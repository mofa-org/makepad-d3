//! makepad-d3 × Splash gallery
//!
//! The whole UI below is Splash DSL. Every `d3.*` chart widget in the
//! library is instantiated here — registered into the script VM by
//! `makepad_d3::script_mod(vm)` and addressable like any built-in widget:
//! declarative `data:` props, `ui.chart.set_data(...)` calls from script
//! closures, and `on_click`/`on_hover` events flowing back into script.

use makepad_widgets::*;

app_main!(App);

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Commas are required between adjacent bracketed elements — bare
    // `[..] [..]` parses as indexing in Splash.
    let bar_data = [
        [30 86 168 281 303 365],
        [200 130 90 240 180 60],
        [12 40 22 90 55 71]
    ]
    let bar_idx = 0

    let Cell = View{
        width: Fill height: Fill flow: Down spacing: 2
        title := Label{text: "" draw_text.color: #x9aa0b0 draw_text.text_style.font_size: 9}
        body := View{width: Fill height: Fill}
    }

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                pass.clear_color: vec4(0.086 0.094 0.125 1.0)
                window.inner_size: vec2(1660 1050)
                window.title: "makepad-d3 × Splash — full d3.* gallery"
                body +: {
                    padding: 12
                    flow: Down
                    spacing: 8

                    View{ width: Fill height: Fit flow: Right spacing: 12 align: Align{y: 0.5}
                        Label{
                            text: "makepad-d3: 21 chart widgets in Splash DSL"
                            draw_text.color: #fff
                            draw_text.text_style.font_size: 14
                        }
                        ButtonFlat{
                            text: "Next bar dataset"
                            on_click: || {
                                bar_idx = bar_idx + 1
                                if bar_idx > 2 { bar_idx = 0 }
                                ui.bar.set_data(bar_data[bar_idx])
                                ui.status.set_text("bar dataset " + bar_idx)
                            }
                        }
                        status := Label{
                            text: "hover / click marks, drag the 3D charts and the globe"
                            draw_text.color: #x7fd1a8
                            draw_text.text_style.font_size: 10
                        }
                    }

                    // Row 1: base charts
                    View{ width: Fill height: 150 flow: Right spacing: 8
                        bar := d3.BarChart{
                            width: Fill height: Fill
                            data: [30 86 168 281 303 365]
                            labels: ["Jan" "Feb" "Mar" "Apr" "May" "Jun"]
                            on_click: |i| ui.status.set_text("bar clicked: " + i)
                        }
                        d3.PieChart{
                            width: Fill height: Fill
                            inner_radius: 0.55
                            data: [35 25 18 12 10]
                            on_click: |i| ui.status.set_text("slice clicked: " + i)
                        }
                        d3.LineChart{ width: Fill height: Fill }
                        d3.ScatterChart{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("dot clicked: " + i)
                        }
                        d3.AreaChart{ width: Fill height: Fill }
                    }

                    // Row 2: statistical
                    View{ width: Fill height: 150 flow: Right spacing: 8
                        d3.Histogram{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("histogram bin: " + i)
                        }
                        d3.Heatmap{
                            width: Fill height: Fill
                            on_click: |r c| ui.status.set_text("heatmap cell: " + r + "," + c)
                        }
                        d3.RadarChart{ width: Fill height: Fill }
                        d3.BoxPlot{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("box series: " + i)
                        }
                        d3.Sunburst{ width: Fill height: Fill }
                    }

                    // Row 3: hierarchies + flows
                    View{ width: Fill height: 150 flow: Right spacing: 8
                        d3.Treemap{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("treemap leaf: " + i)
                        }
                        d3.CirclePack{ width: Fill height: Fill }
                        d3.TreeChart{ width: Fill height: Fill }
                        d3.Sankey{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("sankey node: " + i)
                        }
                        d3.ChordDiagram{ width: Fill height: Fill }
                    }

                    // Row 4: networks + densities
                    View{ width: Fill height: 150 flow: Right spacing: 8
                        d3.ArcDiagram{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("arc node: " + i)
                        }
                        d3.ForceGraph{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("graph node: " + i)
                        }
                        d3.Hexbin{ width: Fill height: Fill }
                        d3.Ridgeline{ width: Fill height: Fill }
                        d3.Horizon{ width: Fill height: Fill }
                    }

                    // Row 5: contour, geo, 3D
                    View{ width: Fill height: 190 flow: Right spacing: 8
                        d3.Contour{ width: Fill height: Fill }
                        d3.Globe{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("globe marker: " + i)
                        }
                        d3.Surface3D{ width: Fill height: Fill }
                        d3.Scatter3D{ width: Fill height: Fill }
                        d3.Bar3D{ width: Fill height: Fill }
                    }

                    // Row 6: sandboxed splash app (isolated VM, runsplash-style)
                    Label{
                        text: "Sandboxed splash app (d3.Splash host — an isolated VM with d3.* registered):"
                        draw_text.color: #x9aa0b0
                        draw_text.text_style.font_size: 9
                    }
                    host := d3.Splash{ width: Fill height: Fit }
                }
            }
        }
    }
}

#[derive(Script, ScriptHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
}

/// Body for the sandboxed `d3.Splash` host — evaluated in an isolated VM,
/// exactly like a ```runsplash fence in an AI-chat markdown stream.
const SANDBOX_BODY: &str = "flow: Right spacing: 8 padding: 4 \
    d3.PieChart{width: 200 height: 120 inner_radius: 0.4 data: [4 3 2 1]} \
    d3.AreaChart{width: Fill height: 120 data: [3 7 4 9 6 12 8 15 11]} \
    d3.Sankey{width: Fill height: 120}";

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        if let Some(mut host) = self
            .ui
            .widget(cx, ids!(host))
            .borrow_mut::<makepad_d3::splash::D3Splash>()
        {
            host.set_text(cx, SANDBOX_BODY);
        }
    }
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);
        makepad_d3::script_mod(vm);
        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
