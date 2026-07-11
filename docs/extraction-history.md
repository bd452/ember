# Ember Extraction and Rename Plan

Status: proposed
Date: 2026-07-11

## Decision

Rename **Ember** to **Ember** and extract it into
`bd452/ember`, checked out locally at `/Users/bryce/projects/ember`.

Ember plays on the Kindle/fire lineage without borrowing another UI framework's
name. It is short, memorable, and maps cleanly across every public surface:

| Surface | Current | Proposed |
| --- | --- | --- |
| Product | Ember | Ember |
| GitHub repository | n/a | `bd452/ember` |
| Parent component path | n/a | `components/ember` |
| Rust library crate | `ember` | `ember` |
| Rust demo crate/binary | `ember-demo` | `ember-demo` |
| Shared library | `libember.so` | `libember.so` |
| C header / symbol prefix | `ember.h` / `ember_*` | `ember.h` / `ember_*` |
| KPM framework package | `com.bd452.ember` | `com.bd452.ember` |
| KPM demo package | `com.bd452.emberdemo` | `com.bd452.emberdemo` |

The rename should be a clean break. The current version is `0.1.0`, the demo
has no runtime dependency on the library package, and the public C interface is
still young. Carrying duplicate crate names, libraries, headers, symbols, or KPM
IDs would create permanent compatibility work for an API that has not shipped
as stable. Release notes can say "formerly Ember," but the implementation
should not retain aliases.

Before publication, verify that `bd452/ember` is available and that the name
does not create a meaningful conflict with an established Kindle UI project.

## Ownership Boundary

The standalone `ember` repository owns:

- the Rust workspace and lockfile;
- `ember`, `ember-demo`, and `fbink-sys`;
- the pinned FBInk source dependency;
- the framework and demo KPM package sources;
- container and koxtoolchain build helpers needed for independent builds;
- the Kindle deploy/run helper;
- framework, C ABI, device, build, and project-context documentation;
- host tests, cross-build checks, and standalone release validation.

`kinstaller-repo` owns:

- the pinned `components/ember` submodule commit;
- repository-wide package ordering;
- copying the two built `.kpkg` files into `packages/`;
- the public package-index `manifest.json` and GitHub Pages deployment;
- any explicit transition policy for the old package IDs.

The parent must not keep a second editable copy of Ember source or package
recipes. The submodule is the source of truth and the parent is its distributor.

## Migration Sequence

### 1. Freeze and record the source state

1. Confirm `main` is clean except for this plan.
2. Create a local snapshot branch such as `codex/ember-pre-extraction`.
3. Record the snapshot commit in both repositories' project-context docs.
4. Return to `main` before changing the ownership boundary.

This matches the successful Kindle Substrate extraction and provides an exact
pre-rename recovery point.

### 2. Build the standalone repository without renaming yet

Create `/Users/bryce/projects/ember` from the Ember introduction commit
through the snapshot, preserving relevant history rather than starting with an
unattributed file copy. Import these logical units:

- `ember`;
- `ember-demo`;
- `fbink-sys`;
- the Rust workspace manifest and lockfile;
- `apps/com.bd452.ember`;
- `apps/com.bd452.emberdemo`;
- FBInk as a nested submodule;
- the build, packaging, version-sync, toolchain, and Kindle-run helpers those
  targets actually use;
- the Dockerfile, license, ignore rules, and relevant documentation.

First make this imported repository pass under the old names. Separating the
history extraction from the rename makes failures attributable: repository
boundary problems are fixed before identifier churn begins.

### 3. Rename every public and internal surface atomically

Rename directories, Cargo package names, Rust imports, features, binary names,
documentation, generated headers, C symbols, shared-library output, package
directories, package manifests, install/uninstall scripts, launcher filenames,
and deploy-script defaults in one standalone-repo change.

Specific rename checks:

- regenerate `ember.h` with cbindgen; do not hand-edit only the filename;
- rename all exported `ember_*` functions and update C ABI tests;
- ensure `cargo metadata` contains no `ember` packages or path dependencies;
- ensure built artifacts are `libember.so` and `ember-demo` for both Kindle ABIs;
- ensure package payloads and scripts contain no old filesystem paths;
- allow old-name mentions only in migration history/release notes.

