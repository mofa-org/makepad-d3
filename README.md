# Makepad D3

A D3.js-compatible data visualization library for [Makepad](https://github.com/makepad/makepad)'s GPU-accelerated rendering.

## Quick Run

```bash
git clone https://github.com/mofa-org/makepad-d3.git
cd makepad-d3
cargo run --example chart_zoo
```

This launches the Chart Zoo with 40+ interactive chart types including Bar, Line, Pie, Sankey, Force Graph, Treemap, Globe Map, and more.

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
