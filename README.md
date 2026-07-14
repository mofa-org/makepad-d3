# Makepad D3

A D3.js-compatible data visualization library for [Makepad](https://github.com/makepad/makepad)'s GPU-accelerated rendering.

> **Makepad 2.0 / Splash status (2026-07):** This library now targets
> **Makepad 2.0**, whose Script/**Splash** runtime replaced the old Live
> system (`live_design!`). makepad-d3 registers a scriptable **`d3.*`
> widget namespace** into the Splash VM, so charts can be written directly
> in Splash DSL — including inside sandboxed `runsplash`-style mini apps
> via the `d3.Splash` host widget. Design + migration record:
> [`docs/SPLASH_INTEGRATION_DESIGN.md`](docs/SPLASH_INTEGRATION_DESIGN.md).

## Quick Run

Makepad 2.0 is consumed as a **sibling path dependency** (its repo vendors a
pre-2.0 crate copy under `old/`, which makes git dependencies ambiguous):

```bash
git clone https://github.com/makepad/makepad.git --branch dev   # sibling checkout
git clone https://github.com/mofa-org/makepad-d3.git
cd makepad-d3
cargo run --example splash_demo
```

This opens a dashboard whose entire UI is Splash DSL: `d3.BarChart`,
`d3.PieChart`, `d3.LineChart`, `d3.ScatterChart` with declarative `data:`,
script-driven `ui.chart.set_data(...)` buttons, `on_click`/`on_hover`
closures, and a sandboxed `d3.Splash` isolate running its own splash body.

## Using d3 charts from Splash DSL

### 1. Register the `d3.*` namespace (one line of Rust)

```rust
use makepad_widgets::*;

app_main!(App);

#[derive(Script, ScriptHook)]
pub struct App {
    #[live] ui: WidgetRef,
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        makepad_d3::script_mod(vm);      // <- adds the d3.* namespace
        self::script_mod(vm)             // your app's script_mod! block
    }
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
```

`makepad_d3::script_mod` also injects `d3` into the widgets prelude, so any
scope that starts with `use mod.prelude.widgets.*` (which includes every
`Splash`-hosted body) can write `d3.BarChart{...}` with no extra imports.

### 2. The chart widgets

All charts share the same contract — declarative props, script methods, and
event closures:

| Widget | `data:` shape | Extra props |
|---|---|---|
| `d3.BarChart` | `[v v v ...]` + `labels: ["a" "b" ...]` | `bar_color`, `hover_color` |
| `d3.LineChart` | numbers, `[[x y], ...]`, or `[{x:.. y:..}, ...]` | `line_color`, `line_width`, `dot_color`, `dot_radius` |
| `d3.AreaChart` | same as LineChart | `line_color`, `line_width`, `fill_color` |
| `d3.ScatterChart` | same as LineChart | `dot_color`, `hover_color`, `dot_radius` |
| `d3.PieChart` | `[v v v ...]` + `labels:` | `inner_radius` (0–0.95, donut hole; slices use d3 Category10) |

Shared props: `width`/`height`, `plot_margin: Inset{...}`, `grid_color`,
`label_color`. A chart with no `data:` renders a small demo dataset.

**Script methods** (call on a `:=` id via `ui.`):

| Method | Effect |
|---|---|
| `ui.chart.set_data(values)` | replace the data, refit, redraw |
| `ui.chart.set_labels(labels)` | replace category/slice labels |
| `ui.chart.set_domain(min, max)` | pin the y-domain (disables auto-fit) |
| `ui.chart.data()` | read the values back as a script array |

**Events** — closures receive the mark index:

```splash
d3.BarChart{
    on_click: |i| ui.status.set_text("clicked bar " + i)
    on_hover: |i| ui.status.set_text("hovering " + i)
}
```

### 3. A complete Splash snippet

```splash
View{ width: Fill height: Fit flow: Down spacing: 10
    chart := d3.BarChart{
        height: 300
        data: [30 86 168 281 303 365]
        labels: ["Jan" "Feb" "Mar" "Apr" "May" "Jun"]
        on_click: |i| ui.status.set_text("clicked bar " + i)
    }
    View{ width: Fill height: Fit flow: Right spacing: 10
        Button{text: "Update" on_click: || ui.chart.set_data([2 7 1 8 2 8])}
        Button{text: "Pin 0..400" on_click: || ui.chart.set_domain(0, 400)}
        status := Label{text: "-"}
    }
    // x/y charts take pairs or objects — note the commas between pairs
    d3.LineChart{ height: 260 data: [[0 5], [1 18], [2 12], [3 40]] }
}
```

### 4. Sandboxed splash apps (`d3.Splash`)

Stock `runsplash` sandboxes only see the built-in widgets. `d3.Splash` is a
drop-in host that evaluates a body string in an **isolated script VM with
`d3.*` registered** — feed it from Rust exactly like the Markdown widget
streams ```` ```runsplash ```` fences:

```splash
host := d3.Splash{ width: Fill height: Fit }
```

```rust
// e.g. in MatchEvent::handle_startup, or as a markdown code-block template
if let Some(mut host) = self.ui.widget(cx, ids!(host))
    .borrow_mut::<makepad_d3::splash::D3Splash>()
{
    host.set_text(cx, "flow: Right spacing: 12 \
        d3.PieChart{width: 240 height: 180 data: [4 3 2 1]} \
        d3.AreaChart{width: Fill height: 180 data: [3 7 4 9 6 12 8]}");
}
```

Sandbox notes: inline `on_click` handlers can use `ui.`, but body-level
helper `fn`s cannot (no `ui` global — needs a makepad-side hook, see design
doc §8.3/§14); networking is off; the body is prefixed with
`View{height:Fit, ` so it starts with properties/children of that view.

### 5. Gotchas

- **Nested array literals need commas**: `[[0 5], [1 18]]` — adjacent
  `[..] [..]` parses as indexing.
- **Put fixed-size charts before `Fill` siblings** in a `flow: Right` row,
  otherwise the deferred `Fill` child resolves after the fixed one drew and
  they overlap.
- Known upstream gap: `DrawText::draw_abs` (axis/slice labels) does not
  render on the current makepad dev tip — the built-in `mod.widgets` charts
  have the same behavior; grids, marks, and `Label` widgets are unaffected.
- The pre-2.0 Chart Zoo (40+ charts) is the porting backlog and does not
  compile against 2.0 yet (`docs/SPLASH_INTEGRATION_DESIGN.md` §11 Phase 4).

## Features

- **Scales**: Linear, Log, Pow, Symlog, Time, Category, Band, Quantize, Quantile, Threshold
- **Shapes**: Line, Area, Arc, Pie, Stack generators with 7 curve interpolations
- **Layouts**: Force-directed graphs, Treemap, Tree, Circle packing
- **Geographic**: Mercator, Orthographic, Equirectangular, Albers projections
- **Colors**: RGB, HSL, LAB, HCL color spaces with perceptual interpolation
- **Interactions**: Zoom, Brush, Tooltip behaviors
- **Components**: Legend, Crosshair, Annotations, Reference lines

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
makepad-d3 = { git = "https://github.com/mofa-org/makepad-d3.git" }
```

## Quick Start

```rust
use makepad_d3::prelude::*;

// Create chart data
let data = ChartData::new()
    .with_labels(vec!["Jan", "Feb", "Mar", "Apr"])
    .add_dataset(
        Dataset::new("Revenue")
            .with_data(vec![100.0, 200.0, 150.0, 300.0])
            .with_hex_color(0x4285F4)
    );

// Create scales
let x_scale = CategoryScale::new()
    .with_labels(data.labels.clone())
    .with_range(50.0, 550.0);

let y_scale = LinearScale::new()
    .with_domain(0.0, 300.0)
    .with_range(350.0, 50.0);  // Inverted for screen coordinates
```

## Module Overview

| Module | Description |
|--------|-------------|
| `data` | ChartData, Dataset, DataPoint structures |
| `scale` | Data-to-pixel mapping functions |
| `axis` | Axis generation, ticks, labels, formatting |
| `shape` | Path, Line, Area, Arc, Pie, Stack generators |
| `shape::curve` | Linear, Catmull-Rom, Natural, Monotone, Basis, Cardinal, Step |
| `color` | Color spaces, interpolation, schemes |
| `layout::force` | Force-directed graph simulation |
| `layout::hierarchy` | Tree, Treemap, Pack layouts |
| `geo` | Geographic projections, GeoJSON support |
| `interaction` | Zoom, Brush, Tooltip behaviors |
| `component` | Legend, Crosshair, Annotation, ReferenceLine |

## Chart Zoo Example

Run the comprehensive chart gallery with 40+ chart types:

```bash
cargo run --example chart_zoo
```

### Available Charts

| Category | Charts |
|----------|--------|
| **Basic** | Bar, Line, Area, Scatter, Pie, Donut |
| **Statistical** | Histogram, Box Plot, Violin, Beeswarm |
| **Hierarchical** | Treemap, Sunburst, Circle Pack, Tree |
| **Network** | Force Graph, Sankey, Chord Diagram, Arc Diagram |
| **Time Series** | Candlestick, Horizon, Calendar |
| **Geographic** | Globe Map, Choropleth |
| **Specialized** | Heatmap, Contour, Hexbin, Radar, Parallel Coordinates |

## Scale Types

### Linear Scale

```rust
let scale = LinearScale::new()
    .with_domain(0.0, 100.0)
    .with_range(0.0, 500.0)
    .with_nice(true)        // Round to nice values
    .with_clamp(true);      // Clamp out-of-domain values

let pixel = scale.scale(50.0);  // -> 250.0
```

### Category Scale

```rust
let scale = CategoryScale::new()
    .with_labels(vec!["A", "B", "C", "D"])
    .with_range(0.0, 400.0)
    .with_padding(0.1);     // Gap between bands

let x = scale.scale_index(1);   // -> position for "B"
let width = scale.bandwidth();  // -> width of each band
```

### Time Scale

```rust
let scale = TimeScale::new()
    .with_domain(start_date, end_date)
    .with_range(0.0, 800.0);

let ticks = scale.ticks(&TickOptions::default());
```

## Curve Interpolation

```rust
use makepad_d3::shape::curve::*;

// Straight lines
let linear = LinearCurve;

// Smooth curves passing through all points
let catmull = CatmullRomCurve::new().with_alpha(0.5);

// Natural cubic spline
let natural = NaturalCurve;

// Preserves monotonicity (no overshoots)
let monotone = MonotoneCurve::x();

// Step function
let step = StepCurve::after();
```

## Force-Directed Layout

```rust
use makepad_d3::layout::force::*;

let mut simulation = ForceSimulation::new()
    .with_nodes(nodes)
    .with_links(links)
    .add_force("charge", ManyBodyForce::new().with_strength(-30.0))
    .add_force("link", LinkForce::new().with_distance(50.0))
    .add_force("center", CenterForce::new(width / 2.0, height / 2.0));

// Run simulation
while simulation.tick() {
    // Update positions
}
```

## Sankey Diagram

The Sankey implementation follows D3's algorithm with:

- **Node values**: Sink = incoming, Source = outgoing, Intermediate = max(in, out)
- **Global scale factor (ky)**: Computed from densest layer
- **SankeyJustify**: Smart sink placement based on source layer count
- **Relaxation**: D3-style iterative position optimization

## Color Interpolation

```rust
use makepad_d3::color::*;

// Perceptually uniform interpolation in LAB space
let color = lerp_color(
    rgba(0.2, 0.4, 0.8, 1.0),
    rgba(0.9, 0.3, 0.3, 1.0),
    0.5
);

// Sequential color scale
let scale = SequentialScale::new(vec![
    rgba(1.0, 1.0, 1.0, 1.0),
    rgba(0.0, 0.0, 1.0, 1.0),
]);
let color = scale.get_color(0.7);
```

## Geographic Projections

```rust
use makepad_d3::geo::*;

let projection = MercatorProjection::new()
    .with_center(-98.0, 39.0)
    .with_scale(1000.0)
    .with_translate(width / 2.0, height / 2.0);

// Project coordinates
if let Some((x, y)) = projection.project(lon, lat) {
    // Draw at (x, y)
}

// Render GeoJSON
let path = GeoPath::new(projection);
let segments = path.render(&geometry);
```

## License

MIT

## References

- [D3.js](https://d3js.org/) - Original JavaScript library
- [Makepad](https://github.com/makepad/makepad) - GPU UI framework
- [D3 Sankey](https://github.com/d3/d3-sankey) - Sankey layout algorithm
- [D3 Force](https://github.com/d3/d3-force) - Force simulation
