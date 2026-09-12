"""Journaled gclient rolls; Git patch-stack operations remain owned by spo."""
from __future__ import annotations

import json
import os
import re
import uuid
from pathlib import Path

from .common import TaskError, git, require, state_lock, try_git, write_json

UPSTREAM = "refs/spo/upstream"


def revision(source):
    text = (source / "DEPS").read_text(encoding="utf-8")
    match = re.search(r"['\"]chromium_revision['\"]\s*:\s*['\"]([0-9a-fA-F]{40})['\"]", text)
    require(match, "missing literal chromium_revision in DEPS")
    return match[1]


def targets(context):
    if context.dry_run:
        configured = json.loads((context.source / "patches/config.json").read_text(encoding="utf-8"))
        result = []
        for item in configured:
            repo = (context.workspace / item["repo"]).resolve()
            name = item.get("name") or Path(item["patch_dir"]).name
            base = revision(context.source) if name == "chromium" else item.get("base") or git(repo, "rev-parse", "--verify", item["base_ref"])
            patch_dir = (context.workspace / item["patch_dir"]).resolve()
            require(patch_dir.is_dir(), f"configured patch directory does not exist: {patch_dir}; check patches/config.json and the checkout layout after a project rename")
            result.append({"name": name, "repo": str(repo), "patch_dir": str(patch_dir), "base": base, "exclude": item.get("exclude", [])})
        return result
    return context.bridge("load", workspace=str(context.workspace), config=str(context.source / "patches/config.json"), bases={"chromium": revision(context.source)})


def declarations(context, sha):
    require(re.fullmatch(r"[0-9a-fA-F]{40}", sha), "--chromium-revision requires a full SHA")
    path = context.source / "DEPS"
    original = path.read_text(encoding="utf-8")
    old = revision(context.source)
    require(original.count(old) == 1, "ambiguous Chromium revision declaration")
    return [{"path": str(path), "original": original, "updated": original.replace(old, sha, 1)}]


def sync_args(context, configured):
    framework = context.source.relative_to(context.workspace).as_posix()
    args = ["sync", "--no-history", "--nohooks", "--revision", f"{framework}@unmanaged"]
    for target in configured:
        repo = Path(target["repo"]).relative_to(context.workspace).as_posix()
        args.extend(["--revision", f"{repo}@{target['base']}"])
    return args


def invoke(context, args):
    executable = os.environ.get("TUFFITE_GCLIENT") or str(context.workspace / "depot_tools" / ("gclient.bat" if os.name == "nt" else "gclient"))
    context.command([executable, *args], cwd=context.workspace, extra_env={"TUFFITE_SKIP_PATCHES": "1"})


def fingerprint(context, configured):
    values = {}
    for path in (context.source / "DEPS", context.workspace / ".gclient", context.source / "patches/config.json"):
        values[str(path)] = git(context.source, "hash-object", str(path))
    for target in configured:
        values[f"{target['name']}:base"] = target["base"]
        for name in [".patches", *context.bridge("filenames", target)]:
            path = Path(target["patch_dir"]) / name
            values[str(path)] = git(target["repo"], "hash-object", str(path))
    return values


def inspect(context, target):
    return context.bridge("inspect", target)


def bootstrap(context):
    if os.environ.get("TUFFITE_SKIP_PATCHES") == "1":
        print("Patch hook deferred to python3 x.py roll.")
        return
    for target in targets(context):
        if context.dry_run:
            print(f"{target['name']}: apply patches on {target['base']}")
        else:
            context.bridge("apply", target)


