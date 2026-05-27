# GNX Language Specification

**James Bonanno**, 27 May 2026

**Document Version**: 0.1 (Draft)

---

## Abstract

GNX (Graphica Node eXchange) is a **declarative, text-first description language** for two-dimensional engineering diagrams. A `.gnx` document describes everything a viewer needs to reproduce a drawing: the drafting sheet, named style classes, primitive shapes ("nodes"), and the connections between them ("wires"). It is the on-disk format for the `egui_grafica` canvas and the editor surface for `grafica-ide`.

This document specifies GNX, including:

- **Lexical structure** — line-oriented tokens, four comment styles, double-quoted strings, signed decimal numbers.
- **Top-level grammar** — `canvas { ... }` root block containing exactly one `settings` block, zero or more `style` blocks, and an ordered sequence of `node` and `wire` blocks.
- **Style classes** — re-usable overlay (border / fill / text) and port templates referenced by nodes through a `: stylename` suffix, with inline fields overriding style values.
- **Coordinate model** — world-unit positions interpreted through the document's unit declaration (`pixels`, `mils`, `millimeters`, `inches`).
- **Page model** — optional drafting sheet with paper size, orientation, and engineering-style title block.
- **Round-trip invariant** — `parse(pretty(scene)) ≡ scene` for every valid `scene`.

---

## Motivation

The canvas world has two dominant authoring modes: visual editors that hide their data behind binary or XML formats (Visio, draw.io, Lucid), and pure-text languages with no editor surface (TikZ, mermaid, PlantUML). The first locks the drawing inside a tool; the second is hostile to the geometric authoring that diagrams actually need.

GNX takes a third path: a **dual-surface format** that is simultaneously a hand-readable text file and the canonical save form of a direct-manipulation canvas. The editor and the file are not coupled through serialisation tricks — they share a single grammar, parsed once and pretty-printed back the same way. The result:

1. **Diffable** — every diagram change is a readable text diff. No more "the binary changed" merge conflicts.
2. **Scriptable** — generate `.gnx` from anywhere that can write a string. Templates, fixture builders, codegen targets, ad-hoc shell pipelines.
3. **Reviewable** — pull requests on diagram-bearing documentation no longer require opening a tool.
4. **Authorable in either direction** — type the DSL and watch the canvas update, or draw on the canvas and watch the DSL fall out.

GNX is intentionally small. It describes diagrams, not behaviour; styling, not simulation; geometry, not semantics of what the nodes *do*. A node-graph compute model layered on top — or a circuit, or a state machine — is a downstream concern.

---

## Design Principles

1. **Round-trip first** — every legal document survives `parse → pretty → parse` unchanged. Style auto-extraction and comment preservation honour this invariant.
2. **Style classes over property repetition** — when two or more nodes share the same overlay and ports, they reference a named style. The pretty-printer factors styles out automatically; the parser merges them back in on load.
3. **Inline overrides** — any field declared inside a node body wins over the style. A single deviant fill or port placement does not force a new style.
4. **Comments as document structure** — comments anchor to the following item (file header, settings, node, wire). They survive round-trip and surface on the canvas wherever the host renders them.
5. **One physical unit base per document** — `units` is declared once in `settings`. Every coordinate inside the document is in those units. The renderer scales the page board (intrinsically defined in inches) to match.
6. **Forgiving lexer, strict parser** — whitespace, statement separators, and comment forms are flexible. Keywords, ordering inside a block, and enum spellings are not.
7. **No host-specific extensions** — a `.gnx` document does not depend on `egui_grafica` to be meaningful. Any conformant parser can reproduce the diagram.

---

## File Types

| Extension | Purpose | Contents |
|-----------|---------|----------|
| `.gnx`    | Canvas / Diagram | Settings, optional styles, nodes, wires |

There is exactly one file type. Reusable style libraries are not split into a separate file in the v0.1 spec — every `style` block lives in the document that uses it. A multi-document style import mechanism is a v0.2 candidate.

---

## Lexical Structure

