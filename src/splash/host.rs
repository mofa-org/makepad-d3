//! `d3.Splash` — a runtime Splash-DSL host with the `d3.*` namespace
//! registered in its sandbox.
//!
//! The built-in `Splash` widget evaluates a Splash body string in an
//! isolated script VM, but that VM only registers the platform and
//! `makepad_widgets` modules — third-party widget crates are invisible in
//! `runsplash`-style sandboxes. `D3Splash` is a drop-in replacement that
//! additionally runs [`crate::script_mod`] in the isolate, so sandboxed
//! bodies can instantiate `d3.BarChart{...}` and friends directly.
//!
//! Feed it a body from Rust with `set_text` (the same streaming contract
//! the Markdown widget uses for ```runsplash fences), or from script via
//! `ui.host.set_text("d3.PieChart{data: [1 2 3]}")`.
//!
//! Known degradations vs. the built-in `Splash` (both need `pub(crate)`
//! makepad APIs; see the upstream-hook proposal in
//! `docs/SPLASH_INTEGRATION_DESIGN.md` §8.3):
//! - no `ui` global inside the sandbox, so body-level helper `fn`s cannot
//!   address widgets — inline `on_click: || {...}` handlers still can;
//! - dropped hosts do not reclaim their isolate VM until app shutdown;
//! - no network access inside the sandbox.

// The `script_mod!` macro generates a public registration function that
// cannot carry a doc comment.
#![allow(missing_docs)]

use makepad_widgets::widget_async::{CxSplashVmExt, SplashVmId, MAIN_SPLASH_VM_ID};
use makepad_widgets::widget_tree::CxWidgetExt;
use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets_internal.*
    use mod.widgets.*

    mod.d3.SplashBase = #(D3Splash::register_widget(vm))

    mod.d3.Splash = set_type_default() do mod.d3.SplashBase{
        width: Fill height: Fit
    }
}

const SPLASH_PREFIX: &str = "use mod.prelude.widgets.*\nView{height:Fit, ";
const SPLASH_EVAL_INSTRUCTION_LIMIT: usize = 200_000;

/// Sandboxed Splash-body host with `d3.*` registered (see module docs).
#[derive(Script, ScriptHook, WidgetRef, WidgetRegister)]
pub struct D3Splash {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    #[live]
    body: ArcStringMut,
    #[rust]
    vm_id: SplashVmId,
}

impl D3Splash {
    /// Stable identity for the body script, based on pointer address.
    fn self_id(&self) -> usize {
        self as *const Self as usize
    }

    fn eval_body(&mut self, cx: &mut Cx) {
        let body = self.body.as_ref();
        if body.is_empty() {
            return;
        }

        if self.vm_id == MAIN_SPLASH_VM_ID {
            self.vm_id = cx.alloc_splash_vm_with_network(false);
            // The whole point of this host: the d3 module joins the sandbox.
            cx.with_script_vm_id(self.vm_id, crate::script_mod);
        }

        let self_id = self.self_id();
        // Full code string: prefix + body (no closing brace - parser auto-closes)
        let code = format!("{}{}", SPLASH_PREFIX, body);

        // ScriptMod identity is stable (same file/line/column each call)
        let script_mod = ScriptMod {
            cargo_manifest_path: String::new(),
            module_path: String::new(),
            file: String::new(),
            line: self_id,
            column: 0,
            code: String::new(),
            values: vec![],
        };

        let vm_id = self.vm_id;
        let new_view = cx.with_script_vm_id(vm_id, |vm| {
            let value = vm.with_instruction_limit(SPLASH_EVAL_INSTRUCTION_LIMIT, |vm| {
                vm.eval_with_append_source(script_mod, &code, NIL.into())
            });
            if !value.is_err() && !value.is_nil() {
                Some(View::script_from_value(vm, value))
            } else {
                None
            }
        });

        if let Some(view) = new_view {
            self.view = view;
            cx.widget_tree_mark_dirty(self.uid);
        }
    }
}

impl D3SplashRef {
    /// Replace the sandboxed body and re-evaluate it (streaming-friendly:
    /// unchanged text is a no-op).
    pub fn set_text(&self, cx: &mut Cx, v: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_text(cx, v);
        }
    }
}

impl WidgetNode for D3Splash {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }
}

impl Widget for D3Splash {
    fn script_call(
        &mut self,
        vm: &mut ScriptVm,
        method: LiveId,
        args: ScriptValue,
    ) -> ScriptAsyncResult {
        if method == live_id!(text) {
            let str_val = vm.bx.heap.new_string_from_str(self.body.as_ref());
            return ScriptAsyncResult::Return(str_val);
        }
        if method == live_id!(set_text) {
            let value = super::vm_data::arg(vm, args, 0);
            if !value.is_nil() {
                let new_body = vm.bx.heap.temp_string_with(|heap, out| {
                    heap.cast_to_string(value, out);
                    out.to_string()
                });
                vm.with_cx_mut(|cx| {
                    self.set_text(cx, &new_body);
                });
            }
            return ScriptAsyncResult::Return(NIL);
        }
        ScriptAsyncResult::MethodNotFound
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn text(&self) -> String {
        self.body.as_ref().to_string()
    }

    fn set_text(&mut self, cx: &mut Cx, v: &str) {
        if self.body.as_ref() != v {
            self.body.set(v);
            self.eval_body(cx);
            self.redraw(cx);
        }
    }
}
