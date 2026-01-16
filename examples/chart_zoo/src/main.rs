//! Chart Zoo - Showcase of all makepad-d3 chart types
//!
//! A comprehensive example demonstrating all chart types and their variations.

use makepad_widgets::*;
use makepad_d3::prelude::*;

mod charts;
use charts::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    FONT_DATA = {
        font_family: {
            latin = font("crate://self/resources/Manrope-Regular.ttf", 0.0, 0.0),
        }
    }

    // Clickable chart card for main page
    ChartCard = <RoundedView> {
        width: Fill,
        height: 280,
        margin: 8,
        padding: 12,
        cursor: Hand,

        show_bg: true,
        draw_bg: {
            color: #ffffff,
            border_radius: 8.0,
        }

        flow: Overlay,
    }

    // Chart title overlay
    ChartTitle = <View> {
        width: Fill,
        height: Fill,
        align: {x: 0.5, y: 0.0},
        padding: {top: 0},

        label = <RoundedView> {
            width: Fit,
            height: Fit,
            padding: {left: 12, right: 12, top: 6, bottom: 6},
            show_bg: true,
            draw_bg: {
                color: #ffffffdd,
                border_radius: 4.0,
            }

            label = <Label> {
                width: Fit,
                height: Fit,
                draw_text: {
                    color: #333333,
                    text_style: <FONT_DATA> { font_size: 13.0 }
                }
            }
        }
    }

    // Detail page chart card (larger)
    DetailChartCard = <RoundedView> {
        width: Fill,
        height: 350,
        margin: 10,
        padding: 15,

        show_bg: true,
        draw_bg: {
            color: #ffffff,
            border_radius: 8.0,
        }

        flow: Overlay,
    }

    // Back button style
    BackButton = <Button> {
        width: Fit,
        height: Fit,
        padding: {left: 16, right: 16, top: 8, bottom: 8},

        draw_bg: {
            instance color: #666666,
            instance color_hover: #4A90D9,
            instance border_radius: 6.0,

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let color = mix(self.color, self.color_hover, self.hover);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            color: #ffffff,
            text_style: <FONT_DATA> { font_size: 14.0 }
        }
        text: "< Back to Charts"
    }

    // Floating back button (blue, positioned by parent View)
    FloatingBackButton = <Button> {
        width: Fit,
        height: Fit,
        padding: {left: 16, right: 16, top: 10, bottom: 10},

        draw_bg: {
            instance color: #4A90D9,
            instance color_hover: #3A7BC8,
            instance border_radius: 8.0,

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let color = mix(self.color, self.color_hover, self.hover);
                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                sdf.fill(color);
                return sdf.result;
            }
        }
        draw_text: {
            color: #ffffff,
            text_style: <FONT_DATA> { font_size: 14.0 }
        }
        text: "< Back"
    }

    // Section header
    SectionHeader = <Label> {
        width: Fit,
        height: Fit,
        margin: {bottom: 10, top: 20},
        draw_text: {
            color: #555555,
            text_style: <FONT_DATA> { font_size: 16.0 }
        }
    }

    App = {{App}} {
        ui: <Window> {
            show_bg: true,
            width: Fill,
            height: Fill,

            draw_bg: {
                color: #f0f0f0
            }

            window: {
                inner_size: vec2(1400, 900)
            }

            body = <View> {
                width: Fill,
                height: Fill,
                flow: Overlay,

                // ============ Main Page - Chart Grid ============
                main_page = <ScrollXYView> {
                    visible: true,
                    flow: Down,
                    spacing: 0,
                    padding: 20,

                    // Header
                    <View> {
                        width: Fill,
                        height: Fit,
                        margin: {bottom: 20},
                        flow: Down,
                        align: {x: 0.5},
                        spacing: 10,

                        <Label> {
                            text: "Makepad D3 Chart Zoo"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 32.0 }
                            }
                        }
                        <Label> {
                            text: "Click any chart to see variations: gradients, animations, axis styles, and more"
                            draw_text: {
                                color: #666666,
                                text_style: <FONT_DATA> { font_size: 14.0 }
                            }
                        }
                    }

                    // Row 1: D3 Advanced Visualizations (Featured)
                    <SectionHeader> { text: "D3 Advanced Visualizations" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        force_graph_card = <ChartCard> {
                            force_graph = <ForceGraphWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Force-Directed Graph" } } }
                        }

                        chord_card = <ChartCard> {
                            chord = <ChordDiagramWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Chord Diagram" } } }
                        }

                        sankey_card = <ChartCard> {
                            sankey = <SankeyWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Sankey Diagram" } } }
                        }

                        treemap_card = <ChartCard> {
                            treemap = <TreemapWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Treemap" } } }
                        }
                    }

                    // Row 1b: More Hierarchy
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        circle_pack_card = <ChartCard> {
                            circle_pack = <CirclePackWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Circle Packing" } } }
                        }

                        sunburst_card = <ChartCard> {
                            sunburst = <SunburstWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Sunburst" } } }
                        }

                        <View> { width: Fill, height: 280 }
                        <View> { width: Fill, height: 280 }
                    }

                    // Row 2: Geographic Projections
                    <SectionHeader> { text: "Geographic & Projections" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        globe_card = <ChartCard> {
                            globe_map = <GlobeMapWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Orthographic Globe" } } }
                        }

                        bar_card = <ChartCard> {
                            bar_chart = <BarChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Bar Chart" } } }
                        }

                        line_card = <ChartCard> {
                            line_chart = <LineChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Line Chart" } } }
                        }

                        pie_card = <ChartCard> {
                            pie_chart = <PieChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Pie Chart" } } }
                        }
                    }

                    // Row 3: Statistical Charts
                    <SectionHeader> { text: "Statistical & Scientific" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        scatter_card = <ChartCard> {
                            scatter_chart = <ScatterChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Scatter Plot" } } }
                        }

                        histogram_card = <ChartCard> {
                            histogram = <HistogramWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Histogram" } } }
                        }

                        box_plot_card = <ChartCard> {
                            box_plot = <BoxPlotWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Box Plot" } } }
                        }

                        heatmap_card = <ChartCard> {
                            heatmap = <HeatmapWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Heatmap" } } }
                        }
                    }

                    // Row 3b: Scientific visualizations
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        contour_card = <ChartCard> {
                            contour = <ContourChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Contour" } } }
                        }

                        quiver_card = <ChartCard> {
                            quiver = <QuiverChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Vector Field" } } }
                        }

                        surface_card = <ChartCard> {
                            surface = <SurfacePlotWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "3D Surface" } } }
                        }

                        <View> { width: Fill, height: 280 }
                    }

                    // Row 3c: Financial Charts
                    <SectionHeader> { text: "Financial & Time Series" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        candlestick_card = <ChartCard> {
                            candlestick = <CandlestickWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Candlestick" } } }
                        }

                        area_card = <ChartCard> {
                            area_chart = <AreaChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Area Chart" } } }
                        }

                        stacked_bar_card = <ChartCard> {
                            stacked_bar = <BarChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stacked Bar" } } }
                        }

                        stacked_area_card = <ChartCard> {
                            stacked_area = <AreaChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stacked Area" } } }
                        }
                    }

                    // Row 4: Specialized Charts
                    <SectionHeader> { text: "Specialized Charts" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        radial_bar_card = <ChartCard> {
                            radial_bar = <RadialBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Radial Bar" } } }
                        }

                        donut_card = <ChartCard> {
                            donut_chart = <PieChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Donut Chart" } } }
                        }

                        multi_line_card = <ChartCard> {
                            multi_line = <LineChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Multi-Series Line" } } }
                        }

                        horizontal_bar_card = <ChartCard> {
                            horizontal_bar = <BarChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Horizontal Bar" } } }
                        }
                    }

                    // Row 5: Multi-Dimensional & Hierarchical
                    <SectionHeader> { text: "Multi-Dimensional & Hierarchical" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        radar_card = <ChartCard> {
                            radar_chart = <RadarChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Radar/Spider" } } }
                        }

                        tree_card = <ChartCard> {
                            tree_chart = <TreeChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Tree/Dendrogram" } } }
                        }

                        parallel_card = <ChartCard> {
                            parallel_chart = <ParallelCoordsWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Parallel Coords" } } }
                        }

                        <View> { width: Fill, height: Fill }
                    }

                    // Row 6: Distribution & Density Charts (NEW)
                    <SectionHeader> { text: "Distribution & Density" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        beeswarm_card = <ChartCard> {
                            beeswarm = <BeeswarmChart> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Beeswarm" } } }
                        }

                        bubble_card = <ChartCard> {
                            bubble = <BubbleChart> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Bubble Chart" } } }
                        }

                        hexbin_card = <ChartCard> {
                            hexbin = <HexbinChart> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Hexbin" } } }
                        }

                        calendar_card = <ChartCard> {
                            calendar = <CalendarChart> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Calendar Heatmap" } } }
                        }
                    }

                    // Row 7: Time Series & Comparison (NEW)
                    <SectionHeader> { text: "Time Series & Comparison" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        streamgraph_card = <ChartCard> {
                            streamgraph = <Streamgraph> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Streamgraph" } } }
                        }

                        ridgeline_card = <ChartCard> {
                            ridgeline = <RidgelinePlot> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Ridgeline" } } }
                        }

                        horizon_card = <ChartCard> {
                            horizon = <HorizonChart> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Horizon Chart" } } }
                        }

                        slope_card = <ChartCard> {
                            slope = <SlopeChart> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Slope Chart" } } }
                        }
                    }

                    // Row 8: Network & Text (NEW)
                    <SectionHeader> { text: "Network & Text" }
                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 0,

                        arc_diagram_card = <ChartCard> {
                            arc_diagram = <ArcDiagram> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Arc Diagram" } } }
                        }

                        word_cloud_card = <ChartCard> {
                            word_cloud = <WordCloud> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Word Cloud" } } }
                        }

                        edge_bundling_card = <ChartCard> {
                            edge_bundling = <EdgeBundlingWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Edge Bundling" } } }
                        }

                        <View> { width: Fill, height: 280 }
                    }

                    // Footer
                    <View> {
                        width: Fill,
                        height: Fit,
                        margin: {top: 40},
                        align: {x: 0.5},

                        <Label> {
                            text: "Built with Makepad D3 - GPU Accelerated Data Visualization"
                            draw_text: {
                                color: #999999,
                                text_style: <FONT_DATA> { font_size: 12.0 }
                            }
                        }
                    }
                }

                // ============ Line Chart Detail Page ============
                line_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 20,
                        margin: {bottom: 20},
                        align: {y: 0.5},

                        <Label> {
                            text: "Line Chart Variations"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 24.0 }
                            }
                        }
                    }

                    // Interpolation styles
                    <SectionHeader> { text: "Interpolation Styles" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <LineChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Linear" } } }
                        }
                        <DetailChartCard> {
                            <SmoothLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Smooth Curve" } } }
                        }
                        <DetailChartCard> {
                            <StepLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Step" } } }
                        }
                    }

                    // Styling
                    <SectionHeader> { text: "Styling Options" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <GradientLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Gradient Line" } } }
                        }
                        <DetailChartCard> {
                            <DashedLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Dashed Line" } } }
                        }
                        <DetailChartCard> {
                            <ThickLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Thick Line" } } }
                        }
                    }

                    // Multi-series
                    <SectionHeader> { text: "Multi-Series" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <MultiSeriesLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Multi-Series" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }
                }

                // ============ Bar Chart Detail Page ============
                bar_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 20,
                        margin: {bottom: 20},
                        align: {y: 0.5},

                        <Label> {
                            text: "Bar Chart Variations"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 24.0 }
                            }
                        }
                    }

                    // Basic variations
                    <SectionHeader> { text: "Basic Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <BarChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Basic Vertical" } } }
                        }
                        <DetailChartCard> {
                            <HorizontalBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Horizontal" } } }
                        }
                        <DetailChartCard> {
                            <GroupedBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Grouped" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <StackedBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stacked" } } }
                        }
                        <DetailChartCard> {
                            <DivergingBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Diverging" } } }
                        }
                        <DetailChartCard> {
                            <RoundedBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Rounded" } } }
                        }
                    }

                    // Styling
                    <SectionHeader> { text: "Gradient & Styling" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <GradientBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Rainbow Gradient" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }
                }

                // ============ Pie Chart Detail Page ============
                pie_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 20,
                        margin: {bottom: 20},
                        align: {y: 0.5},

                        <Label> {
                            text: "Pie Chart Variations"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 24.0 }
                            }
                        }
                    }

                    // Basic variations
                    <SectionHeader> { text: "Basic Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <PieChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Basic Pie" } } }
                        }
                        <DetailChartCard> {
                            <DonutWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Doughnut (50%)" } } }
                        }
                        <DetailChartCard> {
                            <ThinRingWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Thin Ring" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <ExplodedPieWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Exploded" } } }
                        }
                        <DetailChartCard> {
                            <SemiCircleWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Semi-Circle" } } }
                        }
                        <DetailChartCard> {
                            <ManySegmentsWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Many Segments" } } }
                        }
                    }

                    // Styling
                    <SectionHeader> { text: "Gradient & Effects" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <GradientPieWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Radial Gradient" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }
                }

                // ============ Scatter Chart Detail Page ============
                scatter_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 20,
                        margin: {bottom: 20},
                        align: {y: 0.5},

                        <Label> {
                            text: "Scatter Chart Variations"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 24.0 }
                            }
                        }
                    }

                    // Basic variations
                    <SectionHeader> { text: "Basic Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <ScatterChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Basic Scatter" } } }
                        }
                        <DetailChartCard> {
                            <MultiDatasetScatterWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Multi-Dataset" } } }
                        }
                        <DetailChartCard> {
                            <BubbleChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Bubble (x,y,r)" } } }
                        }
                    }

                    // Styling
                    <SectionHeader> { text: "Point Styles" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <LargePointsWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Large Points" } } }
                        }
                        <DetailChartCard> {
                            <SmallPointsWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Small Points" } } }
                        }
                        <DetailChartCard> {
                            <ColorGradientScatterWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Color Gradient" } } }
                        }
                    }

                    // Density
                    <SectionHeader> { text: "Data Density" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <DenseScatterWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Dense Correlation" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }
                }

                // ============ Area Chart Detail Page ============
                area_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 20,
                        margin: {bottom: 20},
                        align: {y: 0.5},

                        <Label> {
                            text: "Area Chart Variations"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 24.0 }
                            }
                        }
                    }

                    // Basic variations
                    <SectionHeader> { text: "Basic Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <AreaChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Basic Area" } } }
                        }
                        <DetailChartCard> {
                            <SmoothAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Smooth Curve" } } }
                        }
                        <DetailChartCard> {
                            <SteppedAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stepped" } } }
                        }
                    }

                    // Multi-series
                    <SectionHeader> { text: "Multi-Series" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <StackedAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stacked Area" } } }
                        }
                        <DetailChartCard> {
                            <GradientAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Gradient Fill" } } }
                        }
                        <DetailChartCard> {
                            <StreamGraphWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stream Graph" } } }
                        }
                    }

                    // Overlapping
                    <SectionHeader> { text: "Overlapping Series" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <MultiAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Multi-Area Overlay" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }
                }

                // ============ D3 Force Graph Detail Page ============
                force_graph_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill,
                        height: Fit,
                        flow: Right,
                        spacing: 20,
                        margin: {bottom: 20},
                        align: {y: 0.5},

                        <Label> {
                            text: "Force-Directed Graph - Interactive Network Visualization"
                            draw_text: {
                                color: #333333,
                                text_style: <FONT_DATA> { font_size: 24.0 }
                            }
                        }
                    }

                    // Large interactive view
                    <SectionHeader> { text: "Interactive Force Simulation" }
                    <DetailChartCard> {
                        height: 500,
                        <ForceGraphWidget> { width: Fill, height: Fill }
                    }

                    // Description
                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Hover over nodes to highlight connections. The graph uses D3-style forces:\n• ManyBodyForce: Repels nodes from each other\n• LinkForce: Attracts connected nodes\n• CenterForce: Keeps the graph centered\n• CollideForce: Prevents node overlap"
                            draw_text: {
                                color: #555555,
                                text_style: <FONT_DATA> { font_size: 13.0 }
                            }
                        }
                    }
                }

                // ============ Treemap Detail Page ============
                treemap_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Treemap - D3 Style Hierarchical Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Market Sectors (Squarify Tiling)" }
                    <DetailChartCard> {
                        height: 450,
                        <TreemapWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Market sector allocation treemap showing Technology, Healthcare, Finance, Energy, and Consumer sectors.\nRectangle area represents market value. Uses squarify tiling for optimal aspect ratios."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "D3 Flare Package Hierarchy (Tableau10 Colors)" }
                    <DetailChartCard> {
                        height: 450,
                        treemap_flare = <TreemapWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10},
                        <Label> {
                            text: "D3 flare visualization toolkit hierarchy - analytics, animate, data, display, scale, vis.\nInspired by the classic D3 treemap example using schemeTableau10 color palette."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Circle Pack Detail Page ============
                circle_pack_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Circle Packing - Nested Circle Layout"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Circle Packing Layout" }
                    <DetailChartCard> {
                        height: 500,
                        <CirclePackWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Circle packing displays hierarchical data as nested circles.\nEach circle's size represents its value, with children packed inside parents."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Sunburst Detail Page ============
                sunburst_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Sunburst - D3 Style Radial Partition Layout"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Programming Languages by Paradigm" }
                    <DetailChartCard> {
                        height: 1000,
                        <SunburstWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Programming language categorization by paradigm: OOP, Functional, Systems, Scripting.\nArc width represents relative usage/popularity of each language."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "D3 Flare Package Hierarchy (Rainbow Colors)" }
                    <DetailChartCard> {
                        height: 1000,
                        sunburst_flare = <SunburstWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10},
                        <Label> {
                            text: "D3 flare visualization toolkit hierarchy with rainbow color interpolation.\nCategories: analytics, animate, data, display, vis. Inspired by classic D3 sunburst example."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Globe Detail Page ============
                globe_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Orthographic Globe - Geographic Projection"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "3D Globe Visualization" }
                    <DetailChartCard> {
                        height: 500,
                        <GlobeMapWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Orthographic projection displays the Earth as seen from space.\nFeatures graticule lines (latitude/longitude grid) and continental outlines."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Histogram Detail Page ============
                histogram_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Histogram - Distribution Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Normal Distribution Histogram" }
                    <DetailChartCard> {
                        height: 400,
                        <HistogramWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Histogram shows the distribution of data by grouping values into bins.\nBar height represents the frequency of values in each bin range."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Box Plot Detail Page ============
                box_plot_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Box Plot - Statistical Summary"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Box and Whisker Plot" }
                    <DetailChartCard> {
                        height: 400,
                        <BoxPlotWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Box plots show the five-number summary: minimum, Q1, median, Q3, maximum.\nOutliers are shown as individual points outside the whiskers."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Heatmap Detail Page ============
                heatmap_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Heatmap - Matrix Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Activity Heatmap" }
                    <DetailChartCard> {
                        height: 400,
                        <HeatmapWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Heatmap displays matrix data using color intensity.\nUseful for showing patterns in time-series, correlations, or activity data."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Candlestick Detail Page ============
                candlestick_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Candlestick - Financial Chart"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "OHLC Candlestick Chart" }
                    <DetailChartCard> {
                        height: 400,
                        <CandlestickWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Candlestick charts show Open-High-Low-Close (OHLC) price data.\nGreen candles indicate price increase, red indicates decrease."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Radial Bar Detail Page ============
                radial_bar_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Radial Bar Chart - Circular Layout"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Monthly Data Radial View" }
                    <DetailChartCard> {
                        height: 450,
                        <RadialBarWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Radial bar charts display categorical data in a circular layout.\nIdeal for cyclic data like months, days of week, or hours."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Donut Detail Page ============
                donut_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Donut Chart - Ring Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Donut Chart Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <DonutWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Standard Donut" } } }
                        }
                        <DetailChartCard> {
                            <ThinRingWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Thin Ring" } } }
                        }
                        <DetailChartCard> {
                            <GradientPieWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Gradient Donut" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Donut charts are pie charts with a center hole.\nThe inner radius can be adjusted for different visual effects."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Multi-Line Detail Page ============
                multi_line_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Multi-Series Line Chart"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Multiple Data Series" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <MultiSeriesLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Overlaid Series" } } }
                        }
                        <DetailChartCard> {
                            <SmoothLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Smooth Curves" } } }
                        }
                        <DetailChartCard> {
                            <DashedLineWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Dashed Style" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Multi-series line charts compare multiple datasets over the same domain.\nDifferent colors and styles help distinguish each series."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Horizontal Bar Detail Page ============
                horizontal_bar_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Horizontal Bar Chart"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Horizontal Bar Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <HorizontalBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Basic Horizontal" } } }
                        }
                        <DetailChartCard> {
                            <RoundedBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Rounded Bars" } } }
                        }
                        <DetailChartCard> {
                            <GradientBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Gradient Bars" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Horizontal bar charts are useful when category labels are long.\nThey make it easier to read and compare values."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Stacked Bar Detail Page ============
                stacked_bar_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Stacked Bar Chart"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Stacked Bar Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <StackedBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stacked Vertical" } } }
                        }
                        <DetailChartCard> {
                            <GroupedBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Grouped" } } }
                        }
                        <DetailChartCard> {
                            <DivergingBarWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Diverging" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Stacked bar charts show part-to-whole relationships.\nEach segment represents a component of the total value."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Stacked Area Detail Page ============
                stacked_area_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Stacked Area Chart"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Stacked Area Variations" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <StackedAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stacked Area" } } }
                        }
                        <DetailChartCard> {
                            <StreamGraphWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Stream Graph" } } }
                        }
                        <DetailChartCard> {
                            <GradientAreaWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Gradient Fill" } } }
                        }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Stacked area charts show how multiple series contribute to a total over time.\nStream graphs center the baseline for a more organic look."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Chord Diagram Detail Page ============
                chord_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Chord Diagrams - D3 Style Relationship Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Phone Brand Switching (D3 Basic Chord)" }
                    <DetailChartCard> {
                        height: 900,
                        chord_phone = <ChordDiagramWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Shows how consumers switch between phone brands based on survey data.\nOuter arcs represent the proportion of respondents owning each brand.\nRibbons show the flow of users from their previous phone brand to current one."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "Country Debt (D3 Directed Chord)" }
                    <DetailChartCard> {
                        height: 900,
                        chord_debt = <ChordDiagramWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Visualizes debts between countries from the 2011 Euro crisis.\nDirected ribbons with arrow-like shape show the direction of debt.\nWider end indicates the debtor, narrower end points to the creditor."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "Software Dependencies (Rainbow Colors)" }
                    <DetailChartCard> {
                        height: 900,
                        chord_dependency = <ChordDiagramWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Shows dependencies among software packages in a class hierarchy.\nRainbow color scheme (d3.interpolateRainbow) distinguishes packages.\nDirected ribbons show import direction between packages."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "Hair Color Relationships (D3 Chord Diagram II)" }
                    <DetailChartCard> {
                        height: 900,
                        chord_hair = <ChordDiagramWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10},
                        <Label> {
                            text: "Shows relationships between hair colors adapted from Circos.\nData represents co-occurrences of hair colors in a population study.\nRibbons colored by target index show the strength of each relationship."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Sankey Diagram Detail Page ============
                sankey_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Sankey Diagrams - D3 Style Flow Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "UK Energy Flow (D3 Basic Sankey)" }
                    <DetailChartCard> {
                        height: 900,
                        sankey_energy = <SankeyWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Visualizes energy flow from primary sources through transformation to end use.\nShows how coal, gas, nuclear, and renewables are converted to electricity and heat.\nInspired by UK Department of Energy & Climate Change data."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "Titanic Survival (Parallel Sets Style)" }
                    <DetailChartCard> {
                        height: 400,
                        sankey_titanic = <SankeyWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Shows Titanic passenger survival rates by class and gender.\nLike parallel coordinates but for categorical data.\nFlow width represents the number of passengers in each category."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "Nike Quarterly Revenue (Financial Flow)" }
                    <DetailChartCard> {
                        height: 400,
                        sankey_nike = <SankeyWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10},
                        <Label> {
                            text: "Nike Q3 FY2019 revenue breakdown by product category and region.\nShows flow from Footwear/Apparel/Equipment through regions to total revenue.\nBased on Nike SEC filings financial data."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Radar Chart Detail Page ============
                radar_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Radar/Spider Chart - Multi-Axis Comparison"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Radar Chart" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <RadarChartWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Performance Comparison" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Radar charts display multivariate data on axes starting from the same point.\nUseful for comparing items across multiple dimensions."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Tree Chart Detail Page ============
                tree_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Tree/Dendrogram - D3 Style Hierarchical Layouts"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Basic Tidy Tree Layout" }
                    <DetailChartCard> {
                        height: 400,
                        <TreeChartWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Basic tree visualization with smooth bezier curve connections.\nUses Reingold-Tilford algorithm for tidy node positioning."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "D3 Flare Package Hierarchy (Tidy Tree)" }
                    <DetailChartCard> {
                        height: 400,
                        tree_flare = <TreeChartWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "D3 flare visualization toolkit hierarchy showing package structure.\nCategories: analytics, animate, data, display, vis with their respective modules."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "D3 Random Tree (Growing Animation)" }
                    <DetailChartCard> {
                        height: 600,
                        tree_cluster = <TreeChartWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10},
                        <Label> {
                            text: "Random tree that grows by periodically adding a child to a random node.\nNodes and links animate smoothly from old positions to new positions.\nStops at 500 nodes."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Parallel Coordinates Detail Page ============
                parallel_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Parallel Coordinates - Multi-Dimensional Data"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Parallel Coordinates" }
                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 0,

                        <DetailChartCard> {
                            <ParallelCoordsWidget> { width: Fill, height: Fill }
                            <ChartTitle> { label = { label = { text: "Multi-Attribute Comparison" } } }
                        }
                        <View> { width: Fill, height: Fill }
                        <View> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Parallel coordinates plot each data point as a polyline across parallel axes.\nUseful for identifying patterns and clusters in high-dimensional data."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Contour Chart Detail Page ============
                contour_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Contour Plot - Density & Topographic Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Smooth Contours (Volcano - D3 Style)" }
                    <DetailChartCard> {
                        height: 450,
                        contour_smooth = <ContourChartWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Volcano-like topographic data with grayscale color scheme.\nInspired by D3's smooth contours example using Maungawhau elevation data.\nHigher resolution grid (61x85) with 20 contour levels for smooth appearance."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "Original Contours (Viridis Color Scheme)" }
                    <DetailChartCard> {
                        height: 400,
                        contour_original = <ContourChartWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10, bottom: 25},
                        <Label> {
                            text: "Gaussian peaks pattern with viridis-like color scheme.\nShows multiple peaks, valleys, and ripple effects.\nUses marching squares algorithm for contour line extraction."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }

                    <SectionHeader> { text: "GeoTIFF Contours (Global Temperature - Magma)" }
                    <DetailChartCard> {
                        height: 350,
                        contour_geotiff = <ContourChartWidget> { width: Fill, height: Fill }
                    }
                    <View> {
                        width: Fill, height: Fit, margin: {top: 10},
                        <Label> {
                            text: "Simulated global surface temperature data in equirectangular projection.\nInspired by D3's GeoTIFF contours example with Magma color scheme.\nShows temperature variations from poles to equator with continental effects."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Quiver/Vector Field Detail Page ============
                quiver_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Vector Field - Flow & Direction Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Vector Field (Quiver Plot)" }
                    <DetailChartCard> {
                        height: 500,
                        <QuiverChartWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Vector field plots show direction and magnitude using arrows.\nColors indicate velocity/strength magnitude (blue=low, red=high).\nShows vortex and source/sink patterns in fluid dynamics."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ 3D Surface Plot Detail Page ============
                surface_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "3D Surface Plot - Mathematical Surface Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "3D Surface with Color Map" }
                    <DetailChartCard> {
                        height: 500,
                        <SurfacePlotWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "3D surface plots display z = f(x,y) with perspective projection.\nColor maps (Viridis) encode height values for better perception.\nIncludes rotation animation and wireframe overlay."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Beeswarm Chart Detail Page ============
                beeswarm_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Beeswarm Chart - Distribution Visualization"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Beeswarm Plot" }
                    <DetailChartCard> {
                        height: 500,
                        <BeeswarmChart> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Beeswarm plots show individual data points arranged along an axis.\nForce simulation prevents overlap while preserving the distribution shape.\nUseful for comparing distributions across categories."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Bubble Chart Detail Page ============
                bubble_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Bubble Chart - Multi-dimensional Scatter"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Bubble Chart" }
                    <DetailChartCard> {
                        height: 500,
                        <BubbleChart> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Bubble charts extend scatter plots with a third dimension: size.\nEach bubble's radius represents a value (e.g., population, revenue).\nColors can indicate categories for additional encoding."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Hexbin Chart Detail Page ============
                hexbin_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Hexbin Chart - Density Binning"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Hexagonal Binning" }
                    <DetailChartCard> {
                        height: 500,
                        <HexbinChart> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Hexbin plots aggregate points into hexagonal bins.\nColor intensity shows density/count in each bin.\nEfficient for visualizing large point datasets."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Calendar Chart Detail Page ============
                calendar_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Calendar Heatmap - Daily Activity"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Calendar Heatmap" }
                    <DetailChartCard> {
                        height: 500,
                        <CalendarChart> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Calendar heatmaps show daily values over time (GitHub-style).\nEach cell represents a day, color intensity shows activity level.\nGreat for tracking habits, contributions, or metrics over time."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Streamgraph Detail Page ============
                streamgraph_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Streamgraph - Flowing Time Series"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Streamgraph" }
                    <DetailChartCard> {
                        height: 500,
                        <Streamgraph> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Streamgraphs are stacked area charts centered around the baseline.\nThe flowing shape emphasizes change over time.\nUsed for music listening history, topic trends, etc."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Ridgeline Plot Detail Page ============
                ridgeline_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Ridgeline Plot - Distribution Comparison"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Ridgeline (Joy Plot)" }
                    <DetailChartCard> {
                        height: 500,
                        <RidgelinePlot> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Ridgeline plots show multiple distributions stacked with slight overlap.\nAlso known as Joy plots (after Joy Division album cover).\nExcellent for comparing distributions across time or categories."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Horizon Chart Detail Page ============
                horizon_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Horizon Chart - Compact Time Series"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Horizon Chart" }
                    <DetailChartCard> {
                        height: 500,
                        <HorizonChart> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Horizon charts fold values into layered bands for compact display.\nPositive values shown in blue, negative in red.\nCompares many time series in limited vertical space."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Slope Chart Detail Page ============
                slope_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Slope Chart - Before/After Comparison"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Slope Chart" }
                    <DetailChartCard> {
                        height: 500,
                        <SlopeChart> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Slope charts compare values between two time points.\nLine direction shows increase (green) or decrease (red).\nEffective for ranking changes and performance comparisons."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Arc Diagram Detail Page ============
                arc_diagram_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Arc Diagram - Network Connections"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Arc Diagram" }
                    <DetailChartCard> {
                        height: 500,
                        <ArcDiagram> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Arc diagrams show network connections with curved arcs.\nNodes arranged on a line, arcs connect related items.\nGood for sequential data and character interactions."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Word Cloud Detail Page ============
                word_cloud_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Word Cloud - Text Frequency"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Word Cloud" }
                    <DetailChartCard> {
                        height: 500,
                        <WordCloud> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Word clouds display text with size proportional to frequency.\nSpiral placement algorithm arranges words without overlap.\nPopular for text analysis and keyword visualization."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Edge Bundling Detail Page ============
                edge_bundling_detail_page = <ScrollXYView> {
                    visible: false,
                    flow: Down,
                    spacing: 0,
                    padding: {left: 20, right: 20, top: 60, bottom: 20},

                    <View> {
                        width: Fill, height: Fit, flow: Right, spacing: 20, margin: {bottom: 20}, align: {y: 0.5},
                        <Label> {
                            text: "Edge Bundling - Network Relationships"
                            draw_text: { color: #333333, text_style: <FONT_DATA> { font_size: 24.0 } }
                        }
                    }

                    <SectionHeader> { text: "Hierarchical Edge Bundling" }
                    <DetailChartCard> {
                        height: 1200,
                        <EdgeBundlingWidget> { width: Fill, height: Fill }
                    }

                    <View> {
                        width: Fill, height: Fit, margin: {top: 15},
                        <Label> {
                            text: "Edge bundling visualizes connections in networks by routing edges through a hierarchy.\nNodes are arranged in a circle grouped by category. Edges are bundled using Bezier curves.\nHover over nodes to highlight incoming (blue) and outgoing (red) connections."
                            draw_text: { color: #555555, text_style: <FONT_DATA> { font_size: 13.0 } }
                        }
                    }
                }

                // ============ Floating Back Button (stays on top-right) ============
                <View> {
                    width: Fill,
                    height: Fit,
                    flow: Right,
                    align: {x: 1.0, y: 0.0},
                    margin: {top: 20, right: 20},

                    floating_back_button = <FloatingBackButton> {
                        visible: false,
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum CurrentPage {
    #[default]
    Main,
    LineDetail,
    BarDetail,
    PieDetail,
    ScatterDetail,
    AreaDetail,
    ForceGraphDetail,
    TreemapDetail,
    CirclePackDetail,
    SunburstDetail,
    GlobeDetail,
    HistogramDetail,
    BoxPlotDetail,
    HeatmapDetail,
    CandlestickDetail,
    RadialBarDetail,
    DonutDetail,
    MultiLineDetail,
    HorizontalBarDetail,
    StackedBarDetail,
    StackedAreaDetail,
    ChordDetail,
    SankeyDetail,
    RadarDetail,
    TreeDetail,
    ParallelDetail,
    ContourDetail,
    QuiverDetail,
    SurfaceDetail,
    // New chart detail pages
    BeeswarmDetail,
    BubbleDetail,
    HexbinDetail,
    CalendarDetail,
    StreamgraphDetail,
    RidgelineDetail,
    HorizonDetail,
    SlopeDetail,
    ArcDiagramDetail,
    WordCloudDetail,
    EdgeBundlingDetail,
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    current_page: CurrentPage,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        // Register makepad_widgets first, then custom chart widgets
        makepad_widgets::live_design(cx);
        charts::live_design(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());
        self.match_event(cx, event);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Handle chart card clicks to navigate to detail pages
        // Basic charts
        if self.ui.view(id!(bar_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::BarDetail);
        }
        if self.ui.view(id!(line_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::LineDetail);
        }
        if self.ui.view(id!(pie_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::PieDetail);
        }
        if self.ui.view(id!(scatter_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::ScatterDetail);
        }
        if self.ui.view(id!(area_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::AreaDetail);
        }

        // D3 advanced visualization clicks
        if self.ui.view(id!(force_graph_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::ForceGraphDetail);
        }
        if self.ui.view(id!(treemap_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::TreemapDetail);
        }
        if self.ui.view(id!(circle_pack_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::CirclePackDetail);
        }
        if self.ui.view(id!(sunburst_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::SunburstDetail);
        }

        // Geographic
        if self.ui.view(id!(globe_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::GlobeDetail);
        }

        // Statistical charts
        if self.ui.view(id!(histogram_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::HistogramDetail);
        }
        if self.ui.view(id!(box_plot_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::BoxPlotDetail);
        }
        if self.ui.view(id!(heatmap_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::HeatmapDetail);
        }

        // Financial charts
        if self.ui.view(id!(candlestick_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::CandlestickDetail);
        }
        if self.ui.view(id!(stacked_bar_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::StackedBarDetail);
        }
        if self.ui.view(id!(stacked_area_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::StackedAreaDetail);
        }

        // Specialized charts
        if self.ui.view(id!(radial_bar_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::RadialBarDetail);
        }
        if self.ui.view(id!(donut_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::DonutDetail);
        }
        if self.ui.view(id!(multi_line_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::MultiLineDetail);
        }
        if self.ui.view(id!(horizontal_bar_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::HorizontalBarDetail);
        }

        // Network/Flow charts
        if self.ui.view(id!(chord_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::ChordDetail);
        }
        if self.ui.view(id!(sankey_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::SankeyDetail);
        }

        // Multi-dimensional & Hierarchical charts
        if self.ui.view(id!(radar_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::RadarDetail);
        }
        if self.ui.view(id!(tree_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::TreeDetail);
        }
        if self.ui.view(id!(parallel_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::ParallelDetail);
        }

        // Scientific charts
        if self.ui.view(id!(contour_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::ContourDetail);
        }
        if self.ui.view(id!(quiver_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::QuiverDetail);
        }
        if self.ui.view(id!(surface_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::SurfaceDetail);
        }

        // New chart types
        if self.ui.view(id!(beeswarm_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::BeeswarmDetail);
        }
        if self.ui.view(id!(bubble_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::BubbleDetail);
        }
        if self.ui.view(id!(hexbin_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::HexbinDetail);
        }
        if self.ui.view(id!(calendar_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::CalendarDetail);
        }
        if self.ui.view(id!(streamgraph_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::StreamgraphDetail);
        }
        if self.ui.view(id!(ridgeline_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::RidgelineDetail);
        }
        if self.ui.view(id!(horizon_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::HorizonDetail);
        }
        if self.ui.view(id!(slope_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::SlopeDetail);
        }
        if self.ui.view(id!(arc_diagram_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::ArcDiagramDetail);
        }
        if self.ui.view(id!(word_cloud_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::WordCloudDetail);
        }
        if self.ui.view(id!(edge_bundling_card)).finger_up(actions).is_some() {
            self.navigate_to(cx, CurrentPage::EdgeBundlingDetail);
        }

        // Handle floating back button
        if self.ui.button(id!(floating_back_button)).clicked(actions) {
            self.navigate_to(cx, CurrentPage::Main);
        }
    }
}

impl App {
    fn navigate_to(&mut self, cx: &mut Cx, page: CurrentPage) {
        self.current_page = page;

        // Hide all pages
        self.ui.view(id!(main_page)).set_visible(cx, false);
        self.ui.view(id!(line_detail_page)).set_visible(cx, false);
        self.ui.view(id!(bar_detail_page)).set_visible(cx, false);
        self.ui.view(id!(pie_detail_page)).set_visible(cx, false);
        self.ui.view(id!(scatter_detail_page)).set_visible(cx, false);
        self.ui.view(id!(area_detail_page)).set_visible(cx, false);
        self.ui.view(id!(force_graph_detail_page)).set_visible(cx, false);
        self.ui.view(id!(treemap_detail_page)).set_visible(cx, false);
        self.ui.view(id!(circle_pack_detail_page)).set_visible(cx, false);
        self.ui.view(id!(sunburst_detail_page)).set_visible(cx, false);
        self.ui.view(id!(globe_detail_page)).set_visible(cx, false);
        self.ui.view(id!(histogram_detail_page)).set_visible(cx, false);
        self.ui.view(id!(box_plot_detail_page)).set_visible(cx, false);
        self.ui.view(id!(heatmap_detail_page)).set_visible(cx, false);
        self.ui.view(id!(candlestick_detail_page)).set_visible(cx, false);
        self.ui.view(id!(radial_bar_detail_page)).set_visible(cx, false);
        self.ui.view(id!(donut_detail_page)).set_visible(cx, false);
        self.ui.view(id!(multi_line_detail_page)).set_visible(cx, false);
        self.ui.view(id!(horizontal_bar_detail_page)).set_visible(cx, false);
        self.ui.view(id!(stacked_bar_detail_page)).set_visible(cx, false);
        self.ui.view(id!(stacked_area_detail_page)).set_visible(cx, false);
        self.ui.view(id!(chord_detail_page)).set_visible(cx, false);
        self.ui.view(id!(sankey_detail_page)).set_visible(cx, false);
        self.ui.view(id!(radar_detail_page)).set_visible(cx, false);
        self.ui.view(id!(tree_detail_page)).set_visible(cx, false);
        self.ui.view(id!(parallel_detail_page)).set_visible(cx, false);
        self.ui.view(id!(contour_detail_page)).set_visible(cx, false);
        self.ui.view(id!(quiver_detail_page)).set_visible(cx, false);
        self.ui.view(id!(surface_detail_page)).set_visible(cx, false);
        self.ui.view(id!(beeswarm_detail_page)).set_visible(cx, false);
        self.ui.view(id!(bubble_detail_page)).set_visible(cx, false);
        self.ui.view(id!(hexbin_detail_page)).set_visible(cx, false);
        self.ui.view(id!(calendar_detail_page)).set_visible(cx, false);
        self.ui.view(id!(streamgraph_detail_page)).set_visible(cx, false);
        self.ui.view(id!(ridgeline_detail_page)).set_visible(cx, false);
        self.ui.view(id!(horizon_detail_page)).set_visible(cx, false);
        self.ui.view(id!(slope_detail_page)).set_visible(cx, false);
        self.ui.view(id!(arc_diagram_detail_page)).set_visible(cx, false);
        self.ui.view(id!(word_cloud_detail_page)).set_visible(cx, false);
        self.ui.view(id!(edge_bundling_detail_page)).set_visible(cx, false);

        // Show/hide floating back button (visible on detail pages, hidden on main)
        let show_floating_back = page != CurrentPage::Main;
        self.ui.button(id!(floating_back_button)).set_visible(cx, show_floating_back);

        // Show the target page
        match page {
            CurrentPage::Main => self.ui.view(id!(main_page)).set_visible(cx, true),
            CurrentPage::LineDetail => self.ui.view(id!(line_detail_page)).set_visible(cx, true),
            CurrentPage::BarDetail => self.ui.view(id!(bar_detail_page)).set_visible(cx, true),
            CurrentPage::PieDetail => self.ui.view(id!(pie_detail_page)).set_visible(cx, true),
            CurrentPage::ScatterDetail => self.ui.view(id!(scatter_detail_page)).set_visible(cx, true),
            CurrentPage::AreaDetail => self.ui.view(id!(area_detail_page)).set_visible(cx, true),
            CurrentPage::ForceGraphDetail => self.ui.view(id!(force_graph_detail_page)).set_visible(cx, true),
            CurrentPage::TreemapDetail => {
                self.ui.view(id!(treemap_detail_page)).set_visible(cx, true);
                self.ui.treemap_widget(id!(treemap_flare)).initialize_flare_data(cx);
            }
            CurrentPage::CirclePackDetail => self.ui.view(id!(circle_pack_detail_page)).set_visible(cx, true),
            CurrentPage::SunburstDetail => {
                self.ui.view(id!(sunburst_detail_page)).set_visible(cx, true);
                self.ui.sunburst_widget(id!(sunburst_flare)).initialize_flare_data(cx);
            }
            CurrentPage::GlobeDetail => self.ui.view(id!(globe_detail_page)).set_visible(cx, true),
            CurrentPage::HistogramDetail => self.ui.view(id!(histogram_detail_page)).set_visible(cx, true),
            CurrentPage::BoxPlotDetail => self.ui.view(id!(box_plot_detail_page)).set_visible(cx, true),
            CurrentPage::HeatmapDetail => self.ui.view(id!(heatmap_detail_page)).set_visible(cx, true),
            CurrentPage::CandlestickDetail => self.ui.view(id!(candlestick_detail_page)).set_visible(cx, true),
            CurrentPage::RadialBarDetail => self.ui.view(id!(radial_bar_detail_page)).set_visible(cx, true),
            CurrentPage::DonutDetail => self.ui.view(id!(donut_detail_page)).set_visible(cx, true),
            CurrentPage::MultiLineDetail => self.ui.view(id!(multi_line_detail_page)).set_visible(cx, true),
            CurrentPage::HorizontalBarDetail => self.ui.view(id!(horizontal_bar_detail_page)).set_visible(cx, true),
            CurrentPage::StackedBarDetail => self.ui.view(id!(stacked_bar_detail_page)).set_visible(cx, true),
            CurrentPage::StackedAreaDetail => self.ui.view(id!(stacked_area_detail_page)).set_visible(cx, true),
            CurrentPage::ChordDetail => {
                self.ui.view(id!(chord_detail_page)).set_visible(cx, true);
                // Initialize the four chord diagram variants
                self.ui.chord_diagram_widget(id!(chord_hair)).initialize_hair_color_data(cx);
                self.ui.chord_diagram_widget(id!(chord_phone)).initialize_phone_data(cx);
                self.ui.chord_diagram_widget(id!(chord_debt)).initialize_debt_data(cx);
                self.ui.chord_diagram_widget(id!(chord_dependency)).initialize_dependency_data(cx);
            }
            CurrentPage::SankeyDetail => {
                self.ui.view(id!(sankey_detail_page)).set_visible(cx, true);
                // Initialize the three Sankey diagram variants
                self.ui.sankey_widget(id!(sankey_energy)).initialize_energy_uk_data(cx);
                self.ui.sankey_widget(id!(sankey_titanic)).initialize_titanic_data(cx);
                self.ui.sankey_widget(id!(sankey_nike)).initialize_nike_data(cx);
            }
            CurrentPage::RadarDetail => self.ui.view(id!(radar_detail_page)).set_visible(cx, true),
            CurrentPage::TreeDetail => {
                self.ui.view(id!(tree_detail_page)).set_visible(cx, true);
                self.ui.tree_chart_widget(id!(tree_flare)).initialize_flare_data(cx);
                self.ui.tree_chart_widget(id!(tree_cluster)).initialize_random_data(cx);
            }
            CurrentPage::ParallelDetail => self.ui.view(id!(parallel_detail_page)).set_visible(cx, true),
            CurrentPage::ContourDetail => {
                self.ui.view(id!(contour_detail_page)).set_visible(cx, true);
                // Initialize the original viridis-style contour
                self.ui.contour_chart_widget(id!(contour_original)).initialize_original_style(cx);
                // Initialize the GeoTIFF-style contour
                self.ui.contour_chart_widget(id!(contour_geotiff)).initialize_geotiff_style(cx);
            }
            CurrentPage::QuiverDetail => self.ui.view(id!(quiver_detail_page)).set_visible(cx, true),
            CurrentPage::SurfaceDetail => self.ui.view(id!(surface_detail_page)).set_visible(cx, true),
            CurrentPage::BeeswarmDetail => self.ui.view(id!(beeswarm_detail_page)).set_visible(cx, true),
            CurrentPage::BubbleDetail => self.ui.view(id!(bubble_detail_page)).set_visible(cx, true),
            CurrentPage::HexbinDetail => self.ui.view(id!(hexbin_detail_page)).set_visible(cx, true),
            CurrentPage::CalendarDetail => self.ui.view(id!(calendar_detail_page)).set_visible(cx, true),
            CurrentPage::StreamgraphDetail => self.ui.view(id!(streamgraph_detail_page)).set_visible(cx, true),
            CurrentPage::RidgelineDetail => self.ui.view(id!(ridgeline_detail_page)).set_visible(cx, true),
            CurrentPage::HorizonDetail => self.ui.view(id!(horizon_detail_page)).set_visible(cx, true),
            CurrentPage::SlopeDetail => self.ui.view(id!(slope_detail_page)).set_visible(cx, true),
            CurrentPage::ArcDiagramDetail => self.ui.view(id!(arc_diagram_detail_page)).set_visible(cx, true),
            CurrentPage::WordCloudDetail => self.ui.view(id!(word_cloud_detail_page)).set_visible(cx, true),
            CurrentPage::EdgeBundlingDetail => self.ui.view(id!(edge_bundling_detail_page)).set_visible(cx, true),
        }

        self.ui.redraw(cx);
    }
}

app_main!(App);

fn main() {
    app_main();
}