### Encoding

`.gnx` files are UTF-8. The lexer reads `char`s; identifiers and keywords are ASCII; string literals carry arbitrary Unicode.

### Whitespace

Spaces, tabs, and carriage returns are insignificant except as token separators. Newlines act as **statement terminators** inside a block — one statement per line is the canonical form, but a `;` is also accepted (parser-level).

### Comments

Four forms are recognised; all four collapse to the same comment payload and survive the round-trip via anchored `CommentBlock`s.

| Form | Origin | Conventional use |
|------|--------|------------------|
| `// …` | Rust line | Ordinary inline comment |
| `/// …` | Rust outer doc | Attached to the **following** item |
| `//! …` | Rust inner doc | Attached to the **enclosing** block (typically the file header) |
| `# …` | Legacy GNX | Back-compat; treated identically to `//` on parse, emitted as `//` on save |

A comment runs from the marker to end-of-line. There are no block comments. A pretty-printer emits the `//` canonical form; the `///` / `//!` distinctions are accepted on read but not preserved.

### Identifiers

```
identifier := [A-Za-z_] [A-Za-z0-9_]*
```

Identifiers are case-sensitive. Node IDs, port IDs, and style names share the same lexical class.

### Numbers

```
number := [-+]? [0-9]+ ( '.' [0-9]+ )?
```

All numbers are interpreted as `f32` (IEEE 754 single precision). No exponent form, no hex form. Leading sign is permitted.

### Strings

Double-quoted, with C-style escapes:

```
string := '"' ( escape | non-quote-char )* '"'
escape := '\\' ( 'n' | 't' | '"' | '\\' )
```

Strings carry arbitrary Unicode bytes between the quotes. Newlines inside a string must be escaped as `\n`.

### Punctuation

| Token | Role |
|-------|------|
| `{` `}` | Block delimiters |
| `:` | Type / style annotation (`node a : rect : my_style`) and port reference (`a.p1`) |
| `.` | Port reference inside a wire (`a.p1 -> b.p2`) |
| `->` | Wire direction |

### Keywords

Reserved at the top of their respective block contexts. They are not reserved globally — a node ID `style` is legal (though confusing).

| Keyword | Context |
|---------|---------|
| `canvas` | File root |
| `settings` | Inside `canvas` |
| `style` | Inside `canvas` |
| `node` | Inside `canvas` |
| `wire` | Inside `canvas` |
| `text` | Inside `node`, `style` |
| `port` | Inside `node`, `style` |
| `at`, `size`, `rotation` | Node transform |
| `border`, `fill` | Node / style overlay |
| `routing`, `stroke`, `arrow`, `label` | Inside `wire` |
| `grid`, `grid_style`, `dot_size`, `units`, `snap`, `show_grid`, `paper`, `orientation`, `background` | Inside `settings` |
| `value`, `anchor`, `font`, `bold`, `italic`, `color` | Inside `text` |
| `type` | Port data-type tag (Section 10) |
| `untyped`, `bool`, `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `float`, `real`, `eng`, `generic` | Port data-type names |

---

## Top-Level Grammar

```ebnf
document    := comment* "canvas" string "{" canvas_body "}" ;
canvas_body := comment* settings_block
               ( comment* style_block
               | comment* node_block
               | comment* wire_block )* ;
```

The order is:

1. **Exactly one** `settings { … }` block — required, first item.
2. **Zero or more** `style { … }` blocks — appear before nodes by convention; the parser accepts them anywhere after `settings`.
3. **Zero or more** `node { … }` blocks.
4. **Zero or more** `wire { … }` blocks.

A `.gnx` document is anchored by its `canvas "…" { }` root. The string is a user-facing document name and may be empty.

---

## Settings Block

```ebnf
settings_block := "settings" "{" settings_field* "}" ;
settings_field := "grid" number
                | "grid_style" ( "lines" | "dots" )
                | "dot_size" number
                | "units" ( "pixels" | "mils" | "millimeters" | "inches" )
                | "snap" onoff
                | "show_grid" onoff
                | "routing" ( "orthogonal" | "bezier" | "straight" )
                | "paper" string
                | "orientation" string
                | "background" ( "light" | "slate" | "charcoal" | "dark" ) ;
