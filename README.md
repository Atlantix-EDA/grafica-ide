# grafica-ide

[![Rust 2024](https://img.shields.io/badge/rust-2024_edition-orange?logo=rust)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![egui](https://img.shields.io/badge/egui-0.34-yellow)](https://github.com/emilk/egui)
[![egui_mobius](https://img.shields.io/badge/egui__mobius-0.4-blue)](https://github.com/saturn77/egui_mobius)
[![egui_grafica](https://img.shields.io/badge/egui__grafica-0.4-green)](https://github.com/saturn77/egui_mobius/tree/master/crates/egui_grafica)
[![License: MIT](https://img.shields.io/badge/license-MIT-lightgrey)](LICENSE)

A DSL-driven graphics IDE — three things in one application:

- **Visio / draw.io-style diagramming.** Drop rectangles, circles,
  ellipses, parallelograms, and text on a canvas; route connections;
  drag, group, edit. The shape ribbon and the canvas behave like the
  diagramming tools you already know.
- **Scriptable.** The same canvas is also a first-class `.canvas`
  domain-specific language. Edit text on the left; the diagram updates
  on the right. Save the DSL — that's the source of truth, not a
  binary file format you can't grep.
- **Node-graph editor.** The connection model is relational
  (port-to-port), with auto-routing, free-ended segments, and live
  re-routing. The diagram engine is a node-graph engine.

Built as a composition of citizens on the [`egui_mobius`][mobius]
framework:

| Pane     | Crate          | Role |
|----------|----------------|------|
| Canvas   | [`egui_grafica`][grafica] | Diagram + node graph, GPU-rendered via wgpu |
| Editor   | [`egui_quill`][quill]     | Live `.canvas` DSL editor |
| Logger   | [`egui_lens`][lens]       | Reactive event log |
| Project  | grafica-ide               | File open / recent / library tree |

Panels are panes in an [`egui_dock`][dock] workspace, coordinated by
[`egui_citizen`][citizen]'s `Dispatcher` (one-hot activation, message
routing, stable per-citizen identity for future window-detach work).

## Status

Early scaffold. The shape of the application is in place:

- Dock layout: Project · Editor · Canvas across the top, Logger across
  the bottom.
- DSL editor → canvas pipeline: edits parse on a 300 ms debounce and
  push a new `Scene` into the canvas's `Registry`. Parse errors land
  in the Logger.
- Top ribbon, Settings tab (UI scale, timezone, clock format, default
  directory), Tokyo Night Storm theme.

What's coming next is in the task plan — bidirectional DSL sync, a
node-graph widget on the shape ribbon, real project trees, breakaway
panels.

## Run

grafica-ide currently uses **path-deps on a sibling checkout of
[`egui_mobius`][mobius]** — the citizens have not yet published to
crates.io. Layout expected:

```text
your-workspace/
├── egui_mobius/        # clone of saturn77/egui_mobius
└── grafica-ide/        # this repo
```

Then:

```sh
cargo run --release
```

The first build pulls in `wgpu` and friends — give it a few minutes.

## Architecture

Three layers, cleanly separated:

1. **`egui_mobius` framework** — `Dynamic<T>` reactive cells,
   `Citizen` trait, `Dispatcher`, the `.canvas` DSL kernel.
2. **`egui_grafica` canvas citizen** — the diagram engine. Owns the
   `Scene` registry, the interaction state machine, and a retained
   wgpu rendering pipeline.
3. **grafica-ide application** — composes the canvas with editor /
   logger / project citizens, hosts them on `egui_dock`, wires the
   DSL pipeline.

This file's job is the third layer. The first two live upstream in
the `egui_mobius` repo.

## License

MIT — see [LICENSE](LICENSE).

[mobius]:  https://github.com/saturn77/egui_mobius
[grafica]: https://github.com/saturn77/egui_mobius/tree/master/crates/egui_grafica
[quill]:   https://github.com/saturn77/egui_mobius/tree/master/crates/egui_quill
[lens]:    https://github.com/saturn77/egui_mobius/tree/master/crates/egui_lens
[citizen]: https://github.com/saturn77/egui_mobius/tree/master/crates/egui_citizen
[dock]:    https://github.com/Adanos020/egui_dock
