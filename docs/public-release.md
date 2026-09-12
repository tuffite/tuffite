# Public source release

Tuffite is an experimental source release. Production hardening, signed updates,
DNS-resolution-aware network policy, and supported platform releases remain
tracked in production-readiness.md. Public sources do not imply production safety.

## Repositories and refs

Publish the Framework and website main branches after committing and reviewing
the changes. The outer workspace must also be available to contributors because
the documented first checkout depends on it. Make its Framework pin refer to the
reviewed public commit before announcing onboarding.

The current local main branches each start from a sanitized single commit.
Legacy Framework tags v0.1.0 and v0.1.1 and old remote-tracking branches still
retain company author addresses and personal-path planning documents. Keep those
refs private; publishing main does not require deleting local recovery history.
Never use push --mirror or push --tags for the initial public release.

Run the read-only ref audit for each exact ref that will be published:

```sh
python3 scripts/audit_public_refs.py --repo . --ref main
python3 scripts/audit_public_refs.py --repo ../../www --ref main --ref release
# Inspect legacy tags separately; these are expected to need cleanup:
python3 scripts/audit_public_refs.py --repo . --ref v0.1.0 --ref v0.1.1
```

The audit checks reachable blobs and commit identities without printing matched
credentials. Add --private-domain with each private email domain to flag company
author addresses; there is no organization-specific domain built into the tool. It is a pattern scan, not a proof that arbitrary sensitive content
is absent. If an existing public remote retains unwanted tags/branches, inspect
its refs and agree on their removal or replacement before changing them. A clean
local main does not remove already published history, forks, or releases.
Explicit branch pushes publish only the selected history. Publishing, deleting
remote tags, force-pushing existing branches, and changing repository visibility
are separate maintainer actions after review.

## Native notices and validation

Repackage all existing Framework Distributions; previous artifacts lack required
notices and must not be uploaded or reused as public packages.

```sh
python3 x.py package --profile release --target-cpu x64 --skip-build
python3 x.py verify dist/tuffite-framework-macos-x64
```

Choose the target CPU, output, and Distribution path for the actual platform.
The package contains runtime/licenses/LICENSE.tuffite.txt,
LICENSE.chromium.txt, and THIRD_PARTY_NOTICES.chromium.txt. Chromium's own license
tool generates the latter from the selected GN dependency graph; errors or empty
notices stop packaging. They are declared and hashed as runtime files, so the
application packager copies them automatically. Application authors are responsible
for their Rust/frontend/resource dependency notices as well.

## SDK and website publishing

Public registry 0.0.0 packages are placeholders. The website's default quick start
uses the source workspace and Playground until the actual 0.1.0 SDK and matching
Framework Distributions are published and verified on clean supported machines.
Do not silently install the placeholder as the development CLI.

Before a package release, run Cargo packaging/verification for tuffite-abi (2.5.0),
tuffite-macros, tuffite-runtime, and tuffite in dependency order. Review package
contents and license inclusion; then publish in that order after verifying the
matching Framework assets. SDK publishing and native asset upload do not happen
as part of source cleanup.

The website builds Rustdoc in its dedicated target cache, clears stale docs,
and packages project/source/dependency licenses. Its tests validate bilingual
content and TOML examples. Preparing a release creates a local release branch;
only an explicit reviewed push publishes it.
