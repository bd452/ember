# Ember

Reactive UI for Kindle e-readers — a Rust port of the Ember reactive UI
framework (Swift / UIKit).

The reactive model matches the original Swift framework: **signals** hold state,
**components** build a widget tree once, and signal writes mutate widget
properties directly (no re-render, no diffing). Because Kindle has no UIKit,
this crate also ships a retained widget tree, a stack layout solver, an
FBInk-based e-ink renderer, and a touch event loop.

## Features

| Feature | Default | Purpose |
|---------|---------|---------|
| *(none)* | — | Core reactive UI + mock renderer. Host tests and desktop smoke builds. |
| `fbink` | off | FBInk framebuffer backend, touch input, and `App::run` on device. |
| `capi` | off | C ABI (`libember.so` + [`include/ember.h`](include/ember.h)). |

## Quick start (Rust)

```rust
use ember::{
    hstack, vstack, Align, App, BuildCtx, Button, Component, ExitHandle,
    IntoNode, Label, Node, Signal,
};

struct Counter {
    count: Signal<i32>,
    exit: ExitHandle,
}

impl Component for Counter {
    fn build(&mut self, ctx: &mut BuildCtx) -> Node {
        let label = Label::new("").size(3);
        {
            let label = label.clone();
            ctx.bind(&self.count, move |v| label.set_text(format!("Count: {v}")));
        }

        let dec = {
            let count = self.count.clone();
            Button::new(" - ").size(3).on_tap(move || count.update(|v| v - 1))
        };
        let inc = {
            let count = self.count.clone();
            Button::new(" + ").size(3).on_tap(move || count.update(|v| v + 1))
        };
        let exit = {
            let exit = self.exit.clone();
            Button::new("Exit").on_tap(move || exit.request_exit())
        };

        vstack(
            16,
            Align::Center,
            vec![
                label.into_node(),
                hstack(24, Align::Center, vec![dec.into_node(), inc.into_node()]),
                exit.into_node(),
            ],
        )
    }
}
```

On device (with `--features fbink`):

```rust
let exit = ExitHandle::new();
let renderer = ember::render::fbink::FbinkRenderer::open()?;
let mut app = App::new(
    Box::new(Counter {
        count: Signal::new(0),
        exit: exit.clone(),
    }),
    renderer,
    exit,
);
app.run()?;
```

On the host, use `MockRenderer` and call `render_frame` / `tap_at` instead of
`run` — see the [demo](../ember-demo) and the [docs](docs/).

## Documentation

| Doc | Contents |
|-----|----------|
| [docs/README.md](docs/README.md) | Docs index |
| [docs/concepts.md](docs/concepts.md) | Reactive model, architecture, lifecycle |
| [docs/usage.md](docs/usage.md) | Building screens: signals, components, widgets, layout, `slot` / `for_each` |
| [docs/device.md](docs/device.md) | FBInk rendering, damage/refresh, touch input, event loop |
| [docs/c-abi.md](docs/c-abi.md) | Using `libember` from C / other languages |
| [docs/building.md](docs/building.md) | Cold-start: Dockerfile / `build-in-container.sh`, host tests, kox, deploy |

## Crate layout

```
ember/src/
  signal.rs          Signal<T>, write coalescing, observers
  disposable.rs      Disposable (Drop-disposes)
  lifecycle.rs       Ordered teardown scope
  component.rs       Component trait, BuildCtx, mounting
  node.rs            Node tree, vstack / hstack
  structural/        slot, for_each (post-mount tree changes)
  widget/            Label, Button, Spacer, Stack
  layout.rs          Stack measure / place
  render/            Renderer trait, FBInk + mock backends
  input/             evdev touch decode + calibration
  app.rs             App, ExitHandle, on-device run loop
  ffi.rs             C ABI (feature capi)
```

## License

MIT — see the workspace `Cargo.toml`.
