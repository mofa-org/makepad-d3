# Makepad to D3.js Gap Analysis

> Comprehensive analysis of what's needed to support D3.js-like data visualization capabilities in Makepad, including evaluation of existing libraries (makepad-chart, Linebender ecosystem).

**Analysis Date**: January 2026
**Analyst**: Claude (Anthropic)

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [D3.js Overview](#d3js-overview)
3. [Makepad Current Capabilities](#makepad-current-capabilities)
4. [Gap Analysis by Module](#gap-analysis-by-module)
5. [makepad-chart Library Analysis](#makepad-chart-library-analysis)
6. [Linebender Ecosystem Analysis](#linebender-ecosystem-analysis)
7. [Combined Gap Assessment](#combined-gap-assessment)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Technical Recommendations](#technical-recommendations)
10. [Appendix: API Comparisons](#appendix-api-comparisons)

---

## Executive Summary

### The Challenge

D3.js is a mature JavaScript library with 15+ years of development, providing comprehensive abstractions for data visualization. Makepad is a GPU-accelerated Rust UI framework with excellent low-level rendering but minimal high-level data visualization abstractions.

### Key Findings

| Aspect | Status | Notes |
|--------|--------|-------|
| **Core Rendering** | ✅ Strong | Makepad's GPU shaders excel at rendering |
| **Animation System** | ✅ Complete | 30+ easing functions, comparable to D3 |
| **Scale System** | 🟡 Partial | makepad-chart provides Linear/Category scales |
| **Axis Generation** | 🟠 Basic | Implicit rendering only, no dedicated component |
| **Shape Generators** | 🟡 Partial | Basic shapes exist, data-driven generators missing |
| **Interactions** | 🔴 Limited | Only hover effects; no zoom/pan/brush |
| **Advanced Layouts** | 🔴 Missing | No force simulation, hierarchy, or geographic |

### Bottom Line

- **makepad-chart** closes ~50-60% of the practical gap for standard business charts
- **Linebender** (Kurbo, Peniko, Color) provides excellent primitives for building the rest
- **Remaining effort**: ~10,000-15,000 lines of Rust for full D3 parity

---

## D3.js Overview

### What is D3.js?

D3 (Data-Driven Documents) is a JavaScript library for producing dynamic, interactive data visualizations in web browsers. It uses SVG, Canvas, and HTML standards.

**Key Statistics:**
- 112,000+ GitHub stars
- 463,000+ dependent projects
- 30+ interconnected modules
- 15+ years of development

### D3 Module Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         D3.js Ecosystem                          │
├─────────────────────────────────────────────────────────────────┤
│  DATA PROCESSING                                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ d3-array │ │d3-random │ │d3-format │ │ d3-time  │           │
│  │ min/max  │ │ random   │ │ numbers  │ │  dates   │           │
│  │ group    │ │ normal   │ │ locale   │ │ parsing  │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
├─────────────────────────────────────────────────────────────────┤
│  SCALES & AXES                                                   │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                        │
│  │ d3-scale │ │ d3-axis  │ │d3-scale- │                        │
│  │ linear   │ │  top     │ │chromatic │                        │
│  │ log/pow  │ │  bottom  │ │ viridis  │                        │
│  │ time     │ │  left    │ │ category │                        │
│  │ ordinal  │ │  right   │ │ schemes  │                        │
│  └──────────┘ └──────────┘ └──────────┘                        │
├─────────────────────────────────────────────────────────────────┤
│  SHAPES & RENDERING                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ d3-shape │ │ d3-path  │ │ d3-chord │ │d3-contour│           │
│  │ line     │ │  moveTo  │ │  matrix  │ │ density  │           │
│  │ area     │ │  lineTo  │ │  ribbon  │ │ contour  │           │
│  │ arc/pie  │ │  curve   │ │  groups  │ │          │           │
│  │ stack    │ │  close   │ │          │ │          │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
├─────────────────────────────────────────────────────────────────┤
│  LAYOUTS                                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                        │
│  │d3-hierarchy│ │ d3-force │ │  d3-geo  │                        │
│  │ tree     │ │ simulation│ │projections│                        │
│  │ treemap  │ │ collide  │ │  paths   │                        │
│  │ pack     │ │ link     │ │ shapes   │                        │
│  │ partition│ │ many-body│ │ streams  │                        │
│  └──────────┘ └──────────┘ └──────────┘                        │
├─────────────────────────────────────────────────────────────────┤
│  INTERACTION                                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │ d3-zoom  │ │ d3-brush │ │ d3-drag  │ │d3-dispatch│          │
│  │ pan      │ │ 1D/2D    │ │ drag     │ │ events   │           │
│  │ scale    │ │ selection│ │ drop     │ │ custom   │           │
│  │ constrain│ │ extent   │ │          │ │          │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
├─────────────────────────────────────────────────────────────────┤
│  ANIMATION & COLOR                                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │d3-transition│ │ d3-ease │ │d3-interpolate│ │ d3-color │      │
│  │ duration │ │ linear   │ │  number  │ │  rgb     │           │
│  │ delay    │ │ cubic    │ │  color   │ │  hsl     │           │
│  │ attr     │ │ elastic  │ │  string  │ │  lab     │           │
│  │ style    │ │ bounce   │ │  zoom    │ │  hcl     │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

### D3 Core Concepts

#### 1. Scales (Domain → Range Mapping)

```javascript
// Linear scale: continuous → continuous
const x = d3.scaleLinear()
    .domain([0, 100])      // Input: data values
    .range([0, 800]);      // Output: pixel positions

x(50);  // Returns 400

// Band scale: discrete → continuous (for bar charts)
const x = d3.scaleBand()
    .domain(['A', 'B', 'C'])
    .range([0, 300])
    .padding(0.1);

x('B');           // Returns pixel position
x.bandwidth();    // Returns bar width
```

#### 2. Axes (Visual representation of scales)

```javascript
const xAxis = d3.axisBottom(xScale)
    .ticks(10)
    .tickFormat(d3.format(',.0f'));

svg.append('g')
    .attr('transform', `translate(0, ${height})`)
    .call(xAxis);
```

#### 3. Shape Generators

```javascript
// Line generator
const line = d3.line()
    .x(d => xScale(d.date))
    .y(d => yScale(d.value))
    .curve(d3.curveCatmullRom);

path.attr('d', line(data));

// Arc generator (for pie charts)
const arc = d3.arc()
    .innerRadius(0)
    .outerRadius(100);

const pie = d3.pie().value(d => d.value);
```

#### 4. Transitions

```javascript
selection.transition()
    .duration(750)
    .delay(100)
    .ease(d3.easeCubicInOut)
    .attr('x', d => xScale(d.value));
```

---

## Makepad Current Capabilities

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Makepad Framework                           │
├─────────────────────────────────────────────────────────────────┤
│  PLATFORM LAYER                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Metal (macOS/iOS) | Vulkan (Linux/Android) | WebGL (Web) │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  DRAW LAYER                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │DrawQuad  │ │DrawLine  │ │DrawText  │ │ DrawIcon │           │
│  │rectangles│ │ lines    │ │SDF fonts │ │ sprites  │           │
│  │gradients │ │ bezier   │ │ glyphs   │ │ atlas    │           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
├─────────────────────────────────────────────────────────────────┤
│  VECTOR LAYER (draw/vector/)                                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                        │
│  │   Path   │ │ Geometry │ │  Bender  │                        │
│  │ MoveTo   │ │   Arc    │ │ tessellate│                        │
│  │ LineTo   │ │  Line    │ │  stroke  │                        │
│  │ CurveTo  │ │ Bezier   │ │  clip    │                        │
│  └──────────┘ └──────────┘ └──────────┘                        │
├─────────────────────────────────────────────────────────────────┤
│  WIDGET LAYER                                                    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │  Button  │ │  Slider  │ │  Label   │ │   View   │           │
│  │ CheckBox │ │ TextInput│ │  Image   │ │  Scroll  │           │
│  │ DropDown │ │  List    │ │   SVG    │ │  Splitter│           │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │
├─────────────────────────────────────────────────────────────────┤
│  ANIMATION LAYER                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Animator: 30+ easing functions, keyframes, state machine │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### Drawing Primitives

#### DrawQuad
```rust
// Basic rectangle with shader support
pub struct DrawQuad {
    pub rect_pos: Vec2,
    pub rect_size: Vec2,
    pub draw_clip: Vec4,
}
```

#### DrawLine
```rust
// Line with Bezier support
impl DrawLine {
    pub fn draw_line_abs(&mut self, cx: &mut Cx2d, p1: DVec2, p2: DVec2);
    pub fn draw_bezier_abs(&mut self, cx: &mut Cx2d, p1: DVec2, cp: DVec2, p2: DVec2, steps: usize);
}
```

#### Path Primitives
```rust
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    ArcTo(Point, radius, x_rotation, large_arc, sweep),
    QuadraticTo(control, end),
    CubicTo(cp1, cp2, end),
    Close,
}
```

### Animation System

Makepad's animator provides 30+ easing functions:

```rust
pub enum Ease {
    Linear,
    // Quadratic
    InQuad, OutQuad, InOutQuad,
    // Cubic
    InCubic, OutCubic, InOutCubic,
    // Quartic
    InQuart, OutQuart, InOutQuart,
    // Quintic
    InQuint, OutQuint, InOutQuint,
    // Sine
    InSine, OutSine, InOutSine,
    // Exponential
    InExp, OutExp, InOutExp,
    // Circular
    InCirc, OutCirc, InOutCirc,
    // Elastic
    InElastic, OutElastic, InOutElastic,
    // Back (overshoot)
    InBack, OutBack, InOutBack,
    // Bounce
    InBounce, OutBounce, InOutBounce,
    // Custom
    Bezier(f64, f64, f64, f64),
    Pow { begin: f64, end: f64 },
    ExpDecay { d1: f64, d2: f64 },
}
```

### Data Binding

```rust
// DataBindingStore for widget ↔ data synchronization
pub struct DataBindingStore {
    pub nodes: Vec<LiveNode>,
}

impl DataBindingStore {
    pub fn data_to_widgets(&self, cx: &mut Cx, actions: &Actions, ui: &WidgetRef);
    pub fn widgets_to_data(&mut self, cx: &mut Cx, actions: &Actions, ui: &WidgetRef);
}
```

### What Makepad Lacks for D3-like Visualization

| Category | Missing Components |
|----------|-------------------|
| **Scales** | Linear, log, time, ordinal, band, quantize scales |
| **Axes** | Axis generators, tick calculation, label formatting |
| **Shapes** | Data-driven line, area, arc, pie, stack generators |
| **Layouts** | Force simulation, tree, treemap, pack, partition |
| **Geographic** | Map projections, GeoJSON, spherical math |
| **Interactions** | Zoom behavior, brush selection, drag behavior |

---

## Gap Analysis by Module

### 1. Scale System

#### D3 Provides

| Scale Type | Purpose | Example |
|------------|---------|---------|
| `scaleLinear` | Continuous → continuous | Position encoding |
| `scaleLog` | Logarithmic mapping | Wide-range data |
| `scalePow` | Power/exponential | Non-linear emphasis |
| `scaleTime` | Date/time → continuous | Time series |
| `scaleOrdinal` | Discrete → discrete | Color encoding |
| `scaleBand` | Discrete → continuous bands | Bar charts |
| `scalePoint` | Discrete → continuous points | Dot plots |
| `scaleQuantize` | Continuous → discrete buckets | Choropleth |
| `scaleQuantile` | Data-driven buckets | Statistical |
| `scaleThreshold` | Custom breakpoints | Classification |
| `scaleSequential` | Continuous → color | Heatmaps |
| `scaleDiverging` | Bidirectional color | Diverging data |

#### Makepad Has
- ❌ No scale abstractions
- ❌ No domain/range mapping
- ❌ No tick generation
- ❌ No `nice()` rounding

#### Gap Level: 🔴 Critical

### 2. Axis Generation

#### D3 Provides
```javascript
d3.axisBottom(scale)   // Ticks below
d3.axisTop(scale)      // Ticks above
d3.axisLeft(scale)     // Ticks left
d3.axisRight(scale)    // Ticks right

// Customization
axis.ticks(10)
axis.tickValues([1, 2, 5, 10])
axis.tickFormat(d3.format('$,.2f'))
axis.tickSize(6)
axis.tickPadding(3)
```

#### Makepad Has
- ❌ No axis generators
- ❌ No tick algorithms
- ✅ DrawText for labels (manual positioning)
- ✅ DrawLine for tick marks (manual drawing)

#### Gap Level: 🔴 Critical

### 3. Shape Generators

#### D3 Provides

| Generator | Output | Use Case |
|-----------|--------|----------|
| `d3.line()` | SVG path | Line charts |
| `d3.area()` | SVG path | Area charts |
| `d3.arc()` | SVG path | Pie/donut charts |
| `d3.pie()` | Angle data | Pie layout |
| `d3.stack()` | Stacked data | Stacked charts |
| `d3.symbol()` | Symbol paths | Scatter plots |
| `d3.linkHorizontal()` | Bezier path | Node-link diagrams |
| `d3.linkVertical()` | Bezier path | Tree diagrams |

#### D3 Curve Types (15+)
- `curveLinear` - Straight segments
- `curveStep` / `curveStepBefore` / `curveStepAfter`
- `curveBasis` / `curveBasisClosed`
- `curveBundle`
- `curveCardinal` / `curveCardinalClosed`
- `curveCatmullRom` / `curveCatmullRomClosed`
- `curveMonotoneX` / `curveMonotoneY`
- `curveNatural`

#### Makepad Has
- ✅ DrawLine with Bezier
- ✅ Path primitives (MoveTo, LineTo, CurveTo)
- ✅ Arc geometry type
- ❌ No data-driven generators
- ❌ Limited curve interpolators

#### Gap Level: 🟠 Major

### 4. Animation & Transitions

#### D3 Provides
```javascript
selection.transition()
    .duration(750)
    .delay((d, i) => i * 100)
    .ease(d3.easeCubicInOut)
    .attr('x', d => scale(d))
    .attrTween('d', arcTween);
```

#### Makepad Has
```rust
// Animator with comparable easing
animator.animator_play(cx, id!(hover.on));

// Keyframe system
KeyFrame { time: 0.0, value: 0.0, ease: Ease::OutCubic }
```

#### Gap Level: 🟢 Comparable

### 5. Interactions

#### D3 Zoom Behavior
```javascript
const zoom = d3.zoom()
    .scaleExtent([1, 8])
    .translateExtent([[0, 0], [width, height]])
    .on('zoom', (event) => {
        g.attr('transform', event.transform);
    });

svg.call(zoom);
```

#### D3 Brush Behavior
```javascript
const brush = d3.brushX()
    .extent([[0, 0], [width, height]])
    .on('brush', brushed)
    .on('end', brushended);
```

#### Makepad Has
- ✅ Mouse/touch events
- ✅ Scroll handling
- ❌ No zoom transform system
- ❌ No brush selection
- ❌ No drag behavior abstraction

#### Gap Level: 🟠 Major

### 6. Hierarchical Layouts

#### D3 Provides

| Layout | Algorithm | Output |
|--------|-----------|--------|
| `d3.tree()` | Tidy tree | Node positions |
| `d3.cluster()` | Dendrogram | Leaf-aligned tree |
| `d3.treemap()` | Squarified | Nested rectangles |
| `d3.pack()` | Circle packing | Nested circles |
| `d3.partition()` | Adjacency | Icicle/sunburst |

#### Makepad Has
- ❌ No hierarchy data structure
- ❌ No layout algorithms

#### Gap Level: 🔴 Critical

### 7. Force Simulation

#### D3 Provides
```javascript
const simulation = d3.forceSimulation(nodes)
    .force('charge', d3.forceManyBody().strength(-300))
    .force('link', d3.forceLink(links).distance(30))
    .force('center', d3.forceCenter(width/2, height/2))
    .force('collide', d3.forceCollide(radius));

simulation.on('tick', () => {
    // Update node positions
});
```

#### Makepad Has
- ❌ No physics engine
- ❌ No force composition
- ❌ No collision detection

#### Gap Level: 🔴 Critical

### 8. Geographic Projections

#### D3 Provides
- 50+ map projections (Mercator, Orthographic, Albers, etc.)
- GeoJSON/TopoJSON parsing
- Great circle interpolation
- Spherical geometry
- Path generation from geography

#### Makepad Has
- ❌ No projection math
- ❌ No GeoJSON support
- ❌ No spherical calculations

#### Gap Level: 🔴 Critical

---

## makepad-chart Library Analysis

### Overview

**Repository**: `../echart/makepad-chart`
**Size**: ~9,300 lines of Rust across 35+ source files
**Philosophy**: Chart.js-style high-level chart widgets (not D3's low-level approach)

### Architecture

```
makepad-chart/src/
├── core/           # Data structures and configuration
│   ├── data.rs     # ChartData, Dataset, DataPoint
│   ├── options.rs  # ChartOptions, TickOptions
│   ├── colors.rs   # Color palettes and utilities
│   └── types.rs    # EasingType, enums
├── scale/          # Domain → pixel mapping
│   ├── linear.rs   # LinearScale (269 lines)
│   ├── category.rs # CategoryScale (327 lines)
│   ├── traits.rs   # Scale trait
│   └── utils.rs    # nice_step, nice_bounds, format_number
├── coord/          # Coordinate systems
│   ├── cartesian.rs # 2D Cartesian coordinates
│   └── polar.rs    # Polar coordinates (pie, radar)
├── element/        # GPU shader primitives
│   ├── bar.rs      # DrawBar
│   ├── line.rs     # DrawChartLine
│   ├── point.rs    # DrawPoint
│   ├── arc.rs      # DrawArc (pie slices)
│   ├── triangle.rs # DrawTriangle (area fills)
│   └── grid.rs     # DrawGridLine
├── chart/          # Chart widget implementations
│   ├── bar_chart.rs
│   ├── horizontal_bar_chart.rs
│   ├── line_chart.rs
│   ├── scatter_chart.rs
│   ├── bubble_chart.rs
│   ├── pie_chart.rs
│   ├── radar_chart.rs
│   ├── polar_area_chart.rs
│   ├── combo_chart.rs
│   └── chord_chart.rs
├── animation/      # Animation system
│   ├── animator.rs # ChartAnimator state machine
│   └── easing.rs   # 28 easing functions
├── interaction/    # Hit testing
│   └── hit_test.rs # HitTester, HitData
└── component/      # Stubs (incomplete)
    ├── axis.rs     # Empty stub
    ├── tooltip.rs  # Empty stub
    └── legend.rs   # Empty stub
```

### Scale Implementation

#### LinearScale

```rust
// src/scale/linear.rs
pub struct LinearScale {
    data_min: f64,
    data_max: f64,
    pixel_start: f64,
    pixel_end: f64,
    begin_at_zero: bool,
    nice: bool,
    clamp: bool,
}

impl Scale for LinearScale {
    fn get_pixel_for_value(&self, value: f64) -> f64;
    fn get_value_for_pixel(&self, pixel: f64) -> f64;
    fn build_ticks(&self, options: &TickOptions) -> Vec<Tick>;
}
```

**Features:**
- ✅ Domain/range mapping
- ✅ `begin_at_zero` option
- ✅ `nice()` bounds calculation
- ✅ Tick generation with `nice_step()`
- ✅ Inverted ranges (for Y axis)
- ✅ Value clamping

#### CategoryScale

```rust
// src/scale/category.rs
pub struct CategoryScale {
    labels: Vec<String>,
    pixel_start: f64,
    pixel_end: f64,
    offset: bool,  // Center items or align to grid
}

impl CategoryScale {
    fn get_band_width(&self) -> f64;
    fn get_bar_width(&self, bar_percent: f64) -> f64;
    fn get_pixel_for_index(&self, index: usize) -> f64;
}
```

**Features:**
- ✅ Discrete label mapping
- ✅ Band width calculation
- ✅ Offset mode (bar vs line charts)
- ✅ Tick skipping for dense labels

### Chart Types (11 Implemented)

| Chart | File | Features |
|-------|------|----------|
| **Bar** | `bar_chart.rs` | Vertical bars, stacking, grouping, gradients |
| **Horizontal Bar** | `horizontal_bar_chart.rs` | Horizontal variant |
| **Line** | `line_chart.rs` | Points, area fill, tension, stepped modes |
| **Scatter** | `scatter_chart.rs` | XY points with styling |
| **Bubble** | `bubble_chart.rs` | Variable radius points |
| **Pie/Doughnut** | `pie_chart.rs` | Circular slices, inner radius |
| **Radar** | `radar_chart.rs` | Multi-axis comparison |
| **Polar Area** | `polar_area_chart.rs` | Equal-angle sectors |
| **Combo** | `combo_chart.rs` | Bar + line hybrid |
| **Chord** | `chord_chart.rs` | Flow/relationship diagram |

### Animation System

```rust
// src/animation/easing.rs - 28 easing functions
pub enum EasingType {
    Linear,
    EaseInQuad, EaseOutQuad, EaseInOutQuad,
    EaseInCubic, EaseOutCubic, EaseInOutCubic,
    EaseInQuart, EaseOutQuart, EaseInOutQuart,
    EaseInQuint, EaseOutQuint, EaseInOutQuint,
    EaseInSine, EaseOutSine, EaseInOutSine,
    EaseInExpo, EaseOutExpo, EaseInOutExpo,
    EaseInCirc, EaseOutCirc, EaseInOutCirc,
    EaseInBack, EaseOutBack, EaseInOutBack,
    EaseInElastic, EaseOutElastic, EaseInOutElastic,
    EaseInBounce, EaseOutBounce, EaseInOutBounce,
}

// Animation modes
- Standard: All elements animate together
- Delay: Staggered animation per element
- Progressive: Left-to-right reveal (time series)
```

### Data Model

```rust
// src/core/data.rs
pub struct ChartData {
    pub labels: Vec<String>,
    pub datasets: Vec<Dataset>,
}

pub struct Dataset {
    pub label: String,
    pub data: Vec<DataPoint>,
    pub background_color: Option<Vec4>,
    pub border_color: Option<Vec4>,
    pub hidden: bool,
    // Line-specific
    pub fill: bool,
    pub tension: f64,
    // Point-specific
    pub point_radius: f64,
    pub point_style: DataPointStyle,
    // Bar-specific
    pub bar_thickness: Option<f64>,
    pub bar_percentage: f64,
}

pub struct DataPoint {
    pub x: Option<f64>,      // Optional X (else use index)
    pub y: f64,              // Y value
    pub y_min: Option<f64>,  // For floating bars
    pub r: Option<f64>,      // For bubbles
    pub label: Option<String>,
    pub meta: Option<String>,
}
```

### GPU Shader Elements

#### DrawArc (Pie/Donut slices)

```rust
// src/element/arc.rs - GPU shader for arcs
live_design! {
    pub DrawArc = {{DrawArc}} {
        fn pixel(self) -> vec4 {
            // Distance from center
            let distance = sqrt(px * px + py * py);

            // Angle using atan2
            let pixel_ang = atan(py, px);

            // Angle masking for sweep
            let sweep_val = self.end_angle - self.start_angle;
            let ang_mask = step(wrap_ang, sweep_val);

            // Anti-aliased edges
            let outer_aa = smoothstep(outer_rad - edge_aa, outer_rad + edge_aa, distance);

            // Gradient support (radial or angular)
            if self.gradient_enabled > 0.5 { ... }
        }
    }
}
```

### Maturity Assessment

| Aspect | Score | Notes |
|--------|-------|-------|
| Chart Types | 9/10 | 11 types, covers most use cases |
| Core Rendering | 9/10 | GPU-accelerated, smooth |
| Data Binding | 8/10 | Flexible API, builder pattern |
| Animation | 9/10 | 28 easings, multiple modes |
| Interaction | 4/10 | Hover only, no zoom/pan |
| Axis/Grid | 6/10 | Implicit, no dedicated component |
| Theming | 5/10 | Limited customization |
| Error Handling | 4/10 | Minimal validation |
| Testing | 6/10 | Unit tests for scales |

### What makepad-chart Provides vs D3

| D3 Feature | makepad-chart Status |
|------------|---------------------|
| Linear scale | ✅ Full implementation |
| Category scale | ✅ Full implementation |
| Time scale | ❌ Missing |
| Log/Pow scales | ❌ Missing |
| Axis generator | 🟡 Implicit only |
| Line generator | ✅ With Catmull-Rom |
| Area generator | ✅ Triangle-based |
| Arc generator | ✅ GPU shader |
| Pie layout | ✅ Built into PieChart |
| Stack layout | ❌ Missing |
| 28 easing functions | ✅ Full match |
| Zoom behavior | ❌ Missing |
| Brush selection | ❌ Missing |
| Force simulation | ❌ Missing |
| Hierarchy layouts | ❌ Missing |

---

## Linebender Ecosystem Analysis

### Overview

Linebender is a Rust graphics collective building a complete 2D graphics stack. Key libraries provide primitives that could accelerate D3-like development.

### Ecosystem Map

```
┌─────────────────────────────────────────────────────────────────┐
│                    LINEBENDER ECOSYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│  UI FRAMEWORKS                                                   │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │  Xilem   │  │ Masonry  │  │  Druid   │                      │
│  │ Reactive │  │ Widget   │  │ (Legacy) │                      │
│  │   UI     │  │  Tree    │  │          │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
├─────────────────────────────────────────────────────────────────┤
│  RENDERING                                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                        Vello                              │   │
│  │     GPU Compute-Centric 2D Renderer (wgpu/WebGPU)        │   │
│  │     Prefix-sum algorithms, 177fps on M1 Max              │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│  PRIMITIVES                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Kurbo   │  │  Peniko  │  │  Color   │  │  Parley  │       │
│  │ Geometry │  │ Styling  │  │  Spaces  │  │  Text    │       │
│  │  Curves  │  │ Brushes  │  │  CSS L4  │  │  Layout  │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
├─────────────────────────────────────────────────────────────────┤
│  CONTENT                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  Resvg   │  │  Velato  │  │Vello_svg │  │  Norad   │       │
│  │   SVG    │  │  Lottie  │  │ SVG→Vello│  │   UFO    │       │
│  │ Renderer │  │ Animate  │  │          │  │  Fonts   │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Kurbo - Geometry & Curves

**Purpose**: Foundation for all vector graphics operations

#### Types

| Type | Description | D3 Equivalent |
|------|-------------|---------------|
| `Point` | 2D point (f64, f64) | `[x, y]` |
| `Vec2` | 2D vector | - |
| `Line` | Line segment | Path segment |
| `Rect` | Rectangle | - |
| `RoundedRect` | Rounded corners | - |
| `Circle` | Circle | `d3.arc()` |
| `Ellipse` | Ellipse | `d3.arc()` |
| `Arc` | Elliptical arc | `d3.arc()` |
| `QuadBez` | Quadratic Bézier | Curve segment |
| `CubicBez` | Cubic Bézier | Curve segment |
| `BezPath` | Complete path | SVG `d` attribute |
| `Affine` | 2D transform | `d3.zoomTransform` |

#### Traits

```rust
// Parametric curve evaluation
trait ParamCurve {
    fn eval(&self, t: f64) -> Point;
    fn subsegment(&self, range: Range<f64>) -> Self;
    fn start(&self) -> Point;
    fn end(&self) -> Point;
}

// Arc length
trait ParamCurveArclen {
    fn arclen(&self, accuracy: f64) -> f64;
    fn inv_arclen(&self, arclen: f64, accuracy: f64) -> f64;
}

// Curvature
trait ParamCurveCurvature {
    fn curvature(&self, t: f64) -> f64;
}

// Nearest point
trait ParamCurveNearest {
    fn nearest(&self, p: Point, accuracy: f64) -> Nearest;
}

// Shape operations
trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self, accuracy: f64) -> f64;
    fn bounding_box(&self) -> Rect;
    fn path_elements(&self, tolerance: f64) -> impl Iterator<Item = PathEl>;
}
```

#### Relevance for D3

- ✅ All geometric primitives for charts
- ✅ Curve mathematics for smooth interpolation
- ✅ Arc length for text-on-path
- ✅ Affine transforms for zoom/pan
- ✅ Path operations (flatten, stroke, dash)

### Peniko - Styling

**Purpose**: Define how shapes are rendered

#### Types

| Type | Description | CSS Equivalent |
|------|-------------|----------------|
| `Brush` | Fill content | `fill` |
| `Color` | RGBA color | CSS color |
| `Gradient` | Color transitions | `linear-gradient` |
| `GradientKind::Linear` | Linear gradient | `linearGradient` |
| `GradientKind::Radial` | Radial gradient | `radialGradient` |
| `GradientKind::Sweep` | Conic gradient | `conic-gradient` |
| `ColorStop` | Gradient stop | `<stop>` |
| `Fill` | Fill rule | `fill-rule` |
| `Stroke` | Stroke properties | `stroke-*` |
| `BlendMode` | Compositing | `mix-blend-mode` |

#### Stroke Definition

```rust
pub struct Stroke {
    pub width: f64,
    pub join: Join,       // Miter, Round, Bevel
    pub miter_limit: f64,
    pub start_cap: Cap,   // Butt, Round, Square
    pub end_cap: Cap,
    pub dash_pattern: Dashes,
    pub dash_offset: f64,
}
```

### Color - Color Science

**Purpose**: Professional color space handling

#### Supported Color Spaces

| Space | Type | Use Case |
|-------|------|----------|
| `Srgb` | RGB | Standard web |
| `LinearSrgb` | Linear RGB | Interpolation |
| `DisplayP3` | Wide gamut | HDR |
| `Rec2020` | Ultra-wide | Professional |
| `Lab` | Perceptual | Difference |
| `Lch` | Lab polar | Gradients |
| `Oklab` | Improved Lab | Modern color |
| `Oklch` | OkLab polar | Best gradients |
| `Hsl` | HSL | Color pickers |
| `Hwb` | HWB | CSS Color 4 |
| `XyzD50` | CIE XYZ | Reference |

#### Color Operations

```rust
// Conversion
let lab: AlphaColor<Lab> = srgb.convert();

// Interpolation
let interpolator = Interpolator::new(color1, color2);
let mid = interpolator.eval(0.5);

// Gradient
let colors = color::gradient(&stops, 256);

// CSS parsing
let color = parse_color("oklch(70% 0.15 180)")?;
```

#### Relevance for D3

- ✅ `d3.interpolateLab` equivalent
- ✅ Perceptually uniform scales
- ✅ CSS Color Level 4 parsing
- ✅ Exceeds D3's color capabilities

### Vello - GPU Renderer

**Purpose**: High-performance 2D rendering

#### Features

| Feature | Description |
|---------|-------------|
| Compute Shaders | GPU-parallel via prefix-sum |
| WebGPU/wgpu | Cross-platform |
| Antialiasing | Up to MSAA 16x |
| Scene Graph | Hierarchical commands |
| Layers | Grouping, clipping, effects |

#### API

```rust
let mut scene = Scene::new();

// Fill shape
scene.fill(
    Fill::NonZero,
    Affine::IDENTITY,
    &Brush::Solid(Color::RED),
    None,
    &circle,
);

// Stroke path
scene.stroke(
    &Stroke::new(2.0),
    Affine::IDENTITY,
    &Brush::Solid(Color::BLACK),
    None,
    &path,
);

// Gradient
let gradient = Gradient::new_linear(p1, p2)
    .with_stops([Color::RED, Color::BLUE]);
scene.fill(Fill::NonZero, transform, &gradient.into(), None, &rect);

// Layers
scene.push_layer(blend, alpha, transform, &clip);
// ... draw ...
scene.pop_layer();
```

#### Performance

- 177 fps on M1 Max (paris-30k scene)
- Designed for interactive graphics

### Parley - Text Layout

**Purpose**: Professional typography

#### Components

| Component | Purpose |
|-----------|---------|
| **Fontique** | Font enumeration, fallback |
| **HarfRust** | Text shaping |
| **Skrifa** | Font parsing |
| **ICU4X** | Unicode, BiDi, line breaking |

#### Capabilities

- Rich text with multiple styles
- Font fallback (emoji, CJK)
- Bidirectional text
- Line breaking and wrapping
- Text selection and editing
- Glyph positioning

### Resvg - SVG Rendering

**Purpose**: Static SVG support

| Feature | Support |
|---------|---------|
| Shapes | ✅ Full |
| Gradients | ✅ Full |
| Patterns | ✅ Full |
| Masks/Clips | ✅ Full |
| Filters | ✅ Most |
| Text | ✅ To paths |
| Animation | ❌ No |

### Velato - Lottie Animations

**Purpose**: After Effects animations

| Feature | Status |
|---------|--------|
| Shapes | ✅ |
| Transforms | ✅ |
| Keyframes | ✅ |
| Easing | ✅ Basic |
| Masks | ✅ |
| Text | ❌ |
| Images | ❌ |

### Linebender Value for D3-like Visualization

| Library | Value | Priority |
|---------|-------|----------|
| **Kurbo** | ⭐⭐⭐⭐⭐ Essential geometry | Critical |
| **Peniko** | ⭐⭐⭐⭐⭐ Styling primitives | Critical |
| **Color** | ⭐⭐⭐⭐⭐ Color science | High |
| **Parley** | ⭐⭐⭐⭐ Text for labels | High |
| **Vello** | ⭐⭐⭐⭐ GPU rendering | Medium |
| **Resvg** | ⭐⭐⭐ SVG import | Medium |
| **Velato** | ⭐⭐ Lottie animations | Low |
| **Xilem** | ⭐⭐ UI framework | Low |

---

## Combined Gap Assessment

### Gap Closure with All Libraries

| D3 Module | Makepad Native | makepad-chart | Linebender | Combined Status |
|-----------|----------------|---------------|------------|-----------------|
| **Geometry** | Basic | - | ✅ Kurbo | 🟢 **Closed** |
| **Curves** | Bezier only | Catmull-Rom | ✅ Kurbo | 🟢 **Closed** |
| **Colors** | RGBA | 10 palettes | ✅ Color | 🟢 **Closed** |
| **Gradients** | Shader | Basic | ✅ Peniko | 🟢 **Closed** |
| **Text** | Basic | - | ✅ Parley | 🟢 **Closed** |
| **Linear Scale** | ❌ | ✅ | ❌ | 🟢 **Closed** |
| **Category Scale** | ❌ | ✅ | ❌ | 🟢 **Closed** |
| **Time Scale** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Log/Pow Scales** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Axis Generator** | ❌ | 🟡 Implicit | ❌ | 🟠 Partial |
| **Line Generator** | Manual | ✅ | Kurbo | 🟢 **Closed** |
| **Area Generator** | Manual | ✅ | Kurbo | 🟢 **Closed** |
| **Arc Generator** | ❌ | ✅ GPU | ✅ Kurbo | 🟢 **Closed** |
| **Pie Layout** | ❌ | ✅ | ❌ | 🟢 **Closed** |
| **Stack Layout** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Symbol Generator** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Animation** | ✅ 30 easings | ✅ 28 easings | Velato | 🟢 **Closed** |
| **Zoom/Pan** | ❌ | ❌ | Affine | 🟠 Partial |
| **Brush Selection** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Force Simulation** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Tree Layout** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Treemap** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Pack Layout** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Partition** | ❌ | ❌ | ❌ | 🔴 Missing |
| **Geographic** | ❌ | ❌ | ❌ | 🔴 Missing |

### Summary by Category

| Category | Gap Status | What's Available |
|----------|------------|------------------|
| **Primitives** | 🟢 Closed | Kurbo, Peniko, Color, Parley |
| **Basic Scales** | 🟢 Closed | makepad-chart Linear/Category |
| **Advanced Scales** | 🔴 Missing | Time, Log, Pow, Quantize |
| **Chart Types** | 🟢 Good | 11 types in makepad-chart |
| **Axes** | 🟠 Partial | Implicit only |
| **Animation** | 🟢 Closed | 28-30 easing functions |
| **Interactions** | 🟠 Partial | Hover only, no zoom/brush |
| **Layouts** | 🔴 Missing | Force, hierarchy |
| **Geographic** | 🔴 Missing | Projections, GeoJSON |

### Overall Gap Estimate

| Scenario | Gap Closed | Remaining Effort |
|----------|------------|------------------|
| Basic business charts | ~75% | Low |
| Interactive dashboards | ~50% | Medium |
| Network graphs | ~20% | High |
| Geographic maps | ~5% | Very High |
| Full D3 parity | ~40% | 10-15K lines |

---

## Implementation Roadmap

### Phase 1: Core Extensions (1-2 months)

**Goal**: Complete the scale and axis system

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| `TimeScale` | High | Medium | Date parsing |
| `LogScale` | Medium | Low | Math |
| `PowScale` | Medium | Low | Math |
| `QuantizeScale` | Medium | Low | - |
| `ThresholdScale` | Medium | Low | - |
| `Axis` component | High | Medium | Scales |
| Tick formatting | High | Low | - |
| Grid component | Medium | Low | Axis |

**Deliverables**:
- Time-series chart support
- Configurable axes with all orientations
- Tick customization API

### Phase 2: Shape Generators (1 month)

**Goal**: Data-driven shape generation

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| `StackGenerator` | High | Medium | - |
| `SymbolGenerator` | Medium | Low | - |
| More curve types | Medium | Medium | Kurbo |
| `LinkGenerator` | Low | Low | - |

**Deliverables**:
- Stacked bar/area charts
- Custom point symbols
- Smooth curve options

### Phase 3: Interactions (2 months)

**Goal**: Pan, zoom, and selection

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| `ZoomBehavior` | High | High | Affine math |
| `BrushBehavior` | High | High | Events |
| `DragBehavior` | Medium | Medium | Events |
| Tooltip component | High | Medium | - |
| Legend component | Medium | Medium | - |

**Deliverables**:
- Interactive zoom/pan
- Range selection
- Data tooltips

### Phase 4: Advanced Layouts (3-4 months)

**Goal**: Network and hierarchical visualizations

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| `ForceSimulation` | High | Very High | Physics |
| `TreeLayout` | High | High | Hierarchy |
| `TreemapLayout` | Medium | High | Hierarchy |
| `PackLayout` | Medium | High | Hierarchy |
| `PartitionLayout` | Low | Medium | Hierarchy |

**Deliverables**:
- Network graphs
- Tree diagrams
- Treemaps and sunbursts

### Phase 5: Geographic (4-6 months)

**Goal**: Map visualizations

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| Projection math | High | Very High | - |
| GeoJSON parser | High | Medium | - |
| Path generator | High | High | Projections |
| Graticule | Medium | Medium | - |

**Deliverables**:
- Choropleth maps
- Point maps
- Geographic paths

---

## Technical Recommendations

### Option A: Extend makepad-chart

**Best for**: Quick wins, maintaining Makepad integration

```rust
// Add to makepad-chart/src/scale/
mod time;    // TimeScale
mod log;     // LogScale
mod pow;     // PowScale

// Add to makepad-chart/src/component/
mod axis;    // Full axis component
mod tooltip; // Tooltip rendering
mod legend;  // Interactive legend
```

**Pros**:
- Builds on existing foundation
- Native Makepad integration
- GPU-optimized rendering

**Cons**:
- Limited to Makepad ecosystem
- Less mathematical rigor than Linebender

### Option B: Integrate Linebender Primitives

**Best for**: Mathematical precision, cross-platform

```rust
// Use Kurbo for geometry
use kurbo::{BezPath, Arc, Affine, Shape};

// Use Color for color science
use color::{OkLch, Interpolator};

// Create Makepad bindings
impl MakepadDrawable for BezPath {
    fn draw(&self, cx: &mut Cx2d) { ... }
}
```

**Pros**:
- Professional-grade primitives
- Perceptually correct colors
- Well-tested math

**Cons**:
- Integration overhead
- Potential API mismatch

### Option C: Hybrid Approach (Recommended)

**Best for**: Balance of integration and quality

```
┌─────────────────────────────────────────────────────────────┐
│                    Visualization Layer                       │
│  ┌─────────────────────────────────────────────────────┐   │
│  │               makepad-chart-extended                 │   │
│  │  Scales | Axes | Shapes | Charts | Interactions     │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│  PRIMITIVES (Choose per component)                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
│  │   Kurbo    │  │   Color    │  │  Makepad   │           │
│  │  Geometry  │  │   Spaces   │  │  DrawText  │           │
│  │   (math)   │  │  (colors)  │  │  (native)  │           │
│  └────────────┘  └────────────┘  └────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  RENDERING (Makepad native)                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │          Makepad GPU Shaders (Metal/Vulkan/WebGL)    │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**Implementation**:

1. **Keep Makepad rendering** - Already optimized
2. **Use Kurbo for math** - Curves, transforms, geometry
3. **Use Color for color science** - Interpolation, spaces
4. **Extend makepad-chart** - Add missing scales, axes
5. **Build interactions** - Zoom, brush on Makepad events

---

## Appendix: API Comparisons

### Scale API Comparison

#### D3
```javascript
const x = d3.scaleLinear()
    .domain([0, 100])
    .range([0, 800])
    .nice()
    .clamp(true);

x(50);           // 400
x.invert(400);   // 50
x.ticks(10);     // [0, 10, 20, ...]
```

#### makepad-chart
```rust
let mut scale = LinearScale::new()
    .with_begin_at_zero(true)
    .with_nice(true)
    .with_clamp(true);
scale.set_data_range(0.0, 100.0);
scale.set_pixel_range(0.0, 800.0);

scale.get_pixel_for_value(50.0);  // 400.0
scale.get_value_for_pixel(400.0); // 50.0
scale.build_ticks(&options);      // Vec<Tick>
```

### Shape API Comparison

#### D3
```javascript
const line = d3.line()
    .x(d => xScale(d.date))
    .y(d => yScale(d.value))
    .curve(d3.curveCatmullRom);

path.attr('d', line(data));
```

#### makepad-chart (Current)
```rust
// Manual line drawing in LineChart
for i in 0..points.len() - 1 {
    self.draw_line.draw_line(cx, points[i], points[i + 1], width);
}

// With tension (Catmull-Rom)
let control_points = Self::calculate_control_points_static(&points, tension, &mode);
self.draw_cubic_lines(cx, &points, &control_points, width);
```

#### Proposed (with Kurbo)
```rust
// Data-driven line generator
let line = LineGenerator::new()
    .x(|d| x_scale.get_pixel_for_value(d.x))
    .y(|d| y_scale.get_pixel_for_value(d.y))
    .curve(CurveType::CatmullRom { tension: 0.5 });

let path: BezPath = line.generate(&data);
path.draw(cx);
```

### Animation API Comparison

#### D3
```javascript
selection.transition()
    .duration(750)
    .ease(d3.easeCubicInOut)
    .attr('width', d => scale(d));
```

#### makepad-chart
```rust
let animator = ChartAnimator::new(750.0)
    .with_easing(EasingType::EaseInOutCubic);
animator.start(time);

// In draw loop
let progress = animator.get_progress();
let width = target_width * progress;
```

### Color API Comparison

#### D3
```javascript
const color = d3.lab(50, 20, -30);
const interpolate = d3.interpolateLab(color1, color2);
interpolate(0.5);
```

#### Linebender Color
```rust
let color: AlphaColor<Lab> = AlphaColor::new([50.0, 20.0, -30.0], 1.0);
let interpolator = Interpolator::new(color1, color2);
interpolator.eval(0.5);
```

---

## Conclusion

### Current State

The combination of **Makepad**, **makepad-chart**, and **Linebender** provides:

- ✅ GPU-accelerated rendering
- ✅ Basic scale system (linear, category)
- ✅ 11 chart types
- ✅ Professional animation (28+ easings)
- ✅ Excellent geometry primitives (Kurbo)
- ✅ Professional color science (Color)

### Gaps to Address

| Gap | Impact | Effort |
|-----|--------|--------|
| Advanced scales | High | Low-Medium |
| Axis component | High | Medium |
| Zoom/Pan | High | High |
| Brush selection | Medium | High |
| Force simulation | Medium | Very High |
| Hierarchy layouts | Medium | High |
| Geographic | Low | Very High |

### Recommended Path Forward

1. **Short term**: Extend makepad-chart with time scales, axis component, tooltips
2. **Medium term**: Add zoom/pan using Kurbo Affine, integrate Color for gradients
3. **Long term**: Implement force simulation and hierarchy layouts
4. **Future**: Geographic support if needed

### Estimated Total Effort

| Milestone | Lines of Code | Time |
|-----------|---------------|------|
| 80% D3 for business charts | ~5,000 | 2-3 months |
| 90% D3 with interactions | ~8,000 | 4-5 months |
| 95% D3 with layouts | ~12,000 | 6-8 months |
| Full D3 parity | ~15,000+ | 10-12 months |

---

*Document generated by Claude (Anthropic) - January 2026*
