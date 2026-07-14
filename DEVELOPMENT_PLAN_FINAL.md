# Makepad D3 Visualization Library - Final Development Plan

**Version:** 2.0 (Merged)
**Created:** 2026-01-15
**Status:** Partially superseded — see revision notice below

---

> ## ⚠️ Revision notice (2026-07-13): Makepad 2.0 "Splash" migration
>
> Makepad's `dev` branch shipped **2.0**, which removes the Live system
> (`live_design!`, `#[derive(Live, LiveHook, ...)]`) that this plan's widget
> layer was designed against, replacing it with the **Script/Splash system**
> (`script_mod!`, `#[derive(Script, ScriptHook, Widget)]`, runtime Splash DSL
> apps, `DrawVector` GPU vector paths).
>
> The updated design lives in
> **[`docs/SPLASH_INTEGRATION_DESIGN.md`](docs/SPLASH_INTEGRATION_DESIGN.md)**.
> It defines how makepad-d3 registers a `d3.*` widget namespace into the
> Splash VM so charts can be used inside Splash apps (including AI-chat
> `runsplash` mini-apps), the script-facing data/event contract, the
> `render3d` shader port, and the phased migration.
>
> **Update 2026-07-13:** the migration was executed (design doc §14). The
> library now builds on makepad 2.0; `d3.*` charts work directly in Splash
> DSL (`cargo run --example splash_demo`). The pre-2.0 chart_zoo example is
> the remaining porting backlog (Phase 4).
>
> How that affects this document:
> - **Still valid:** the d3 math core roadmap — scales, axes, shapes, colors,
>   layouts, geo (Phases 1–7 below). That code is pure Rust and unaffected.
> - **Superseded:** every widget/rendering/`live_design!` code sample and the
>   app-integration sections; follow the Splash design doc instead.
> - **Dependency:** makepad 2.0 is consumed as a **sibling path dependency**
>   (`../makepad`, dev branch) — its repo vendors a pre-2.0 crate copy under
>   `old/`, which makes git dependencies ambiguous (design doc §14).

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Project Structure](#2-project-structure)
3. [Priority & Effort Framework](#3-priority--effort-framework)
4. [Phase 0: Foundation (Week 1-2)](#4-phase-0-foundation-week-1-2)
5. [Phase 1: Core Scales (Week 3-6)](#5-phase-1-core-scales-week-3-6)
6. [Phase 2: Axis System (Week 7-10)](#6-phase-2-axis-system-week-7-10)
7. [Phase 3: Shape Generators (Week 11-16)](#7-phase-3-shape-generators-week-11-16)
8. [Phase 4: Color System (Week 17-18)](#8-phase-4-color-system-week-17-18)
9. [Phase 5: Interactions (Week 19-26)](#9-phase-5-interactions-week-19-26)
10. [Phase 6: Layout Algorithms (Week 27-40)](#10-phase-6-layout-algorithms-week-27-40)
11. [Phase 7: Geographic (Week 41-52)](#11-phase-7-geographic-week-41-52)
12. [MVP Definition](#12-mvp-definition)
13. [Quality Gates](#13-quality-gates)
14. [Risk Management](#14-risk-management)
15. [Appendix](#15-appendix)

---

## 1. Executive Summary

### Vision
Build a comprehensive D3.js-compatible data visualization library for Makepad, leveraging Rust's performance and Makepad's GPU acceleration.

### Goals

| Milestone | Timeline | Deliverables |
|-----------|----------|--------------|
| **MVP** | Week 10 | Time series charts, axes, basic interactivity |
| **v1.0** | Week 26 | Full chart library with zoom/pan/brush |
| **v2.0** | Week 40 | Network graphs, hierarchical visualizations |
| **v3.0** | Week 52 | Geographic projections |

### Success Criteria
- 80% D3.js API parity for common visualizations
- 60fps rendering with 10,000 data points
- <50ms initial render time
- 100% public API documentation
- >80% test coverage

---

## 2. Project Structure

### Repository Layout

```
makepad-d3/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── docs/
│   ├── api/                    # Generated API docs
│   ├── guides/                 # User guides
│   │   ├── getting-started.md
│   │   ├── scales.md
│   │   ├── axes.md
│   │   ├── shapes.md
│   │   └── interactions.md
│   └── examples/               # Example documentation
├── src/
│   ├── lib.rs                  # Public API exports
│   ├── prelude.rs              # Common imports
│   ├── error.rs                # Error types
│   ├── data/
│   │   ├── mod.rs
│   │   ├── point.rs            # DataPoint
│   │   ├── dataset.rs          # Dataset
│   │   └── chart_data.rs       # ChartData
│   ├── scale/
│   │   ├── mod.rs
│   │   ├── traits.rs           # Scale trait
│   │   ├── linear.rs
│   │   ├── log.rs
│   │   ├── pow.rs
│   │   ├── symlog.rs
│   │   ├── time.rs
│   │   ├── category.rs
│   │   ├── band.rs
│   │   ├── point.rs
│   │   ├── quantize.rs
│   │   ├── quantile.rs
│   │   ├── threshold.rs
│   │   └── sequential.rs
│   ├── axis/
│   │   ├── mod.rs
│   │   ├── axis.rs             # Axis widget
│   │   ├── orientation.rs
│   │   ├── tick.rs
│   │   ├── format.rs           # Number/date formatting
│   │   └── grid.rs
│   ├── shape/
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
│   │       ├── bundle.rs
│   │       ├── cardinal.rs
│   │       ├── catmull_rom.rs
│   │       ├── monotone.rs
│   │       └── natural.rs
│   ├── color/
│   │   ├── mod.rs
│   │   ├── scale.rs
│   │   ├── interpolate.rs
│   │   └── schemes/
│   │       ├── mod.rs
│   │       ├── categorical.rs
│   │       ├── sequential.rs
│   │       └── diverging.rs
│   ├── component/
│   │   ├── mod.rs
│   │   ├── tooltip.rs
│   │   ├── legend.rs
│   │   ├── annotation.rs
│   │   └── crosshair.rs
│   ├── interaction/
│   │   ├── mod.rs
│   │   ├── zoom.rs
│   │   ├── brush.rs
│   │   ├── drag.rs
│   │   └── hit_test.rs
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── hierarchy.rs
│   │   ├── tree.rs
│   │   ├── cluster.rs
│   │   ├── treemap.rs
│   │   ├── pack.rs
│   │   ├── partition.rs
│   │   └── force/
│   │       ├── mod.rs
│   │       ├── simulation.rs
│   │       ├── center.rs
│   │       ├── collide.rs
│   │       ├── link.rs
│   │       ├── many_body.rs
│   │       └── position.rs
│   └── geo/
│       ├── mod.rs
│       ├── projection/
│       │   ├── mod.rs
│       │   ├── mercator.rs
│       │   ├── equirectangular.rs
│       │   ├── orthographic.rs
│       │   └── albers.rs
│       ├── path.rs
│       └── geojson.rs
├── examples/
│   ├── basic/
│   │   ├── line_chart.rs
│   │   ├── bar_chart.rs
│   │   ├── scatter_chart.rs
│   │   └── pie_chart.rs
│   ├── time_series/
│   │   ├── stock_chart.rs
│   │   └── multi_series.rs
│   ├── interactive/
│   │   ├── zoom_pan.rs
│   │   ├── brush_select.rs
│   │   └── linked_charts.rs
│   ├── advanced/
│   │   ├── force_graph.rs
│   │   ├── treemap.rs
│   │   └── sunburst.rs
│   └── geographic/
│       ├── world_map.rs
│       └── choropleth.rs
├── tests/
│   ├── integration/
│   ├── visual/                 # Visual regression tests
│   └── performance/
└── benches/
    ├── scale_bench.rs
    ├── render_bench.rs
    └── layout_bench.rs
```

### Dependencies

```toml
# Cargo.toml
[package]
name = "makepad-d3"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[dependencies]
makepad-widgets = "0.6"
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Optional: Linebender integration
kurbo = { version = "0.10", optional = true }
color = { version = "0.3", optional = true }

[dev-dependencies]
criterion = "0.5"
proptest = "1.0"

[features]
default = []
kurbo = ["dep:kurbo"]
color = ["dep:color"]
full = ["kurbo", "color"]

[[bench]]
name = "scale_bench"
harness = false
```

---

## 3. Priority & Effort Framework

### Priority Definitions

| Priority | Label | Meaning | Example |
|----------|-------|---------|---------|
| **P0** | 🔴 Critical | Blocks all downstream work | Scale trait, data structures |
| **P1** | 🟠 High | Required for MVP | TimeScale, Axis, LineGenerator |
| **P2** | 🟡 Medium | Required for v1.0 | Tooltip, Zoom, ColorScales |
| **P3** | 🟢 Low | Nice to have for v2.0+ | ForceSimulation, Geographic |

### Effort Estimation

| Effort | Days | Description | Example |
|--------|------|-------------|---------|
| **XS** | 0.5 | Trivial change | Add method to existing struct |
| **S** | 1-2 | Simple feature | SymlogScale, QuantizeScale |
| **M** | 3-5 | Moderate complexity | LogScale, StackGenerator |
| **L** | 6-10 | Complex feature | TimeScale, Axis widget |
| **XL** | 11-20 | Major system | ZoomBehavior, ForceSimulation |
| **XXL** | 21+ | Very large | Geographic projections |

### Team Roles

| Team | Responsibilities | Skills Required |
|------|------------------|-----------------|
| **Core** | Data structures, traits, utilities | Rust fundamentals |
| **Scale** | All scale implementations | Math, algorithms |
| **UI** | Axis, tooltip, legend widgets | Makepad, UI/UX |
| **Graphics** | Shape generators, curves | Geometry, Beziers |
| **Color** | Color scales, schemes | Color science |
| **Interaction** | Zoom, brush, hit testing | Events, gestures |
| **Layout** | Force, hierarchy, tree | Graph algorithms |
| **Geo** | Projections, GeoJSON | Cartography, math |

---

## 4. Phase 0: Foundation (Week 1-2)

**Goal:** Establish project infrastructure and core abstractions

### Sprint 0.1: Project Setup

---

#### TASK-0001: Repository Setup
**Priority:** P0 🔴 | **Effort:** XS (0.5 day) | **Owner:** Core Team

**Description:**
Initialize the repository with proper structure, CI/CD, and development tools.

**Tasks:**
- [ ] Create repository with structure from Section 2
- [ ] Configure `Cargo.toml` with dependencies
- [ ] Set up GitHub Actions workflow
- [ ] Add pre-commit hooks (rustfmt, clippy)
- [ ] Create issue/PR templates
- [ ] Add LICENSE (MIT/Apache-2.0)

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `.github/workflows/ci.yml` | CI pipeline | 50 |
| `.pre-commit-config.yaml` | Pre-commit hooks | 20 |
| `Cargo.toml` | Package manifest | 40 |
| `README.md` | Project overview | 100 |

**Acceptance Criteria:**
- [ ] `cargo build` succeeds
- [ ] `cargo test` runs (even with no tests)
- [ ] `cargo clippy` passes with no warnings
- [ ] CI pipeline runs on push/PR

**Verification:**
```bash
# Verify build
cargo build --all-features

# Verify CI locally
act -j build  # Using act for local GitHub Actions

# Verify formatting
cargo fmt --check
cargo clippy -- -D warnings
```

**Documentation:**
- [ ] README with project description
- [ ] CONTRIBUTING.md with development setup

---

#### TASK-0002: Error Types
**Priority:** P0 🔴 | **Effort:** S (1 day) | **Owner:** Core Team

**Description:**
Define error types for the library using thiserror.

**API Specification:**
```rust
// src/error.rs

use thiserror::Error;

/// Errors that can occur in makepad-d3
#[derive(Error, Debug, Clone, PartialEq)]
pub enum D3Error {
    #[error("Invalid domain: {message}")]
    InvalidDomain { message: String },

    #[error("Invalid range: {message}")]
    InvalidRange { message: String },

    #[error("Value out of bounds: {value} not in [{min}, {max}]")]
    OutOfBounds { value: f64, min: f64, max: f64 },

    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

/// Result type alias for makepad-d3
pub type D3Result<T> = Result<T, D3Error>;

impl D3Error {
    pub fn invalid_domain(msg: impl Into<String>) -> Self {
        Self::InvalidDomain { message: msg.into() }
    }

    pub fn invalid_range(msg: impl Into<String>) -> Self {
        Self::InvalidRange { message: msg.into() }
    }

    pub fn out_of_bounds(value: f64, min: f64, max: f64) -> Self {
        Self::OutOfBounds { value, min, max }
    }

    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData { message: msg.into() }
    }

    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError { message: msg.into() }
    }

    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError { message: msg.into() }
    }
}
```

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `src/error.rs` | Error types | 80 |
| `src/error/tests.rs` | Error tests | 40 |

**Acceptance Criteria:**
- [ ] All error variants compile
- [ ] Error messages are descriptive
- [ ] `Display` implementation works correctly
- [ ] Errors are `Send + Sync`

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = D3Error::invalid_domain("min > max");
        assert_eq!(err.to_string(), "Invalid domain: min > max");
    }

    #[test]
    fn test_out_of_bounds() {
        let err = D3Error::out_of_bounds(150.0, 0.0, 100.0);
        assert!(err.to_string().contains("150"));
        assert!(err.to_string().contains("0"));
        assert!(err.to_string().contains("100"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<D3Error>();
    }
}
```

**Documentation:**
- [ ] Rustdoc for all error variants
- [ ] Examples showing error handling

---

#### TASK-0003: Core Data Structures
**Priority:** P0 🔴 | **Effort:** M (3 days) | **Owner:** Core Team

**Description:**
Define the fundamental data structures for chart data.

**API Specification:**
```rust
// src/data/point.rs

use serde::{Deserialize, Serialize};

/// A single data point in a chart
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DataPoint {
    /// X coordinate value (None = use index)
    pub x: Option<f64>,

    /// Y coordinate value (primary value)
    pub y: f64,

    /// Y minimum for floating bars/ranges
    pub y_min: Option<f64>,

    /// Radius for bubble charts
    pub r: Option<f64>,

    /// Display label
    pub label: Option<String>,

    /// Additional metadata (for tooltips)
    pub meta: Option<String>,
}

impl DataPoint {
    /// Create a new data point with x and y values
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x: Some(x),
            y,
            ..Default::default()
        }
    }

    /// Create from y value only (x will be index)
    pub fn from_y(y: f64) -> Self {
        Self { y, ..Default::default() }
    }

    /// Create a floating/range data point
    pub fn range(y_min: f64, y_max: f64) -> Self {
        Self {
            y: y_max,
            y_min: Some(y_min),
            ..Default::default()
        }
    }

    /// Create a bubble data point
    pub fn bubble(x: f64, y: f64, r: f64) -> Self {
        Self {
            x: Some(x),
            y,
            r: Some(r),
            ..Default::default()
        }
    }

    /// Builder: set label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Builder: set meta
    pub fn with_meta(mut self, meta: impl Into<String>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Get effective X value (or index if None)
    pub fn x_or(&self, index: usize) -> f64 {
        self.x.unwrap_or(index as f64)
    }

    /// Get Y range (y_min to y)
    pub fn y_range(&self) -> (f64, f64) {
        (self.y_min.unwrap_or(0.0), self.y)
    }
}

// Convenience conversions
impl From<f64> for DataPoint {
    fn from(y: f64) -> Self {
        Self::from_y(y)
    }
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl From<(f64, f64, f64)> for DataPoint {
    fn from((x, y, r): (f64, f64, f64)) -> Self {
        Self::bubble(x, y, r)
    }
}
```

```rust
// src/data/dataset.rs

use super::DataPoint;
use makepad_widgets::*;
use serde::{Deserialize, Serialize};

/// Point marker styles
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointStyle {
    #[default]
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Star,
    None,
}

/// A dataset containing multiple data points
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Dataset {
    /// Display label for this dataset
    pub label: String,

    /// Data points
    pub data: Vec<DataPoint>,

    /// Background/fill color
    #[serde(skip)]
    pub background_color: Option<Vec4>,

    /// Border/stroke color
    #[serde(skip)]
    pub border_color: Option<Vec4>,

    /// Border width in pixels
    pub border_width: f64,

    /// Whether this dataset is hidden
    pub hidden: bool,

    // Line chart options
    /// Fill area under line
    pub fill: bool,

    /// Line tension (0 = straight, 0.4 = smooth)
    pub tension: f64,

    // Point options
    /// Point radius in pixels
    pub point_radius: f64,

    /// Point style
    pub point_style: PointStyle,

    // Bar chart options
    /// Bar width as fraction of category width (0-1)
    pub bar_percent: f64,

    /// Bar border radius
    pub bar_radius: f64,
}

impl Dataset {
    /// Create a new dataset with a label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            border_width: 1.0,
            tension: 0.0,
            point_radius: 3.0,
            point_style: PointStyle::Circle,
            bar_percent: 0.8,
            ..Default::default()
        }
    }

    /// Set data from y values
    pub fn with_data(mut self, data: impl IntoIterator<Item = f64>) -> Self {
        self.data = data.into_iter().map(DataPoint::from_y).collect();
        self
    }

    /// Set data from (x, y) pairs
    pub fn with_xy_data(mut self, data: impl IntoIterator<Item = (f64, f64)>) -> Self {
        self.data = data.into_iter().map(DataPoint::from).collect();
        self
    }

    /// Set data from DataPoints directly
    pub fn with_points(mut self, data: Vec<DataPoint>) -> Self {
        self.data = data;
        self
    }

    /// Set background color
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set background color from hex (0xRRGGBB)
    pub fn with_hex_color(mut self, hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        self.background_color = Some(vec4(r, g, b, 1.0));
        self
    }

    /// Set border color
    pub fn with_border_color(mut self, color: Vec4) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Enable area fill
    pub fn with_fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// Set line tension
    pub fn with_tension(mut self, tension: f64) -> Self {
        self.tension = tension.clamp(0.0, 1.0);
        self
    }

    /// Set point radius
    pub fn with_point_radius(mut self, radius: f64) -> Self {
        self.point_radius = radius;
        self
    }

    /// Set point style
    pub fn with_point_style(mut self, style: PointStyle) -> Self {
        self.point_style = style;
        self
    }

    /// Set hidden state
    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Get number of data points
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get Y extent (min, max)
    pub fn y_extent(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for point in &self.data {
            if let Some(y_min) = point.y_min {
                min = min.min(y_min);
            }
            min = min.min(point.y);
            max = max.max(point.y);
        }
        Some((min, max))
    }

    /// Get X extent (min, max)
    pub fn x_extent(&self) -> Option<(f64, f64)> {
        if self.data.is_empty() {
            return None;
        }
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for (i, point) in self.data.iter().enumerate() {
            let x = point.x_or(i);
            min = min.min(x);
            max = max.max(x);
        }
        Some((min, max))
    }
}
```

```rust
// src/data/chart_data.rs

use super::{Dataset, DataPoint};
use serde::{Deserialize, Serialize};

/// Container for all chart data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ChartData {
    /// Category labels (x-axis for bar charts, legend for pie)
    pub labels: Vec<String>,

    /// Datasets to render
    pub datasets: Vec<Dataset>,
}

impl ChartData {
    /// Create new empty chart data
    pub fn new() -> Self {
        Self::default()
    }

    /// Set category labels
    pub fn with_labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Add a dataset
    pub fn add_dataset(mut self, dataset: Dataset) -> Self {
        self.datasets.push(dataset);
        self
    }

    /// Set datasets
    pub fn with_datasets(mut self, datasets: Vec<Dataset>) -> Self {
        self.datasets = datasets;
        self
    }

    /// Get Y extent across all visible datasets
    pub fn y_extent(&self) -> Option<(f64, f64)> {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut found = false;

        for dataset in &self.datasets {
            if dataset.hidden {
                continue;
            }
            if let Some((d_min, d_max)) = dataset.y_extent() {
                min = min.min(d_min);
                max = max.max(d_max);
                found = true;
            }
        }

        if found { Some((min, max)) } else { None }
    }

    /// Get X extent across all visible datasets
    pub fn x_extent(&self) -> Option<(f64, f64)> {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut found = false;

        for dataset in &self.datasets {
            if dataset.hidden {
                continue;
            }
            if let Some((d_min, d_max)) = dataset.x_extent() {
                min = min.min(d_min);
                max = max.max(d_max);
                found = true;
            }
        }

        if found { Some((min, max)) } else { None }
    }

    /// Get total Y value (for pie charts)
    pub fn total(&self) -> f64 {
        self.datasets
            .first()
            .map(|d| d.data.iter().filter(|p| p.y > 0.0).map(|p| p.y).sum())
            .unwrap_or(0.0)
    }

    /// Get number of data points in first dataset
    pub fn len(&self) -> usize {
        self.datasets.first().map(|d| d.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty() || self.datasets.iter().all(|d| d.is_empty())
    }

    /// Get number of visible datasets
    pub fn visible_count(&self) -> usize {
        self.datasets.iter().filter(|d| !d.hidden).count()
    }

    /// Toggle dataset visibility
    pub fn toggle_dataset(&mut self, index: usize) {
        if let Some(dataset) = self.datasets.get_mut(index) {
            dataset.hidden = !dataset.hidden;
        }
    }

    /// Validate data consistency
    pub fn validate(&self) -> Result<(), crate::error::D3Error> {
        use crate::error::D3Error;

        // Check labels match data length
        if !self.labels.is_empty() && !self.datasets.is_empty() {
            let expected_len = self.labels.len();
            for (i, dataset) in self.datasets.iter().enumerate() {
                if !dataset.data.is_empty() && dataset.data.len() != expected_len {
                    return Err(D3Error::invalid_data(format!(
                        "Dataset {} has {} points, expected {} (labels count)",
                        i, dataset.data.len(), expected_len
                    )));
                }
            }
        }
        Ok(())
    }
}
```

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `src/data/mod.rs` | Module exports | 10 |
| `src/data/point.rs` | DataPoint struct | 120 |
| `src/data/dataset.rs` | Dataset struct | 180 |
| `src/data/chart_data.rs` | ChartData struct | 150 |
| `src/data/tests.rs` | Unit tests | 200 |

**Acceptance Criteria:**
- [ ] All structs implement `Clone`, `Debug`, `Default`
- [ ] Serde serialization works
- [ ] Builder pattern methods are chainable
- [ ] Extent calculations handle edge cases
- [ ] Validation catches inconsistent data

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_point_from_y() {
        let point = DataPoint::from_y(42.0);
        assert_eq!(point.y, 42.0);
        assert_eq!(point.x, None);
    }

    #[test]
    fn test_data_point_conversions() {
        let p1: DataPoint = 42.0.into();
        assert_eq!(p1.y, 42.0);

        let p2: DataPoint = (1.0, 2.0).into();
        assert_eq!(p2.x, Some(1.0));
        assert_eq!(p2.y, 2.0);

        let p3: DataPoint = (1.0, 2.0, 3.0).into();
        assert_eq!(p3.r, Some(3.0));
    }

    #[test]
    fn test_dataset_y_extent() {
        let dataset = Dataset::new("test")
            .with_data(vec![10.0, 50.0, 30.0, -5.0]);

        let (min, max) = dataset.y_extent().unwrap();
        assert_eq!(min, -5.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_dataset_y_extent_with_range() {
        let dataset = Dataset::new("test")
            .with_points(vec![
                DataPoint::range(-10.0, 10.0),
                DataPoint::range(5.0, 20.0),
            ]);

        let (min, max) = dataset.y_extent().unwrap();
        assert_eq!(min, -10.0);
        assert_eq!(max, 20.0);
    }

    #[test]
    fn test_chart_data_y_extent_skips_hidden() {
        let data = ChartData::new()
            .add_dataset(Dataset::new("visible").with_data(vec![0.0, 50.0]))
            .add_dataset(Dataset::new("hidden").with_data(vec![-100.0, 100.0]).with_hidden(true));

        let (min, max) = data.y_extent().unwrap();
        assert_eq!(min, 0.0);
        assert_eq!(max, 50.0);
    }

    #[test]
    fn test_chart_data_validation() {
        let data = ChartData::new()
            .with_labels(vec!["A", "B", "C"])
            .add_dataset(Dataset::new("test").with_data(vec![1.0, 2.0])); // Wrong length!

        assert!(data.validate().is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = ChartData::new()
            .with_labels(vec!["A", "B"])
            .add_dataset(Dataset::new("test").with_data(vec![1.0, 2.0]));

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ChartData = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.labels, original.labels);
        assert_eq!(parsed.datasets.len(), original.datasets.len());
    }
}
```

**Documentation:**
- [ ] Module-level documentation in `src/data/mod.rs`
- [ ] Struct-level rustdoc with examples
- [ ] `docs/guides/data-structures.md` user guide

---

#### TASK-0004: Scale Trait
**Priority:** P0 🔴 | **Effort:** S (2 days) | **Owner:** Core Team

**Description:**
Define the core `Scale` trait that all scale implementations must follow.

**API Specification:**
```rust
// src/scale/traits.rs

use crate::error::D3Result;

/// Options for tick generation
#[derive(Clone, Debug)]
pub struct TickOptions {
    /// Target number of ticks (approximate)
    pub count: usize,

    /// Maximum ticks to show (hard limit)
    pub max_count: usize,

    /// Minimum ticks to show
    pub min_count: usize,

    /// Include domain bounds as ticks
    pub include_bounds: bool,

    /// Custom step size (overrides count)
    pub step_size: Option<f64>,
}

impl Default for TickOptions {
    fn default() -> Self {
        Self {
            count: 10,
            max_count: 20,
            min_count: 2,
            include_bounds: false,
            step_size: None,
        }
    }
}

impl TickOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn with_max_count(mut self, max: usize) -> Self {
        self.max_count = max;
        self
    }

    pub fn with_step_size(mut self, step: f64) -> Self {
        self.step_size = Some(step);
        self
    }

    pub fn with_bounds(mut self, include: bool) -> Self {
        self.include_bounds = include;
        self
    }
}

/// A tick mark on a scale
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    /// The value in the domain
    pub value: f64,

    /// The formatted label
    pub label: String,

    /// Position in pixels (computed from scale)
    pub position: f64,
}

impl Tick {
    pub fn new(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            position: 0.0,
        }
    }

    pub fn with_position(mut self, position: f64) -> Self {
        self.position = position;
        self
    }
}

/// Core trait for all scales
///
/// A scale maps values from a domain (input space) to a range (output space).
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, LinearScale};
///
/// let mut scale = LinearScale::new();
/// scale.set_domain(0.0, 100.0);
/// scale.set_range(0.0, 500.0);
///
/// assert_eq!(scale.scale(50.0), 250.0);
/// assert_eq!(scale.invert(250.0), 50.0);
/// ```
pub trait Scale: Send + Sync {
    /// Get the scale type identifier
    fn scale_type(&self) -> &'static str;

    /// Set the input domain (data space)
    fn set_domain(&mut self, min: f64, max: f64);

    /// Set the output range (pixel space)
    fn set_range(&mut self, start: f64, end: f64);

    /// Get the current domain bounds
    fn domain(&self) -> (f64, f64);

    /// Get the current range bounds
    fn range(&self) -> (f64, f64);

    /// Map a domain value to range value
    fn scale(&self, value: f64) -> f64;

    /// Map a range value back to domain value
    fn invert(&self, value: f64) -> f64;

    /// Generate tick marks for this scale
    fn ticks(&self, options: &TickOptions) -> Vec<Tick>;

    /// Check if the range is inverted (start > end)
    fn is_inverted(&self) -> bool {
        let (start, end) = self.range();
        start > end
    }

    /// Clamp a value to the domain bounds
    fn clamp_domain(&self, value: f64) -> f64 {
        let (min, max) = self.domain();
        value.clamp(min.min(max), min.max(max))
    }

    /// Clamp a value to the range bounds
    fn clamp_range(&self, value: f64) -> f64 {
        let (start, end) = self.range();
        value.clamp(start.min(end), start.max(end))
    }

    /// Normalize a domain value to [0, 1]
    fn normalize(&self, value: f64) -> f64 {
        let (min, max) = self.domain();
        if (max - min).abs() < f64::EPSILON {
            0.5
        } else {
            (value - min) / (max - min)
        }
    }

    /// Copy configuration from another scale of the same type
    fn copy_from(&mut self, other: &Self) where Self: Sized;

    /// Clone into a boxed trait object
    fn clone_box(&self) -> Box<dyn Scale>;
}

/// Extension trait for scale configuration
pub trait ScaleExt: Scale {
    /// Configure and return self (for chaining)
    fn domain(self, min: f64, max: f64) -> Self where Self: Sized;
    fn range(self, start: f64, end: f64) -> Self where Self: Sized;
}

/// Marker trait for continuous scales (linear, log, pow, time)
pub trait ContinuousScale: Scale {
    /// Extend domain to "nice" round values
    fn nice(&mut self);

    /// Check if clamping is enabled
    fn is_clamped(&self) -> bool;

    /// Enable/disable clamping
    fn set_clamp(&mut self, clamp: bool);
}

/// Marker trait for discrete/ordinal scales (category, band, point)
pub trait DiscreteScale: Scale {
    /// Get the bandwidth (space allocated for each item)
    fn bandwidth(&self) -> f64;

    /// Get the step (distance between items)
    fn step(&self) -> f64;

    /// Set padding between items (0-1)
    fn set_padding(&mut self, padding: f64);
}
```

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `src/scale/mod.rs` | Module exports | 30 |
| `src/scale/traits.rs` | Trait definitions | 200 |
| `src/scale/traits/tests.rs` | Trait tests | 50 |

**Acceptance Criteria:**
- [ ] `Scale` trait compiles with all methods
- [ ] `TickOptions` has sensible defaults
- [ ] Trait is object-safe (can be `Box<dyn Scale>`)
- [ ] `Send + Sync` bounds allow use in async code

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test that Scale can be made into a trait object
    #[test]
    fn test_scale_is_object_safe() {
        fn accepts_boxed_scale(_scale: Box<dyn Scale>) {}
        // This test passes if it compiles
    }

    #[test]
    fn test_tick_options_default() {
        let opts = TickOptions::default();
        assert_eq!(opts.count, 10);
        assert_eq!(opts.max_count, 20);
        assert!(!opts.include_bounds);
    }

    #[test]
    fn test_tick_options_builder() {
        let opts = TickOptions::new()
            .with_count(5)
            .with_step_size(10.0)
            .with_bounds(true);

        assert_eq!(opts.count, 5);
        assert_eq!(opts.step_size, Some(10.0));
        assert!(opts.include_bounds);
    }
}
```

**Documentation:**
- [ ] Comprehensive rustdoc for `Scale` trait
- [ ] Code examples in documentation
- [ ] `docs/guides/scales.md` overview

---

#### TASK-0005: Linear Scale
**Priority:** P0 🔴 | **Effort:** M (3 days) | **Owner:** Core Team

**Description:**
Implement the linear scale (continuous domain → continuous range).

**API Specification:**
```rust
// src/scale/linear.rs

use super::traits::{Scale, ContinuousScale, ScaleExt, Tick, TickOptions};
use super::utils::{nice_step, nice_bounds, format_number};

/// Linear scale for continuous numeric data
///
/// Maps a continuous input domain to a continuous output range using
/// linear interpolation.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, LinearScale};
///
/// let scale = LinearScale::new()
///     .domain(0.0, 100.0)
///     .range(0.0, 500.0);
///
/// assert_eq!(scale.scale(0.0), 0.0);
/// assert_eq!(scale.scale(50.0), 250.0);
/// assert_eq!(scale.scale(100.0), 500.0);
/// ```
#[derive(Clone, Debug)]
pub struct LinearScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    clamp: bool,
    nice: bool,
}

impl LinearScale {
    /// Create a new linear scale with default domain [0, 1] and range [0, 1]
    pub fn new() -> Self {
        Self {
            domain_min: 0.0,
            domain_max: 1.0,
            range_start: 0.0,
            range_end: 1.0,
            clamp: false,
            nice: false,
        }
    }

    /// Enable nice domain bounds
    pub fn with_nice(mut self, nice: bool) -> Self {
        self.nice = nice;
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Extend domain to start at zero (for bar charts)
    pub fn with_zero(mut self) -> Self {
        if self.domain_min > 0.0 {
            self.domain_min = 0.0;
        }
        if self.domain_max < 0.0 {
            self.domain_max = 0.0;
        }
        self
    }

    /// Get the scale ratio (domain range / pixel range)
    fn ratio(&self) -> f64 {
        let domain_span = self.domain_max - self.domain_min;
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            0.0
        } else {
            domain_span / range_span
        }
    }
}

impl Default for LinearScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for LinearScale {
    fn scale_type(&self) -> &'static str {
        "linear"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min;
        self.domain_max = max;

        if self.nice {
            let (nice_min, nice_max) = nice_bounds(min, max);
            self.domain_min = nice_min;
            self.domain_max = nice_max;
        }
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) {
        (self.domain_min, self.domain_max)
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        let value = if self.clamp {
            value.clamp(
                self.domain_min.min(self.domain_max),
                self.domain_min.max(self.domain_max),
            )
        } else {
            value
        };

        let t = self.normalize(value);
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn invert(&self, pixel: f64) -> f64 {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_min;
        }

        let t = (pixel - self.range_start) / range_span;
        self.domain_min + t * (self.domain_max - self.domain_min)
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let span = self.domain_max - self.domain_min;

        // Determine step size
        let step = options.step_size.unwrap_or_else(|| {
            nice_step(span, options.count)
        });

        // Calculate starting tick
        let start = (self.domain_min / step).ceil() * step;

        let mut ticks = Vec::new();
        let epsilon = step * 0.0001;

        // Add min bound if requested
        if options.include_bounds && start > self.domain_min + epsilon {
            let pos = self.scale(self.domain_min);
            ticks.push(Tick::new(self.domain_min, format_number(self.domain_min)).with_position(pos));
        }

        // Generate ticks
        let mut value = start;
        while value <= self.domain_max + epsilon && ticks.len() < options.max_count {
            // Skip if too close to previous
            let skip = ticks.last()
                .map(|t| (t.value - value).abs() < epsilon)
                .unwrap_or(false);

            if !skip {
                let pos = self.scale(value);
                ticks.push(Tick::new(value, format_number(value)).with_position(pos));
            }
            value += step;
        }

        // Add max bound if requested
        if options.include_bounds {
            let last_value = ticks.last().map(|t| t.value).unwrap_or(f64::MIN);
            if (self.domain_max - last_value).abs() > epsilon {
                let pos = self.scale(self.domain_max);
                ticks.push(Tick::new(self.domain_max, format_number(self.domain_max)).with_position(pos));
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_min = other.domain_min;
        self.domain_max = other.domain_max;
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.clamp = other.clamp;
        self.nice = other.nice;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for LinearScale {
    fn nice(&mut self) {
        let (nice_min, nice_max) = nice_bounds(self.domain_min, self.domain_max);
        self.domain_min = nice_min;
        self.domain_max = nice_max;
    }

    fn is_clamped(&self) -> bool {
        self.clamp
    }

    fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }
}

impl ScaleExt for LinearScale {
    fn domain(mut self, min: f64, max: f64) -> Self {
        self.set_domain(min, max);
        self
    }

    fn range(mut self, start: f64, end: f64) -> Self {
        self.set_range(start, end);
        self
    }
}
```

```rust
// src/scale/utils.rs

/// Calculate a "nice" step size for tick generation
///
/// Returns a step size that produces clean tick values (1, 2, 5, 10, 20, 50, etc.)
pub fn nice_step(span: f64, target_count: usize) -> f64 {
    if span.abs() < f64::EPSILON || target_count == 0 {
        return 1.0;
    }

    let raw_step = span / target_count as f64;
    let magnitude = 10.0_f64.powf(raw_step.abs().log10().floor());
    let normalized = raw_step / magnitude;

    let nice_normalized = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice_normalized * magnitude
}

/// Extend bounds to "nice" round values
pub fn nice_bounds(min: f64, max: f64) -> (f64, f64) {
    if (max - min).abs() < f64::EPSILON {
        return (min - 1.0, max + 1.0);
    }

    let span = max - min;
    let step = nice_step(span, 10);

    let nice_min = (min / step).floor() * step;
    let nice_max = (max / step).ceil() * step;

    (nice_min, nice_max)
}

/// Format a number for display
///
/// Uses appropriate precision and SI prefixes for readability
pub fn format_number(value: f64) -> String {
    let abs = value.abs();

    if abs == 0.0 {
        return "0".to_string();
    }

    // Use SI prefixes for large/small numbers
    if abs >= 1_000_000_000.0 {
        format!("{:.1}G", value / 1_000_000_000.0)
    } else if abs >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else if abs >= 1.0 {
        // Remove unnecessary trailing zeros
        let formatted = format!("{:.2}", value);
        formatted.trim_end_matches('0').trim_end_matches('.').to_string()
    } else if abs >= 0.01 {
        format!("{:.2}", value)
    } else if abs >= 0.001 {
        format!("{:.3}", value)
    } else {
        format!("{:.2e}", value)
    }
}

/// Linear interpolation
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Inverse linear interpolation (find t given value)
pub fn inverse_lerp(a: f64, b: f64, value: f64) -> f64 {
    if (b - a).abs() < f64::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Map a value from one range to another
pub fn map_range(value: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    let t = inverse_lerp(in_min, in_max, value);
    lerp(out_min, out_max, t)
}
```

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `src/scale/linear.rs` | LinearScale impl | 250 |
| `src/scale/utils.rs` | Utility functions | 100 |
| `src/scale/linear/tests.rs` | Unit tests | 200 |

**Acceptance Criteria:**
- [ ] Correct linear mapping
- [ ] Inverted ranges work (for Y axis)
- [ ] Clamping works correctly
- [ ] Nice bounds produce readable ticks
- [ ] Performance: 10M operations < 100ms

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_mapping() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        assert_eq!(scale.scale(0.0), 0.0);
        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(100.0), 500.0);
    }

    #[test]
    fn test_invert() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        assert_eq!(scale.invert(0.0), 0.0);
        assert_eq!(scale.invert(250.0), 50.0);
        assert_eq!(scale.invert(500.0), 100.0);
    }

    #[test]
    fn test_inverted_range() {
        // Y-axis: higher values map to lower pixels
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(500.0, 0.0);  // Inverted!

        assert_eq!(scale.scale(0.0), 500.0);
        assert_eq!(scale.scale(100.0), 0.0);
        assert!(scale.is_inverted());
    }

    #[test]
    fn test_clamping() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0)
            .with_clamp(true);

        assert_eq!(scale.scale(-50.0), 0.0);   // Clamped to min
        assert_eq!(scale.scale(150.0), 500.0); // Clamped to max
    }

    #[test]
    fn test_no_clamping() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        assert_eq!(scale.scale(-50.0), -250.0);  // Extrapolated
        assert_eq!(scale.scale(150.0), 750.0);   // Extrapolated
    }

    #[test]
    fn test_nice_bounds() {
        let (nice_min, nice_max) = nice_bounds(0.3, 9.7);
        assert_eq!(nice_min, 0.0);
        assert_eq!(nice_max, 10.0);
    }

    #[test]
    fn test_nice_step() {
        assert_eq!(nice_step(100.0, 10), 10.0);
        assert_eq!(nice_step(95.0, 10), 10.0);
        assert_eq!(nice_step(45.0, 10), 5.0);
        assert_eq!(nice_step(18.0, 10), 2.0);
    }

    #[test]
    fn test_tick_generation() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        let ticks = scale.ticks(&TickOptions::new().with_count(5));

        assert!(!ticks.is_empty());
        assert!(ticks.len() <= 10);

        // First tick should be at or near 0
        assert!(ticks[0].value <= 0.0 + f64::EPSILON);

        // Last tick should be at or near 100
        let last = ticks.last().unwrap();
        assert!(last.value >= 100.0 - f64::EPSILON);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(1.0), "1");
        assert_eq!(format_number(1.5), "1.5");
        assert_eq!(format_number(1.50), "1.5");
        assert_eq!(format_number(1000.0), "1K");
        assert_eq!(format_number(1500.0), "1.5K");
        assert_eq!(format_number(1000000.0), "1M");
        assert_eq!(format_number(1000000000.0), "1G");
        assert_eq!(format_number(0.005), "0.005");
    }

    #[test]
    fn test_scale_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LinearScale>();
    }

    #[test]
    fn test_clone_box() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale(50.0), 250.0);
    }

    // Performance benchmark (run with cargo bench)
    #[bench]
    fn bench_scale_operations(b: &mut test::Bencher) {
        let scale = LinearScale::new()
            .domain(0.0, 1000.0)
            .range(0.0, 800.0);

        b.iter(|| {
            for i in 0..10000 {
                test::black_box(scale.scale(i as f64));
            }
        });
    }
}
```

**Verification Script:**
```bash
#!/bin/bash
# verify_linear_scale.sh

echo "=== Verifying LinearScale Implementation ==="

# 1. Compilation check
echo "1. Checking compilation..."
cargo build --features full 2>&1 | tee /tmp/build.log
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "FAIL: Compilation failed"
    exit 1
fi
echo "PASS: Compilation successful"

# 2. Unit tests
echo "2. Running unit tests..."
cargo test scale::linear --no-fail-fast 2>&1 | tee /tmp/test.log
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "FAIL: Unit tests failed"
    exit 1
fi
echo "PASS: Unit tests passed"

# 3. Documentation
echo "3. Checking documentation..."
cargo doc --no-deps 2>&1 | tee /tmp/doc.log
if grep -q "warning: missing documentation" /tmp/doc.log; then
    echo "WARN: Missing documentation"
fi
echo "PASS: Documentation builds"

# 4. Clippy
echo "4. Running clippy..."
cargo clippy -- -D warnings 2>&1 | tee /tmp/clippy.log
if [ ${PIPESTATUS[0]} -ne 0 ]; then
    echo "FAIL: Clippy warnings"
    exit 1
fi
echo "PASS: Clippy clean"

# 5. Coverage (requires cargo-tarpaulin)
echo "5. Checking coverage..."
cargo tarpaulin --out Html --output-dir coverage/ -- scale::linear 2>&1 | tee /tmp/coverage.log
COVERAGE=$(grep "Coverage:" /tmp/coverage.log | tail -1 | grep -oP '\d+\.\d+')
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
    echo "WARN: Coverage is ${COVERAGE}%, target is 80%"
else
    echo "PASS: Coverage is ${COVERAGE}%"
fi

echo ""
echo "=== LinearScale Verification Complete ==="
```

**Documentation:**
- [ ] API documentation with examples
- [ ] Mathematical explanation of linear interpolation
- [ ] Common use cases

---

#### TASK-0006: Category Scale
**Priority:** P0 🔴 | **Effort:** M (3 days) | **Owner:** Core Team

**Description:**
Implement the category scale (discrete domain → continuous range bands).

**API Specification:**
```rust
// src/scale/category.rs

use super::traits::{Scale, DiscreteScale, Tick, TickOptions};

/// Scale for categorical/discrete data
///
/// Maps discrete categories to continuous bands or points.
/// Used primarily for bar charts and grouped visualizations.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, CategoryScale};
///
/// let scale = CategoryScale::new()
///     .with_labels(vec!["A", "B", "C", "D"])
///     .range(0.0, 400.0);
///
/// // With offset=true (default), items are centered in bands
/// assert_eq!(scale.scale(0.0), 50.0);  // Center of first band
/// assert_eq!(scale.scale(1.0), 150.0); // Center of second band
/// assert_eq!(scale.bandwidth(), 100.0);
/// ```
#[derive(Clone, Debug)]
pub struct CategoryScale {
    labels: Vec<String>,
    range_start: f64,
    range_end: f64,
    padding_inner: f64,
    padding_outer: f64,
    align: f64,
    offset: bool,
}

impl CategoryScale {
    /// Create a new category scale
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            range_start: 0.0,
            range_end: 100.0,
            padding_inner: 0.0,
            padding_outer: 0.0,
            align: 0.5,
            offset: true,
        }
    }

    /// Set category labels
    pub fn with_labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    /// Set range
    pub fn range(mut self, start: f64, end: f64) -> Self {
        self.range_start = start;
        self.range_end = end;
        self
    }

    /// Set inner padding (between bands) as fraction 0-1
    pub fn with_padding_inner(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self
    }

    /// Set outer padding (at edges) as fraction 0-1
    pub fn with_padding_outer(mut self, padding: f64) -> Self {
        self.padding_outer = padding.clamp(0.0, 1.0);
        self
    }

    /// Set uniform padding (inner and outer)
    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self.padding_outer = padding.clamp(0.0, 1.0);
        self
    }

    /// Set alignment within step (0-1)
    pub fn with_align(mut self, align: f64) -> Self {
        self.align = align.clamp(0.0, 1.0);
        self
    }

    /// Set offset mode
    ///
    /// When true, items are centered between grid lines (bar charts).
    /// When false, items are placed on grid lines (line charts).
    pub fn with_offset(mut self, offset: bool) -> Self {
        self.offset = offset;
        self
    }

    /// Get number of categories
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Get label at index
    pub fn label(&self, index: usize) -> Option<&str> {
        self.labels.get(index).map(String::as_str)
    }

    /// Get all labels
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// Find index for a label
    pub fn index_of(&self, label: &str) -> Option<usize> {
        self.labels.iter().position(|l| l == label)
    }

    /// Get pixel position for category index
    pub fn scale_index(&self, index: usize) -> f64 {
        if self.labels.is_empty() {
            return self.range_start;
        }

        let step = self.step();
        let bandwidth = self.bandwidth();
        let base = self.range_start + self.padding_outer * step + index as f64 * step;

        if self.offset {
            base + bandwidth / 2.0 + self.align * (step - bandwidth)
        } else {
            base
        }
    }

    /// Get category index for pixel position
    pub fn invert_index(&self, pixel: f64) -> usize {
        if self.labels.is_empty() {
            return 0;
        }

        let step = self.step();
        if step == 0.0 {
            return 0;
        }

        let adjusted = pixel - self.range_start - self.padding_outer * step;
        let index = (adjusted / step).floor() as i64;
        index.clamp(0, (self.labels.len() - 1) as i64) as usize
    }
}

impl Default for CategoryScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for CategoryScale {
    fn scale_type(&self) -> &'static str {
        "category"
    }

    fn set_domain(&mut self, _min: f64, _max: f64) {
        // Category scale doesn't use numeric domain
        // Labels are set via with_labels()
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, (self.labels.len().saturating_sub(1)) as f64)
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        self.scale_index(value.round() as usize)
    }

    fn invert(&self, pixel: f64) -> f64 {
        self.invert_index(pixel) as f64
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let mut ticks = Vec::with_capacity(self.labels.len());

        // Calculate step for skipping labels if too many
        let step = if self.labels.len() > options.max_count && options.max_count > 0 {
            (self.labels.len() as f64 / options.max_count as f64).ceil() as usize
        } else {
            1
        };

        for (i, label) in self.labels.iter().enumerate() {
            if i % step == 0 {
                let pos = self.scale_index(i);
                ticks.push(Tick::new(i as f64, label.clone()).with_position(pos));
            }
        }

        ticks
    }

    fn copy_from(&mut self, other: &Self) {
        self.labels = other.labels.clone();
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.padding_inner = other.padding_inner;
        self.padding_outer = other.padding_outer;
        self.align = other.align;
        self.offset = other.offset;
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl DiscreteScale for CategoryScale {
    fn bandwidth(&self) -> f64 {
        if self.labels.is_empty() {
            return 0.0;
        }

        let n = self.labels.len() as f64;
        let range = (self.range_end - self.range_start).abs();
        let step = range / (n + self.padding_outer * 2.0 - self.padding_inner);

        step * (1.0 - self.padding_inner)
    }

    fn step(&self) -> f64 {
        if self.labels.is_empty() {
            return 0.0;
        }

        let n = self.labels.len() as f64;
        let range = (self.range_end - self.range_start).abs();

        range / (n + self.padding_outer * 2.0 - self.padding_inner)
    }

    fn set_padding(&mut self, padding: f64) {
        self.padding_inner = padding.clamp(0.0, 1.0);
        self.padding_outer = padding.clamp(0.0, 1.0);
    }
}
```

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `src/scale/category.rs` | CategoryScale impl | 220 |
| `src/scale/category/tests.rs` | Unit tests | 180 |

**Acceptance Criteria:**
- [ ] Correct band positioning
- [ ] Padding works correctly
- [ ] Label lookup works
- [ ] Tick generation handles dense labels

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_category() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0);

        assert_eq!(scale.len(), 4);
        assert_eq!(scale.bandwidth(), 100.0);
        assert_eq!(scale.step(), 100.0);
    }

    #[test]
    fn test_scale_with_offset() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0)
            .with_offset(true);

        // Items should be centered in bands
        assert_eq!(scale.scale_index(0), 50.0);
        assert_eq!(scale.scale_index(1), 150.0);
        assert_eq!(scale.scale_index(2), 250.0);
        assert_eq!(scale.scale_index(3), 350.0);
    }

    #[test]
    fn test_scale_without_offset() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0)
            .with_offset(false);

        // Items should be at start of bands
        assert_eq!(scale.scale_index(0), 0.0);
        assert_eq!(scale.scale_index(1), 100.0);
        assert_eq!(scale.scale_index(2), 200.0);
        assert_eq!(scale.scale_index(3), 300.0);
    }

    #[test]
    fn test_invert() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0)
            .with_offset(true);

        assert_eq!(scale.invert_index(50.0), 0);
        assert_eq!(scale.invert_index(150.0), 1);
        assert_eq!(scale.invert_index(250.0), 2);
        assert_eq!(scale.invert_index(350.0), 3);
    }

    #[test]
    fn test_padding() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"])
            .range(0.0, 400.0)
            .with_padding_inner(0.2);

        // Bandwidth should be reduced by padding
        assert!(scale.bandwidth() < 100.0);
    }

    #[test]
    fn test_label_lookup() {
        let scale = CategoryScale::new()
            .with_labels(vec!["A", "B", "C", "D"]);

        assert_eq!(scale.index_of("B"), Some(1));
        assert_eq!(scale.index_of("X"), None);
        assert_eq!(scale.label(2), Some("C"));
    }

    #[test]
    fn test_tick_generation() {
        let scale = CategoryScale::new()
            .with_labels(vec!["Jan", "Feb", "Mar", "Apr"])
            .range(0.0, 400.0);

        let ticks = scale.ticks(&TickOptions::default());

        assert_eq!(ticks.len(), 4);
        assert_eq!(ticks[0].label, "Jan");
        assert_eq!(ticks[1].label, "Feb");
        assert_eq!(ticks[2].label, "Mar");
        assert_eq!(ticks[3].label, "Apr");
    }

    #[test]
    fn test_tick_skipping() {
        let scale = CategoryScale::new()
            .with_labels((0..100).map(|i| format!("Item{}", i)))
            .range(0.0, 1000.0);

        let ticks = scale.ticks(&TickOptions::new().with_max_count(10));

        assert!(ticks.len() <= 10);
    }

    #[test]
    fn test_empty_scale() {
        let scale = CategoryScale::new();

        assert!(scale.is_empty());
        assert_eq!(scale.bandwidth(), 0.0);
        assert_eq!(scale.step(), 0.0);
    }
}
```

**Documentation:**
- [ ] API documentation with bar chart example
- [ ] Padding and alignment explanation
- [ ] Comparison with band scale (D3)

---

### Phase 0 Completion Checklist

**Sprint 0 Summary:**

| Task ID | Task | Priority | Effort | Status |
|---------|------|----------|--------|--------|
| TASK-0001 | Repository Setup | P0 | XS | ⬜ |
| TASK-0002 | Error Types | P0 | S | ⬜ |
| TASK-0003 | Core Data Structures | P0 | M | ⬜ |
| TASK-0004 | Scale Trait | P0 | S | ⬜ |
| TASK-0005 | Linear Scale | P0 | M | ⬜ |
| TASK-0006 | Category Scale | P0 | M | ⬜ |

**Quality Gates:**
- [ ] All tasks complete
- [ ] `cargo build` passes
- [ ] `cargo test` passes (>90% coverage for completed code)
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo doc` builds with no warnings
- [ ] All public APIs documented

**Deliverables:**
- [ ] Core library structure
- [ ] Error handling system
- [ ] Data structures for charts
- [ ] Scale trait and two implementations
- [ ] Unit tests for all components

---

## 5. Phase 1: Core Scales (Week 3-6)

**Goal:** Complete the scale system with time, log, and discretizing scales

### Sprint 1.1: Time Scale (Week 3-4)

---

#### TASK-1001: Time Scale
**Priority:** P1 🟠 | **Effort:** L (8 days) | **Owner:** Scale Team

**Description:**
Implement a time scale for date/time data with calendar-aware tick generation.

**API Specification:**
```rust
// src/scale/time.rs

use super::traits::{Scale, ContinuousScale, ScaleExt, Tick, TickOptions};
use chrono::{DateTime, Utc, Duration, Datelike, Timelike, NaiveDateTime};

/// Time interval for tick generation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl TimeInterval {
    /// Get the approximate duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        match self {
            Self::Millisecond(n) => *n as f64,
            Self::Second(n) => *n as f64 * 1000.0,
            Self::Minute(n) => *n as f64 * 60_000.0,
            Self::Hour(n) => *n as f64 * 3_600_000.0,
            Self::Day(n) => *n as f64 * 86_400_000.0,
            Self::Week(n) => *n as f64 * 604_800_000.0,
            Self::Month(n) => *n as f64 * 2_592_000_000.0, // ~30 days
            Self::Year(n) => *n as f64 * 31_536_000_000.0, // 365 days
        }
    }

    /// Get a reasonable default format string for this interval
    pub fn default_format(&self) -> &'static str {
        match self {
            Self::Millisecond(_) => "%H:%M:%S.%3f",
            Self::Second(_) => "%H:%M:%S",
            Self::Minute(_) | Self::Hour(_) => "%H:%M",
            Self::Day(_) | Self::Week(_) => "%b %d",
            Self::Month(_) => "%b %Y",
            Self::Year(_) => "%Y",
        }
    }

    /// Find the appropriate interval for a given duration and target tick count
    pub fn for_duration(duration_ms: f64, target_ticks: usize) -> Self {
        let target_interval = duration_ms / target_ticks as f64;

        // Intervals in ascending order of duration
        let intervals = [
            (1.0, Self::Millisecond(1)),
            (5.0, Self::Millisecond(5)),
            (10.0, Self::Millisecond(10)),
            (50.0, Self::Millisecond(50)),
            (100.0, Self::Millisecond(100)),
            (500.0, Self::Millisecond(500)),
            (1000.0, Self::Second(1)),
            (5000.0, Self::Second(5)),
            (15000.0, Self::Second(15)),
            (30000.0, Self::Second(30)),
            (60000.0, Self::Minute(1)),
            (300000.0, Self::Minute(5)),
            (900000.0, Self::Minute(15)),
            (1800000.0, Self::Minute(30)),
            (3600000.0, Self::Hour(1)),
            (10800000.0, Self::Hour(3)),
            (21600000.0, Self::Hour(6)),
            (43200000.0, Self::Hour(12)),
            (86400000.0, Self::Day(1)),
            (172800000.0, Self::Day(2)),
            (604800000.0, Self::Week(1)),
            (1209600000.0, Self::Week(2)),
            (2592000000.0, Self::Month(1)),
            (7776000000.0, Self::Month(3)),
            (15552000000.0, Self::Month(6)),
            (31536000000.0, Self::Year(1)),
            (63072000000.0, Self::Year(2)),
            (157680000000.0, Self::Year(5)),
            (315360000000.0, Self::Year(10)),
        ];

        intervals
            .iter()
            .find(|(ms, _)| *ms >= target_interval)
            .map(|(_, interval)| *interval)
            .unwrap_or(Self::Year(10))
    }
}

/// Scale for date/time data
///
/// Maps dates to pixel positions with calendar-aware tick generation.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, TimeScale};
/// use chrono::{Utc, TimeZone};
///
/// let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
/// let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
///
/// let scale = TimeScale::new()
///     .with_domain(start, end)
///     .range(0.0, 1000.0);
///
/// // Mid-year maps to approximately mid-range
/// let mid = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();
/// assert!((scale.scale_time(mid) - 500.0).abs() < 50.0);
/// ```
#[derive(Clone, Debug)]
pub struct TimeScale {
    domain_start: DateTime<Utc>,
    domain_end: DateTime<Utc>,
    range_start: f64,
    range_end: f64,
    clamp: bool,
    nice: bool,
    format: Option<String>,
}

impl TimeScale {
    /// Create a new time scale with current time as domain
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            domain_start: now,
            domain_end: now + Duration::hours(24),
            range_start: 0.0,
            range_end: 100.0,
            clamp: false,
            nice: false,
            format: None,
        }
    }

    /// Set the time domain
    pub fn with_domain(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.domain_start = start;
        self.domain_end = end;
        self
    }

    /// Set the range
    pub fn range(mut self, start: f64, end: f64) -> Self {
        self.range_start = start;
        self.range_end = end;
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Set custom tick format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Get domain start
    pub fn domain_start(&self) -> DateTime<Utc> {
        self.domain_start
    }

    /// Get domain end
    pub fn domain_end(&self) -> DateTime<Utc> {
        self.domain_end
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        (self.domain_end - self.domain_start).num_milliseconds() as f64
    }

    /// Scale a DateTime to pixel position
    pub fn scale_time(&self, time: DateTime<Utc>) -> f64 {
        let time = if self.clamp {
            time.clamp(
                self.domain_start.min(self.domain_end),
                self.domain_start.max(self.domain_end),
            )
        } else {
            time
        };

        let t = self.normalize_time(time);
        self.range_start + t * (self.range_end - self.range_start)
    }

    /// Invert pixel position to DateTime
    pub fn invert_time(&self, pixel: f64) -> DateTime<Utc> {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_start;
        }

        let t = (pixel - self.range_start) / range_span;
        let duration = self.domain_end - self.domain_start;
        let offset_ms = (duration.num_milliseconds() as f64 * t) as i64;

        self.domain_start + Duration::milliseconds(offset_ms)
    }

    /// Normalize time to [0, 1]
    fn normalize_time(&self, time: DateTime<Utc>) -> f64 {
        let duration = self.domain_end - self.domain_start;
        if duration.num_milliseconds() == 0 {
            return 0.5;
        }

        let offset = time - self.domain_start;
        offset.num_milliseconds() as f64 / duration.num_milliseconds() as f64
    }

    /// Generate time ticks
    pub fn time_ticks(&self, options: &TickOptions) -> Vec<TimeTick> {
        let duration_ms = self.duration_ms();
        let interval = TimeInterval::for_duration(duration_ms, options.count);
        let format = self.format.as_deref()
            .unwrap_or_else(|| interval.default_format());

        let mut ticks = Vec::new();
        let mut current = self.floor_to_interval(self.domain_start, interval);

        while current <= self.domain_end && ticks.len() < options.max_count {
            if current >= self.domain_start {
                let pos = self.scale_time(current);
                let label = current.format(format).to_string();
                ticks.push(TimeTick {
                    time: current,
                    value: current.timestamp_millis() as f64,
                    label,
                    position: pos,
                });
            }
            current = self.ceil_to_interval(current, interval);
        }

        ticks
    }

    /// Floor datetime to interval boundary
    fn floor_to_interval(&self, time: DateTime<Utc>, interval: TimeInterval) -> DateTime<Utc> {
        match interval {
            TimeInterval::Millisecond(n) => {
                let ms = time.timestamp_subsec_millis();
                let floored = (ms / n) * n;
                time - Duration::milliseconds((ms - floored) as i64)
            }
            TimeInterval::Second(n) => {
                let s = time.second();
                let floored = (s / n) * n;
                time.with_second(floored).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            TimeInterval::Minute(n) => {
                let m = time.minute();
                let floored = (m / n) * n;
                time.with_minute(floored).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            TimeInterval::Hour(n) => {
                let h = time.hour();
                let floored = (h / n) * n;
                time.with_hour(floored).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            TimeInterval::Day(n) => {
                let d = time.day();
                let floored = ((d - 1) / n) * n + 1;
                time.with_day(floored).unwrap()
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            TimeInterval::Week(_) => {
                // Floor to Monday
                let weekday = time.weekday().num_days_from_monday();
                (time - Duration::days(weekday as i64))
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            TimeInterval::Month(n) => {
                let m = time.month();
                let floored = ((m - 1) / n) * n + 1;
                time.with_month(floored).unwrap()
                    .with_day(1).unwrap()
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
            TimeInterval::Year(n) => {
                let y = time.year();
                let floored = (y / n as i32) * n as i32;
                time.with_year(floored).unwrap()
                    .with_month(1).unwrap()
                    .with_day(1).unwrap()
                    .with_hour(0).unwrap()
                    .with_minute(0).unwrap()
                    .with_second(0).unwrap()
                    .with_nanosecond(0).unwrap()
            }
        }
    }

    /// Ceil datetime to next interval boundary
    fn ceil_to_interval(&self, time: DateTime<Utc>, interval: TimeInterval) -> DateTime<Utc> {
        match interval {
            TimeInterval::Millisecond(n) => time + Duration::milliseconds(n as i64),
            TimeInterval::Second(n) => time + Duration::seconds(n as i64),
            TimeInterval::Minute(n) => time + Duration::minutes(n as i64),
            TimeInterval::Hour(n) => time + Duration::hours(n as i64),
            TimeInterval::Day(n) => time + Duration::days(n as i64),
            TimeInterval::Week(n) => time + Duration::weeks(n as i64),
            TimeInterval::Month(n) => {
                let new_month = time.month() + n;
                if new_month > 12 {
                    time.with_year(time.year() + (new_month / 12) as i32).unwrap()
                        .with_month(((new_month - 1) % 12) + 1).unwrap()
                } else {
                    time.with_month(new_month).unwrap()
                }
            }
            TimeInterval::Year(n) => {
                time.with_year(time.year() + n as i32).unwrap()
            }
        }
    }
}

/// A tick mark with time information
#[derive(Clone, Debug)]
pub struct TimeTick {
    pub time: DateTime<Utc>,
    pub value: f64,
    pub label: String,
    pub position: f64,
}

impl Default for TimeScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for TimeScale {
    fn scale_type(&self) -> &'static str {
        "time"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        // Interpret as milliseconds since epoch
        self.domain_start = DateTime::from_timestamp_millis(min as i64)
            .unwrap_or_else(Utc::now);
        self.domain_end = DateTime::from_timestamp_millis(max as i64)
            .unwrap_or_else(Utc::now);
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) {
        (
            self.domain_start.timestamp_millis() as f64,
            self.domain_end.timestamp_millis() as f64,
        )
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        // Value is milliseconds since epoch
        let time = DateTime::from_timestamp_millis(value as i64)
            .unwrap_or(self.domain_start);
        self.scale_time(time)
    }

    fn invert(&self, pixel: f64) -> f64 {
        self.invert_time(pixel).timestamp_millis() as f64
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        self.time_ticks(options)
            .into_iter()
            .map(|tt| Tick {
                value: tt.value,
                label: tt.label,
                position: tt.position,
            })
            .collect()
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_start = other.domain_start;
        self.domain_end = other.domain_end;
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.clamp = other.clamp;
        self.nice = other.nice;
        self.format = other.format.clone();
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for TimeScale {
    fn nice(&mut self) {
        let interval = TimeInterval::for_duration(self.duration_ms(), 10);
        self.domain_start = self.floor_to_interval(self.domain_start, interval);
        self.domain_end = self.ceil_to_interval(self.domain_end, interval);
    }

    fn is_clamped(&self) -> bool {
        self.clamp
    }

    fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }
}
```

**Deliverables:**
| File | Purpose | LOC |
|------|---------|-----|
| `src/scale/time.rs` | TimeScale impl | 400 |
| `src/scale/time/tests.rs` | Unit tests | 250 |

**Acceptance Criteria:**
- [ ] Correct time mapping
- [ ] Calendar-aware tick generation
- [ ] Handles timezone correctly
- [ ] Appropriate format strings per interval
- [ ] Works with various durations (hours to years)

**Test Cases:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_basic_time_scale() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_domain(start, end)
            .range(0.0, 1000.0);

        assert_eq!(scale.scale_time(start), 0.0);
        assert!((scale.scale_time(end) - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_mid_year() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        let mid = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();

        let scale = TimeScale::new()
            .with_domain(start, end)
            .range(0.0, 1000.0);

        let pos = scale.scale_time(mid);
        assert!((pos - 500.0).abs() < 50.0);
    }

    #[test]
    fn test_invert() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_domain(start, end)
            .range(0.0, 1000.0);

        let inverted = scale.invert_time(500.0);
        assert!(inverted > start);
        assert!(inverted < end);
    }

    #[test]
    fn test_interval_selection() {
        // 1 hour duration -> should use minute intervals
        let hour_ms = 3_600_000.0;
        let interval = TimeInterval::for_duration(hour_ms, 10);
        assert!(matches!(interval, TimeInterval::Minute(_)));

        // 1 year duration -> should use month intervals
        let year_ms = 31_536_000_000.0;
        let interval = TimeInterval::for_duration(year_ms, 12);
        assert!(matches!(interval, TimeInterval::Month(_)));
    }

    #[test]
    fn test_tick_generation() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_domain(start, end)
            .range(0.0, 1000.0);

        let ticks = scale.time_ticks(&TickOptions::new().with_count(12));

        assert!(!ticks.is_empty());
        assert!(ticks.len() <= 15); // Approximate

        // Ticks should be in order
        for i in 1..ticks.len() {
            assert!(ticks[i].time > ticks[i-1].time);
        }
    }

    #[test]
    fn test_hourly_ticks() {
        let start = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 6, 15, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_domain(start, end)
            .range(0.0, 1000.0);

        let ticks = scale.time_ticks(&TickOptions::new().with_count(12));

        // Should have ~12 hourly ticks
        assert!(ticks.len() >= 8);
        assert!(ticks.len() <= 24);
    }
}
```

**Documentation:**
- [ ] API docs with time series example
- [ ] Interval selection explanation
- [ ] Format string reference
- [ ] `docs/guides/time-scale.md`

---

### Sprint 1.2: Additional Scales (Week 5-6)

---

#### TASK-1002: Log Scale
**Priority:** P1 🟠 | **Effort:** M (3 days) | **Owner:** Scale Team

**Description:**
Implement logarithmic scale for data spanning multiple orders of magnitude.

**API Specification:**
```rust
// src/scale/log.rs

use super::traits::{Scale, ContinuousScale, ScaleExt, Tick, TickOptions};

/// Logarithmic scale for exponential data
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, LogScale};
///
/// let scale = LogScale::new()
///     .domain(1.0, 1000.0)
///     .range(0.0, 300.0);
///
/// assert_eq!(scale.scale(1.0), 0.0);
/// assert_eq!(scale.scale(10.0), 100.0);
/// assert_eq!(scale.scale(100.0), 200.0);
/// assert_eq!(scale.scale(1000.0), 300.0);
/// ```
#[derive(Clone, Debug)]
pub struct LogScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    base: f64,
    clamp: bool,
}

impl LogScale {
    pub fn new() -> Self {
        Self {
            domain_min: 1.0,
            domain_max: 10.0,
            range_start: 0.0,
            range_end: 1.0,
            base: 10.0,
            clamp: false,
        }
    }

    pub fn with_base(mut self, base: f64) -> Self {
        self.base = base.max(1.001);
        self
    }

    fn log(&self, x: f64) -> f64 {
        x.ln() / self.base.ln()
    }

    fn pow(&self, x: f64) -> f64 {
        self.base.powf(x)
    }
}

impl Scale for LogScale {
    fn scale_type(&self) -> &'static str { "log" }

    fn set_domain(&mut self, min: f64, max: f64) {
        self.domain_min = min.max(f64::EPSILON);
        self.domain_max = max.max(f64::EPSILON);
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) { (self.domain_min, self.domain_max) }
    fn range(&self) -> (f64, f64) { (self.range_start, self.range_end) }

    fn scale(&self, value: f64) -> f64 {
        let value = if self.clamp {
            value.clamp(self.domain_min, self.domain_max)
        } else {
            value.max(f64::EPSILON)
        };

        let log_min = self.log(self.domain_min);
        let log_max = self.log(self.domain_max);
        let log_val = self.log(value);

        let t = (log_val - log_min) / (log_max - log_min);
        self.range_start + t * (self.range_end - self.range_start)
    }

    fn invert(&self, pixel: f64) -> f64 {
        let t = (pixel - self.range_start) / (self.range_end - self.range_start);
        let log_min = self.log(self.domain_min);
        let log_max = self.log(self.domain_max);
        self.pow(log_min + t * (log_max - log_min))
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        let mut ticks = Vec::new();
        let log_min = self.log(self.domain_min).floor() as i32;
        let log_max = self.log(self.domain_max).ceil() as i32;

        for exp in log_min..=log_max {
            let value = self.pow(exp as f64);
            if value >= self.domain_min && value <= self.domain_max {
                let pos = self.scale(value);
                ticks.push(Tick::new(value, format!("{:.0e}", value)).with_position(pos));
            }
            if ticks.len() >= options.max_count { break; }
        }
        ticks
    }

    fn copy_from(&mut self, other: &Self) { *self = other.clone(); }
    fn clone_box(&self) -> Box<dyn Scale> { Box::new(self.clone()) }
}

impl ContinuousScale for LogScale {
    fn nice(&mut self) {
        self.domain_min = self.pow(self.log(self.domain_min).floor());
        self.domain_max = self.pow(self.log(self.domain_max).ceil());
    }
    fn is_clamped(&self) -> bool { self.clamp }
    fn set_clamp(&mut self, clamp: bool) { self.clamp = clamp; }
}

impl ScaleExt for LogScale {
    fn domain(mut self, min: f64, max: f64) -> Self { self.set_domain(min, max); self }
    fn range(mut self, start: f64, end: f64) -> Self { self.set_range(start, end); self }
}
```

**Acceptance Criteria:**
- [ ] Correct logarithmic mapping
- [ ] Configurable base (10, 2, e)
- [ ] Handles domain > 0 only
- [ ] Tick generation at powers of base

---

#### TASK-1003: Power Scale
**Priority:** P2 🟡 | **Effort:** S (2 days) | **Owner:** Scale Team

**Description:**
Implement power scale (sqrt, square, cubic).

**API Specification:**
```rust
// src/scale/pow.rs

#[derive(Clone, Debug)]
pub struct PowScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    exponent: f64,
    clamp: bool,
}

impl PowScale {
    pub fn new() -> Self {
        Self { domain_min: 0.0, domain_max: 1.0, range_start: 0.0, range_end: 1.0, exponent: 1.0, clamp: false }
    }

    pub fn sqrt() -> Self { Self::new().with_exponent(0.5) }
    pub fn square() -> Self { Self::new().with_exponent(2.0) }

    pub fn with_exponent(mut self, exp: f64) -> Self {
        self.exponent = exp;
        self
    }
}

impl Scale for PowScale {
    fn scale(&self, value: f64) -> f64 {
        let sign = value.signum();
        let abs_val = value.abs().powf(self.exponent);
        let t = (sign * abs_val - self.domain_min.abs().powf(self.exponent) * self.domain_min.signum())
            / (self.domain_max.abs().powf(self.exponent) * self.domain_max.signum()
               - self.domain_min.abs().powf(self.exponent) * self.domain_min.signum());
        self.range_start + t * (self.range_end - self.range_start)
    }
    // ... other trait methods similar to LinearScale
}
```

**Acceptance Criteria:**
- [ ] Configurable exponent
- [ ] sqrt() convenience constructor
- [ ] Handles negative values with signed power

---

#### TASK-1004: Symlog Scale
**Priority:** P2 🟡 | **Effort:** S (2 days) | **Owner:** Scale Team

**Description:**
Bi-symmetric log scale for data crossing zero.

**API Specification:**
```rust
// src/scale/symlog.rs

#[derive(Clone, Debug)]
pub struct SymlogScale {
    domain_min: f64,
    domain_max: f64,
    range_start: f64,
    range_end: f64,
    constant: f64, // Linear region constant
    clamp: bool,
}

impl SymlogScale {
    pub fn new() -> Self {
        Self { domain_min: -1.0, domain_max: 1.0, range_start: 0.0, range_end: 1.0, constant: 1.0, clamp: false }
    }

    pub fn with_constant(mut self, c: f64) -> Self {
        self.constant = c.max(0.001);
        self
    }

    fn symlog(&self, x: f64) -> f64 {
        x.signum() * ((x.abs() / self.constant) + 1.0).ln()
    }

    fn symexp(&self, y: f64) -> f64 {
        y.signum() * self.constant * (y.abs().exp() - 1.0)
    }
}
```

**Acceptance Criteria:**
- [ ] Smooth transition through zero
- [ ] Configurable linear region
- [ ] Correct inverse mapping

---

## 6. Phase 2: Axis System (Week 7-10)

**Goal:** Create reusable axis components for chart rendering

### Sprint 2.1: Core Axis (Week 7-8)

---

#### TASK-2001: Axis Widget
**Priority:** P1 🟠 | **Effort:** L (6 days) | **Owner:** UI Team

**Description:**
Implement the core Axis widget that renders tick marks, labels, and grid lines.

**API Specification:**
```rust
// src/axis/axis.rs

use makepad_widgets::*;
use crate::scale::{Scale, Tick, TickOptions};

live_design! {
    AxisBase = {{Axis}} {
        width: Fill,
        height: 40,

        draw_line: {
            color: #333
        }
        draw_tick: {
            color: #333
        }
        draw_label: {
            text_style: { font_size: 10.0 }
            color: #666
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AxisOrientation {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(Live, LiveHook, Widget)]
pub struct Axis {
    #[redraw] #[live] draw_line: DrawLine,
    #[redraw] #[live] draw_tick: DrawLine,
    #[redraw] #[live] draw_label: DrawText,

    #[live] orientation: AxisOrientation,
    #[live] tick_size: f64,
    #[live] tick_padding: f64,
    #[live] label_offset: f64,
    #[live] show_domain_line: bool,
    #[live] show_grid: bool,
    #[live] grid_length: f64,

    #[rust] ticks: Vec<Tick>,
    #[rust] scale_range: (f64, f64),
}

impl Axis {
    pub fn set_ticks(&mut self, ticks: Vec<Tick>) {
        self.ticks = ticks;
    }

    pub fn set_scale<S: Scale>(&mut self, scale: &S, options: &TickOptions) {
        self.ticks = scale.ticks(options);
        self.scale_range = scale.range();
    }
}

impl Widget for Axis {
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);

        match self.orientation {
            AxisOrientation::Bottom => self.draw_bottom_axis(cx, rect),
            AxisOrientation::Top => self.draw_top_axis(cx, rect),
            AxisOrientation::Left => self.draw_left_axis(cx, rect),
            AxisOrientation::Right => self.draw_right_axis(cx, rect),
        }

        DrawStep::done()
    }
}

impl Axis {
    fn draw_bottom_axis(&mut self, cx: &mut Cx2d, rect: Rect) {
        let y = rect.pos.y;

        // Draw domain line
        if self.show_domain_line {
            self.draw_line.draw_line(cx,
                DVec2 { x: self.scale_range.0, y },
                DVec2 { x: self.scale_range.1, y }
            );
        }

        // Draw ticks and labels
        for tick in &self.ticks {
            let x = tick.position;

            // Tick mark
            self.draw_tick.draw_line(cx,
                DVec2 { x, y },
                DVec2 { x, y: y + self.tick_size }
            );

            // Grid line
            if self.show_grid {
                self.draw_tick.draw_line(cx,
                    DVec2 { x, y },
                    DVec2 { x, y: y - self.grid_length }
                );
            }

            // Label
            self.draw_label.draw_abs(cx,
                DVec2 { x: x - 20.0, y: y + self.tick_size + self.tick_padding },
                &tick.label
            );
        }
    }

    // Similar methods for other orientations...
}
```

**Acceptance Criteria:**
- [ ] All four orientations work correctly
- [ ] Tick marks render at correct positions
- [ ] Labels are readable and positioned properly
- [ ] Grid lines optional and configurable
- [ ] Works with any Scale implementation

**Test Cases:**
```rust
#[test]
fn test_axis_with_linear_scale() {
    let scale = LinearScale::new().domain(0.0, 100.0).range(0.0, 500.0);
    let mut axis = Axis::default();
    axis.set_scale(&scale, &TickOptions::default());

    assert!(!axis.ticks.is_empty());
    assert_eq!(axis.scale_range, (0.0, 500.0));
}
```

---

#### TASK-2002: Number Formatting
**Priority:** P1 🟠 | **Effort:** M (3 days) | **Owner:** UI Team

**Description:**
Implement flexible number formatting for axis labels.

**API Specification:**
```rust
// src/axis/format.rs

pub enum NumberFormat {
    Auto,
    Fixed(usize),           // Fixed decimal places
    Precision(usize),       // Significant digits
    Percent,                // Multiply by 100, add %
    SI,                     // k, M, G, etc.
    Currency { prefix: String, decimals: usize },
    Custom(Box<dyn Fn(f64) -> String + Send + Sync>),
}

impl NumberFormat {
    pub fn format(&self, value: f64) -> String {
        match self {
            Self::Auto => format_auto(value),
            Self::Fixed(d) => format!("{:.1$}", value, *d),
            Self::Precision(p) => format!("{:.1$e}", value, p.saturating_sub(1)),
            Self::Percent => format!("{:.1}%", value * 100.0),
            Self::SI => format_si(value),
            Self::Currency { prefix, decimals } => format!("{}{:.1$}", prefix, value, *decimals),
            Self::Custom(f) => f(value),
        }
    }
}

fn format_si(value: f64) -> String {
    let (scaled, suffix) = if value.abs() >= 1e12 { (value / 1e12, "T") }
        else if value.abs() >= 1e9 { (value / 1e9, "G") }
        else if value.abs() >= 1e6 { (value / 1e6, "M") }
        else if value.abs() >= 1e3 { (value / 1e3, "k") }
        else if value.abs() >= 1.0 { (value, "") }
        else if value.abs() >= 1e-3 { (value * 1e3, "m") }
        else if value.abs() >= 1e-6 { (value * 1e6, "μ") }
        else { (value * 1e9, "n") };

    format!("{:.1}{}", scaled, suffix)
}
```

**Acceptance Criteria:**
- [ ] Auto format selects appropriate precision
- [ ] SI prefixes work correctly
- [ ] Currency formatting handles locale
- [ ] Custom formatters work

---

## 7. Phase 3: Shape Generators (Week 11-16)

**Goal:** Implement D3-style shape generators for paths

### Sprint 3.1: Line and Area (Week 11-12)

---

#### TASK-3001: Line Generator
**Priority:** P1 🟠 | **Effort:** L (5 days) | **Owner:** Graphics Team

**Description:**
Create a line generator that converts data points to drawable paths.

**API Specification:**
```rust
// src/shape/line.rs

use crate::shape::curve::Curve;

pub struct LineGenerator<X, Y> {
    x: X,
    y: Y,
    defined: Box<dyn Fn(usize, &DataPoint) -> bool>,
    curve: Box<dyn Curve>,
}

impl LineGenerator<fn(&DataPoint, usize) -> f64, fn(&DataPoint, usize) -> f64> {
    pub fn new() -> Self {
        Self {
            x: |d, i| d.x_or(i),
            y: |d, _| d.y,
            defined: Box::new(|_, d| d.y.is_finite()),
            curve: Box::new(LinearCurve),
        }
    }
}

impl<X, Y> LineGenerator<X, Y>
where
    X: Fn(&DataPoint, usize) -> f64,
    Y: Fn(&DataPoint, usize) -> f64,
{
    pub fn x<F>(self, f: F) -> LineGenerator<F, Y>
    where F: Fn(&DataPoint, usize) -> f64 {
        LineGenerator { x: f, y: self.y, defined: self.defined, curve: self.curve }
    }

    pub fn y<F>(self, f: F) -> LineGenerator<X, F>
    where F: Fn(&DataPoint, usize) -> f64 {
        LineGenerator { x: self.x, y: f, defined: self.defined, curve: self.curve }
    }

    pub fn curve(mut self, curve: impl Curve + 'static) -> Self {
        self.curve = Box::new(curve);
        self
    }

    pub fn generate(&self, data: &[DataPoint]) -> Vec<PathSegment> {
        let points: Vec<_> = data.iter().enumerate()
            .filter(|(i, d)| (self.defined)(*i, d))
            .map(|(i, d)| DVec2 { x: (self.x)(d, i), y: (self.y)(d, i) })
            .collect();

        self.curve.generate(&points)
    }
}

#[derive(Clone, Debug)]
pub enum PathSegment {
    MoveTo(DVec2),
    LineTo(DVec2),
    CurveTo { cp1: DVec2, cp2: DVec2, end: DVec2 },
    ClosePath,
}
```

**Acceptance Criteria:**
- [ ] Generates correct path segments
- [ ] Handles missing data (defined filter)
- [ ] Works with all curve types
- [ ] Efficient for large datasets

---

#### TASK-3002: Curve Interpolators
**Priority:** P1 🟠 | **Effort:** L (8 days) | **Owner:** Graphics Team

**Description:**
Implement D3-compatible curve interpolation algorithms.

**API Specification:**
```rust
// src/shape/curve/mod.rs

pub trait Curve: Send + Sync {
    fn generate(&self, points: &[DVec2]) -> Vec<PathSegment>;
}

// Linear interpolation
pub struct LinearCurve;
impl Curve for LinearCurve {
    fn generate(&self, points: &[DVec2]) -> Vec<PathSegment> {
        if points.is_empty() { return vec![]; }
        let mut path = vec![PathSegment::MoveTo(points[0])];
        path.extend(points[1..].iter().map(|p| PathSegment::LineTo(*p)));
        path
    }
}

// Catmull-Rom spline
pub struct CatmullRomCurve { pub alpha: f64 }
impl Curve for CatmullRomCurve {
    fn generate(&self, points: &[DVec2]) -> Vec<PathSegment> {
        // Centripetal Catmull-Rom implementation
        // ...
    }
}

// Monotone cubic (preserves monotonicity)
pub struct MonotoneCurve;
impl Curve for MonotoneCurve {
    fn generate(&self, points: &[DVec2]) -> Vec<PathSegment> {
        // Fritsch-Carlson monotone cubic
        // ...
    }
}

// Step functions
pub enum StepPosition { Before, Middle, After }
pub struct StepCurve { pub position: StepPosition }

// Cardinal spline
pub struct CardinalCurve { pub tension: f64 }

// Basis spline (B-spline)
pub struct BasisCurve;

// Natural cubic spline
pub struct NaturalCurve;
```

**Acceptance Criteria:**
- [ ] Linear, Step, Basis, Cardinal work
- [ ] Catmull-Rom matches D3 output
- [ ] Monotone preserves data monotonicity
- [ ] Natural spline is C2 continuous

---

#### TASK-3003: Area Generator
**Priority:** P1 🟠 | **Effort:** M (4 days) | **Owner:** Graphics Team

**Description:**
Area generator for filled regions between curves.

**API Specification:**
```rust
// src/shape/area.rs

pub struct AreaGenerator<X, Y0, Y1> {
    x: X,
    y0: Y0,  // Baseline
    y1: Y1,  // Top line
    curve: Box<dyn Curve>,
}

impl AreaGenerator<...> {
    pub fn generate(&self, data: &[DataPoint]) -> Vec<PathSegment> {
        let top_points: Vec<_> = /* ... */;
        let bottom_points: Vec<_> = /* ... reversed */;

        let mut path = self.curve.generate(&top_points);
        path.extend(self.curve.generate(&bottom_points));
        path.push(PathSegment::ClosePath);
        path
    }
}
```

---

### Sprint 3.2: Arc and Pie (Week 13-14)

---

#### TASK-3004: Arc Generator
**Priority:** P1 🟠 | **Effort:** M (4 days) | **Owner:** Graphics Team

**Description:**
Generate arc paths for pie/donut charts and radial visualizations.

**API Specification:**
```rust
// src/shape/arc.rs

pub struct ArcGenerator {
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    end_angle: f64,
    corner_radius: f64,
    pad_angle: f64,
}

impl ArcGenerator {
    pub fn new() -> Self {
        Self {
            inner_radius: 0.0,
            outer_radius: 100.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::TAU,
            corner_radius: 0.0,
            pad_angle: 0.0,
        }
    }

    pub fn centroid(&self) -> DVec2 {
        let r = (self.inner_radius + self.outer_radius) / 2.0;
        let a = (self.start_angle + self.end_angle) / 2.0;
        DVec2 { x: r * a.cos(), y: r * a.sin() }
    }

    pub fn generate(&self) -> Vec<PathSegment> {
        // Generate arc path with optional corner radius
        // Uses SVG-style arc commands internally
    }
}
```

---

#### TASK-3005: Pie Layout
**Priority:** P1 🟠 | **Effort:** M (3 days) | **Owner:** Graphics Team

**Description:**
Compute pie slice angles from data values.

**API Specification:**
```rust
// src/shape/pie.rs

pub struct PieLayout {
    start_angle: f64,
    end_angle: f64,
    pad_angle: f64,
    sort: Option<Box<dyn Fn(&f64, &f64) -> Ordering>>,
}

#[derive(Clone, Debug)]
pub struct PieSlice {
    pub value: f64,
    pub index: usize,
    pub start_angle: f64,
    pub end_angle: f64,
    pub pad_angle: f64,
}

impl PieLayout {
    pub fn compute(&self, values: &[f64]) -> Vec<PieSlice> {
        let total: f64 = values.iter().sum();
        let range = self.end_angle - self.start_angle;
        let pad_total = self.pad_angle * values.len() as f64;
        let value_range = range - pad_total;

        let mut slices = Vec::with_capacity(values.len());
        let mut angle = self.start_angle;

        for (i, &value) in values.iter().enumerate() {
            let slice_angle = (value / total) * value_range;
            slices.push(PieSlice {
                value,
                index: i,
                start_angle: angle,
                end_angle: angle + slice_angle,
                pad_angle: self.pad_angle,
            });
            angle += slice_angle + self.pad_angle;
        }

        slices
    }
}
```

---

### Sprint 3.3: Stack Layout (Week 15-16)

---

#### TASK-3006: Stack Generator
**Priority:** P1 🟠 | **Effort:** M (5 days) | **Owner:** Graphics Team

**Description:**
Stack data series for stacked bar/area charts.

**API Specification:**
```rust
// src/shape/stack.rs

pub enum StackOrder { None, Ascending, Descending, InsideOut, Reverse }
pub enum StackOffset { None, Expand, Diverging, Silhouette, Wiggle }

pub struct StackGenerator {
    order: StackOrder,
    offset: StackOffset,
}

#[derive(Clone, Debug)]
pub struct StackedSeries {
    pub key: String,
    pub points: Vec<(f64, f64)>, // (y0, y1) for each data point
}

impl StackGenerator {
    pub fn stack(&self, data: &ChartData) -> Vec<StackedSeries> {
        let n_points = data.len();
        let n_series = data.datasets.len();

        // Initialize with zeros
        let mut result: Vec<StackedSeries> = data.datasets.iter()
            .map(|d| StackedSeries {
                key: d.label.clone(),
                points: vec![(0.0, 0.0); n_points],
            })
            .collect();

        // Apply ordering
        let order = self.compute_order(data);

        // Stack values
        for i in 0..n_points {
            let mut y0 = 0.0;
            for &series_idx in &order {
                let y = data.datasets[series_idx].data.get(i)
                    .map(|p| p.y).unwrap_or(0.0);
                result[series_idx].points[i] = (y0, y0 + y);
                y0 += y;
            }
        }

        // Apply offset
        self.apply_offset(&mut result);

        result
    }
}
```

---

## 8. Phase 4: Color System (Week 17-18)

**Goal:** Implement color scales and schemes

---

#### TASK-4001: Color Scales
**Priority:** P2 🟡 | **Effort:** M (4 days) | **Owner:** Color Team

**Description:**
Implement sequential, diverging, and categorical color scales.

**API Specification:**
```rust
// src/color/scale.rs

pub trait ColorScale: Send + Sync {
    fn color(&self, t: f64) -> Vec4;
}

pub struct SequentialScale {
    colors: Vec<Vec4>,
}

impl SequentialScale {
    pub fn new(colors: Vec<Vec4>) -> Self { Self { colors } }

    pub fn viridis() -> Self { /* D3 viridis colors */ }
    pub fn plasma() -> Self { /* D3 plasma colors */ }
    pub fn inferno() -> Self { /* D3 inferno colors */ }
    pub fn blues() -> Self { /* Blue gradient */ }
}

impl ColorScale for SequentialScale {
    fn color(&self, t: f64) -> Vec4 {
        let t = t.clamp(0.0, 1.0);
        let n = self.colors.len() - 1;
        let i = (t * n as f64).floor() as usize;
        let j = (i + 1).min(n);
        let local_t = t * n as f64 - i as f64;
        lerp_color(self.colors[i], self.colors[j], local_t as f32)
    }
}

pub struct DivergingScale {
    negative: Vec<Vec4>,
    positive: Vec<Vec4>,
    mid: Vec4,
}

pub struct CategoricalScale {
    colors: Vec<Vec4>,
}

impl CategoricalScale {
    pub fn category10() -> Self { /* D3 category10 */ }
    pub fn tableau10() -> Self { /* Tableau colors */ }
}
```

---

## 9. Phase 5: Interactions (Week 19-26)

**Goal:** Add zoom, brush, and tooltip interactions

---

#### TASK-5001: Tooltip Component
**Priority:** P2 🟡 | **Effort:** M (4 days) | **Owner:** Interaction Team

**Description:**
Floating tooltip that follows cursor and displays data.

**API Specification:**
```rust
// src/component/tooltip.rs

live_design! {
    TooltipBase = {{Tooltip}} {
        width: Fit, height: Fit,
        padding: 8,

        draw_bg: {
            color: #222
            radius: 4.0
        }

        content = <View> {
            flow: Down,
            title = <Label> { draw_text: { color: #fff, font_size: 12 } }
            value = <Label> { draw_text: { color: #ccc, font_size: 11 } }
        }
    }
}

#[derive(Live, Widget)]
pub struct Tooltip {
    #[live] draw_bg: DrawQuad,
    #[walk] walk: Walk,
    #[rust] visible: bool,
    #[rust] position: DVec2,
    #[rust] content: TooltipContent,
}

pub struct TooltipContent {
    pub title: String,
    pub items: Vec<(String, String, Vec4)>, // (label, value, color)
}

impl Tooltip {
    pub fn show(&mut self, pos: DVec2, content: TooltipContent) {
        self.visible = true;
        self.position = pos;
        self.content = content;
    }

    pub fn hide(&mut self) { self.visible = false; }
}
```

---

#### TASK-5002: Zoom Behavior
**Priority:** P2 🟡 | **Effort:** XL (12 days) | **Owner:** Interaction Team

**Description:**
Implement zoom and pan behavior with constraints.

**API Specification:**
```rust
// src/interaction/zoom.rs

pub struct ZoomTransform {
    pub k: f64,   // Scale factor
    pub x: f64,   // X translation
    pub y: f64,   // Y translation
}

pub struct ZoomBehavior {
    scale_extent: (f64, f64),
    translate_extent: Option<Rect>,
    wheel_delta: f64,
    on_zoom: Option<Box<dyn FnMut(&ZoomTransform)>>,
}

impl ZoomBehavior {
    pub fn new() -> Self {
        Self {
            scale_extent: (0.1, 10.0),
            translate_extent: None,
            wheel_delta: 0.002,
            on_zoom: None,
        }
    }

    pub fn handle_wheel(&mut self, transform: &mut ZoomTransform, delta: f64, center: DVec2) {
        let k0 = transform.k;
        let k1 = (k0 * (1.0 + delta * self.wheel_delta))
            .clamp(self.scale_extent.0, self.scale_extent.1);

        // Zoom centered on cursor
        transform.k = k1;
        transform.x = center.x - (center.x - transform.x) * k1 / k0;
        transform.y = center.y - (center.y - transform.y) * k1 / k0;

        self.constrain(transform);
        if let Some(ref mut on_zoom) = self.on_zoom { on_zoom(transform); }
    }

    pub fn handle_pan(&mut self, transform: &mut ZoomTransform, delta: DVec2) {
        transform.x += delta.x;
        transform.y += delta.y;
        self.constrain(transform);
        if let Some(ref mut on_zoom) = self.on_zoom { on_zoom(transform); }
    }
}
```

---

#### TASK-5003: Brush Selection
**Priority:** P2 🟡 | **Effort:** L (8 days) | **Owner:** Interaction Team

**Description:**
Rectangular selection brush for filtering data.

**API Specification:**
```rust
// src/interaction/brush.rs

pub enum BrushType { X, Y, XY }

pub struct BrushBehavior {
    brush_type: BrushType,
    extent: Option<Rect>,
    on_brush: Option<Box<dyn FnMut(Option<Rect>)>>,
    on_end: Option<Box<dyn FnMut(Option<Rect>)>>,
}

pub struct BrushSelection {
    pub x0: f64, pub y0: f64,
    pub x1: f64, pub y1: f64,
}

impl BrushBehavior {
    pub fn x() -> Self { Self::new(BrushType::X) }
    pub fn y() -> Self { Self::new(BrushType::Y) }
    pub fn xy() -> Self { Self::new(BrushType::XY) }

    pub fn handle_start(&mut self, pos: DVec2) { /* ... */ }
    pub fn handle_move(&mut self, pos: DVec2) { /* ... */ }
    pub fn handle_end(&mut self) { /* ... */ }
}
```

---

## 10. Phase 6: Layout Algorithms (Week 27-40)

**Goal:** Implement force simulation and hierarchy layouts

### Overview

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| TASK-6001 | Force Simulation Core | P3 | XL |
| TASK-6002 | Force: ManyBody | P3 | L |
| TASK-6003 | Force: Link | P3 | M |
| TASK-6004 | Force: Collide | P3 | M |
| TASK-6005 | Hierarchy Base | P3 | L |
| TASK-6006 | Tree Layout | P3 | L |
| TASK-6007 | Treemap Layout | P3 | L |
| TASK-6008 | Pack Layout | P3 | M |

*Detailed API specs for Phase 6 follow the same pattern as earlier phases.*

---

## 11. Phase 7: Geographic (Week 41-52)

**Goal:** Add geographic projections and GeoJSON support

### Overview

| Task | Description | Priority | Effort |
|------|-------------|----------|--------|
| TASK-7001 | Projection Trait | P3 | M |
| TASK-7002 | Mercator Projection | P3 | M |
| TASK-7003 | Equirectangular | P3 | S |
| TASK-7004 | Orthographic | P3 | M |
| TASK-7005 | GeoJSON Parser | P3 | L |
| TASK-7006 | GeoPath Generator | P3 | L |

*Geographic features are deferred to v3.0 release.*

---

## 12. MVP Definition

### MVP Scope (Week 1-10)

**Included Features:**
- ✅ Core data structures (DataPoint, Dataset, ChartData)
- ✅ Scale trait and implementations
  - LinearScale
  - CategoryScale
  - TimeScale
  - LogScale
- ✅ Axis component (all orientations)
- ✅ Number/date formatting
- ✅ Grid lines
- ✅ Basic examples

**Excluded from MVP:**
- ❌ Advanced scales (Pow, Symlog, Quantize)
- ❌ Shape generators (Line, Area, Stack)
- ❌ Color scales
- ❌ Interactions (Zoom, Brush)
- ❌ Layout algorithms
- ❌ Geographic projections

### MVP Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Time series chart | Working | Visual verification |
| Log scale chart | Working | Visual verification |
| Bar chart | Working | Visual verification |
| Performance | 10k points < 50ms | Benchmark |
| Test coverage | >80% | cargo-tarpaulin |
| Documentation | 100% public API | cargo doc |

### MVP Deliverables Checklist

- [ ] `src/error.rs` - Error types
- [ ] `src/data/` - Data structures
- [ ] `src/scale/traits.rs` - Scale trait
- [ ] `src/scale/linear.rs` - Linear scale
- [ ] `src/scale/category.rs` - Category scale
- [ ] `src/scale/time.rs` - Time scale
- [ ] `src/scale/log.rs` - Log scale
- [ ] `src/axis/` - Axis component
- [ ] `examples/basic/` - Basic examples
- [ ] `docs/` - Documentation

---

## 12A. GPU 3D Rendering System (New)

**Goal:** Implement GPU-accelerated 3D rendering for surface plots, scatter plots, and globe visualizations

**Reference Documents:**
- [GPU_3D_RENDERING_ARCHITECTURE.md](./docs/GPU_3D_RENDERING_ARCHITECTURE.md) - Detailed architecture
- [3D_OPTIMIZATION_PLAN.md](./docs/3D_OPTIMIZATION_PLAN.md) - Implementation plan

### Motivation

The current 3D rendering (e.g., `surface_plot.rs`) uses CPU-based projection and individual draw calls, resulting in poor performance:

| Resolution | Current (CPU) | Target (GPU) | Speedup |
|------------|---------------|--------------|---------|
| 25×25 | ~5ms | ~0.1ms | 50× |
| 100×100 | ~80ms | ~0.2ms | 400× |
| 500×500 | >1s | ~1ms | 1000× |

### Architecture Overview

Based on analysis of `makepad-urdf-player` GPU rendering:

```
┌─────────────────────────────────────────────────────────┐
│                    GPU RENDERING                        │
├─────────────────────────────────────────────────────────┤
│  SETUP (once):                                          │
│  1. Generate mesh geometry on CPU                       │
│  2. Upload vertex buffer (pos + normal + uv + id)       │
│  3. Upload index buffer (triangle indices)              │
│                                                         │
│  PER FRAME (64 bytes only):                             │
│  1. Update 4×Vec4 transform matrix                      │
│  2. GPU vertex shader: transform + diffuse lighting     │
│  3. GPU fragment shader: colormap + specular            │
│  4. Hardware depth buffer: automatic sorting            │
└─────────────────────────────────────────────────────────┘
```

### Key Components

| Component | Purpose | Location |
|-----------|---------|----------|
| `Transform3D` | 64-byte transform matrix | `src/render3d/types.rs` |
| `MeshData` | Interleaved vertex buffer | `src/render3d/mesh.rs` |
| `GeometryMesh3D` | GPU geometry wrapper | `src/render3d/geometry.rs` |
| `DrawMesh3D` | Main 3D shader | `src/render3d/draw_mesh.rs` |
| `Camera3D` | Orbital camera | `src/render3d/camera.rs` |

### Vertex Layout (9 floats = 36 bytes per vertex)

```
┌──────────────────────────────────────────────────────────┐
│ pos.x │ pos.y │ pos.z │ id  │ nx  │ ny  │ nz  │ u   │ v  │
│ f32   │ f32   │ f32   │ f32 │ f32 │ f32 │ f32 │ f32 │ f32│
└──────────────────────────────────────────────────────────┘
```

### Shader Pipeline

**Vertex Shader:**
- Transform position via 4×4 matrix (column-major)
- Transform normals (rotation only)
- Compute Lambertian diffuse lighting
- Two-tone shading (darker undersides)

**Fragment Shader:**
- Blinn-Phong specular highlights
- Colormap lookup (Viridis, Plasma, etc.)
- Viewport clipping

### Task Overview

| Task | Description | Priority | Effort | Days |
|------|-------------|----------|--------|------|
| TASK-3D-001 | Core 3D Types | P0 | S | 1 |
| TASK-3D-002 | MeshData Structure | P0 | S | 1 |
| TASK-3D-003 | GeometryMesh3D | P0 | S | 1 |
| TASK-3D-004 | DrawMesh3D Shader | P0 | L | 3 |
| TASK-3D-005 | Colormap Shaders | P1 | M | 2 |
| TASK-3D-006 | Camera3D | P0 | M | 2 |
| TASK-3D-007 | Camera Events | P1 | S | 1 |
| TASK-3D-008 | GPU Surface Plot | P0 | L | 3 |
| TASK-3D-009 | Wireframe Mode | P2 | S | 1 |
| TASK-3D-010 | 3D Scatter Plot | P1 | M | 2 |
| TASK-3D-011 | 3D Bar Chart | P1 | M | 2 |
| TASK-3D-012 | Globe Optimization | P2 | S | 1 |
| TASK-3D-013 | Benchmarks | P1 | S | 1 |
| TASK-3D-014 | Documentation | P2 | S | 1 |
| TASK-3D-015 | Example Updates | P2 | S | 1 |

**Total: ~23 days**

### New Module Structure

```
src/render3d/
├── mod.rs           # Module exports
├── types.rs         # Transform3D, Vec3
├── mesh.rs          # MeshData, vertex layout
├── geometry.rs      # GeometryMesh3D
├── draw_mesh.rs     # DrawMesh3D shader
├── camera.rs        # Camera3D orbital camera
└── colormap.rs      # Shader colormaps
```

### Success Criteria

- [ ] Surface plot renders at 60fps for 100×100 grid
- [ ] Scatter plot handles 10,000 points at 60fps
- [ ] Camera orbit/pan/zoom is smooth
- [ ] Colormaps match existing CPU implementations
- [ ] All examples updated and working

### Dependencies

Add to `Cargo.toml`:
```toml
glam = "0.29"  # For Mat4, Vec3 math
```

---

## 13. Quality Gates

### Per-Task Quality Checklist

Every task must meet these criteria before being marked complete:

```markdown
## Quality Checklist for [TASK-XXXX]

### Code Quality
- [ ] Code compiles with no warnings
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt` applied
- [ ] No `unwrap()` in library code (use `?` or `expect()`)
- [ ] No `panic!()` in library code
- [ ] Error handling uses `D3Result<T>`

### Testing
- [ ] Unit tests written for all public methods
- [ ] Edge cases tested (empty input, zero values, etc.)
- [ ] Integration tests if applicable
- [ ] Test coverage >80% for new code

### Documentation
- [ ] Rustdoc for all public items
- [ ] Examples in documentation
- [ ] Module-level documentation
- [ ] User guide updated if needed

### Performance
- [ ] Benchmarks added if performance-critical
- [ ] No obvious O(n²) or worse algorithms for large n
- [ ] Memory allocations minimized in hot paths

### Review
- [ ] Code review by another team member
- [ ] API design review if new public API
```

### Sprint Quality Gates

At the end of each sprint:

1. **Build Gate**
   ```bash
   cargo build --all-features --all-targets
   cargo build --release --all-features
   ```

2. **Test Gate**
   ```bash
   cargo test --all-features
   cargo test --release --all-features
   ```

3. **Lint Gate**
   ```bash
   cargo clippy --all-features -- -D warnings
   cargo fmt --check
   ```

4. **Doc Gate**
   ```bash
   cargo doc --all-features --no-deps
   # Check for missing docs warnings
   ```

5. **Coverage Gate**
   ```bash
   cargo tarpaulin --all-features --out Html
   # Target: >80%
   ```

### Phase Quality Gates

At the end of each phase:

| Gate | Target | Tool |
|------|--------|------|
| Build | ✅ Clean | cargo build |
| Tests | ✅ Pass | cargo test |
| Lint | 0 warnings | cargo clippy |
| Format | ✅ Clean | cargo fmt |
| Coverage | >80% | cargo-tarpaulin |
| Docs | 100% public | cargo doc |
| Examples | All run | cargo run --example |
| Benchmarks | Baseline set | cargo bench |

---

## 14. Risk Management

### Risk Register

| ID | Risk | Probability | Impact | Mitigation | Owner |
|----|------|-------------|--------|------------|-------|
| R1 | Makepad breaking changes | Medium | High | Pin to specific commit, adapter layer | Core |
| R2 | Performance issues | Medium | Medium | Early benchmarking, profiling | Core |
| R3 | Kurbo API changes | Low | Medium | Pin version, wrapper types | Graphics |
| R4 | Color crate limitations | Low | Low | Fallback to manual impl | Color |
| R5 | Scope creep | High | High | Strict MVP, change control | PM |
| R6 | Team availability | Medium | Medium | Cross-training, documentation | PM |
| R7 | Complex algorithms | Medium | Medium | Research spike, external libs | Layout |

### Contingency Plans

**R1: Makepad Breaking Changes**
- Maintain adapter layer between Makepad and library
- Document minimum supported Makepad version
- Create compatibility tests

**R5: Scope Creep**
- All new features require written approval
- Features outside MVP deferred to next phase
- Weekly scope review meetings

**R7: Complex Algorithms**
- Research spike before implementation
- Consider existing Rust crates
- Simplify if needed (e.g., skip Barnes-Hut)

---

## 15. Appendix

### A. Glossary

| Term | Definition |
|------|------------|
| **Scale** | Function mapping domain (data) to range (pixels) |
| **Domain** | Input space of a scale (e.g., 0-100) |
| **Range** | Output space of a scale (e.g., 0-500px) |
| **Tick** | Mark on axis showing a scale value |
| **Band** | Space allocated to a category in band scale |
| **Bandwidth** | Width of a band |
| **Nice** | Extending domain to round values |
| **Clamp** | Restricting output to range bounds |

### B. Reference Links

- [D3.js Documentation](https://d3js.org/)
- [D3 Scale Source](https://github.com/d3/d3-scale)
- [Makepad Repository](https://github.com/makepad/makepad)
- [Kurbo Documentation](https://docs.rs/kurbo)
- [Linebender Color](https://docs.rs/color)

### C. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-15 | Initial GLM plan |
| 2.0 | 2026-01-15 | Merged Claude plan, added verification |
| 2.1 | 2026-01-18 | Added Section 12A: GPU 3D Rendering System |

---

*Document maintained by Development Team*
*Last updated: 2026-01-18*
