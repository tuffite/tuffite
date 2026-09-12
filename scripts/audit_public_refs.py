#!/usr/bin/env python3
"""Read-only privacy/credential scan of explicitly selected publication refs."""
import argparse
import json
import re
import subprocess
from pathlib import Path

RULES = {
    "private_key": rb"-----BEGIN (?:RSA |EC |OPENSSH |DSA |ENCRYPTED )?PRIVATE KEY-----",
    "github_token": rb"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,})\b",
    "aws_access_key": rb"\b(?:AKIA|ASIA)[A-Z0-9]{16}\b",
    "google_api_key": rb"\bAIza[A-Za-z0-9_-]{30,}\b",
    "credential_url": rb"https?://[^\s/\"']+:[^\s/@\"']+@",
    "personal_path": rb"(?:/(?:Users|home)/[^/\s\"']+|[A-Za-z]:[\\/]+Users[\\/]+[^\\/\s\"']+)",
}


def git(repo, *args):
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, check=True)
    return result.stdout


def audit(repo, refs, private_domains):
    patterns = {name: re.compile(pattern) for name, pattern in RULES.items()}
    for domain in private_domains:
        patterns[f"private_email_domain:{domain}"] = re.compile(
            rb"@" + re.escape(domain.encode()) + rb"\b", re.IGNORECASE
        )
    findings = []
    for ref in refs:
        commit = git(repo, "rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}").decode().strip()
        objects = git(repo, "rev-list", "--objects", commit).decode().splitlines()
        with subprocess.Popen(["git", "-C", str(repo), "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE) as batch:
            for entry in objects:
                oid, _, path = entry.partition(" ")
                batch.stdin.write((oid + "\n").encode())
                batch.stdin.flush()
                header = batch.stdout.readline().decode().split()
                data = batch.stdout.read(int(header[2]))
                batch.stdout.read(1)
                if header[1] not in ("blob", "commit"):
                    continue
                for rule, pattern in patterns.items():
                    for match in pattern.finditer(data):
                        findings.append({"ref": ref, "object": oid, "path": path or "<commit metadata>", "rule": rule, "line": data.count(b"\n", 0, match.start()) + 1})
            batch.stdin.close()
            batch.wait()
    return findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--ref", action="append", required=True, help="Exact publication ref; repeat for each branch/tag")
    parser.add_argument("--private-domain", action="append", default=[], help="Private author/content email domain to flag")
    args = parser.parse_args()
    try:
        findings = audit(args.repo, args.ref, args.private_domain)
    except (OSError, subprocess.CalledProcessError) as error:
        parser.exit(2, f"Cannot audit selected refs: {error}\n")
    print(json.dumps({"refs": args.ref, "findings": findings}, indent=2))
    return bool(findings)


if __name__ == "__main__":
    raise SystemExit(main())
