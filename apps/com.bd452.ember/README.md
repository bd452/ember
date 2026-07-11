# com.bd452.ember

KPM package for the Ember library: `libember.so` (C ABI + FBInk
backend, statically linked) and `ember.h`.

Rust sources: [`ember`](../../ember/). Docs:
[`ember/docs/`](../../ember/docs/).

No runtime dependencies — FBInk is linked into the `.so`.

## Build

```sh
# macOS / Docker
./scripts/build-in-container.sh apps/com.bd452.ember/build.sh

# Linux x86_64 (kox installed)
./apps/com.bd452.ember/build.sh
```

Stages `package/lib/{kindlehf,kindlepw2}/libember.so` and
`package/include/ember.h`, then packs a `.kpkg`.
