# Changelog

## 0.2.0 — 2026-07-14

### Makepad 2.0 "Splash" migration

- Target **Makepad 2.0** (Script/Splash runtime; the Live system /
  `live_design!` was removed upstream). Consumed as a sibling path
  dependency `../makepad/widgets` — a git dependency is not possible while
  the makepad repo vendors a pre-2.0 crate copy under `old/`.
- `src/render3d/draw.rs` shaders ported to `script_mod!` / `script_shader`.

### The `d3.*` Splash namespace (new)

`makepad_d3::script_mod(vm)` registers **21 chart widgets** usable directly
from Splash DSL, with declarative `data:` props, `set_data(...)`-style
script methods, and `on_click`/`on_hover` script-closure events:

- Basic: `BarChart`, `LineChart`, `AreaChart`, `ScatterChart`, `PieChart`
- Statistical: `Histogram`, `Heatmap`, `RadarChart`, `BoxPlot`
- Hierarchies: `Treemap`, `Sunburst`, `CirclePack`, `TreeChart`
- Flows: `Sankey`, `ChordDiagram`, `ArcDiagram`
- Networks/density: `ForceGraph`, `Hexbin`, `Ridgeline`, `Horizon`,
  `Contour`
- Geographic: `Globe` (draggable orthographic)
- 3D (drag to orbit): `Surface3D`, `Scatter3D`, `Bar3D`
- Sandbox host: `d3.Splash` — evaluates runtime Splash bodies in an
  isolated VM with `d3.*` registered (runsplash-style)

### Docs & examples

- `examples/splash_demo`: full-gallery app, entire UI in Splash DSL.
- `docs/SPLASH_INTEGRATION_DESIGN.md`: architecture + implementation record.
- `docs/d3-splash.md`: Splash authoring guide for all widgets (for humans
  and AI generation).

### Known limitations

- Axis/label text is invisible pending an upstream makepad fix
  (`DrawText::draw_abs` does not render on the current dev tip; makepad's
  built-in charts show the same behavior).
- The pre-2.0 `chart_zoo` example is retired (superseded by the `d3.*`
  gallery); its remaining exotic variants live on as porting references.

## 0.1.0 — 2026-01

- Initial release: d3-compatible core (scales, axes, shapes, colors,
  layouts, geo, 3D math) + Live-system chart widgets (`chart_zoo`).
