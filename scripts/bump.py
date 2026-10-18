"""Plan and update the shared Framework/SDK release without resolving dependencies."""
from __future__ import annotations

import json
import re
import tomllib

from .common import require


def replace_once(text, pattern, replacement, label):
    updated, count = re.subn(pattern, replacement, text, flags=re.MULTILINE)
    require(count == 1, f"expected one version declaration for {label}, found {count}")
    return updated


def release_version(current, requested):
    pattern = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    require(re.fullmatch(pattern, current), f"unsupported current release: {current}")
    parts = tuple(map(int, current.split('.')))
    if requested in ("major", "minor", "patch"):
        index = ("major", "minor", "patch").index(requested)
        target = (*parts[:index], parts[index] + 1, *((0,) * (2 - index)))
    else:
        require(re.fullmatch(pattern, requested), "version must be major, minor, patch, or X.Y.Z")
        target = tuple(map(int, requested.split('.')))
    require(all(value <= 65535 for value in target), "ABI version components must fit u16")
    require(target > parts, f"new release must be greater than {current}")
    return '.'.join(map(str, target))


def plan(root, requested):
    """Validate every input before returning any edits; leave third-party versions intact."""
    workspace = tomllib.loads((root / 'Cargo.toml').read_text())['workspace']
    current = workspace['package']['version']
    version = release_version(current, requested)
    edits = {}

    def update(path, text):
        original = path.read_text()
        if text != original:
            edits[path] = text

    manifest = root / 'Cargo.toml'
    update(manifest, replace_once(manifest.read_text(),
        r'(^\[workspace\.package\]\nversion = ")[^"]+("$)',
        lambda m: m[1] + version + m[2], 'workspace'))
    names = set()
    for member in workspace['members']:
        manifest = root / member / 'Cargo.toml'
        data = tomllib.loads(manifest.read_text())
        require(data['package']['version'] == {'workspace': True},
                f"{manifest}: package version must inherit workspace")
        names.add(data['package']['name'])
    # The manifests use inline path dependency tables; reject unsupported forms.
    for member in workspace['members']:
        manifest = root / member / 'Cargo.toml'
        text = manifest.read_text()
        data = tomllib.loads(text)
        for section in ('dependencies', 'build-dependencies', 'dev-dependencies'):
            for alias, dependency in data.get(section, {}).items():
                if not isinstance(dependency, dict) or not dependency.get('path'):
                    continue
                if dependency.get('package', alias) not in names:
                    continue
                if 'version' not in dependency:
                    continue
                require(dependency['version'] == current, f"{manifest}: {alias} version is out of sync")
                text = re.sub(r'^(\s*' + re.escape(alias) + r'\s*=\s*\{[^\n]*\bversion\s*=\s*)"' + re.escape(current) + '"',
                              lambda m: m[1] + f'"{version}"', text, flags=re.MULTILINE)
        # Check that each declared internal version was actually replaced.
        checked = tomllib.loads(text)
        for section in ('dependencies', 'build-dependencies', 'dev-dependencies'):
            for alias, dependency in checked.get(section, {}).items():
                if isinstance(dependency, dict) and dependency.get('path') and dependency.get('package', alias) in names and 'version' in dependency:
                    require(dependency['version'] == version, f"{manifest}: unsupported dependency format for {alias}")
        update(manifest, text)
    lock = root / 'Cargo.lock'
    text = lock.read_text()
    for name in sorted(names):
        text = replace_once(text, r'(^name = "' + re.escape(name) + r'"\nversion = ")' + re.escape(current) + r'("$)',
                            lambda m: m[1] + version + m[2], f'Cargo.lock: {name}')
    update(lock, text)
    lock = root / 'bun.lock'
    text = lock.read_text()
    root_package = json.loads((root / 'package.json').read_text())
    for glob in root_package['workspaces']:
        for directory in sorted(root.glob(glob)):
            manifest = directory / 'package.json'
            package = json.loads(manifest.read_text())
            if 'version' not in package:
                continue
            require(package['version'] == current, f"{manifest}: version is out of sync")
            update(manifest, replace_once(manifest.read_text(), r'(^\s*"version": ")[^"]+("[, ]*$)',
                                         lambda m: m[1] + version + m[2], str(manifest)))
            key = directory.relative_to(root).as_posix()
            text = replace_once(text, r'("' + re.escape(key) + r'": \{\n\s*"name": "' + re.escape(package['name']) + r'",\n\s*"version": ")' + re.escape(current) + r'(")',
                                lambda m: m[1] + version + m[2], f'bun.lock: {key}')
    update(lock, text)
    return current, version, edits


def run(context, args):
    current, version, edits = plan(context.source, args.version)
    print(f'Tuffite {current} -> {version} (including ABI)')
    for path, text in edits.items():
        print(f'  {path.relative_to(context.source)}')
    if context.dry_run:
        return
    for path, text in edits.items():
        path.write_text(text, encoding='utf-8')
    print('Versions updated. Rebuild and package matching Framework Distributions before publishing.')
