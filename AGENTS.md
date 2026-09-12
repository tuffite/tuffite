# Tuffite in Chromium development guidance

## Repository boundary

- This repository is checked out at `src/tuffite` inside a gclient workspace.
- Keep all Tuffite framework implementation here: Rust crates, public headers,
  JavaScript packages, Playground, native implementation, GN, Mojo, and
  component-owned Blink/Content integration sources.
- Keep Chromium-owned changes in `patches/chromium`; never commit them as files
  in this repository.
- Do not add another native lock file. The outer workspace's committed
  `.gclient` pins the Framework solution; this repository's `DEPS` is the sole
  Chromium URL/revision declaration. gclient follows that dependency recursively.

## Chromium patch boundaries

- Guard Chromium additions with `tuffite_in_chromium` in GN or
  `BUILDFLAG(TUFFITE_IN_CHROMIUM)` in C/C++ whenever the surrounding syntax
  permits it.
- When an unconditional Chromium edit is unavoidable, enclose the smallest
  practical region in balanced `TUFFITE_BEGIN` / `TUFFITE_END` comments.
- Do not add boundary markers inside this repository's own source files.
- Chromium-owned `BUILD.gn` files must not list Tuffite sources or labels
  directly. Define lists in the component integration directory's
  `tuffite_in_chromium.gni`, import it directly from Chromium, and consume
  its variables inside `if (tuffite_in_chromium)`.
- Feature-owned implementation and IDL sources belong under `components/<feature>`.
  Sources compiled directly into Chromium targets may live in dedicated feature
  integration directories (for example Explorer `browser/content` and
  `renderer/blink`), alongside the `.gni` files imported by Chromium.
  Keep private dependency exceptions scoped to those integration directories.
- Prefix imported variables with `tuffite_`; include the feature name where it
  prevents ambiguity.
- Use source-absolute labels such as `//tuffite/...`; imported GN source strings
  are resolved in the consuming target's directory, not relative to the `.gni`.

## Development workflow

- Prepare tools at the outer root with `mise run setup`, which performs the first pinned checkout. The Python gclient hook applies patches through spo.
- Run Framework commands from `src/tuffite`: `python3 x.py roll`, `python3 x.py build --run`, and `cargo spo`.
- `python3 x.py roll` preserves the current Framework branch and uses gclient for dependencies.
- Use `cargo spo` for Git-backed patch authoring. Patch files are ordered mailboxes;
  one file may contain multiple consecutive commits. `.patches` and patch files belong here.
- Export with `cargo spo export --all --verify-apply`; verify canonical output with
  `cargo spo check --all --strict`. Export reads commits; it does not commit WIP implicitly.
- Preserve interrupted operations; use `cargo spo continue <target>` / `abort <target>`
  or `python3 x.py roll --continue` / `--abort`. Never clear operation state to bypass a conflict.
- Keep patches small, ordered, and independently reviewable. Chromium commits must not include `tuffite/` files.
- `python3 x.py` is the sole Framework task entry; GN/Ninja operations use the internal
  Python 3 modules in `scripts/`. No-argument invocation prints help.
  The first-checkout hook uses spo.
- Application developers use the `tuff` binary. Reusable Distribution APIs live in
  `tuffite::build::framework`; keep SDK build helpers independent of spo.