Start Ember at `0.1.0`. This is a product/package rename, not a compatible
upgrade of `com.bd452.ember`, so the old ID's version sequence should not be
reused to imply in-place upgrade semantics.

### 4. Make the standalone repository publication-ready

Add:

- a root README with the quick start and architecture;
- `docs/project-context.md` describing provenance and the parent/distributor
  relationship;
- MIT `LICENSE`, `.gitignore`, recursive-clone instructions, and contribution
  basics;
- CI for host tests, C ABI tests, formatting/linting, and both Kindle target
  builds;
- a release workflow that produces the two KPM artifacts (or attaches them to
  a GitHub release) without modifying `kinstaller-repo`;
- a documented version-bump process with one source of truth.

Create the public repository as `bd452/ember`, push the preserved history, and
use a tag such as `v0.1.0` only after the full validation gate passes.

### 5. Cut `kinstaller-repo` over to the submodule

Add:

```text
components/ember -> https://github.com/bd452/ember.git
```

Then remove the in-parent Rust crates and the two old package-source
directories. Update the root build and package-update scripts to invoke:

```text
components/ember/apps/com.bd452.ember/build.sh
components/ember/apps/com.bd452.emberdemo/build.sh
```

Update the Pages workflow to test the component workspace, cache its Cargo
target, and initialize submodules recursively. Update root/app docs and the
Kindle launcher to point through the component boundary.

The parent should continue committing the generated `.kpkg` artifacts and
index entries, just as it does for Kindle Substrate.

### 6. Retire the old package IDs explicitly

Because KPM package IDs are identities, not display labels, do not silently
change the contents behind `com.bd452.ember`.

For the initial cutover:

1. publish `com.bd452.ember` and `com.bd452.emberdemo` as new packages;
2. remove the old Ember entries from the active index if there are no
   external users;
3. keep the already-published old artifact files only if immutable historical
   URLs are desirable;
4. document manual removal of the old package before installing Ember;
5. do not make the new demo depend on the framework package, because it remains
   self-contained through static linking.

If external users are discovered before cutover, replace step 2 with one final
deprecated Ember package that contains only a clear migration notice. Do
not ship two live framework ABIs indefinitely.

## Validation Gates

The extraction is complete only when all of these pass:

### Standalone repository

- recursive clone starts clean, including FBInk;
- `cargo test -p ember` passes;
- `cargo test -p ember --features capi` passes;
- the host `ember-demo` smoke run passes;
- framework and demo cross-build for `kindlehf` and `kindlepw2`;
- both KPM packages build from the standalone checkout;
- generated C header matches the Rust ABI and exports only `ember_*` symbols;
- `rg -i 'ember'` finds only intentional migration-history text;
- builds do not dirty the FBInk submodule.

### Parent integration

- recursive parent clone initializes `components/ember` and nested FBInk;
- a full container build succeeds through the submodule boundary;
- `packages/com.bd452.ember` and `packages/com.bd452.emberdemo` are refreshed;
- the root index contains the new IDs, names, descriptions, versions, and URLs;
- GitHub Pages CI tests the standalone workspace through the pinned commit;
- timestamp-only `.kpkg` churn is excluded from the migration commit.

### Physical Kindle

- install `com.bd452.ember` on a supported Kindle and verify its library/header
  payload;
- install and launch `com.bd452.emberdemo`;
- verify touch input, layout, partial refresh, full-refresh thresholds, exit,
  pillow restoration, and relaunch;
- verify the on-device process is `ember-demo` and no old Ember paths are
  required.

Build and CI success prove the repository boundary; they do not replace the
physical-device gate.

## Proposed Commit Structure

Keep review and rollback simple with four conceptual commits:

1. `Preserve Ember source history in standalone repository`
2. `Rename Ember to Ember`
3. `Make Ember independently buildable and publishable`
4. `Publish Ember from pinned standalone repository` (in `kinstaller-repo`)

The GitHub repository should be created only after commits 1–3 are locally
verified. The parent cutover should happen only after the standalone remote is
reachable, so `.gitmodules` never points at a nonexistent source of truth.

## Out of Scope

- redesigning the reactive model or refresh heuristics during extraction;
- publishing the Rust crate to crates.io in the first cutover;
- maintaining permanent Ember compatibility aliases;
- claiming device validation based only on host tests or cross-builds.