onoff := "on" | "off" | "true" | "false" ;
```

### Semantics

| Field | Meaning |
|-------|---------|
| `grid` | Grid step in **world units**. |
| `grid_style` | `lines` draws faint axis-aligned rulings; `dots` draws a marker at every intersection. |
| `dot_size` | World-unit diameter of dot markers when `grid_style dots`. Ignored for lines. |
| `units` | World-unit interpretation. Drives page-board scaling (1 inch = 1, 1000, 25.4, or 96 world units). |
| `snap` | When `on`, placements and resizes snap to the nearest grid intersection. |
| `show_grid` | Renderer flag — the grid still exists for snapping when `off`. |
| `routing` | Default routing for new wires (`orthogonal`, `bezier`, `straight`). |
| `paper` | Named paper size — `"Letter"`, `"Legal"`, `"Tabloid"`, `"A5"`, `"A4"`, `"A3"`, `"ANSI C"`, `"ANSI D"`. Omit (or pass `none` upstream) to disable the page board. |
| `orientation` | `"portrait"` or `"landscape"`. Only meaningful with a `paper`. |
| `background` | Canvas tone — `light`, `slate`, `charcoal`, `dark`. |

---

## Style Block

```ebnf
style_block := "style" identifier "{" style_field* "}" ;
style_field := border_field | fill_field | text_block | port_field ;
```

A style carries any subset of the overlay surface (border, fill, text) plus a list of ports. Every field is optional — a style that contains only `fill` and `port` lines is perfectly legal.

### Inheritance Semantics

When a node references a style with `node x : rect : my_style { … }`:

1. The named style is looked up in the document's style table.
2. Its fields **pre-seed** the node — `border`, `fill`, `text` become the node's overlay starting state; the style's `port` list is copied to the node verbatim.
3. The node body is then parsed top-to-bottom. Each inline field **overrides** the style's value.
4. Inline `port` declarations whose ID matches a style port replace that port in place; new IDs append.

A style with the same content but a different name is a different style. Names are compared as exact ASCII strings.

### Auto-extraction (Printer)

When a `Scene` is pretty-printed:

1. Every node's `(overlay, ports)` tuple is hashed against the other nodes.
2. Any tuple shared by **two or more nodes** is factored into a `style` block named after the first node that carried an explicit `style_ref`, or `s0`, `s1`, … if none did.
3. Each member node emits `node id : kind : stylename { at … size … rotation … }` and **omits** every field equal to the style.
4. Nodes whose tuple is unique inline the full field set as before.

The printer also preserves any `style` block parsed but unreferenced by current nodes (e.g., a library style the user authored), so a round-trip never loses a declaration.

---

## Node Block

```ebnf
node_block := "node" identifier ":" node_kind ( ":" identifier )? "{" node_field* "}" ;
node_kind  := "rect" | "circle" | "ellipse" | "parallelogram" ;
node_field := "at" number number
            | "size" number number
            | "rotation" number
            | border_field | fill_field | text_block | port_field ;