def run(context, args):
    # Preserve the previous Rust runner's journal paths and schema for recovery.
    path = Path(git(context.source, "rev-parse", "--path-format=absolute", "--git-path", "xtask/roll.json"))
    if context.dry_run:
        configured = targets(context)
        if args.chromium_revision:
            declarations(context, args.chromium_revision)
            print(f"Update Chromium revision in DEPS to {args.chromium_revision}")
            for target in configured:
                if target["name"] == "chromium":
                    target["base"] = args.chromium_revision
        print(f"Framework: {context.source}\nPreserve the Framework branch; patches: {args.patches}; WIP: {args.dirty}")
        for target in configured:
            print(f"{target['name']}: preserve stack, sync {target['repo']}, then apply {target['base']}")
        invoke(context, sync_args(context, configured))
        if not args.skip_hooks:
            invoke(context, ["runhooks"])
        return
    with state_lock(path):
        if path.exists():
            require(not args.chromium_revision, "a roll is already active; finish it before changing the revision")
            state = json.loads(path.read_text(encoding="utf-8"))
            if args.abort:
                return abort(context, path, state)
            require(args.resume, f"unfinished roll ({state['phase']}); use --continue or --abort")
            if args.skip_hooks:
                require(state["phase"] in ("preparing", "syncing", "hooks"), "hooks have already been handled in this roll")
                state["skip_hooks"] = True
                write_json(path, state)
            recover(context, args, path, state)
            return advance(context, path, state)
        require(not (args.resume or args.abort or args.wip_restored or args.wip_saved), "no roll is active")
        configured = targets(context)
        receipt = path.with_name("rolled.json")
        if not args.chromium_revision and args.patches != "local" and receipt.exists():
            previous = json.loads(receipt.read_text(encoding="utf-8"))
            if previous == fingerprint(context, configured):
                statuses = [context.bridge("status", target) for target in configured]
                if all(item["current"] and not item["pending"] for item in statuses):
                    print("Dependencies and patches already match this Framework checkout.")
                    return
        changes = declarations(context, args.chromium_revision) if args.chromium_revision else []
        saved = []
        for target in configured:
            baseline_target = target.copy()
            if target["name"] == "chromium" and args.chromium_revision:
                target["base"] = args.chromium_revision
            status = inspect(context, target)
            require(not status["pending"] and not status["native"], f"{target['name']} has an unfinished operation; use cargo spo to recover it first")
            require(args.dirty == "stash" or not status["dirty"], f"{target['name']} has WIP; use --dirty stash")
            repo = target["repo"]
            head = git(repo, "rev-parse", "HEAD")
            # Recover the applied baseline before changing the requested DEPS revision.
            # The bridge verifies ancestry and committed patch content; it never assumes HEAD.
            old_base = context.bridge("baseline", baseline_target)
            old_target = {**target, "base": old_base}
            saved.append({"target": target, "head": head, "branch": try_git(repo, "symbolic-ref", "--quiet", "--short", "HEAD"), "old_base": old_base, "old_groups": context.bridge("groups", old_target, base=old_base), "stash": None})
        state = {"phase": "preparing", "cursor": 0, "targets": saved, "declarations": changes, "local_patches": args.patches == "local", "stash_allowed": args.dirty == "stash", "skip_hooks": args.skip_hooks, "aborted": False}
        write_json(path, state)
        advance(context, path, state)


def recover(context, args, path, state):
    saved = state["targets"][state["cursor"]] if state["cursor"] < len(state["targets"]) else None
    if state["phase"] == "saving-wip":
        require(args.wip_saved, "WIP save was interrupted; inspect the stash and continue with --wip-saved <oid>")
        repo = saved["target"]["repo"]
        oid = git(repo, "rev-parse", f"{args.wip_saved}^{{commit}}")
        require(git(repo, "rev-parse", f"{oid}^1") == saved["head"], "stash was created from a different HEAD")
        require(not inspect(context, saved["target"])["dirty"], "WIP is not safely saved yet")
        saved["stash"] = oid
        state["phase"] = "preparing"
        write_json(path, state)
    if state["phase"] == "restoring-wip":
        require(args.wip_restored, "WIP restoration was interrupted; resolve it and continue with --wip-restored")
        require(not git(saved["target"]["repo"], "ls-files", "--unmerged"), "unresolved conflicts remain")
        state["phase"] = "wip-restored"
        write_json(path, state)


def check_declarations(state):
    for declaration in state["declarations"]:
        actual = Path(declaration["path"]).read_text(encoding="utf-8")
        require(actual in (declaration["original"], declaration["updated"]), f"{declaration['path']} changed outside this roll")


