# ember-demo

On-device / host demo for [`ember`](../ember/).

- **Host** (default): mounts the demo screen, renders one frame with
  `MockRenderer`, exits. No FBInk or cross toolchain.
- **Device** (`--features fbink`): `FbinkRenderer` + touch `App::run`.

KPM packaging (binaries per platform, launcher, `.kpkg`) lives in
[`apps/com.bd452.emberdemo`](../../apps/com.bd452.emberdemo/) — see that
README for Docker/Linux build and deploy.

Library docs: [`ember/docs/`](../ember/docs/), especially
[building.md](../ember/docs/building.md) and [usage.md](../ember/docs/usage.md).

```sh
# Host smoke
cargo run -p ember-demo

# Device (from repo root, via container helper on macOS)
./scripts/build-in-container.sh apps/com.bd452.emberdemo/build.sh
```
