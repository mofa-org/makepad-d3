//! Splash (Makepad Script) integration — the `d3.*` widget namespace
//!
//! This module makes makepad-d3 charts directly usable from the Splash DSL.
//! It registers a `d3` script module containing chart widgets that render
//! through Makepad's GPU vector path renderer and bind data both
//! declaratively (`data:` / `labels:` properties) and imperatively
//! (`ui.chart.set_data(...)` script calls).
//!
//! # Registering
//!
//! Call [`crate::script_mod`] from your app's `AppMain::script_mod`, after
//! `makepad_widgets::script_mod`:
//!
//! ```rust,ignore
//! impl AppMain for App {
//!     fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
//!         makepad_widgets::script_mod(vm);
//!         makepad_d3::script_mod(vm);
//!         self::script_mod(vm)
//!     }
//!     fn handle_event(&mut self, cx: &mut Cx, event: &Event) { /* .. */ }
//! }
//! ```
//!
//! # Using from Splash
//!
//! ```splash,ignore
//! chart := d3.BarChart{
//!     height: 300
//!     data: [3 1 4 1 5 9 2 6]
//!     labels: ["a" "b" "c" "d" "e" "f" "g" "h"]
//!     on_click: |i| ui.status.set_text("clicked bar " + i)
//! }
//! Button{text: "Randomize" on_click: || ui.chart.set_data([2 7 1 8 2 8])}
//! ```

// The `script_mod!` macro generates a public registration function that
// cannot carry a doc comment.
#![allow(missing_docs)]

use makepad_widgets::*;

pub mod charts;
pub mod charts_3d;
pub mod charts_flow;
pub mod charts_hier;
pub mod charts_net;
pub mod charts_stat;
pub mod host;
pub mod vm_data;

pub use charts::{D3AreaChart, D3BarChart, D3LineChart, D3PieChart, D3ScatterChart};
pub use charts_3d::{D3Bar3D, D3Scatter3D, D3Surface3D};
pub use charts_flow::{D3ArcDiagram, D3ChordDiagram, D3Sankey};
pub use charts_hier::{D3CirclePack, D3Sunburst, D3TreeChart, D3Treemap};
pub use charts_net::{D3Contour, D3ForceGraph, D3Globe, D3Hexbin, D3Horizon, D3Ridgeline};
pub use charts_stat::{D3BoxPlot, D3Heatmap, D3Histogram, D3RadarChart};
pub use host::D3Splash;

// Creates the `d3` script module namespace. Chart and shader registrations
// from the other modules assign into it, so this must run first — the
// crate-level `script_mod` takes care of the ordering.
script_mod! {
    mod.d3 = {}
}
