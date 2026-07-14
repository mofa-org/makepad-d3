//! makepad-d3 × Splash demo
//!
//! The whole UI below is Splash DSL. The `d3.*` chart widgets come from
//! `makepad_d3::script_mod(vm)` — registered into the script VM at startup
//! and addressable like any built-in widget: declarative `data:` props,
//! `ui.chart.set_data(...)` calls from script closures, and `on_click` /
//! `on_hover` events flowing back into script.

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

    startup() do #(App::script_component(vm)){
        ui: Root{
            main_window := Window{
                pass.clear_color: vec4(0.086 0.094 0.125 1.0)
                window.inner_size: vec2(1140 960)
                window.title: "makepad-d3 × Splash"
                body +: {
                    padding: 16
                    flow: Down
                    spacing: 10

                    Label{
                        text: "makepad-d3 charts in Splash DSL"
                        draw_text.color: #fff
                        draw_text.text_style.font_size: 16
                    }
                    Label{
                        text: "d3.BarChart / d3.PieChart / d3.LineChart / d3.ScatterChart — data + events bound through the script VM"
                        draw_text.color: #x9aa0b0
                        draw_text.text_style.font_size: 10
                    }

                    View{
                        width: Fill height: 320 flow: Right spacing: 12

                        // Fixed-width children go before Fill siblings: a Fill
                        // child is deferred by the turtle, so a fixed child
                        // written after it would draw before the Fill slot
                        // resolves and the two would overlap.
                        pie := d3.PieChart{
                            width: 340 height: Fill
                            inner_radius: 0.55
                            data: [35 25 18 12 10]
                            labels: ["Rust" "C" "JS" "Go" "Py"]
                            on_click: |i| ui.status.set_text("slice clicked: " + i)
                        }

                        bar := d3.BarChart{
                            width: Fill height: Fill
                            data: [30 86 168 281 303 365]
                            labels: ["Jan" "Feb" "Mar" "Apr" "May" "Jun"]
                            on_click: |i| ui.status.set_text("bar clicked: " + i)
                            on_hover: |i| ui.status.set_text("bar hover: " + i)
                        }
                    }

                    View{
                        width: Fill height: 280 flow: Right spacing: 12

                        line := d3.LineChart{
                            width: Fill height: Fill
                        }

                        scatter := d3.ScatterChart{
                            width: Fill height: Fill
                            on_click: |i| ui.status.set_text("dot clicked: " + i)
                        }
                    }

                    View{
                        width: Fill height: Fit flow: Right spacing: 10
                        align: Align{y: 0.5}

                        ButtonFlat{
                            text: "Next dataset"
                            on_click: || {
                                bar_idx = bar_idx + 1
                                if bar_idx > 2 { bar_idx = 0 }
                                ui.bar.set_data(bar_data[bar_idx])
                                ui.status.set_text("bar dataset " + bar_idx)
                            }
                        }
                        ButtonFlat{
                            text: "Pie: equal split"
                            on_click: || {
                                ui.pie.set_data([20 20 20 20 20])
                                ui.status.set_text("pie set to equal split")
                            }
                        }
                        ButtonFlat{
                            text: "Line: ramp"
                            on_click: || {
                                ui.line.set_data([[0 5], [1 18], [2 12], [3 40], [4 33], [5 66], [6 58], [7 90]])
                                ui.status.set_text("line replaced with ramp")
                            }
                        }
                        status := Label{
                            text: "hover / click a mark, or press a button"
                            draw_text.color: #x7fd1a8
                            draw_text.text_style.font_size: 11
                        }
                    }

                    Label{
                        text: "Sandboxed splash app (d3.Splash host — an isolated VM with d3.* registered, runsplash-style):"
                        draw_text.color: #x9aa0b0
                        draw_text.text_style.font_size: 10
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
const SANDBOX_BODY: &str = "flow: Right spacing: 12 padding: 8 \
    d3.PieChart{width: 240 height: 180 inner_radius: 0.4 data: [4 3 2 1]} \
    d3.AreaChart{width: Fill height: 180 data: [3 7 4 9 6 12 8 15 11]}";

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