```

### Fields

| Field | Description |
|-------|-------------|
| `at x y` | World-unit top-left of the node's bounding box. |
| `size w h` | World-unit width and height. Resize handles operate on this. |
| `rotation deg` | Counter-clockwise rotation around the bounding-box centre, in degrees. |
| `border style width "#color"` | Outline: `solid`, `dashed`, or `dotted`; width in world units; `#RRGGBB` colour. |
| `fill "#color" alpha` | Body fill: colour and alpha in `[0.0, 1.0]`. |
| `text { … }` | Optional label block. See below. |
| `port kind name anchor [args] [type "string"]` | One port per line. See [Port Block](#port-block). |

### Kinds

| Kind | Visible contour | Bbox interpretation |
|------|-----------------|---------------------|
| `rect` | Axis-aligned rectangle. | Direct. |
| `circle` | Ellipse inscribed in the bbox (square bbox → circle). | Radial projection. |
| `ellipse` | Ellipse inscribed in the bbox. | Radial projection. |
| `parallelogram` | Right-leaning parallelogram. Top edge inset by `h × 0.25`. | Slanted-edge port projection. |

Port positions are computed against the **visible contour**, not the bounding box. A `circle` node's `east 0.5` port sits on the curve at the 3-o'clock point, not on the bbox corner.

### Text Block

```ebnf
text_block := "text" "{" text_field* "}" ;
text_field := "value" string
            | "anchor" text_anchor
            | "font" string number
            | "bold" onoff
            | "italic" onoff
            | "color" string ;
text_anchor := "center" | "top_center" | "bottom_center"
             | "left" | "right"
             | "top_left" | "top_right" | "bottom_left" | "bottom_right" ;
```

`font ""` means "use the host's default proportional family." `value` may contain `\n` for multi-line labels.

### Port Block

A port is a connection point on a node. It carries three pieces of information: a *direction* (in / out / bidir / untyped), a *placement* (where on the node's contour it sits), and an optional *data type* (what shape of value flows across a wire attached to it).

#### Grammar

```ebnf
port_field  := "port" port_dir identifier port_anchor ( "type" port_type )? ;
port_dir    := "in" | "out" | "bidir" | "untyped" ;
port_anchor := "north" number
             | "south" number
             | "east"  number
             | "west"  number
             | "free"  number number ;
port_type   := "untyped"
             | "bool"
             | "int8" | "int16" | "int32" | "int64"
             | "uint8" | "uint16" | "uint32" | "uint64"
             | "float" | "real"
             | "eng"     ( "<" string ">" )?
             | "generic" "<" identifier ">" ;
```

`port_dir` and `port_type` are **orthogonal**. The direction describes whether data flows in, out, both, or is unconstrained. The type describes the shape of that data. Either can be `untyped`, and the two `untyped` values are unrelated — the direction `untyped` means "I don't model direction"; the type `untyped` means "I don't model data".

#### Placement

`north 0.5` means "midpoint of the top edge"; `east 0.0` is the top-right corner-ish; `west 1.0` is the bottom-left corner-ish (parametric along the face). `free fx fy` is normalised body-local coordinates — `free 0.5 0.5` is dead-centre regardless of shape.

#### Direction

| Direction | Meaning |
|-----------|---------|
| `in`      | Inbound port — data flows from the wire into the node. |
| `out`     | Outbound port — data flows from the node onto the wire. |
| `bidir`   | Bidirectional — data flows both ways (bus-like). |
| `untyped` | Direction is not modelled. The default for block-diagram-style placements where the wires document structure rather than data flow. |

#### Data Types

The optional `type` field tags a port with a data type. Wires connecting two typed ports are validated for type compatibility at connect time (host-level concern). Untyped ports skip validation entirely.

**Primitive types:**

| Type | Meaning |
|------|---------|
| `untyped` | No type discipline. The implicit type when the `type` field is absent. Compatible with every other type at the wire level. |
| `bool` | Single boolean — `true` or `false`. |
| `int8`, `int16`, `int32`, `int64` | Signed two's-complement integer of the named width. |
| `uint8`, `uint16`, `uint32`, `uint64` | Unsigned integer of the named width. |
| `float` | IEEE 754 single-precision floating point (32-bit). |
| `real` | IEEE 754 double-precision floating point (64-bit). Named `real` rather than `float64` to match the engineering / HDL convention. |

**Engineering quantities:**

| Form | Meaning |
|------|---------|
| `eng` | Dimensioned real number with no declared unit. Useful when the unit is documented elsewhere or implied by context. |
| `eng<"unit">` | Dimensioned real number with a unit string. The string is opaque — `"m/s"`, `"Pa"`, `"degC"`, `"rad/s^2"` — and the host validates unit compatibility however it chooses. |

`eng<"K">` and `eng<"degC">` are distinct types — a wire between them is a unit mismatch, even though both represent temperature. Hosts may layer conversion semantics on top; the spec only requires the strings be compared as strings.

**Generic types:**

`generic<T>` declares a type variable. The identifier `T` (or any other) acts as a placeholder; the host *unifies* it across a wire at connect time. A passthrough node with `port in i west 0.5 type generic<T>` and `port out o east 0.5 type generic<T>` accepts any input type, and the output is constrained to be the same type as the input.

`generic<T>` is **not** a synonym for `untyped`. An `untyped` port accepts anything with no record of what was attached. A `generic<T>` port records the inferred type and propagates it through the node.

v0.1 hosts may treat `generic<T>` as `untyped` for the purposes of wire validation (i.e., skip unification). The syntax is reserved so that future versions can introduce type inference without a grammar change.

#### Example

```gnx
node adc : rect : default {
  at 0 0
  size 60 60
  rotation 0

  // Block-diagram defaults — direction and type both untyped.
  port untyped n north 0.5
  port untyped s south 0.5

  // Typed signal flow.
  port in  vref west 0.3 type eng<"V">
  port in  ain  west 0.7 type eng<"V">
  port out code east 0.5 type uint16
}

node passthrough : rect : default {
  at 200 0
  size 60 60
  rotation 0

  // Both ports share the same type variable T — the host can
  // infer T from whichever side connects first.
  port in  i west 0.5 type generic<T>
  port out o east 0.5 type generic<T>
}
```

---

## Wire Block

```ebnf
wire_block := "wire" identifier endpoint "->" endpoint "{" wire_field* "}" ;
endpoint   := identifier "." identifier ;
wire_field := "routing" wire_routing
            | "stroke" string number line_style
            | "arrow" arrow_head arrow_head
            | "label" string ;

wire_routing := "orthogonal" | "bezier" | "straight" | manual_routing ;
manual_routing := "manual" "[" ( number number )* "]" ;

line_style := "solid" | "dashed" | "dotted" ;
arrow_head := "none" | "arrow" | "triangle" | "diamond" | "circle" ;
```

### Endpoints

`a.p1 -> b.p2` is a directed connection from node `a`'s port `p1` to node `b`'s port `p2`. The direction influences arrow placement and the routing algorithm's exit logic.

Free-floating endpoints (dangling wires) exist as an in-memory editor state but are **not** representable in `.gnx`. Saving a scene with free ends silently drops them.

### Routing

| Routing | Behaviour |
|---------|-----------|
| `orthogonal` | Auto-routed; one bend, axis-aligned, port-direction-aware stubs. |
| `bezier` | Auto-routed; port-direction-aware control handles. |
| `straight` | Direct line between endpoints. |
| `manual[ x0 y0  x1 y1 … ]` | Hand-laid waypoints in world units. The router connects endpoints through each waypoint in order. |

---

## Coordinate Model

GNX has **one coordinate system**: world units, axis-aligned, y-down. Every position and size in a `.gnx` file is in these units. There is no transform stack and no nested coordinate frames in v0.1.

The `units` setting is purely cosmetic — it controls the suffix in the inspector and the scaling of the engineering-drawing page board. Editing a file from `mils` to `mm` does **not** rescale numbers. If a sheet authored in mils is reinterpreted as mm, every shape is 25.4× larger.

A future v0.2 may add a `world_unit` declaration that the parser uses to physically rescale on unit change.

---

## Page Board

When `settings.paper` is set, the renderer draws an engineering-drawing sheet anchored at world origin:

- **Sheet** — paper-sized rectangle at `(0, 0)`. Outlined only; content sits on top.
- **Frame** — drawing border inset by 0.5 inch on every side.
- **Zone markers** — `A`–`H` along the horizontal frame edges, `1`–`6` along the vertical, bottom-up numbering (so cell `A1` is bottom-left).
- **Title block** — optional bottom-right block carrying TITLE, COMPANY, DWG NO + REV, DATE + SHEET. Stored on the host `Scene`; not yet representable in the DSL itself.

The page board is a host-level feature; the `.gnx` declaration is the `paper` and `orientation` settings. Title-block field content is currently a host-side editor concern.

---

## Round-Trip Invariant

For every valid `Scene` value the implementation produces, the following must hold:

```
parse(pretty_document(&doc)) == doc.scene
```

Equality compares every field: nodes, edges, settings, style-extracted overlays, port lists, comment anchors. The current test suite enforces this with a fixture covering all node kinds, all routing variants, comments at every anchor, and shared-style auto-extraction.

The only documented loss-of-information cases are:

1. **Comment style** — `//`, `///`, `//!`, and `#` all collapse to `//` on emit.
2. **Free-ended wires** — dropped on save.
3. **Title-block fields** — host-only in v0.1.

---

## File Layout Example

A minimal but representative document:

```gnx
//! grafica-ide system diagram
//! Version 1, James Bonanno

canvas "Block Diagram" {
  settings {
    grid 10
    grid_style lines
    dot_size 2
    units mils
    snap on
    show_grid on
    routing orthogonal
    background slate
  }

  // Shared widget style — auto-extracted by the printer when ≥ 2 nodes share it.
  style s0 {
    border solid 2 "#1F2937"
    fill "#DBEAFE" 0.9
    text {
      value "Text"
      anchor center
      font "" 12
      bold off
      italic off
      color "#111827"
    }
    port untyped n north 0.5
    port untyped e east  0.5
    port untyped s south 0.5
    port untyped w west  0.5
  }

  /// The system's primary input stage.
  node alpha : rect : s0 {
    at 850 175
    size 80 50
    rotation 0
    text {
      value "Alpha"
      anchor center
      font "" 12
      bold off
      italic off
      color "#111827"
    }
  }

  node beta : circle : s0 {
    at 1040 240
    size 80 80
    rotation 0
  }

  wire w0 alpha.e -> beta.w {
    routing bezier
    stroke "#374151" 1.5 solid
    arrow arrow none
  }
}
```

This document:

- Carries inner-doc header comments anchored to the canvas block.
- Declares one shared style with full overlay + four quadrant ports.
- Inlines a per-node `text` override on `alpha` (its label differs from the style's "Text" default).
- Connects `alpha.e` to `beta.w` with a bezier wire.

---

## Conformance

A conformant GNX implementation must:

1. Accept every legal document under this specification.
2. Reject documents that violate the grammar with a useful, line-numbered error.
3. Preserve every field of a `Scene` round-trip through `parse → pretty → parse`.
4. Recognise all four comment forms on read.
5. Compute port positions on the visible contour, not the bbox, for `circle`, `ellipse`, and `parallelogram` nodes.

A conformant implementation **may**:

- Emit any comment form on write (the reference implementation emits `//`).
- Reorder nodes, wires, or styles within their declaration sections, provided the round-trip identity holds.
- Auto-extract styles from shared overlays on write.
- Reject extensions not listed in this document (forward compatibility is not guaranteed in v0.1).

---

## Versioning

| Version | Date | Summary |
|---------|------|---------|
| 0.1 | 2026-05-27 | Initial spec — canvas, settings, style, node, wire, four comment forms, page board declaration, round-trip invariant. |

---

## Reference Implementation

The reference parser and pretty-printer live in `crates/egui_grafica/src/lang.rs` in the [`saturn77/egui_mobius`](https://github.com/saturn77/egui_mobius) repository. The model types referenced throughout this spec are defined in `crates/egui_grafica/src/model.rs`.

The syntect grammar for editor highlighting lives at `crates/egui_quill/syntaxes/Graphica.sublime-syntax` (same repo).

This spec lives in the [`Atlantix-EDA/grafica-ide`](https://github.com/Atlantix-EDA/grafica-ide) repository under `docs/lang-spec/` — markdown source plus a Typst rendering and build script for the PDF.
