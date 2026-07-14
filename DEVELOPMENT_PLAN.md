# Makepad D3 Development Plan

> Detailed development plan for implementing D3.js-like data visualization capabilities in Makepad

**Version**: 1.0
**Created**: January 2026
**Status**: Planning

---

## Table of Contents

1. [Overview](#overview)
2. [Priority Framework](#priority-framework)
3. [Phase 1: Foundation](#phase-1-foundation)
4. [Phase 2: Core Visualization](#phase-2-core-visualization)
5. [Phase 3: Interactivity](#phase-3-interactivity)
6. [Phase 4: Advanced Layouts](#phase-4-advanced-layouts)
7. [Phase 5: Geographic](#phase-5-geographic)
8. [Dependency Graph](#dependency-graph)
9. [Risk Assessment](#risk-assessment)
10. [Success Metrics](#success-metrics)

---

## Overview

### Vision

Create a comprehensive data visualization library for Makepad that provides D3.js-level capabilities while leveraging Rust's performance and Makepad's GPU-accelerated rendering.

### Guiding Principles

1. **Incremental Value**: Each phase delivers usable functionality
2. **API Consistency**: Follow established patterns from makepad-chart
3. **Performance First**: Leverage GPU rendering for large datasets
4. **Composability**: Small, reusable components over monolithic widgets
5. **Type Safety**: Use Rust's type system to prevent runtime errors

### Project Structure

```
makepad-d3/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── scale/              # Phase 1
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   ├── continuous/
│   │   │   ├── linear.rs   # Extend from makepad-chart
│   │   │   ├── log.rs
│   │   │   ├── pow.rs
│   │   │   ├── time.rs
│   │   │   └── symlog.rs
│   │   ├── ordinal/
│   │   │   ├── ordinal.rs
│   │   │   ├── band.rs     # Extend from makepad-chart
│   │   │   └── point.rs
│   │   └── discretizing/
│   │       ├── quantize.rs
│   │       ├── quantile.rs
│   │       └── threshold.rs
│   ├── axis/               # Phase 1
│   │   ├── mod.rs
│   │   ├── axis.rs
│   │   ├── ticks.rs
│   │   └── format.rs
│   ├── shape/              # Phase 2
│   │   ├── mod.rs
│   │   ├── line.rs
│   │   ├── area.rs
│   │   ├── arc.rs
│   │   ├── pie.rs
│   │   ├── stack.rs
│   │   ├── symbol.rs
│   │   └── curve/
│   │       ├── mod.rs
│   │       ├── linear.rs
│   │       ├── step.rs
│   │       ├── basis.rs
│   │       ├── cardinal.rs
│   │       ├── catmull_rom.rs
│   │       ├── monotone.rs
│   │       └── natural.rs
│   ├── color/              # Phase 2
│   │   ├── mod.rs
│   │   ├── schemes.rs
│   │   ├── sequential.rs
│   │   ├── diverging.rs
│   │   └── categorical.rs
│   ├── component/          # Phase 2
│   │   ├── mod.rs
│   │   ├── tooltip.rs
│   │   ├── legend.rs
│   │   ├── annotation.rs
│   │   └── crosshair.rs
│   ├── interaction/        # Phase 3
│   │   ├── mod.rs
│   │   ├── zoom.rs
│   │   ├── brush.rs
│   │   ├── drag.rs
│   │   └── hover.rs
│   ├── layout/             # Phase 4
│   │   ├── mod.rs
│   │   ├── force/
│   │   │   ├── mod.rs
│   │   │   ├── simulation.rs
│   │   │   ├── center.rs
│   │   │   ├── collide.rs
│   │   │   ├── link.rs
│   │   │   ├── many_body.rs
│   │   │   └── position.rs
│   │   └── hierarchy/
│   │       ├── mod.rs
│   │       ├── tree.rs
│   │       ├── cluster.rs
│   │       ├── treemap.rs
│   │       ├── pack.rs
│   │       └── partition.rs
│   ├── geo/                # Phase 5
│   │   ├── mod.rs
│   │   ├── projection/
│   │   ├── path.rs
│   │   └── geojson.rs
│   └── util/
│       ├── mod.rs
│       ├── array.rs
│       ├── format.rs
│       └── interpolate.rs
├── examples/
│   ├── basic_charts/
│   ├── interactive/
│   ├── layouts/
│   └── geographic/
└── tests/
```

---

## Priority Framework

### Priority Levels

| Level | Definition | Criteria |
|-------|------------|----------|
| **P0** | Critical | Blocks all other work, required for MVP |
| **P1** | High | Required for production use cases |
| **P2** | Medium | Important for feature completeness |
| **P3** | Low | Nice to have, future enhancement |
| **P4** | Deferred | Out of scope for initial release |

### Prioritization Matrix

```
                    HIGH IMPACT
                        │
         P1             │            P0
    (Important)         │        (Critical)
                        │
   ─────────────────────┼─────────────────────
                        │
         P3             │            P2
      (Future)          │         (Medium)
                        │
                    LOW IMPACT

         LOW EFFORT ◄───┴───► HIGH EFFORT
```

### Feature Priority Summary

| Feature | Priority | Impact | Effort | Phase |
|---------|----------|--------|--------|-------|
| TimeScale | P0 | High | Medium | 1 |
| Axis Component | P0 | High | Medium | 1 |
| Tooltip | P0 | High | Low | 2 |
| LogScale | P1 | Medium | Low | 1 |
| StackGenerator | P1 | High | Medium | 2 |
| ZoomBehavior | P1 | High | High | 3 |
| Legend | P1 | Medium | Medium | 2 |
| BrushBehavior | P2 | Medium | High | 3 |
| ForceSimulation | P2 | Medium | Very High | 4 |
| TreeLayout | P2 | Medium | High | 4 |
| Geographic | P3 | Low | Very High | 5 |

---

## Phase 1: Foundation

**Duration**: 4-6 weeks
**Goal**: Complete scale system and axis generation

### 1.1 Time Scale (P0)

**Priority**: P0 - Critical
**Effort**: 2 weeks
**Dependencies**: None

#### Requirements

- Parse date strings and timestamps
- Map dates to pixel positions
- Generate appropriate time-based ticks
- Support multiple time granularities (year, month, day, hour, minute)
- Handle timezone considerations

#### API Design

```rust
/// Time scale for date/time data
pub struct TimeScale {
    domain: (DateTime<Utc>, DateTime<Utc>),
    range: (f64, f64),
    nice: bool,
    clamp: bool,
}

impl TimeScale {
    pub fn new() -> Self;

    /// Set the input domain (date range)
    pub fn domain(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self;

    /// Set the output range (pixel range)
    pub fn range(self, start: f64, end: f64) -> Self;

    /// Extend domain to nice round dates
    pub fn nice(self, interval: TimeInterval) -> Self;

    /// Get pixel position for a date
    pub fn scale(&self, date: DateTime<Utc>) -> f64;

    /// Get date for a pixel position
    pub fn invert(&self, pixel: f64) -> DateTime<Utc>;

    /// Generate tick values
    pub fn ticks(&self, count: usize) -> Vec<DateTime<Utc>>;

    /// Generate tick values at specific interval
    pub fn ticks_interval(&self, interval: TimeInterval) -> Vec<DateTime<Utc>>;
}

/// Time intervals for tick generation
pub enum TimeInterval {
    Millisecond(u32),
    Second(u32),
    Minute(u32),
    Hour(u32),
    Day(u32),
    Week(u32),
    Month(u32),
    Year(u32),
}
```

#### Implementation Tasks

| Task | Effort | Priority |
|------|--------|----------|
| Core TimeScale struct | 2 days | P0 |
| Domain/range mapping | 1 day | P0 |
| Date parsing utilities | 2 days | P0 |
| Tick generation algorithm | 3 days | P0 |
| Nice bounds calculation | 2 days | P1 |
| Timezone handling | 2 days | P2 |
| Unit tests | 2 days | P0 |

#### Test Cases

```rust
#[test]
fn test_time_scale_basic() {
    let scale = TimeScale::new()
        .domain(
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
        )
        .range(0.0, 1000.0);

    // Mid-year should be ~500
    let mid = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();
    assert!((scale.scale(mid) - 500.0).abs() < 10.0);
}

#[test]
fn test_time_scale_ticks() {
    let scale = TimeScale::new()
        .domain(/* Jan 1 */, /* Dec 31 */)
        .range(0.0, 1000.0);

    let ticks = scale.ticks(12);
    assert_eq!(ticks.len(), 12); // One per month
}
```

### 1.2 Logarithmic Scale (P1)

**Priority**: P1 - High
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
pub struct LogScale {
    domain: (f64, f64),
    range: (f64, f64),
    base: f64,  // Default: 10
    clamp: bool,
}

impl LogScale {
    pub fn new() -> Self;
    pub fn base(self, base: f64) -> Self;
    pub fn domain(self, min: f64, max: f64) -> Self;
    pub fn range(self, start: f64, end: f64) -> Self;
    pub fn scale(&self, value: f64) -> f64;
    pub fn invert(&self, pixel: f64) -> f64;
    pub fn ticks(&self, count: usize) -> Vec<f64>;
}
```

#### Implementation Tasks

| Task | Effort | Priority |
|------|--------|----------|
| Core LogScale struct | 1 day | P1 |
| Log transformation | 1 day | P1 |
| Tick generation (powers) | 2 days | P1 |
| Handle edge cases (≤0) | 1 day | P1 |
| Unit tests | 1 day | P1 |

### 1.3 Power Scale (P2)

**Priority**: P2 - Medium
**Effort**: 3 days
**Dependencies**: None

#### API Design

```rust
pub struct PowScale {
    domain: (f64, f64),
    range: (f64, f64),
    exponent: f64,  // Default: 1 (linear), 0.5 (sqrt), 2 (square)
    clamp: bool,
}
```

### 1.4 Quantize/Quantile/Threshold Scales (P2)

**Priority**: P2 - Medium
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
/// Maps continuous domain to discrete range
pub struct QuantizeScale<T> {
    domain: (f64, f64),
    range: Vec<T>,
}

/// Maps continuous domain to discrete range based on data quantiles
pub struct QuantileScale<T> {
    domain: Vec<f64>,  // Sorted data
    range: Vec<T>,
}

/// Maps continuous domain to discrete range based on explicit thresholds
pub struct ThresholdScale<T> {
    thresholds: Vec<f64>,
    range: Vec<T>,  // len = thresholds.len() + 1
}
```

### 1.5 Axis Component (P0)

**Priority**: P0 - Critical
**Effort**: 2 weeks
**Dependencies**: All scales

#### Requirements

- Support all four orientations (top, right, bottom, left)
- Configurable tick marks and labels
- Custom tick formatting
- Grid line generation
- Animated transitions

#### API Design

```rust
pub enum AxisOrientation {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct Axis<S: Scale> {
    scale: S,
    orientation: AxisOrientation,
    tick_size_inner: f64,
    tick_size_outer: f64,
    tick_padding: f64,
    tick_values: Option<Vec<S::Domain>>,
    tick_format: Option<Box<dyn Fn(&S::Domain) -> String>>,
    tick_count: usize,
}

impl<S: Scale> Axis<S> {
    pub fn new(scale: S, orientation: AxisOrientation) -> Self;

    /// Set number of ticks (approximate)
    pub fn ticks(self, count: usize) -> Self;

    /// Set explicit tick values
    pub fn tick_values(self, values: Vec<S::Domain>) -> Self;

    /// Set tick format function
    pub fn tick_format<F>(self, format: F) -> Self
    where F: Fn(&S::Domain) -> String + 'static;

    /// Set inner tick size
    pub fn tick_size_inner(self, size: f64) -> Self;

    /// Set outer tick size (domain line ends)
    pub fn tick_size_outer(self, size: f64) -> Self;

    /// Set padding between tick and label
    pub fn tick_padding(self, padding: f64) -> Self;

    /// Generate axis elements for rendering
    pub fn generate(&self) -> AxisElements;
}

pub struct AxisElements {
    pub domain_path: BezPath,
    pub ticks: Vec<TickElement>,
}

pub struct TickElement {
    pub position: f64,
    pub line_start: DVec2,
    pub line_end: DVec2,
    pub label_position: DVec2,
    pub label_text: String,
    pub label_anchor: TextAnchor,
}
```

#### Implementation Tasks

| Task | Effort | Priority |
|------|--------|----------|
| Axis struct and builder | 2 days | P0 |
| Tick calculation | 2 days | P0 |
| Domain path generation | 1 day | P0 |
| Label positioning | 2 days | P0 |
| Text anchor calculation | 1 day | P0 |
| Makepad widget wrapper | 2 days | P0 |
| Grid line generation | 1 day | P1 |
| Custom formatters | 1 day | P1 |
| Unit tests | 2 days | P0 |

### 1.6 Number Formatting (P1)

**Priority**: P1 - High
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
/// Format specifier similar to D3's format
pub struct NumberFormat {
    fill: char,
    align: Align,
    sign: Sign,
    symbol: Symbol,
    zero: bool,
    width: Option<usize>,
    comma: bool,
    precision: Option<usize>,
    trim: bool,
    type_: FormatType,
}

impl NumberFormat {
    /// Parse format specifier string (e.g., ",.2f", "$,.2s", "+.0%")
    pub fn parse(specifier: &str) -> Result<Self, FormatError>;

    /// Format a number
    pub fn format(&self, value: f64) -> String;
}

pub enum FormatType {
    Exponent,      // e - exponent notation
    Fixed,         // f - fixed point
    General,       // g - either fixed or exponent
    Rounded,       // r - decimal notation, rounded
    SiPrefix,      // s - SI prefix (k, M, G, etc.)
    Percentage,    // % - multiply by 100, add %
    Binary,        // b - binary
    Octal,         // o - octal
    Hex,           // x - lowercase hex
    HexUpper,      // X - uppercase hex
    Unicode,       // c - unicode character
}

/// Convenience function
pub fn format(specifier: &str) -> impl Fn(f64) -> String {
    let fmt = NumberFormat::parse(specifier).unwrap();
    move |value| fmt.format(value)
}
```

### Phase 1 Milestones

| Milestone | Week | Deliverables |
|-----------|------|--------------|
| M1.1 | 2 | TimeScale with tests |
| M1.2 | 3 | LogScale, PowScale |
| M1.3 | 4 | Quantize/Quantile/Threshold scales |
| M1.4 | 5 | Axis component |
| M1.5 | 6 | Number formatting, integration tests |

### Phase 1 Exit Criteria

- [ ] All continuous scales implemented with >90% test coverage
- [ ] Axis renders correctly for all orientations
- [ ] Time series chart example working
- [ ] Performance: 60fps with 10,000 data points

---

## Phase 2: Core Visualization

**Duration**: 4-6 weeks
**Goal**: Shape generators, color schemes, and essential components

### 2.1 Stack Generator (P1)

**Priority**: P1 - High
**Effort**: 1.5 weeks
**Dependencies**: None

#### Requirements

- Stack data for stacked bar/area charts
- Multiple stacking orders (none, ascending, descending, insideOut)
- Multiple offset modes (none, expand, diverging, silhouette, wiggle)

#### API Design

```rust
pub struct Stack<T> {
    keys: Vec<String>,
    value: Box<dyn Fn(&T, &str) -> f64>,
    order: StackOrder,
    offset: StackOffset,
}

pub enum StackOrder {
    None,           // Use key order
    Ascending,      // Smallest sum first
    Descending,     // Largest sum first
    InsideOut,      // Larger series in middle
    Reverse,        // Reverse of none
}

pub enum StackOffset {
    None,           // Zero baseline
    Expand,         // Normalize to 0-1
    Diverging,      // Positive above, negative below zero
    Silhouette,     // Center around zero
    Wiggle,         // Minimize weighted wiggle (streamgraph)
}

impl<T> Stack<T> {
    pub fn new() -> Self;
    pub fn keys(self, keys: Vec<String>) -> Self;
    pub fn value<F>(self, accessor: F) -> Self
    where F: Fn(&T, &str) -> f64 + 'static;
    pub fn order(self, order: StackOrder) -> Self;
    pub fn offset(self, offset: StackOffset) -> Self;

    /// Generate stacked series
    pub fn stack(&self, data: &[T]) -> Vec<StackSeries>;
}

pub struct StackSeries {
    pub key: String,
    pub points: Vec<StackPoint>,
}

pub struct StackPoint {
    pub index: usize,
    pub lower: f64,  // y0
    pub upper: f64,  // y1
}
```

### 2.2 Symbol Generator (P2)

**Priority**: P2 - Medium
**Effort**: 1 week
**Dependencies**: Kurbo (optional)

#### API Design

```rust
pub enum SymbolType {
    Circle,
    Cross,
    Diamond,
    Square,
    Star,
    Triangle,
    Wye,
    Custom(Box<dyn Fn(f64) -> BezPath>),
}

pub struct Symbol {
    symbol_type: SymbolType,
    size: f64,  // Area in square pixels
}

impl Symbol {
    pub fn new(symbol_type: SymbolType) -> Self;
    pub fn size(self, size: f64) -> Self;

    /// Generate path for symbol at origin
    pub fn generate(&self) -> BezPath;

    /// Generate path at specific position
    pub fn generate_at(&self, x: f64, y: f64) -> BezPath;
}
```

### 2.3 Curve Interpolators (P1)

**Priority**: P1 - High
**Effort**: 2 weeks
**Dependencies**: None

#### API Design

```rust
pub trait Curve {
    /// Start a new line
    fn line_start(&mut self);

    /// End the current line
    fn line_end(&mut self);

    /// Add a point to the curve
    fn point(&mut self, x: f64, y: f64);

    /// Get the resulting path
    fn result(&self) -> BezPath;
}

// Implementations
pub struct CurveLinear { ... }
pub struct CurveStep { step_position: f64 } // 0.0=before, 0.5=middle, 1.0=after
pub struct CurveBasis { ... }
pub struct CurveBasisClosed { ... }
pub struct CurveBundle { beta: f64 }
pub struct CurveCardinal { tension: f64 }
pub struct CurveCardinalClosed { tension: f64 }
pub struct CurveCatmullRom { alpha: f64 }
pub struct CurveCatmullRomClosed { alpha: f64 }
pub struct CurveMonotoneX { ... }
pub struct CurveMonotoneY { ... }
pub struct CurveNatural { ... }
```

#### Implementation Priority

| Curve | Priority | Use Case |
|-------|----------|----------|
| Linear | P0 | Basic lines |
| Step | P1 | Step charts |
| CatmullRom | P1 | Smooth lines (already in makepad-chart) |
| Monotone | P1 | No overshoot (already in makepad-chart) |
| Cardinal | P2 | Configurable tension |
| Basis | P2 | B-spline smoothing |
| Natural | P2 | Natural cubic spline |
| Bundle | P3 | Hierarchical edge bundling |

### 2.4 Color Schemes (P1)

**Priority**: P1 - High
**Effort**: 1.5 weeks
**Dependencies**: Linebender Color (optional)

#### API Design

```rust
/// Categorical color schemes
pub mod categorical {
    pub fn category10() -> [Color; 10];
    pub fn accent() -> [Color; 8];
    pub fn dark2() -> [Color; 8];
    pub fn paired() -> [Color; 12];
    pub fn pastel1() -> [Color; 9];
    pub fn pastel2() -> [Color; 8];
    pub fn set1() -> [Color; 9];
    pub fn set2() -> [Color; 8];
    pub fn set3() -> [Color; 12];
    pub fn tableau10() -> [Color; 10];
}

/// Sequential color schemes (single hue)
pub mod sequential {
    pub fn blues(t: f64) -> Color;
    pub fn greens(t: f64) -> Color;
    pub fn greys(t: f64) -> Color;
    pub fn oranges(t: f64) -> Color;
    pub fn purples(t: f64) -> Color;
    pub fn reds(t: f64) -> Color;
}

/// Sequential color schemes (multi hue)
pub mod sequential_multi {
    pub fn viridis(t: f64) -> Color;
    pub fn inferno(t: f64) -> Color;
    pub fn magma(t: f64) -> Color;
    pub fn plasma(t: f64) -> Color;
    pub fn cividis(t: f64) -> Color;
    pub fn turbo(t: f64) -> Color;
    pub fn warm(t: f64) -> Color;
    pub fn cool(t: f64) -> Color;
}

/// Diverging color schemes
pub mod diverging {
    pub fn brbg(t: f64) -> Color;    // Brown-Blue-Green
    pub fn prgn(t: f64) -> Color;    // Purple-Green
    pub fn piyg(t: f64) -> Color;    // Pink-Yellow-Green
    pub fn puor(t: f64) -> Color;    // Purple-Orange
    pub fn rdbu(t: f64) -> Color;    // Red-Blue
    pub fn rdgy(t: f64) -> Color;    // Red-Grey
    pub fn rdylbu(t: f64) -> Color;  // Red-Yellow-Blue
    pub fn rdylgn(t: f64) -> Color;  // Red-Yellow-Green
    pub fn spectral(t: f64) -> Color;
}

/// Create interpolator for color scale
pub fn scale_sequential<F>(scheme: F) -> SequentialScale
where F: Fn(f64) -> Color + 'static;

pub fn scale_diverging<F>(scheme: F) -> DivergingScale
where F: Fn(f64) -> Color + 'static;
```

### 2.5 Tooltip Component (P0)

**Priority**: P0 - Critical
**Effort**: 1.5 weeks
**Dependencies**: None

#### Requirements

- Auto-positioning (flip when near edges)
- Multiple content types (text, key-value, custom)
- Follow cursor or anchor to data point
- Smooth show/hide transitions
- Customizable styling

#### API Design

```rust
#[derive(Live, LiveHook, Widget)]
pub struct Tooltip {
    #[live] view: View,
    #[live] draw_bg: DrawQuad,
    #[live] draw_text: DrawText,

    #[rust] visible: bool,
    #[rust] position: DVec2,
    #[rust] content: TooltipContent,
    #[rust] anchor: TooltipAnchor,
    #[rust] offset: DVec2,
}

pub enum TooltipContent {
    Text(String),
    KeyValue(Vec<(String, String)>),
    Custom(Box<dyn Fn(&mut Cx2d, Rect)>),
}

pub enum TooltipAnchor {
    Cursor,
    Point(DVec2),
    Element(WidgetRef),
}

impl Tooltip {
    pub fn show(&mut self, cx: &mut Cx, content: TooltipContent, anchor: TooltipAnchor);
    pub fn hide(&mut self, cx: &mut Cx);
    pub fn update_position(&mut self, cx: &mut Cx, cursor: DVec2);
}
```

### 2.6 Legend Component (P1)

**Priority**: P1 - High
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
#[derive(Live, LiveHook, Widget)]
pub struct Legend {
    #[live] view: View,
    #[live] draw_symbol: DrawQuad,
    #[live] draw_text: DrawText,

    #[rust] items: Vec<LegendItem>,
    #[rust] orientation: LegendOrientation,
    #[rust] interactive: bool,
}

pub struct LegendItem {
    pub key: String,
    pub label: String,
    pub color: Vec4,
    pub symbol: SymbolType,
    pub visible: bool,
}

pub enum LegendOrientation {
    Horizontal,
    Vertical,
}

impl Legend {
    pub fn set_items(&mut self, items: Vec<LegendItem>);
    pub fn toggle_item(&mut self, key: &str);
    pub fn is_visible(&self, key: &str) -> bool;
}

// Events
pub enum LegendAction {
    ItemClicked { key: String },
    ItemHovered { key: String },
}
```

### 2.7 Annotation Component (P2)

**Priority**: P2 - Medium
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
pub enum Annotation {
    /// Line annotation (horizontal, vertical, or diagonal)
    Line {
        start: AnnotationPoint,
        end: AnnotationPoint,
        stroke: Stroke,
        label: Option<AnnotationLabel>,
    },

    /// Rectangle annotation (highlight region)
    Rect {
        x: AnnotationRange,
        y: AnnotationRange,
        fill: Vec4,
        stroke: Option<Stroke>,
        label: Option<AnnotationLabel>,
    },

    /// Point annotation with callout
    Point {
        position: AnnotationPoint,
        symbol: SymbolType,
        label: AnnotationLabel,
    },

    /// Text annotation
    Text {
        position: AnnotationPoint,
        text: String,
        style: TextStyle,
    },
}

pub enum AnnotationPoint {
    Data(f64, f64),      // In data coordinates
    Pixel(f64, f64),     // In pixel coordinates
    Relative(f64, f64),  // 0-1 relative to chart area
}

pub enum AnnotationRange {
    Data(f64, f64),
    Pixel(f64, f64),
    Full,
}
```

### Phase 2 Milestones

| Milestone | Week | Deliverables |
|-----------|------|--------------|
| M2.1 | 2 | Stack generator, stacked bar chart example |
| M2.2 | 3 | Curve interpolators (Linear, Step, CatmullRom, Monotone) |
| M2.3 | 4 | Color schemes (categorical, sequential) |
| M2.4 | 5 | Tooltip component |
| M2.5 | 6 | Legend component, annotation component |

### Phase 2 Exit Criteria

- [ ] Stacked area/bar charts working
- [ ] All major curve types implemented
- [ ] Color schemes match D3's visual output
- [ ] Tooltip follows cursor with smart positioning
- [ ] Interactive legend toggles dataset visibility

---

## Phase 3: Interactivity

**Duration**: 6-8 weeks
**Goal**: Zoom, pan, brush, and advanced interactions

### 3.1 Zoom Behavior (P1)

**Priority**: P1 - High
**Effort**: 3 weeks
**Dependencies**: Affine transforms

#### Requirements

- Pan (translate) with mouse drag
- Zoom (scale) with mouse wheel
- Pinch-to-zoom on touch devices
- Configurable constraints (min/max scale, pan extent)
- Smooth animated transitions
- Programmatic zoom control

#### API Design

```rust
/// 2D transformation state
#[derive(Clone, Copy, Debug)]
pub struct ZoomTransform {
    pub k: f64,   // Scale factor
    pub x: f64,   // X translation
    pub y: f64,   // Y translation
}

impl ZoomTransform {
    pub const IDENTITY: Self = Self { k: 1.0, x: 0.0, y: 0.0 };

    /// Apply transform to a point
    pub fn apply(&self, point: DVec2) -> DVec2;

    /// Inverse transform
    pub fn invert(&self, point: DVec2) -> DVec2;

    /// Rescale a scale's range
    pub fn rescale_x<S: Scale>(&self, scale: &S) -> S;
    pub fn rescale_y<S: Scale>(&self, scale: &S) -> S;

    /// Compose transforms
    pub fn then(&self, other: &ZoomTransform) -> ZoomTransform;

    /// Convert to Affine matrix
    pub fn to_affine(&self) -> Affine;
}

pub struct ZoomBehavior {
    /// Scale extent [min, max]
    scale_extent: (f64, f64),

    /// Translate extent [[x0, y0], [x1, y1]]
    translate_extent: Option<(DVec2, DVec2)>,

    /// Current transform
    transform: ZoomTransform,

    /// Gesture state
    gesture: Option<ZoomGesture>,

    /// Event callbacks
    on_start: Option<Box<dyn Fn(&ZoomTransform)>>,
    on_zoom: Option<Box<dyn Fn(&ZoomTransform)>>,
    on_end: Option<Box<dyn Fn(&ZoomTransform)>>,
}

impl ZoomBehavior {
    pub fn new() -> Self;

    /// Set scale constraints
    pub fn scale_extent(self, min: f64, max: f64) -> Self;

    /// Set pan constraints
    pub fn translate_extent(self, extent: (DVec2, DVec2)) -> Self;

    /// Set callback for zoom start
    pub fn on_start<F>(self, callback: F) -> Self
    where F: Fn(&ZoomTransform) + 'static;

    /// Set callback for zoom change
    pub fn on_zoom<F>(self, callback: F) -> Self
    where F: Fn(&ZoomTransform) + 'static;

    /// Set callback for zoom end
    pub fn on_end<F>(self, callback: F) -> Self
    where F: Fn(&ZoomTransform) + 'static;

    /// Handle input event
    pub fn handle_event(&mut self, event: &Event, rect: Rect) -> bool;

    /// Get current transform
    pub fn transform(&self) -> &ZoomTransform;

    /// Programmatic zoom
    pub fn zoom_to(&mut self, transform: ZoomTransform, animated: bool);
    pub fn zoom_in(&mut self, factor: f64, center: DVec2);
    pub fn zoom_out(&mut self, factor: f64, center: DVec2);
    pub fn pan_to(&mut self, x: f64, y: f64, animated: bool);
    pub fn reset(&mut self, animated: bool);
}
```

#### Implementation Tasks

| Task | Effort | Priority |
|------|--------|----------|
| ZoomTransform struct | 2 days | P0 |
| Mouse wheel zoom | 3 days | P0 |
| Mouse drag pan | 2 days | P0 |
| Scale constraints | 2 days | P0 |
| Translate constraints | 2 days | P1 |
| Touch pinch zoom | 3 days | P1 |
| Animated transitions | 2 days | P1 |
| Programmatic API | 2 days | P1 |
| Integration tests | 2 days | P0 |

### 3.2 Brush Behavior (P2)

**Priority**: P2 - Medium
**Effort**: 3 weeks
**Dependencies**: None

#### Requirements

- 1D brush (X or Y axis selection)
- 2D brush (rectangular selection)
- Resize handles
- Clear selection
- Programmatic control
- Snapping to data points (optional)

#### API Design

```rust
pub enum BrushType {
    X,      // Horizontal selection
    Y,      // Vertical selection
    XY,     // 2D rectangular selection
}

pub struct BrushSelection {
    pub x0: Option<f64>,
    pub x1: Option<f64>,
    pub y0: Option<f64>,
    pub y1: Option<f64>,
}

impl BrushSelection {
    pub fn is_empty(&self) -> bool;
    pub fn contains(&self, x: f64, y: f64) -> bool;
    pub fn width(&self) -> Option<f64>;
    pub fn height(&self) -> Option<f64>;
}

pub struct BrushBehavior {
    brush_type: BrushType,
    extent: (DVec2, DVec2),  // Brushable area
    selection: Option<BrushSelection>,

    // Callbacks
    on_start: Option<Box<dyn Fn(&BrushSelection)>>,
    on_brush: Option<Box<dyn Fn(&BrushSelection)>>,
    on_end: Option<Box<dyn Fn(&BrushSelection)>>,
}

impl BrushBehavior {
    pub fn new(brush_type: BrushType) -> Self;
    pub fn extent(self, extent: (DVec2, DVec2)) -> Self;

    pub fn on_start<F>(self, callback: F) -> Self;
    pub fn on_brush<F>(self, callback: F) -> Self;
    pub fn on_end<F>(self, callback: F) -> Self;

    pub fn handle_event(&mut self, event: &Event, rect: Rect) -> bool;
    pub fn selection(&self) -> Option<&BrushSelection>;
    pub fn move_to(&mut self, selection: BrushSelection);
    pub fn clear(&mut self);
}
```

### 3.3 Crosshair Component (P2)

**Priority**: P2 - Medium
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
#[derive(Live, LiveHook, Widget)]
pub struct Crosshair {
    #[live] draw_line: DrawLine,
    #[live] draw_label_bg: DrawQuad,
    #[live] draw_label_text: DrawText,

    #[rust] position: Option<DVec2>,
    #[rust] show_x: bool,
    #[rust] show_y: bool,
    #[rust] snap_to_data: bool,
}

impl Crosshair {
    pub fn update(&mut self, cursor: DVec2, data_points: &[(f64, f64)]);
    pub fn set_format_x<F>(&mut self, format: F) where F: Fn(f64) -> String;
    pub fn set_format_y<F>(&mut self, format: F) where F: Fn(f64) -> String;
}
```

### 3.4 Hover/Selection State (P1)

**Priority**: P1 - High
**Effort**: 1 week
**Dependencies**: None

#### API Design

```rust
/// Manages hover and selection state for data points
pub struct DataState {
    hovered: Option<DataIndex>,
    selected: HashSet<DataIndex>,

    on_hover: Option<Box<dyn Fn(Option<&DataIndex>)>>,
    on_select: Option<Box<dyn Fn(&HashSet<DataIndex>)>>,
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct DataIndex {
    pub dataset: usize,
    pub index: usize,
}

impl DataState {
    pub fn set_hovered(&mut self, index: Option<DataIndex>);
    pub fn toggle_selected(&mut self, index: DataIndex);
    pub fn select(&mut self, index: DataIndex);
    pub fn deselect(&mut self, index: &DataIndex);
    pub fn clear_selection(&mut self);
    pub fn select_range(&mut self, start: DataIndex, end: DataIndex);
}
```

### Phase 3 Milestones

| Milestone | Week | Deliverables |
|-----------|------|--------------|
| M3.1 | 3 | Zoom behavior (wheel + drag) |
| M3.2 | 5 | Zoom constraints and animation |
| M3.3 | 6 | Touch support |
| M3.4 | 7 | Brush behavior |
| M3.5 | 8 | Crosshair, hover/selection state |

### Phase 3 Exit Criteria

- [ ] Zoom/pan works on all platforms (desktop, mobile, web)
- [ ] Brush selection filters data correctly
- [ ] Smooth 60fps during interactions
- [ ] Programmatic zoom API works with animations

---

## Phase 4: Advanced Layouts

**Duration**: 8-12 weeks
**Goal**: Force simulation and hierarchical layouts

### 4.1 Force Simulation (P2)

**Priority**: P2 - Medium
**Effort**: 6 weeks
**Dependencies**: None

#### Requirements

- Velocity Verlet integration
- Composable forces
- Alpha decay for settling
- Reheat capability
- Node fixing/unfixing
- Performance optimization for large graphs

#### API Design

```rust
/// A node in the force simulation
pub struct SimulationNode {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub fx: Option<f64>,  // Fixed x position
    pub fy: Option<f64>,  // Fixed y position
    pub data: Box<dyn Any>,
}

/// A link between nodes
pub struct SimulationLink {
    pub source: usize,
    pub target: usize,
    pub strength: Option<f64>,
    pub distance: Option<f64>,
}

/// Force simulation
pub struct ForceSimulation {
    nodes: Vec<SimulationNode>,
    forces: HashMap<String, Box<dyn Force>>,

    alpha: f64,
    alpha_min: f64,
    alpha_decay: f64,
    alpha_target: f64,
    velocity_decay: f64,

    on_tick: Option<Box<dyn Fn(&[SimulationNode])>>,
    on_end: Option<Box<dyn Fn(&[SimulationNode])>>,
}

impl ForceSimulation {
    pub fn new(nodes: Vec<SimulationNode>) -> Self;

    /// Add a force
    pub fn force(self, name: &str, force: impl Force + 'static) -> Self;

    /// Remove a force
    pub fn remove_force(&mut self, name: &str);

    /// Set alpha (simulation "temperature")
    pub fn alpha(self, alpha: f64) -> Self;
    pub fn alpha_min(self, alpha_min: f64) -> Self;
    pub fn alpha_decay(self, decay: f64) -> Self;
    pub fn alpha_target(self, target: f64) -> Self;

    /// Velocity decay (friction)
    pub fn velocity_decay(self, decay: f64) -> Self;

    /// Callbacks
    pub fn on_tick<F>(self, callback: F) -> Self;
    pub fn on_end<F>(self, callback: F) -> Self;

    /// Control
    pub fn tick(&mut self) -> f64;  // Returns alpha
    pub fn restart(&mut self);
    pub fn stop(&mut self);

    /// Find node at position
    pub fn find(&self, x: f64, y: f64, radius: Option<f64>) -> Option<&SimulationNode>;

    /// Fix/unfix nodes
    pub fn fix_node(&mut self, index: usize, x: f64, y: f64);
    pub fn unfix_node(&mut self, index: usize);
}

/// Force trait
pub trait Force: Send + Sync {
    fn initialize(&mut self, nodes: &[SimulationNode]);
    fn apply(&self, nodes: &mut [SimulationNode], alpha: f64);
}
```

#### Force Types

```rust
/// Center force - keeps nodes centered
pub struct ForceCenter {
    x: f64,
    y: f64,
    strength: f64,
}

/// Collision force - prevents overlap
pub struct ForceCollide {
    radius: Box<dyn Fn(&SimulationNode) -> f64>,
    strength: f64,
    iterations: usize,
}

/// Link force - maintains distances
pub struct ForceLink {
    links: Vec<SimulationLink>,
    distance: f64,
    strength: Option<f64>,
    iterations: usize,
}

/// Many-body force - attraction/repulsion
pub struct ForceManyBody {
    strength: f64,           // Negative = repulsion
    theta: f64,              // Barnes-Hut approximation
    distance_min: f64,
    distance_max: f64,
}

/// Position forces - attract to x/y positions
pub struct ForceX {
    x: Box<dyn Fn(&SimulationNode) -> f64>,
    strength: f64,
}

pub struct ForceY {
    y: Box<dyn Fn(&SimulationNode) -> f64>,
    strength: f64,
}

/// Radial force - attract to radius from center
pub struct ForceRadial {
    radius: f64,
    x: f64,
    y: f64,
    strength: f64,
}
```

#### Implementation Tasks

| Task | Effort | Priority |
|------|--------|----------|
| SimulationNode/Link structs | 1 day | P0 |
| Velocity Verlet integration | 3 days | P0 |
| Alpha decay system | 2 days | P0 |
| ForceCenter | 2 days | P0 |
| ForceLink | 3 days | P0 |
| ForceManyBody (naive) | 3 days | P0 |
| ForceCollide | 3 days | P1 |
| ForceX/Y/Radial | 2 days | P1 |
| Barnes-Hut optimization | 1 week | P2 |
| Node fixing | 2 days | P1 |
| Integration with Makepad | 3 days | P0 |
| Example: Network graph | 3 days | P0 |

### 4.2 Hierarchy Data Structure (P2)

**Priority**: P2 - Medium
**Effort**: 2 weeks
**Dependencies**: None

#### API Design

```rust
/// A node in the hierarchy
pub struct HierarchyNode<T> {
    pub data: T,
    pub depth: usize,
    pub height: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,

    // Layout properties (set by layout algorithms)
    pub x: f64,
    pub y: f64,
    pub value: Option<f64>,
}

/// Hierarchy container
pub struct Hierarchy<T> {
    nodes: Vec<HierarchyNode<T>>,
    root: usize,
}

impl<T> Hierarchy<T> {
    /// Create from nested data
    pub fn from_nested<F>(root: T, children: F) -> Self
    where F: Fn(&T) -> Vec<T>;

    /// Create from flat data with parent references
    pub fn stratify<I, K, F>(data: I, id: F, parent_id: F) -> Self
    where
        I: IntoIterator<Item = T>,
        K: Hash + Eq,
        F: Fn(&T) -> K;

    /// Get root node
    pub fn root(&self) -> &HierarchyNode<T>;

    /// Iterate in various orders
    pub fn each_before<F>(&self, callback: F);  // Pre-order
    pub fn each_after<F>(&self, callback: F);   // Post-order
    pub fn each_breadth<F>(&self, callback: F); // Breadth-first

    /// Get ancestors/descendants
    pub fn ancestors(&self, index: usize) -> Vec<usize>;
    pub fn descendants(&self, index: usize) -> Vec<usize>;
    pub fn leaves(&self) -> Vec<usize>;

    /// Get links (parent-child pairs)
    pub fn links(&self) -> Vec<(usize, usize)>;

    /// Calculate values
    pub fn sum<F>(&mut self, value: F) where F: Fn(&T) -> f64;
    pub fn count(&mut self);

    /// Sort children
    pub fn sort<F>(&mut self, compare: F) where F: Fn(&HierarchyNode<T>, &HierarchyNode<T>) -> Ordering;
}
```

### 4.3 Tree Layout (P2)

**Priority**: P2 - Medium
**Effort**: 2 weeks
**Dependencies**: Hierarchy

#### API Design

```rust
/// Tidy tree layout (Reingold-Tilford algorithm)
pub struct TreeLayout {
    size: Option<(f64, f64)>,
    node_size: Option<(f64, f64)>,
    separation: Box<dyn Fn(&HierarchyNode<Any>, &HierarchyNode<Any>) -> f64>,
}

impl TreeLayout {
    pub fn new() -> Self;

    /// Set tree size (scales to fit)
    pub fn size(self, width: f64, height: f64) -> Self;

    /// Set node size (fixed spacing)
    pub fn node_size(self, width: f64, height: f64) -> Self;

    /// Set separation function
    pub fn separation<F>(self, sep: F) -> Self
    where F: Fn(&HierarchyNode<Any>, &HierarchyNode<Any>) -> f64 + 'static;

    /// Apply layout to hierarchy
    pub fn layout<T>(&self, hierarchy: &mut Hierarchy<T>);
}

/// Cluster layout (dendrogram)
pub struct ClusterLayout {
    size: Option<(f64, f64)>,
    node_size: Option<(f64, f64)>,
    separation: Box<dyn Fn(&HierarchyNode<Any>, &HierarchyNode<Any>) -> f64>,
}
```

### 4.4 Treemap Layout (P2)

**Priority**: P2 - Medium
**Effort**: 2 weeks
**Dependencies**: Hierarchy

#### API Design

```rust
pub enum TreemapTiling {
    Binary,
    Dice,
    Slice,
    SliceDice,
    Squarify { ratio: f64 },
    Resquarify { ratio: f64 },
}

pub struct TreemapLayout {
    size: (f64, f64),
    tile: TreemapTiling,
    round: bool,
    padding: TreemapPadding,
}

pub enum TreemapPadding {
    Uniform(f64),
    Inner(f64),
    Outer(f64),
    Top(f64),
    Custom(Box<dyn Fn(&HierarchyNode<Any>) -> (f64, f64, f64, f64)>),
}

impl TreemapLayout {
    pub fn new() -> Self;
    pub fn size(self, width: f64, height: f64) -> Self;
    pub fn tile(self, tile: TreemapTiling) -> Self;
    pub fn padding(self, padding: TreemapPadding) -> Self;
    pub fn round(self, round: bool) -> Self;

    pub fn layout<T>(&self, hierarchy: &mut Hierarchy<T>);
}
```

### 4.5 Pack Layout (P3)

**Priority**: P3 - Low
**Effort**: 2 weeks
**Dependencies**: Hierarchy

#### API Design

```rust
/// Circle packing layout
pub struct PackLayout {
    size: (f64, f64),
    padding: Box<dyn Fn(&HierarchyNode<Any>) -> f64>,
    radius: Option<Box<dyn Fn(&HierarchyNode<Any>) -> f64>>,
}

impl PackLayout {
    pub fn new() -> Self;
    pub fn size(self, width: f64, height: f64) -> Self;
    pub fn padding<F>(self, padding: F) -> Self;
    pub fn radius<F>(self, radius: F) -> Self;

    pub fn layout<T>(&self, hierarchy: &mut Hierarchy<T>);
}

// After layout, nodes have:
// - x, y: center position
// - r: radius
```

### 4.6 Partition Layout (P3)

**Priority**: P3 - Low
**Effort**: 1 week
**Dependencies**: Hierarchy

#### API Design

```rust
/// Partition layout (icicle/sunburst)
pub struct PartitionLayout {
    size: (f64, f64),
    round: bool,
    padding: f64,
}

impl PartitionLayout {
    pub fn new() -> Self;
    pub fn size(self, width: f64, height: f64) -> Self;
    pub fn round(self, round: bool) -> Self;
    pub fn padding(self, padding: f64) -> Self;

    pub fn layout<T>(&self, hierarchy: &mut Hierarchy<T>);
}

// After layout, nodes have:
// - x0, y0, x1, y1: rectangular bounds
// For sunburst, use polar transform
```

### Phase 4 Milestones

| Milestone | Week | Deliverables |
|-----------|------|--------------|
| M4.1 | 2 | Hierarchy data structure |
| M4.2 | 5 | Force simulation (basic forces) |
| M4.3 | 7 | Tree and cluster layouts |
| M4.4 | 9 | Treemap layout |
| M4.5 | 11 | Pack layout |
| M4.6 | 12 | Partition layout, examples |

### Phase 4 Exit Criteria

- [ ] Force-directed graph renders correctly
- [ ] Force simulation settles in reasonable time
- [ ] All hierarchy layouts match D3 output
- [ ] Examples for each layout type

---

## Phase 5: Geographic

**Duration**: 12-16 weeks
**Goal**: Map projections and geographic visualization

### 5.1 Projections (P3)

**Priority**: P3 - Low
**Effort**: 8 weeks
**Dependencies**: None

#### Core Projections

| Projection | Type | Priority |
|------------|------|----------|
| Mercator | Cylindrical | P0 |
| Equirectangular | Cylindrical | P0 |
| Orthographic | Azimuthal | P1 |
| Stereographic | Azimuthal | P2 |
| Albers | Conic | P1 |
| AlbersUsa | Composite | P2 |
| ConicEqualArea | Conic | P2 |
| Gnomonic | Azimuthal | P3 |

#### API Design

```rust
pub trait Projection {
    /// Project a geographic coordinate to screen
    fn project(&self, lon: f64, lat: f64) -> Option<(f64, f64)>;

    /// Inverse projection
    fn invert(&self, x: f64, y: f64) -> Option<(f64, f64)>;

    /// Get clip extent
    fn clip_extent(&self) -> Option<((f64, f64), (f64, f64))>;
}

pub struct MercatorProjection {
    center: (f64, f64),
    scale: f64,
    translate: (f64, f64),
    clip_extent: Option<((f64, f64), (f64, f64))>,
    precision: f64,
}

impl MercatorProjection {
    pub fn new() -> Self;
    pub fn center(self, lon: f64, lat: f64) -> Self;
    pub fn scale(self, scale: f64) -> Self;
    pub fn translate(self, x: f64, y: f64) -> Self;
    pub fn fit_size(self, size: (f64, f64), geojson: &GeoJson) -> Self;
    pub fn fit_extent(self, extent: ((f64, f64), (f64, f64)), geojson: &GeoJson) -> Self;
}
```

### 5.2 GeoJSON Parser (P3)

**Priority**: P3 - Low
**Effort**: 2 weeks
**Dependencies**: None

#### API Design

```rust
pub enum GeoJson {
    Point(Coordinate),
    MultiPoint(Vec<Coordinate>),
    LineString(Vec<Coordinate>),
    MultiLineString(Vec<Vec<Coordinate>>),
    Polygon(Vec<Vec<Coordinate>>),
    MultiPolygon(Vec<Vec<Vec<Coordinate>>>),
    GeometryCollection(Vec<GeoJson>),
    Feature {
        geometry: Option<Box<GeoJson>>,
        properties: HashMap<String, Value>,
        id: Option<Value>,
    },
    FeatureCollection(Vec<GeoJson>),
}

pub type Coordinate = (f64, f64);  // [lon, lat]

impl GeoJson {
    pub fn parse(json: &str) -> Result<Self, GeoJsonError>;
    pub fn bounds(&self) -> ((f64, f64), (f64, f64));
    pub fn centroid(&self) -> Coordinate;
}
```

### 5.3 Path Generator (P3)

**Priority**: P3 - Low
**Effort**: 3 weeks
**Dependencies**: Projections, GeoJSON

#### API Design

```rust
pub struct GeoPath<P: Projection> {
    projection: P,
    context: Option<PathContext>,
    point_radius: f64,
}

impl<P: Projection> GeoPath<P> {
    pub fn new(projection: P) -> Self;

    /// Generate path for geometry
    pub fn path(&self, geojson: &GeoJson) -> BezPath;

    /// Calculate area
    pub fn area(&self, geojson: &GeoJson) -> f64;

    /// Calculate centroid
    pub fn centroid(&self, geojson: &GeoJson) -> (f64, f64);

    /// Calculate bounds
    pub fn bounds(&self, geojson: &GeoJson) -> ((f64, f64), (f64, f64));

    /// Check if point is inside
    pub fn contains(&self, geojson: &GeoJson, point: (f64, f64)) -> bool;
}
```

### Phase 5 Milestones

| Milestone | Week | Deliverables |
|-----------|------|--------------|
| M5.1 | 3 | Mercator, Equirectangular projections |
| M5.2 | 5 | GeoJSON parser |
| M5.3 | 8 | Path generator |
| M5.4 | 10 | Orthographic, Albers projections |
| M5.5 | 13 | Additional projections |
| M5.6 | 16 | Choropleth example |

---

## Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          DEPENDENCY GRAPH                                │
└─────────────────────────────────────────────────────────────────────────┘

Phase 1 (Foundation)
═══════════════════
┌──────────┐   ┌──────────┐   ┌──────────┐
│TimeScale │   │ LogScale │   │ PowScale │
└────┬─────┘   └────┬─────┘   └────┬─────┘
     │              │              │
     └──────────────┼──────────────┘
                    │
                    ▼
            ┌───────────────┐
            │  Axis (P0)    │◄─────────────┐
            └───────┬───────┘              │
                    │                      │
                    ▼                      │
            ┌───────────────┐              │
            │NumberFormat(P1)│              │
            └───────────────┘              │
                                           │
Phase 2 (Visualization)                    │
═══════════════════════                    │
┌──────────┐   ┌──────────┐               │
│  Stack   │   │  Curves  │               │
│Generator │   │  (P1)    │               │
└────┬─────┘   └────┬─────┘               │
     │              │                      │
     └──────┬───────┘                      │
            │                              │
            ▼                              │
┌───────────────────┐   ┌──────────────┐  │
│  Color Schemes    │   │   Tooltip    │──┘
│      (P1)         │   │    (P0)      │
└─────────┬─────────┘   └──────┬───────┘
          │                    │
          └────────┬───────────┘
                   │
                   ▼
          ┌───────────────┐   ┌──────────────┐
          │    Legend     │   │  Annotation  │
          │    (P1)       │   │    (P2)      │
          └───────────────┘   └──────────────┘

Phase 3 (Interactivity)
═══════════════════════
┌──────────────────────────────────────────┐
│           ZoomTransform (P1)              │
└──────────────────┬───────────────────────┘
                   │
        ┌──────────┴──────────┐
        │                     │
        ▼                     ▼
┌───────────────┐    ┌───────────────┐
│ ZoomBehavior  │    │BrushBehavior  │
│    (P1)       │    │    (P2)       │
└───────┬───────┘    └───────┬───────┘
        │                    │
        └────────┬───────────┘
                 │
                 ▼
        ┌───────────────┐
        │  Crosshair    │
        │    (P2)       │
        └───────────────┘

Phase 4 (Layouts)
═════════════════
┌─────────────────────────────────────────┐
│           Hierarchy (P2)                 │
└──────────────────┬──────────────────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
    ▼              ▼              ▼
┌────────┐   ┌──────────┐   ┌────────┐
│  Tree  │   │ Treemap  │   │  Pack  │
│ (P2)   │   │  (P2)    │   │  (P3)  │
└────────┘   └──────────┘   └────────┘

┌─────────────────────────────────────────┐
│        ForceSimulation (P2)              │
└──────────────────┬──────────────────────┘
                   │
    ┌────────┬─────┴─────┬────────┐
    │        │           │        │
    ▼        ▼           ▼        ▼
┌──────┐ ┌──────┐   ┌──────┐ ┌──────┐
│Center│ │ Link │   │Many- │ │Collide│
│Force │ │Force │   │Body  │ │Force │
└──────┘ └──────┘   └──────┘ └──────┘

Phase 5 (Geographic)
════════════════════
┌─────────────────────────────────────────┐
│           Projections (P3)               │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│           GeoJSON Parser (P3)            │
└──────────────────┬──────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│           GeoPath Generator (P3)         │
└─────────────────────────────────────────┘
```

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Force simulation performance | Medium | High | Use Barnes-Hut approximation, WebGPU compute |
| Touch gesture conflicts | Medium | Medium | Careful event handling, clear gesture states |
| Complex curve math | Low | Medium | Use proven Kurbo library |
| Geographic projection accuracy | Medium | Low | Test against D3 output |
| Large dataset rendering | Medium | High | Implement data decimation, LOD |

### Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep | High | High | Strict P0/P1 prioritization |
| Integration issues | Medium | Medium | Early integration testing |
| Unexpected complexity | Medium | Medium | Buffer time in estimates |
| Dependency on makepad changes | Low | High | Abstract Makepad specifics |

### Mitigation Strategies

1. **Early prototyping**: Build minimal versions of risky components first
2. **Continuous integration**: Automated tests for each commit
3. **Weekly checkpoints**: Review progress against milestones
4. **Fallback plans**: Simpler implementations for complex features

---

## Success Metrics

### Phase 1 Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Scale coverage | 8+ scale types | Feature count |
| Test coverage | >90% | Code coverage tool |
| API consistency | 100% | Code review |
| Documentation | Complete | All public APIs documented |

### Phase 2 Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Chart types enhanced | All 11 | Feature verification |
| Color scheme parity | Match D3 | Visual comparison |
| Tooltip usability | <100ms response | Performance profiling |

### Phase 3 Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Zoom smoothness | 60fps | Frame timing |
| Touch support | All gestures work | Device testing |
| Brush accuracy | ±1px | Visual testing |

### Phase 4 Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Force simulation speed | 1000 nodes @ 60fps | Performance testing |
| Layout accuracy | Match D3 output | Visual comparison |
| API usability | Positive feedback | User testing |

### Overall Success Criteria

| Metric | Target |
|--------|--------|
| D3 feature parity | 80%+ for common use cases |
| Performance | 60fps with 10K data points |
| Code quality | <5 clippy warnings |
| Documentation | 100% public API coverage |
| Examples | 20+ working examples |

---

## Timeline Summary

```
2026
───────────────────────────────────────────────────────────────────────────
│ Jan │ Feb │ Mar │ Apr │ May │ Jun │ Jul │ Aug │ Sep │ Oct │ Nov │ Dec │
───────────────────────────────────────────────────────────────────────────
│◄─────── Phase 1 ───────►│                                              │
│       Foundation        │                                              │
│    (Scales, Axes)       │                                              │
│                         │◄─────── Phase 2 ───────►│                    │
│                         │    Core Visualization   │                    │
│                         │ (Shapes, Colors, UI)    │                    │
│                                                   │◄── Phase 3 ──►│    │
│                                                   │ Interactivity │    │
│                                                   │(Zoom, Brush)  │    │
│                                                                   │◄───
│                                                                   │Ph 4
───────────────────────────────────────────────────────────────────────────
2027
───────────────────────────────────────────────────────────────────────────
│ Jan │ Feb │ Mar │ Apr │ May │ Jun │
───────────────────────────────────────────────────────────────────────────
 ─►│                              │
   │◄────── Phase 4 ─────────────►│
   │    Advanced Layouts          │
   │   (Force, Hierarchy)         │
                                  │◄────── Phase 5 ──────────────────────►
                                  │       Geographic
                                  │   (Projections, GeoJSON)
───────────────────────────────────────────────────────────────────────────
```

---

## Getting Started

### Prerequisites

```toml
# Cargo.toml
[dependencies]
makepad-widgets = "0.6"
kurbo = "0.10"           # Geometry (optional)
color = "0.3"            # Color science (optional)
chrono = "0.4"           # Date/time for TimeScale
serde = "1.0"            # For GeoJSON parsing
serde_json = "1.0"
```

### First Steps

1. **Fork makepad-chart**: Start from existing implementation
2. **Set up CI**: Automated testing and linting
3. **Create project structure**: As outlined above
4. **Implement TimeScale**: First P0 deliverable
5. **Build integration test**: Verify with makepad-chart

### Team Structure (Recommended)

| Role | Focus Area | Phase |
|------|------------|-------|
| Lead Developer | Architecture, core scales | 1-2 |
| Graphics Developer | Shapes, rendering | 2 |
| Interaction Developer | Zoom, brush, events | 3 |
| Algorithm Developer | Force, hierarchy | 4 |
| GIS Developer | Geographic | 5 |

---

*Development Plan v1.0 - January 2026*
