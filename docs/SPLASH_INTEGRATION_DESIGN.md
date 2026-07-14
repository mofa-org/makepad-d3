# Makepad-D3 × Splash Integration Design

**Version:** 1.1
**Created:** 2026-07-13
**Status:** **Implemented** (Phases 0–3 executed 2026-07-13 — see [§14](#14-implementation-status-2026-07-13); Phase 4 chart_zoo port remains) — supersedes the widget-layer portions of `DEVELOPMENT_PLAN_FINAL.md`
**Makepad reference:** `dev` branch @ `4f9ce7a8b` (makepad-widgets **2.0.0**), local checkout `~/home/makepad`

---

## Table of Contents

1. [Why this document exists](#1-why-this-document-exists)
2. [What changed in Makepad 2.0 ("Splash")](#2-what-changed-in-makepad-20-splash)
3. [Goals and non-goals](#3-goals-and-non-goals)
4. [Architecture overview](#4-architecture-overview)
5. [Module registration: the `d3.*` namespace](#5-module-registration-the-d3-namespace)
6. [Chart widget anatomy in the Script system](#6-chart-widget-anatomy-in-the-script-system)
7. [Data binding contract (Splash ⇄ Rust)](#7-data-binding-contract-splash--rust)
8. [Running d3 inside sandboxed Splash apps (runsplash)](#8-running-d3-inside-sandboxed-splash-apps-runsplash)
9. [render3d migration](#9-render3d-migration)
10. [Old → New API mapping (porting table)](#10-old--new-api-mapping-porting-table)
11. [Migration plan](#11-migration-plan)
12. [Risks and open questions](#12-risks-and-open-questions)
13. [Appendix: verified reference index](#13-appendix-verified-reference-index)
14. [Implementation status (2026-07-13)](#14-implementation-status-2026-07-13)

---

## 1. Why this document exists

makepad-d3 was designed (Jan 2026, `DEVELOPMENT_PLAN_FINAL.md`) against Makepad's **Live system**: `live_design!{}` DSL blocks, `#[derive(Live, LiveHook, Widget)]`, and `live_register`. In mid-2026 Makepad shipped **2.0**, which **removes the Live system entirely** and replaces it with the **Script/Splash system** — a runtime scripting VM (the *Splash DSL*) that now defines widgets, styling, shaders, and even whole mini-apps evaluated from strings at runtime.

Our `Cargo.toml` tracks `branch = "dev"`, but `Cargo.lock` pins the pre-2.0 commit `7a50694`. The library still builds against that pin; any `cargo update` will break every widget in the repo. More importantly, the new Splash runtime is the strategic reason to update: **Splash apps — including AI-generated `runsplash` mini-apps in AI Chat and any app that embeds the `Splash` widget — can instantiate widgets from script at runtime.** Registering makepad-d3's charts into that VM makes the whole d3 grammar (scales, shapes, layouts, hierarchies, geo, 3D) scriptable from Splash with zero recompilation.

Makepad 2.0 also ships a *minimal* built-in chart module (`widgets/src/chart.rs`: line/bar/area/scatter/candlestick/OHLC on a pan-zoom `ChartView`). It has **no script-facing data API** (charts self-generate fake data) and no d3-style grammar. makepad-d3's value in the 2.0 world is exactly that gap: real data binding from script, plus the full d3 feature set.

---

## 2. What changed in Makepad 2.0 ("Splash")

Verified against the makepad `dev` checkout; file references in [§13](#13-appendix-verified-reference-index).

### 2.1 The Script VM replaces the Live system

- The engine lives in `platform/script` (`makepad-script`). Core types: `ScriptVm`, `ScriptValue`, `ScriptObject`/`ScriptObjectRef` (GC heap), `ScriptFnRef` (script closures), `ScriptAsyncResult`.
- `live_design!{}` is **gone** (zero occurrences in `platform/` or `widgets/`). Its replacement is the **`script_mod!{}`** proc-macro (`platform/script/derive/src/lib.rs:18`), which embeds Splash source in the Rust file and generates `pub fn script_mod(vm: &mut ScriptVm) -> ScriptValue` for the enclosing module. `#(rust_expr)` splices a Rust expression's `ScriptValue` into the script — this is how Rust widget types enter the VM.
- Derives changed: `#[derive(Live, LiveHook, Widget)]` → **`#[derive(Script, ScriptHook, Widget)]`**. Field attributes `#[live]`, `#[live(default)]`, `#[rust]`, `#[walk]`, `#[layout]`, `#[redraw]`, `#[deref]`, `#[find]` survive with the same meaning. Two new **required** fields on non-`#[deref]` widgets:
  - `#[uid] uid: WidgetUid` — explicit widget uid (makepad commit `e61e3bfbc` "require explicit widget uids");
  - `#[source] source: ScriptObjectRef` — a rooted reference to the script object the widget was built from (commit `9d18257fd`); needed to call script closures (`on_click` etc.) and survive GC.
- Lifecycle hooks moved from `LiveHook` to **`ScriptHook`** (`platform/script/src/traits.rs:16`): `on_before_apply`, `on_after_apply(vm, apply, scope, value)`, `on_after_new`, `on_after_reload`, `on_custom_apply`, plus scoped variants. `Apply::Reload` fires for hot reload (`Event::LiveEdit`) and `Apply::ScriptReapply` for runtime re-apply.
- `LiveId`, `live_id!`, `ids!` **still exist** and are used for method names and widget paths.

### 2.2 Widget registration and styling defaults

The pattern used by every built-in widget (e.g. `widgets/src/chart.rs:3-92`):

```rust
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    // 1. register the Rust type => raw widget constructor
    mod.widgets.LineChartBase = #(LineChart::register_widget(vm))

    // 2. styled default (theme layer), becomes the name apps use
    mod.widgets.LineChart = set_type_default() do mod.widgets.LineChartBase{
        width: Fill
        height: Fill
        line_color: #x4fc3f7
    }
}
```

`register_widget(vm)` is generated by `derive(Widget)` (`widgets/derive_widget/src/derive_widget.rs:388`). A crate aggregates its modules in a crate-level `pub fn script_mod(vm: &mut ScriptVm)` (see `widgets/src/lib.rs:646`), and can run raw script with `script_eval!(vm, {...})` — the widgets crate uses that to compose the `mod.prelude.widgets` scope that Splash bodies import.

Sub-namespaces are plain script objects: the Liquid Glass kit registers `mod.widgets.glass = {}` then `mod.widgets.glass.GlassButtonBase = #(...)` (`widgets/src/glass_panel.rs:16-22`), which is why Splash code writes `glass.GlassButton{...}`. **This is the exact pattern `d3.*` will follow.**

### 2.3 App boot flow

`app_main!(App)` + `impl AppMain for App` (`platform/src/app_main.rs:227-300`):

```rust
#[derive(Script, ScriptHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] ...
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> ScriptValue {
        makepad_widgets::script_mod(vm);  // registers mod.widgets / theme / prelude
        self::script_mod(vm)              // this crate's script_mod! block; returns the App value
    }
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) { ... }
}
app_main!(App);
```

On `Event::Startup` the macro runs `App::script_mod(vm)` and constructs the app from the returned script value (`ScriptNew::script_from_value`). On `Event::LiveEdit` it re-runs it under `vm.with_reload` and re-applies with `Apply::Reload` — that is the 2.0 hot-reload path.

### 2.4 Script-facing widget methods and events

- **Methods:** `Widget::script_call(&mut self, vm, method: LiveId, args: ScriptValue) -> ScriptAsyncResult` (`widgets/src/widget.rs:223`). Reference implementation: `Label` (`widgets/src/label.rs:262-289`) implements `text()` / `set_text()` by reading positional args via `vm.bx.heap.vec_value(args_obj, 0, trap)` and returning `ScriptAsyncResult::Return(...)`. Splash code then calls `ui.my_label.set_text("hi")`.
- **Events → script closures:** widgets hold `#[live] on_click: ScriptFnRef` and fire it with `cx.widget_to_script_call(uid, NIL, self.source.clone(), self.on_click.clone(), &[/*args*/])` (`widgets/src/button.rs:422,463-466`). The `#[source]` field is what makes this possible.
- **Array/heap access:** `vm.bx.heap.vec_len(obj)` / `vec_value(obj, i, trap)` (see `widgets/src/animator.rs:48-72`), `value.as_number() / as_string() / as_object() / as_array() / as_color()`, strings via `heap.new_string_from_str` and `heap.temp_string_with`. `vm.with_cx_mut(|cx| ...)` reaches `Cx` from inside `script_call`.

### 2.5 GPU vector graphics — the new rendering substrate

Two big additions replace most of the custom-SDF plumbing our chart_zoo widgets hand-rolled:

- **`DrawVector`** (`draw/src/shader/draw_vector.rs:265`): immediate-mode GPU path renderer — `move_to / line_to / bezier_to / quad_to / close / rect / rounded_rect / circle / ellipse`, `fill / fill_gpu / fill_opts(join, miter, aa)`, `stroke / stroke_opts`, gradient paints (`add_gradient_row`, `set_paint`), and shadows (`shape_shadow`, `shadow`). One `begin()/end(cx)` session batches an entire chart into a single draw call. The built-in `ChartView` composes exactly this (bg `DrawColor` at depth 0, grid at 0.1, `DrawVector` at 2, `DrawText` at 3).
- **`Vector` widget** (`widgets/src/vector.rs:5-38`): a declarative, SVG-like scene graph **inside Splash script**: `Path` (full SVG `d:` path-data strings, tweenable), `Rect/Circle/Ellipse/Line/Polyline/Polygon`, `Group`, `Gradient/RadGradient/Stop`, `Filter/DropShadow`, transform nodes (`Rotate/Scale/Translate/SkewX/SkewY`), and `Tween` for animation. This means simple d3-style graphics can be authored *entirely in Splash*, and our Rust shape generators can also emit SVG path data for it.

Custom shaders are still available; they are now written in Splash script inside `script_mod!` and bound to `#[repr(C)]` structs with `#[derive(Script, ScriptHook)]` + `#[deref] draw_super: DrawQuad` and registered via `#(DrawX::script_shader(vm))` with inheritance `..mod.draw.DrawQuad` (`draw/src/shader/draw_quad.rs:1-108`). Math pods are `Vec2f/Vec3f/Vec4f`.

### 2.6 Runtime Splash apps ("splash app")

- The **`Splash` widget** (`widgets/src/splash.rs`) hosts a Splash DSL **string** (`body`) at runtime: it allocates an **isolated script VM** (`cx.alloc_splash_vm_with_network(allow_net)`), evaluates `use mod.prelude.widgets.*\nView{height:Fit, ` + body under a 200k-instruction budget, builds a `View` from the result, and injects a `ui` global so handlers can address widgets.
- The **Markdown widget maps ```` ```runsplash ```` fenced blocks to `Splash` widgets** (`widgets/src/markdown.rs:399-401`) — this is how AI Chat renders AI-generated mini-apps.
- Isolated splash VMs register **exactly two module sets, hardcoded**: `makepad_platform::script::script_mod(vm)` and `makepad_widgets::script_mod(vm)` (`widgets/src/widget_async.rs:284-337`). Third-party crates are invisible inside these sandboxes unless the host app adds them (see [§8](#8-running-d3-inside-sandboxed-splash-apps-runsplash)).
- Net-enabled VMs additionally get `mod.net`: `net.http_request`, `http_resource(url)`, `parse_json()`, `url_encode()` — the substrate for a d3-fetch equivalent.
- The authoritative DSL guide for app bodies is **`splash.md`** in the makepad repo root (layout rules, `:=` addressable ids, `on_render` dynamic lists, glass kit, gotchas). `makepad.splash` at the repo root is the Studio hub run-item registry — new example crates must be added there to be runnable from Studio (per makepad `AGENTS.md`).

---

## 3. Goals and non-goals

### Goals

| # | Goal | Acceptance |
|---|------|-----------|
| G1 | **Compile against makepad 2.0** (widgets 2.0.0, `dev` tip) | `cargo check` clean on lib + chart_zoo |
| G2 | **`d3.*` charts usable from any app's Splash code** | An app that calls `makepad_d3::script_mod(vm)` can write `d3.BarChart{ data: [...] }` in its `script_mod!` blocks, with hot reload |
| G3 | **`d3.*` charts usable inside runtime splash apps** (AI-chat-style `runsplash` bodies / `Splash`-widget hosts) | A `D3Splash` host widget evaluates a body containing `d3.*` charts in a sandboxed VM |
| G4 | **Script-facing data contract** | Declarative `data:` property + imperative `ui.chart.set_data(...)`, `on_*` event closures firing back into script |
| G5 | **Keep the d3 math core pure** | `scale/ shape/ color/ layout/ geo/ data/ axis` stay makepad-free (already true: only `render3d` + `lib.rs` touch makepad APIs) |

### Non-goals

- Replacing or forking makepad's built-in `mod.widgets.*Chart` widgets — `d3.*` is a separate, richer namespace; the built-ins stay untouched.
- Full d3.js API parity in the Splash DSL itself (the DSL is not JavaScript; the grammar surface is charts + options + data + events).
- Upstream makepad changes as a hard dependency. An upstream extension hook is proposed ([§8.3](#83-upstream-proposal-splash-vm-module-hook)) but every goal above is achievable without it.

---

## 4. Architecture overview

```
┌────────────────────────────────────────────────────────────────────┐
│ Splash apps (3 host tiers)                                         │
│  A) Rust app script_mod! blocks   B) Splash widget bodies          │
│     d3.LineChart{...}                (runsplash / AI chat)         │
│                                   C) .splash files (Studio)        │
├────────────────────────────────────────────────────────────────────┤
│ mod.d3  (script namespace)                 ← src/splash/mod.rs     │
│   d3.BarChart d3.LineChart d3.AreaChart d3.PieChart d3.Treemap …  │
│   d3.theme (color schemes)   D3Splash (sandbox host widget)        │
├────────────────────────────────────────────────────────────────────┤
│ Widget layer (NEW: src/splash/)                                    │
│   #[derive(Script, ScriptHook, Widget)] chart widgets              │
│   script_call: set_data/set_labels/set_domain/data…                │
│   events: on_click/on_hover/on_brush → ScriptFnRef                 │
│   rendering: DrawVector (paths) + DrawText (labels) + shaders      │
├────────────────────────────────────────────────────────────────────┤
│ d3 math core (UNCHANGED, pure Rust)                                │
│   scale · axis · shape · color · layout · geo · data · interaction │
├────────────────────────────────────────────────────────────────────┤
│ makepad 2.0: DrawVector · Vector · DrawQuad shaders · ScriptVm     │
└────────────────────────────────────────────────────────────────────┘
```

Design principles:

1. **The core stays a math kernel.** Scales, shape generators, layouts, projections keep their pure-Rust builder APIs. Widgets consume them; the VM never sees them directly.
2. **One widget = one d3 chart archetype**, mirroring the built-in `ChartView` composition pattern (`#[deref] chart_view` reuse) but driven by real data and d3 scales/axes instead of hardcoded ticks.
3. **Splash is the user interface; Rust is the engine.** Everything a Splash author can configure is either a `#[live]` property (declarative) or a `script_call` method (imperative). Everything interactive fires a `ScriptFnRef` callback.
4. **Render through `DrawVector` by default.** Our shape generators already emit polylines/polygons/arcs; they map 1:1 onto `move_to/line_to/bezier_to` sessions. Custom SDF shaders remain only where they buy something (heatmap fills, contour bands, 3D).

---

## 5. Module registration: the `d3.*` namespace

Crate-level entry point (new `src/splash/mod.rs`, re-exported from `lib.rs`):

```rust
/// Register the d3 module into a script VM.
/// Call after `makepad_widgets::script_mod(vm)`.
pub fn script_mod(vm: &mut ScriptVm) {
    charts::script_mod(vm);      // d3.BarChart, d3.LineChart, ...
    hierarchy::script_mod(vm);   // d3.Treemap, d3.Sunburst, d3.Pack, ...
    theme::script_mod(vm);       // d3.theme.category10, d3.theme.viridis, ...

    // Make `d3` visible wherever `use mod.prelude.widgets.*` is imported —
    // the same trick makepad uses to compose the prelude (widgets/src/lib.rs:637).
    script_eval!(vm, {
        mod.prelude.widgets.d3 = mod.d3
    });
}
```

Per-module registration follows the glass-kit pattern exactly:

```rust
script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3 = {}

    mod.d3.BarChartBase = #(D3BarChart::register_widget(vm))
    mod.d3.BarChart = set_type_default() do mod.d3.BarChartBase{
        width: Fill
        height: 300              // sandbox-friendly: Fill collapses in Fit parents (splash.md rule)
        plot_margin: Inset{left: 46 top: 12 right: 12 bottom: 28}
        bar_color: #x4285f4
        grid_color: #x2a2a3e
        label_color: #x9aa0b0
        bg_color: #x00000000     // transparent by default: glass-backdrop friendly
    }
    // ... one Base + styled default per chart
}
```

Naming decisions:

- **Namespace `d3`, not additions to `mod.widgets`** — avoids collision with the built-in `LineChart`/`BarChart` and gives Splash authors an unambiguous `d3.` prefix (mirrors `glass.`).
- **Defaults tuned for Splash bodies**: fixed default height (Splash `Fit`-parent rule), transparent background (glass backdrops), theme colors overridable per-instance.
- The app tier stays one line: `makepad_d3::script_mod(vm)` inside `AppMain::script_mod`, after `makepad_widgets::script_mod(vm)`.

---

## 6. Chart widget anatomy in the Script system

Skeleton for the archetype (bar chart shown; all charts share the shape):

```rust
#[derive(Script, ScriptHook, Widget)]
pub struct D3BarChart {
    #[uid] uid: WidgetUid,
    #[source] source: ScriptObjectRef,      // roots the script object; enables callbacks
    #[walk] walk: Walk,
    #[layout] layout: Layout,

    // draw layers (same depth discipline as built-in ChartView)
    #[redraw] #[live] draw_bg: DrawColor,   // depth 0
    #[live] draw_grid: DrawColor,           // depth 0.1
    #[live] draw_vector: DrawVector,        // depth 2 — bars/lines/areas
    #[live] draw_text: DrawText,            // depth 3 — axis labels

    // styling (script-settable)
    #[live] bar_color: Vec4f,
    #[live] grid_color: Vec4f,
    #[live] label_color: Vec4f,
    #[live] bg_color: Vec4f,
    #[live(0.7)] bar_width_fraction: f32,
    #[live] plot_margin: Inset,

    // event callbacks (script closures)
    #[live] on_click: ScriptFnRef,          // |index, x, y|
    #[live] on_hover: ScriptFnRef,          // |index|

    // engine state (never script-visible)
    #[rust] data: ChartData,                // d3 core type
    #[rust] x_scale: CategoryScale,
    #[rust] y_scale: LinearScale,
    #[rust] plot_rect: Rect,
    #[rust] hovered: Option<usize>,
    #[rust] dirty: bool,
}
```

**Rendering** (`draw_walk`): compute scales from `data` + `plot_rect` (d3 core), then one `DrawVector` session: `begin()` → grid lines → `rect()` per bar (or `move_to/line_to/bezier_to` for line/area/curve charts, `circle()` for scatter, arc tessellation for pie/sunburst) → `end(cx)`; axis ticks come from the existing `Axis`/`Tick` core (real d3 tick logic — better than the built-in `nice_ticks`) drawn with `DrawText`.

**Declarative data** — accept `data:` / `labels:` in the script object and convert **at apply time**, never holding raw `ScriptValue` in the widget:

```rust
impl ScriptHook for D3BarChart {
    fn on_after_apply(&mut self, vm: &mut ScriptVm, apply: &Apply,
                      _scope: &mut Scope, value: ScriptValue) {
        // read `data: [..]` / `labels: [..]` fields off the applied object,
        // copy into self.data (Vec<f64>/Vec<String>), set self.dirty
    }
}
```

This covers `Apply::New`, hot reload (`Apply::Reload`) and runtime re-apply (`Apply::ScriptReapply`) with one hook.

**Imperative API** — `script_call`, mirroring the `Label::set_text` idiom:

| Method (Splash side) | Args | Semantics |
|---|---|---|
| `set_data(values)` | array of numbers, `[x y]` pairs, or objects | replace series, refit domain unless pinned, redraw |
| `set_labels(labels)` | array of strings | category labels |
| `set_domain(min, max)` | numbers | pin y-domain (disables auto-fit) |
| `data()` | — | current values back as a script array |
| `set_options({...})` | object | batch styling/behavior updates |

**Events** — fired from `handle_event` hit-testing (reusing the pan/zoom + hover logic already written for chart_zoo):

```rust
if let Some(i) = self.hit_bar(fe.abs) {
    let args = [ScriptValue::from(i as f64), fe.abs.x.into(), fe.abs.y.into()];
    vm_missing_here; // via with_cx: cx.widget_to_script_call(
                     //   uid, NIL, self.source.clone(), self.on_click.clone(), &args)
}
```

so Splash authors write `d3.BarChart{ on_click: |i| ui.status.set_text("bar " + i) }`.

**Chart set** (phased, see [§11](#11-migration-plan)): Bar, Line (curve families via bezier), Area, Scatter, Pie/Donut, Heatmap, Histogram, Radar, Treemap, Sunburst, Pack, Force graph, Sankey, Chord, Geo/choropleth, Surface3D/Bar3D/Scatter3D. The 50 chart_zoo widgets are the porting backlog; each becomes a `d3.*` registration when ported.

---

## 7. Data binding contract (Splash ⇄ Rust)

Accepted `data:` shapes (all normalized into `ChartData` on the Rust side):

```splash
// 1. bare values (index = x)
d3.BarChart{ data: [3 1 4 1 5 9 2 6] labels: ["a" "b" "c" "d" "e" "f" "g" "h"] }

// 2. [x y] pairs
d3.LineChart{ data: [[0 1.2] [1 3.4] [2 2.9]] }

// 3. objects (multi-field, d3-style)
d3.ScatterChart{ data: [{x: 1 y: 2 r: 4} {x: 3 y: 1 r: 9}] }

// 4. multi-series
d3.LineChart{ series: [{name: "2025" data: [..]} {name: "2026" data: [..]}] }
```

Live-update loop, the killer demo for Splash apps (net-enabled sandbox):

```splash
use mod.prelude.widgets.*
let refresh = fn(){
    let res = fetch("https://api.example.com/prices", nil).await()
    if res != nil {
        let js = res.body.to_string().parse_json()
        ui.chart.set_data(js.values)
        ui.status.set_text("updated: " + js.values.len() + " points")
    }
}
View{ width: Fill height: Fit flow: Down spacing: 10
    glass.H1{text: "Live prices"}
    chart := d3.LineChart{ height: 300 }
    status := glass.Caption{text: "-"}
    glass.GlassButtonProminent{text: "Refresh" on_click: || refresh()}
}
```

Splash-authoring constraints we inherit (from `splash.md`) and bake into defaults: containers need `height: Fit`/fixed heights, `:=` ids for anything addressed via `ui.`, `for` loops render build-time only (dynamic chart lists go through `on_render` + `ui.x.render()`), numbers stay numeric (`"" + n` to display).

---

## 8. Running d3 inside sandboxed Splash apps (runsplash)

### 8.1 The gap

`Splash` widget sandboxes register **only** platform + `makepad_widgets` modules (hardcoded in `widget_async.rs:284-337`). `d3.*` is invisible in stock `runsplash` blocks regardless of what the host app registered in its **main** VM.

### 8.2 Solution: `D3Splash` host widget (Tier B)

Ship a drop-in replacement for `Splash` in makepad-d3. Same shape as `widgets/src/splash.rs` (~170 lines), with one addition after VM allocation:

```rust
self.vm_id = cx.alloc_splash_vm_with_network(self.allow_net);
cx.with_script_vm_id(self.vm_id, |vm| {
    makepad_d3::script_mod(vm);       // adds mod.d3 + prelude injection
});
// then eval_with_append_source(prefix + body) exactly like Splash
```

`CxSplashVmExt` (`alloc_splash_vm_with_network`, `with_script_vm_id`) is public, so this needs **no makepad changes**. Apps that render AI-generated markdown opt in by overriding the Markdown widget's `runsplash` template to instantiate `D3Splash` instead of `Splash` (the code-block template is a normal widget template in `mod.widgets.Markdown`'s defaults).

`D3Splash` keeps the sandbox guarantees: isolated heap, instruction budget, opt-in networking — d3 only *adds registered modules*, it does not widen the sandbox.

### 8.3 Upstream proposal: splash VM module hook

File a makepad PR adding an extension registry, e.g.:

```rust
Cx::register_splash_mod(|vm| makepad_d3::script_mod(vm));
```

consulted inside `alloc_splash_vm_with_network`. Then stock `Splash`/`runsplash` (and AI Chat itself) picks up `d3.*` with one registration call at app startup, and `D3Splash` becomes a thin alias. Until merged, Tier B covers the use case.

### 8.4 Prompting layer (AI-generated splash apps)

AI Chat's `runsplash` generation is prompt-driven (`splash.md` is the DSL guide). Once a host embeds `D3Splash`, add a **`d3-splash.md` guide** (this repo, `docs/`) in the same format — widget list, data shapes, gotchas — so agents can be pointed at it to generate d3 charts inside splash apps. Deliverable of Phase 4.

---

## 9. render3d migration

`src/render3d/draw.rs` is the only library code on the Live system (`live_design!` shader block + `#[derive(Live, LiveHook, LiveRegister)]` on `DrawMesh3d` and friends; `camera.rs` doc-comments; `colormap.rs` emits shader snippets).

Port per the `DrawQuad`/`DrawPbr` 2.0 idiom (`draw/src/shader/draw_quad.rs`, `draw/src/shader/draw_pbr.rs`):

1. Each `Draw*` struct → `#[derive(Script, ScriptHook)] #[repr(C)]` with `#[deref] draw_super: DrawQuad` (or `DrawCube`/geometry-appropriate base) and `#[live]` instance fields as `Vec2f/Vec3f/Vec4f/Mat4`.
2. The `live_design!` shader block → `script_mod!` entries: `mod.d3.DrawMesh3d = mod.std.set_type_default() do #(DrawMesh3d::script_shader(vm)){ ..mod.draw.DrawQuad  vertex: fn(){...} pixel: fn(){...} }`. Shader fns are Splash script; `mod.shader` provides `Sdf2d`, math comes from `mod.math`.
3. Colormap snippets in `colormap.rs` become script shader functions registered once under `mod.d3.shaderlib` and inherited with `..`/`+:` composition.
4. The existing GPU architecture doc (`docs/GPU_3D_RENDERING_ARCHITECTURE.md`) stays valid at the pipeline level (passes, depth, instancing); only its `live_design!` code samples are superseded by this section.

---

## 10. Old → New API mapping (porting table)

Mechanical mapping for the ~50 chart_zoo widgets, `main.rs`, and `render3d`:

| Live system (current code) | Script system (makepad 2.0) |
|---|---|
| `live_design!{ ... }` | `script_mod!{ ... }` (Splash syntax: no commas needed, `:=` ids, `+:` merge) |
| `pub X = {{X}} { ... }` | `mod.d3.XBase = #(X::register_widget(vm))` + `mod.d3.X = set_type_default() do mod.d3.XBase{ ... }` |
| `#[derive(Live, LiveHook, Widget)]` | `#[derive(Script, ScriptHook, Widget)]` + add `#[uid] uid: WidgetUid` + `#[source] source: ScriptObjectRef` |
| `#[derive(Live, LiveHook, LiveRegister)]` (shaders) | `#[derive(Script, ScriptHook)] #[repr(C)]` + register via `#(T::script_shader(vm))` |
| `impl LiveHook { fn after_apply(...) }` | `impl ScriptHook { fn on_after_apply(&mut self, vm, apply, scope, value) }` |
| `fn after_new_from_doc(cx)` | `fn on_after_new(&mut self, vm)` (Cx via `vm.with_cx_mut`) |
| `live_register` / `LiveRegister` trait | crate-level `pub fn script_mod(vm)` calling per-module generated `script_mod`s |
| `App { ui: WidgetRef }` + `impl LiveRegister for App` | `impl AppMain::script_mod(vm)` → `makepad_widgets::script_mod(vm); makepad_d3::script_mod(vm); self::script_mod(vm)` |
| `self.ui.button(id!(x))` | unchanged concept; paths via `ids!`, lookups now `self.ui.button(cx, ids!(x))` (Cx-first signatures) |
| `Vec2/Vec3/Vec4` in shader-adjacent fields | `Vec2f/Vec3f/Vec4f` (f32 pods; `DVec2` stays for layout math) |
| `apply_over(cx, live!{...})` | script re-apply (`Apply::ScriptReapply`) / setters; avoid in new code — expose `#[live]` props + `script_call` instead |
| custom SDF quad shaders for lines/bars | prefer `DrawVector` sessions; keep shaders only for fills/effects that need them |
| widget-visible data via Rust-only setters | `script_call` methods + `data:` apply-time parsing (this design, §6–§7) |

---

## 11. Migration plan

Phases are sequential; each leaves the repo green (`cargo check` + chart_zoo runs).

**Phase 0 — Pin & branch hygiene (0.5 day)**
- Add explicit `rev = "7a50694..."` to `Cargo.toml` on `main` (freeze the working baseline; today only `Cargo.lock` protects it).
- Create `splash-2.0` migration branch; bump dep there to `branch = "dev"` @ latest (or rev-pin `4f9ce7a8b`).

**Phase 1 — Library core on 2.0 (1–2 days)**
- Port `render3d` per [§9](#9-render3d-migration); everything else in `src/` compiles untouched (verified: only `render3d` + `lib.rs` docs reference makepad).
- CI gate: `cargo check --lib` on 2.0.

**Phase 2 — `src/splash/` widget layer MVP (1 week)**
- `script_mod` entry + `mod.d3` registration ([§5](#5-module-registration-the-d3-namespace)).
- First four widgets: `d3.BarChart`, `d3.LineChart`, `d3.AreaChart`, `d3.ScatterChart` on `DrawVector` + core scales/axes, with `data:`/`labels:` apply-time parsing, `set_data`/`set_labels`/`set_domain`/`data` script_calls, `on_click`/`on_hover` callbacks.
- New example `examples/splash_demo`: 2.0-style app (`AppMain::script_mod`) whose UI is written in Splash and drives charts from script (button → `set_data`, slider → domain).

**Phase 3 — Sandbox host (3 days)**
- `D3Splash` widget ([§8.2](#82-solution-d3splash-host-widget-tier-b)); demo: markdown view rendering a ```` ```runsplash ```` block with a `d3.*` chart via template override.
- Draft upstream hook PR ([§8.3](#83-upstream-proposal-splash-vm-module-hook)).

**Phase 4 — chart_zoo port in waves (2–4 weeks, parallelizable)**
- Wave 1: pie/donut, histogram, heatmap, radar, box plot.
- Wave 2: hierarchies (treemap, sunburst, pack, tree), flows (sankey, chord, arc diagram).
- Wave 3: force graph, geo (globe/choropleth), statistical (contour, hexbin, ridgeline, horizon).
- Wave 4: 3D (bar3d, scatter3d, surface) on ported render3d.
- Each wave: register in `mod.d3`, add a splash usage snippet to `docs/d3-splash.md`.

**Phase 5 — Docs & release (2 days)**
- `docs/d3-splash.md` authoring guide (splash.md-style, for humans and AI generation).
- README update (2.0 requirement, splash quick start), CHANGELOG, tag `0.2.0`, fold `main` ← `splash-2.0`.

---

## 12. Risks and open questions

| Risk | Impact | Mitigation |
|---|---|---|
| `dev` branch moves fast (2.0 APIs still churning — e.g. explicit-uid change landed recently) | rework during migration | rev-pin the migration branch; re-sync at phase boundaries only |
| `script_call` arg conventions could evolve | method layer churn | isolate all heap/arg parsing in one `src/splash/vm_data.rs` helper module (single choke point) |
| Holding script data refs across GC | crashes/UB | policy: **copy out at apply time**; only `ScriptObjectRef` (`#[source]`) is held, matching built-in widgets |
| Sandbox instruction budget (200k) with big datasets | truncated eval in runsplash | data via `set_data`/`net` (runtime calls are budgeted per-event, not per-eval); document dataset guidance in `d3-splash.md` |
| Name drift vs built-in charts (`mod.widgets.LineChart` vs `d3.LineChart`) | author confusion | namespace is always explicit (`d3.`); docs lead with it |
| Upstream hook rejected | Tier B only | `D3Splash` is fully self-sufficient; revisit if AI Chat exposes its own extension point |

Open questions to resolve during Phase 2:
1. Multi-series declarative syntax: `series:` array of objects vs. child nodes (`d3.Series{}` children) — prototype both, pick the one that composes with `on_render` list rebuilding.
2. Whether `d3.theme.*` should mirror `mod.theme` values (`set_type_default` composition) so charts restyle with app themes automatically.
3. Animation: reuse makepad `Animator` (script-driven keyframes, `widgets/src/animator.rs`) vs. our `ChartAnimator` — decide per-chart during Wave 1.

---

## 13. Appendix: verified reference index

All paths relative to the makepad `dev` checkout @ `4f9ce7a8b` (`~/home/makepad`):

| Topic | Reference |
|---|---|
| Script engine crate | `platform/script/` (`makepad-script`), VM in `src/main.rs`, heap/opcode modules |
| `script_mod!` proc macro | `platform/script/derive/src/lib.rs:18`, `script.rs:9-51` |
| `ScriptHook` trait | `platform/script/src/traits.rs:16-110` |
| App boot + hot reload | `platform/src/app_main.rs:227-300` (`AppMain`, `_app_main_event_closure`) |
| Widget trait, `script_call`, `script_call_live` | `widgets/src/widget.rs:198-232`, `:1786` (`register_widget` decl) |
| `register_widget` codegen | `widgets/derive_widget/src/derive_widget.rs:388` |
| Method-call reference impl | `widgets/src/label.rs:262-289` (`text`/`set_text`) |
| Script closures on widgets | `widgets/src/button.rs:422` (`on_click: ScriptFnRef`), `:463-466` (`widget_to_script_call`) |
| Heap array access | `widgets/src/animator.rs:48-72` (`vec_len`/`vec_value`) |
| Widget registration + defaults pattern | `widgets/src/chart.rs:3-92`; namespace pattern `widgets/src/glass_panel.rs:16-22` |
| Crate aggregation + prelude composition | `widgets/src/lib.rs:540-646` (`script_eval!` prelude merge at `:637`) |
| Built-in charts (gap analysis) | `widgets/src/chart.rs` (ChartView + 6 chart types, fake-data only, no `script_call`) |
| `DrawVector` GPU path API | `draw/src/shader/draw_vector.rs:265,331-700` |
| `Vector` declarative SVG widget | `widgets/src/vector.rs:5-38` (`Path.d` SVG data `:693,750`) |
| 2.0 shader idiom | `draw/src/shader/draw_quad.rs:1-108` (`script_shader`, `..mod.draw.DrawQuad`, `Vec4f` pods) |
| `Splash` host widget | `widgets/src/splash.rs` (prefix `:37-38`, instruction limit `:39`, eval `:47-97`) |
| Sandbox VM registration (hardcoded) | `widgets/src/widget_async.rs:284-337` (`CxSplashVmExt`) |
| `runsplash` → Splash mapping | `widgets/src/markdown.rs:399-401` |
| Splash DSL authoring guide | `splash.md` (repo root); glass kit section |
| Studio run-item registry | `makepad.splash` (repo root); Studio runbook in `AGENTS.md` |
| makepad-d3 baseline | `Cargo.lock` pin `7a50694` (widgets 1.0.0-era) — lib `cargo check` clean 2026-07-13 |

---

## 14. Implementation status (2026-07-13)

Phases 0–3 were executed the same day against makepad `dev` @ `4f9ce7a8b`.
Verified: `cargo check`/`clippy` clean on the new modules, all **813** lib
tests pass, and `examples/splash_demo` runs with zero script errors — all
five `d3.*` charts render from Splash DSL, and a `d3.Splash` sandbox
isolate renders charts from a runtime body string (screenshot-verified).

**What landed**

| Piece | Where |
|---|---|
| Dependency bump to 2.0 | `Cargo.toml` — **path dep on `../makepad/widgets`** |
| render3d Script port (6 shaders) | `src/render3d/draw.rs` (`script_mod!` + `script_shader`, `Vec4`→`Vec4f`, derives swapped) |
| `d3.*` namespace + prelude injection | `src/splash/mod.rs` (`mod.d3 = {}`), `src/splash/charts.rs` (`mod.prelude.widgets.d3 = mod.d3`) |
| Chart widgets (Bar, Line, Area, Scatter, Pie) | `src/splash/charts.rs` — `DrawVector` rendering, d3 scales/ticks, `data:`/`labels:` apply-time parsing, `set_data`/`set_labels`/`set_domain`/`data()` script calls, `on_click`/`on_hover` `ScriptFnRef` events, demo data when empty |
| VM data conversion choke point | `src/splash/vm_data.rs` |
| Sandbox host (Tier B) | `src/splash/host.rs` — `d3.Splash`, isolate VM + `crate::script_mod` registration + body eval |
| Registration entry point | `makepad_d3::script_mod(vm)` in `src/lib.rs` |
| Proof app | `examples/splash_demo` (`cargo run --example splash_demo`) |

**Deviations from the design**

1. **Path dependency instead of a git rev pin** (§11 Phase 0): the makepad
   repo vendors a full pre-2.0 crate copy under `old/`, so two packages
   named `makepad-widgets` exist in the repo and cargo rejects it as a git
   source ("unexpectedly found multiple copies"). Until upstream removes
   or renames `old/`, consumers need a sibling checkout.
2. **Prelude injection moved into the `script_mod!` block** (§5): no need
   for `script_eval!` from Rust — the last line of the charts block does
   `mod.prelude.widgets.d3 = mod.d3`.
3. **`D3Splash` ships with documented degradations** (§8.2): three helpers
   the built-in `Splash` uses are `pub(crate)` in makepad
   (`inject_splash_ui_handle`, `mark_splash_isolate_dead`,
   `handle_splash_network_responses`), so sandbox bodies have no `ui`
   global inside helper `fn`s (inline handlers work), dropped hosts do not
   reclaim their isolate, and networking is off. These become trivial once
   the §8.3 upstream hook (plus making those helpers public) lands.

**Findings for chart/DSL authors (Phase 4 porting notes)**

- **Nested array literals need commas** — `[[1 2] [3 4]]` parses as
  indexing (`[1 2]` indexed by `3`…); write `[[1 2], [3 4]]`.
- **Layout: fixed-size charts before `Fill` siblings** in a `flow: Right`
  row. A `Fill` child is deferred by the turtle; a fixed child written
  after it draws before the deferred slot resolves and the two overlap.
- **Upstream gap: `DrawText::draw_abs` does not render** on the current
  dev tip — chart axis/slice labels are invisible. The built-in
  `mod.widgets` charts (e.g. `CandlestickChart`) exhibit exactly the same
  behavior (verified side by side), while `Label` widgets render fine, so
  this is makepad's text pipeline, not our port. Re-test on makepad
  updates; consider overlay `Label`s only if upstream stalls.

**Remaining:** §11 Phase 4 (port the ~50 chart_zoo widgets into `d3.*` —
`[[example]] chart_zoo` is commented out in `Cargo.toml` until then) and
Phase 5 (authoring guide `docs/d3-splash.md`, release).

---

## 15. Phase 4 + 5 completion (2026-07-14)

Phases 4 and 5 were executed the day after the migration. The `d3.*`
namespace now holds **21 chart widgets** (screenshot-verified in the
`splash_demo` gallery, zero script errors, clippy-clean, 813 tests):

| Wave | Widgets | Module |
|---|---|---|
| MVP | BarChart, LineChart, AreaChart, ScatterChart, PieChart | `src/splash/charts.rs` |
| 1 — statistical | Histogram, Heatmap, RadarChart, BoxPlot | `src/splash/charts_stat.rs` |
| 2 — hierarchies | Treemap, Sunburst, CirclePack, TreeChart | `src/splash/charts_hier.rs` |
| 2 — flows | Sankey, ChordDiagram, ArcDiagram | `src/splash/charts_flow.rs` |
| 3 — networks/density/geo | ForceGraph, Hexbin, Ridgeline, Horizon, Contour, Globe | `src/splash/charts_net.rs` |
| 4 — 3D | Surface3D, Scatter3D, Bar3D | `src/splash/charts_3d.rs` |

Phase 5 deliverables: `docs/d3-splash.md` (Splash authoring guide for all
widgets), `CHANGELOG.md`, version **0.2.0**, README catalog.

Implementation notes:

- **3D rendering strategy** (deviation from §9's shader-first assumption):
  faces come from the render3d core as projected screen quads and are
  painted back-to-front through `DrawVector` with CPU-side lighting —
  simpler than the rect-bound quad shaders and pixel-identical for flat
  faces. The ported `Draw*3D` shader types remain available.
- **Core layout bugs found**: `TreemapLayout` and `PackLayout` in
  `src/layout/hierarchy/` produce overlapping/degenerate geometry on the
  2.0 build (pre-existing; see `treemap-bug-glm.md`). The widgets are
  self-sufficient — inline squarify and greedy tangent bubble packing —
  until the core layouts are fixed.
- **Sankey/Chord** are compact d3-style reimplementations (layered
  longest-path + relaxation; matrix sub-arc allocation with quadratic
  ribbons) rather than ports of the 2.5k-line zoo widgets.
- The zoo's remaining exotic variants (calendar, word cloud, slope,
  beeswarm, streamgraph, parallel coords, …) are covered conceptually by
  the 21 widgets' primitives and stay as reference material only; chart_zoo
  is retired as a build target.