def advance(context, path, state):
    check_declarations(state)
    if state["phase"] == "preparing":
        while state["cursor"] < len(state["targets"]):
            saved = state["targets"][state["cursor"]]
            target = saved["target"]
            head = git(target["repo"], "rev-parse", "HEAD")
            require(head in (saved["head"], saved["old_base"]), f"{target['name']} changed outside this roll")
            if state["local_patches"] and head == saved["head"]:
                old_target = {**target, "base": saved["old_base"]}
                status = inspect(context, old_target)
                context.bridge("continue" if status["pending"] else "export", old_target)
            status = inspect(context, target)
            require(not status["pending"] and not status["native"], f"{target['name']} has another operation")
            if status["dirty"] and saved["stash"] is None:
                require(state["stash_allowed"], "working changes appeared; preserve them before continuing")
                state["phase"] = "saving-wip"
                write_json(path, state)
                saved["stash"] = context.bridge("stash", target)
                state["phase"] = "preparing"
                write_json(path, state)
            git(target["repo"], "update-ref", f"refs/spo/roll-backups/{uuid.uuid4().hex}", saved["head"])
            git(target["repo"], "checkout", "--detach", saved["old_base"])
            state["cursor"] += 1
            write_json(path, state)
        for change in state["declarations"]:
            Path(change["path"]).write_text(change["updated"], encoding="utf-8")
        state.update(phase="syncing", cursor=0)
        write_json(path, state)
    if state["phase"] == "syncing":
        invoke(context, sync_args(context, [item["target"] for item in state["targets"]]))
        for saved in state["targets"]:
            target = saved["target"]
            require(git(target["repo"], "rev-parse", "HEAD") == target["base"], f"gclient did not sync {target['name']} to its requested base")
            git(target["repo"], "config", "core.untrackedCache", "true")
            git(target["repo"], "config", "core.fsmonitor", "true")
        state["phase"] = "hooks"
        write_json(path, state)
    if state["phase"] == "hooks":
        if state.get("skip_hooks", False):
            print("Skipping gclient hooks; reusing the prepared toolchain.")
        else:
            invoke(context, ["runhooks"])
        state.update(phase="applying", cursor=0)
        write_json(path, state)
    if state["phase"] == "applying":
        while state["cursor"] < len(state["targets"]):
            target = state["targets"][state["cursor"]]["target"]
            context.bridge("continue" if inspect(context, target)["pending"] else "apply", target)
            state["cursor"] += 1
            write_json(path, state)
        state.update(phase="restoring", cursor=0)
        write_json(path, state)
    if state["phase"] in ("restoring", "wip-restored"):
        while state["cursor"] < len(state["targets"]):
            saved = state["targets"][state["cursor"]]
            if saved["stash"]:
                if state["phase"] != "wip-restored":
                    require(not inspect(context, saved["target"])["dirty"], "preserve new changes before restoring WIP")
                    state["phase"] = "restoring-wip"
                    write_json(path, state)
                    context.bridge("restore-stash", saved["target"], oid=saved["stash"])
                    state["phase"] = "wip-restored"
                    write_json(path, state)
                context.bridge("drop-stash", saved["target"], oid=saved["stash"])
            state["cursor"] += 1
            state["phase"] = "restoring"
            write_json(path, state)
        if not state.get("aborted", False) and not state.get("skip_hooks", False):
            write_json(path.with_name("rolled.json"), fingerprint(context, [item["target"] for item in state["targets"]]))
        path.unlink()
        print("Roll aborted. Original stacks and declarations restored; other gclient dependencies may need synchronization." if state.get("aborted", False) else "Roll complete. Framework branch preserved; dependency patches applied. Review version declarations before committing.")


def abort(context, path, state):
    require(state["phase"] not in ("restoring-wip", "wip-restored", "saving-wip"), "WIP save/restoration was interrupted; resolve and continue it before aborting")
    check_declarations(state)
    for saved in state["targets"]:
        target = saved["target"]
        if inspect(context, target)["pending"]:
            context.bridge("abort", target)
        status = inspect(context, target)
        require(not status["dirty"] and not status["native"], f"{target['name']} has new work or a native Git operation; preserve it before aborting")
        if saved["branch"]:
            require(git(target["repo"], "rev-parse", f"refs/heads/{saved['branch']}") == saved["head"], f"original branch {saved['branch']} changed outside the roll")
    for saved in state["targets"]:
        target = saved["target"]
        if saved["branch"]:
            git(target["repo"], "checkout", saved["branch"])
        else:
            git(target["repo"], "checkout", "--detach", saved["head"])
        git(target["repo"], "update-ref", UPSTREAM, saved["old_base"])
        context.bridge("record", target, base=saved["old_base"], groups=saved["old_groups"])
    for change in state["declarations"]:
        Path(change["path"]).write_text(change["original"], encoding="utf-8")
    state.update(aborted=True, phase="restoring", cursor=0)
    write_json(path, state)
    advance(context, path, state)
