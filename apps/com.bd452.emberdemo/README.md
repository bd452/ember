# com.bd452.emberdemo

Interactive KPM demo for the Ember UI framework: reactive counter, keyed
`for_each` list, touch buttons, and Exit (restores the Kindle home booklet).

Source lives in the Cargo workspace (`ember-demo`), not under
`package/`. This directory is packaging only: cross-build, stage binaries,
icon, and `.kpkg`.

## What it is

| Piece | Location |
|-------|----------|
| Rust app | [`ember-demo`](../../ember-demo/) |
| Framework | [`ember`](../../ember/) |
| Framework docs | [`ember/docs/`](../../ember/docs/) |
| Build / Docker | [`ember/docs/building.md`](../../ember/docs/building.md) |

The binary **statically links** Ember + FBInk. There is **no** runtime
dependency on `com.bd452.fbink` or `com.bd452.ember`.

## Package layout

```text
apps/com.bd452.emberdemo/
  build.sh                 # cross-compile both platforms → package/bin/
  scripts/make-icon.py     # generates package/icon.png
  package/
    manifest.json
    app.sh                 # picks kindlehf vs kindlepw2 by dynamic linker
    launch.sh              # disable pillow → app.sh → restore home
    install.sh / uninstall.sh
    bin/kindlehf/ember-demo    # produced by build.sh (gitignored staging)
    bin/kindlepw2/ember-demo
    icon.png
```

`launch.sh` disables the status-bar overlay (`pillow`) while the demo runs and
on exit asks `appmgrd` to start `app://com.lab126.booklet.home` so the
framework repaints instead of leaving a blank panel.

## Prerequisites

Same as the library cross build — see
[building.md](../../ember/docs/building.md):

1. `git submodule update --init --recursive` (FBInk under `com.bd452.fbink`)
2. Linux x86_64 + koxtoolchain, **or** Docker/`./scripts/build-in-container.sh`
3. Rust stable with both ARM targets (provided inside `kinstaller-build`)

## Build

### Docker / macOS (recommended on non-Linux hosts)

From the **repo root**:

```sh
# Image: docker build --platform linux/amd64 -t kinstaller-build .
./scripts/build-in-container.sh apps/com.bd452.emberdemo/build.sh
```

Or a one-off cargo build of the demo only (kindlehf):

```sh
./scripts/build-in-container.sh bash -lc '
source scripts/koxtoolchain.sh
platform=kindlehf
export CROSS_TC="$(kox_prefix "$platform")"
export PATH="$(kox_tool_bin "$platform"):$PATH"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="$(kox_tool_bin "$platform")/${CROSS_TC}-gcc"
cargo build --manifest-path Cargo.toml \
  -p ember-demo --release --features fbink \
  --target "$(kox_rust_target "$platform")"
'
```

Binary then at:

```text
target-kindle/armv7-unknown-linux-gnueabihf/release/ember-demo
```

(when using the helper’s `CARGO_TARGET_DIR`), or under `target/…` for a
native Linux package `build.sh` (default cargo target dir).

### Native Linux x86_64

```sh
./scripts/setup-koxtoolchain.sh   # once
./apps/com.bd452.emberdemo/build.sh
```

Stages:

- `package/bin/kindlehf/ember-demo`
- `package/bin/kindlepw2/ember-demo`

then packs a `.kpkg` via `scripts/pack-app.sh`.

### Host stub (no Kindle toolchain)

```sh
cd rust
cargo run -p ember-demo
# mounts UI, renders one MockRenderer frame, exits
```

## Install / run on device

**KPM** (after a repo `./build.sh` publish or local `.kpkg`):

```text
;kpm install com.bd452.emberdemo
```

Launch from KUAL / the package launcher.

**SSH dev loop** (jailbroken Kindle; use whole-second `sleep` — BusyBox):

Preferred:

```sh
./scripts/build-in-container.sh apps/com.bd452.emberdemo/build.sh

# Known IP:
KINDLE=root@192.168.1.231 KINDLE_PASSWORD=kindle \
  ./scripts/run-ember-demo-on-kindle.sh

# Unknown IP: tries SSH against hosts already in the local ARP table.
KINDLE_PASSWORD=kindle ./scripts/run-ember-demo-on-kindle.sh
```

Manual equivalent:

```sh
BIN=apps/com.bd452.emberdemo/package/bin/kindlehf/ember-demo
# or: target-kindle/armv7-unknown-linux-gnueabihf/release/ember-demo
KINDLE=root@192.168.1.231

ssh "$KINDLE" 'killall ember-demo 2>/dev/null || true; sleep 1'
scp "$BIN" "$KINDLE:/mnt/us/ember-demo"
ssh "$KINDLE" '
lipc-set-prop com.lab126.pillow disableEnablePillow disable 2>/dev/null || true
cd /mnt/us
nohup ./ember-demo >/mnt/us/ember-demo.out 2>/mnt/us/ember-demo.err &
sleep 2
ps | grep ember | grep -v grep
'
```

## Notes

- Manual launch grabs the touchscreen and paints the framebuffer; it is **not**
  a booklet. Sleep / “swipe to unlock” will fight the grab until booklet
  integration exists — see [device.md](../../ember/docs/device.md).
- Touch calibration uses `EVIOCGABS`; overrides:
  `EMBER_TOUCH_DEV`, `EMBER_TOUCH_SWAP`, `EMBER_TOUCH_INVERT_X/Y`,
  `EMBER_TOUCH_RAW`.
