//! Chart widgets for the chart zoo example
//!
//! These widgets render various chart types using makepad-d3 primitives.

use makepad_widgets::*;

pub mod draw_primitives;
pub mod animation;
pub mod axis_renderer;
pub mod legend_renderer;

// Basic charts
pub mod bar_chart;
pub mod line_chart;
pub mod pie_chart;
pub mod scatter_chart;
pub mod area_chart;

// Chart variants for detail pages
pub mod line_variants;
pub mod bar_variants;
pub mod pie_variants;
pub mod scatter_variants;
pub mod area_variants;

// Advanced D3-style visualizations
pub mod force_graph;
pub mod treemap_chart;
pub mod circle_pack;
pub mod globe_map;
pub mod sunburst;

// Network/Flow charts
pub mod chord_diagram;
pub mod sankey;
pub mod arc_diagram;
pub mod edge_bundling;

// Statistical charts
pub mod histogram;
pub mod box_plot;
pub mod heatmap;
pub mod hexbin;
pub mod beeswarm;

// Financial charts
pub mod candlestick;

// Specialized charts
pub mod radial_bar;
pub mod radar_chart;
pub mod tree_chart;
pub mod parallel_coords;
pub mod slope_chart;
pub mod bubble_chart;
pub mod calendar_chart;

// Time series visualizations
pub mod streamgraph;
pub mod ridgeline;
pub mod horizon_chart;

// Text visualizations
pub mod word_cloud;

// Scientific/Mathematical visualizations
pub mod contour_chart;
pub mod quiver_chart;
pub mod surface_plot;
pub mod apollonius_problem;

pub use draw_primitives::*;
pub use axis_renderer::*;
pub use legend_renderer::*;
pub use bar_chart::*;
pub use line_chart::*;
pub use pie_chart::*;
pub use scatter_chart::*;
pub use area_chart::*;
pub use line_variants::*;
pub use bar_variants::*;
pub use pie_variants::*;
pub use scatter_variants::*;
pub use area_variants::*;
pub use force_graph::*;
pub use treemap_chart::*;
pub use circle_pack::*;
pub use globe_map::*;
pub use sunburst::*;
pub use chord_diagram::*;
pub use sankey::*;
pub use arc_diagram::*;
pub use edge_bundling::*;
pub use histogram::*;
pub use box_plot::*;
pub use heatmap::*;
pub use hexbin::*;
pub use beeswarm::*;
pub use candlestick::*;
pub use radial_bar::*;
pub use radar_chart::*;
pub use tree_chart::*;
pub use parallel_coords::*;
pub use slope_chart::*;
pub use bubble_chart::*;
pub use calendar_chart::*;
pub use streamgraph::*;
pub use ridgeline::*;
pub use horizon_chart::*;
pub use word_cloud::*;
pub use contour_chart::*;
pub use quiver_chart::*;
pub use surface_plot::*;
pub use apollonius_problem::*;

/// Register all chart widget live designs
pub fn live_design(cx: &mut Cx) {
    draw_primitives::live_design(cx);
    axis_renderer::live_design(cx);
    legend_renderer::live_design(cx);
    bar_chart::live_design(cx);
    line_chart::live_design(cx);
    pie_chart::live_design(cx);
    scatter_chart::live_design(cx);
    area_chart::live_design(cx);
    line_variants::live_design(cx);
    bar_variants::live_design(cx);
    pie_variants::live_design(cx);
    scatter_variants::live_design(cx);
    area_variants::live_design(cx);
    force_graph::live_design(cx);
    treemap_chart::live_design(cx);
    circle_pack::live_design(cx);
    globe_map::live_design(cx);
    sunburst::live_design(cx);
    chord_diagram::live_design(cx);
    sankey::live_design(cx);
    arc_diagram::live_design(cx);
    edge_bundling::live_design(cx);
    histogram::live_design(cx);
    box_plot::live_design(cx);
    heatmap::live_design(cx);
    hexbin::live_design(cx);
    beeswarm::live_design(cx);
    candlestick::live_design(cx);
    radial_bar::live_design(cx);
    radar_chart::live_design(cx);
    tree_chart::live_design(cx);
    parallel_coords::live_design(cx);
    slope_chart::live_design(cx);
    bubble_chart::live_design(cx);
    calendar_chart::live_design(cx);
    streamgraph::live_design(cx);
    ridgeline::live_design(cx);
    horizon_chart::live_design(cx);
    word_cloud::live_design(cx);
    contour_chart::live_design(cx);
    quiver_chart::live_design(cx);
    surface_plot::live_design(cx);
    apollonius_problem::live_design(cx);
}
