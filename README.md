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

## Features

### Diagramming — Visio / draw.io shape

- Shape palette: **rectangle**, **square**, **circle**, **ellipse**,
  **parallelogram**, **text** — sticky tools, click to place, `Esc` to
  disarm.
- Movable tool ribbon — dock to any of the four sides.
- Selectable canvas background (Light / Slate / Charcoal / Dark).
- Configurable grid: lines or dots, snap-to-grid, free spacing.
- Selection: click, shift-click multi-select, marquee.

### Node graphs

- **Relational connections** — edges reference `(Node, Port)`, not
  coordinates. Moving a node drags every wire connected to it.
- **Port-direction-aware orthogonal routing** — wires exit
  perpendicular to each port's face (N/S vertical, E/W horizontal).
  No more co-linear runs along a shape's edge.
- **Bezier**, **straight**, and **manual** routing modes alongside
  orthogonal.
- **Adjacent-waypoint follow** — dragging a node drags its wires'
  elbows so the route stays perpendicular to the moved port.
- **Wire segment selection** — click a single run of a multi-segment
  wire; that run highlights on its own.
- **Free-ended wires** — delete a segment, surviving runs stay
  anchored where they were, dangling at the cut.
- **Extend from a dangling end** — drag a free-end marker to place a
  new segment or to snap back onto a port. Free ends also accept
  incoming connections.
- Right-click context menu on wires: edit colour / width / style,
  delete a single segment, delete the whole wire, add a pivot.

### DSL scripting

- First-class `.canvas` declarative DSL — text is the source of truth,
  not a binary format.
- Live parse: editor edits push a new `Scene` into the canvas with a
  300 ms debounce.
- Parse errors land in the Logger tab with the offending line.
- Hand-written lexer + recursive-descent parser; round-trip stable
  (`parse(pretty(scene))` reconstructs the scene).

### Layout & UI

- **Dockable, drag-rearrangeable panels** via [`egui_dock`][dock]:
  Project · Editor · Canvas across the top, Logger across the bottom.
  Panels move freely *within* the application window — splits, tabs,
  drag-rearrange.
- **One-hot citizen activation** through [`egui_citizen`][citizen]'s
  Dispatcher — at most one panel is active at a time, with message
  routing keyed to a stable `CitizenId`.
- Top ribbon: live clock, clickable version chips, About modal,
  Hotkeys menu, Settings access.
- **Settings tab**: UI scale (0.7×–2.0×), timezone (full chrono-tz
  list), 12 / 24-hour clock, default directory for file dialogs.
- **Tokyo Night Storm** theme throughout.
- Project panel: Open File… with native (rfd) + WASM (browser picker)
  file pickers; per-session recent list.

### Rendering

- Retained **wgpu** GPU pipeline through `egui_grafica`'s `gpu` feature.
- Procedural grid shader — zero geometry, crisp at any zoom.
- Instanced node bodies — rect / circle / ellipse / parallelogram via
  a fragment-shader SDF, with inside borders.
- Instanced edge segments — antialiased oriented quads, dash and
  dotted patterns computed in the shader.
- **Dirty-tracked VRAM cache** — pan / zoom re-uploads only the
  viewport uniform; instance buffers stay untouched.

### Interaction

- **Ctrl + right-mouse-drag** to pan; mouse wheel to zoom; middle-drag
  kept as a fallback.
- Hotkeys (cursor-hover gated):
  `G` toggle grid · `X` / `Y` mirror axes · `R` rotate 90° ·
  `Esc` disarm shape tool · `Del` / `Backspace` delete selection.
- Right-click context menus on canvas elements (wire / segment / pivot).
- Geometry kernel: [`hypercurve`][hypercurve] — exact arithmetic for
  hit-testing, intersection, and routing primitives.

## Status & Roadmap

The scaffold runs end-to-end: dock layout, DSL → canvas pipeline,
ribbon, settings, theme, and the full canvas feature surface above.

**Coming next:**

- Node-graph widget on the tool palette (shape with default in / out
  ports — the node-graph primitive).
- Right-click context menu on a *node*: add / edit text, edit border
  colour / width / style.
- Resizable widgets — drag handles on the selected node.
- Bidirectional DSL ↔ canvas live sync (currently one-way: text → canvas).
- WASM build path (scaffold present; full QA is desktop-first).

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

[mobius]:     https://github.com/saturn77/egui_mobius
[grafica]:    https://github.com/saturn77/egui_mobius/tree/master/crates/egui_grafica
[quill]:      https://github.com/saturn77/egui_mobius/tree/master/crates/egui_quill
[lens]:       https://github.com/saturn77/egui_mobius/tree/master/crates/egui_lens
[citizen]:    https://github.com/saturn77/egui_mobius/tree/master/crates/egui_citizen
[dock]:       https://github.com/Adanos020/egui_dock
[hypercurve]: https://github.com/timschmidt/hypercurve
