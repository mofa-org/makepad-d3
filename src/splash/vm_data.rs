//! Conversions between Splash VM values and Rust chart data.
//!
//! All helpers copy data out of the VM heap immediately — chart widgets
//! never hold unrooted [`ScriptValue`]s across garbage collections. These
//! are the single choke point for script argument parsing, so VM API
//! changes only touch this file.

use makepad_widgets::*;

/// Iterate the elements of a script sequence (a heap array like `[1 2 3]`
/// or an object carrying vec storage) and collect them into a Vec.
fn collect_elements(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<ScriptValue>> {
    if let Some(arr) = value.as_array() {
        let len = vm.bx.heap.array_len(arr);
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            out.push(vm.bx.heap.array_index_unchecked(arr, i));
        }
        return Some(out);
    }
    if let Some(obj) = value.as_object() {
        let len = vm.bx.heap.vec_len(obj);
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let trap = vm.bx.threads.cur().trap.pass();
            let v = vm.bx.heap.vec_value(obj, i, trap);
            if v.is_err() {
                break;
            }
            out.push(v);
        }
        return Some(out);
    }
    None
}

/// Read a script sequence as a vec of numbers: `[3 1 4 1 5]`.
///
/// Non-numeric elements are skipped. Returns `None` when the value is not
/// a sequence at all (e.g. `nil`), so callers can distinguish "not set"
/// from "empty".
pub fn to_f64_vec(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<f64>> {
    let items = collect_elements(vm, value)?;
    Some(items.iter().filter_map(|v| v.as_number()).collect())
}

/// Read a script sequence as (x, y) points.
///
/// Accepted element shapes, mirroring d3 conventions:
/// - a bare number: `[3 1 4]` — the index becomes x
/// - an `[x y]` pair: `[[0 1.2] [1 3.4]]`
/// - an object: `[{x: 0 y: 1.2} {x: 1 y: 3.4}]`
pub fn to_xy_vec(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<(f64, f64)>> {
    let items = collect_elements(vm, value)?;
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        if let Some(y) = item.as_number() {
            out.push((i as f64, y));
            continue;
        }
        if let Some(pair) = collect_elements(vm, *item) {
            if pair.len() >= 2 {
                if let (Some(x), Some(y)) = (pair[0].as_number(), pair[1].as_number()) {
                    out.push((x, y));
                }
                continue;
            }
        }
        if let Some(obj) = item.as_object() {
            let trap = vm.bx.threads.cur().trap.pass();
            let x = vm.bx.heap.value(obj, id!(x).into(), trap);
            let trap = vm.bx.threads.cur().trap.pass();
            let y = vm.bx.heap.value(obj, id!(y).into(), trap);
            if let (Some(x), Some(y)) = (x.as_number(), y.as_number()) {
                out.push((x, y));
            }
        }
    }
    Some(out)
}

/// Read a script sequence as strings, casting non-string elements
/// (numbers, bools) through the VM's string conversion.
pub fn to_string_vec(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<String>> {
    let items = collect_elements(vm, value)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if item.is_nil() {
            out.push(String::new());
            continue;
        }
        let s = vm.bx.heap.temp_string_with(|heap, buf| {
            heap.cast_to_string(item, buf);
            buf.to_string()
        });
        out.push(s);
    }
    Some(out)
}

/// Public wrapper over element collection (arrays and object-vecs).
pub fn elements(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<ScriptValue>> {
    collect_elements(vm, value)
}

/// Read a named field off a script object value (`NIL` when absent).
pub fn field(vm: &mut ScriptVm, value: ScriptValue, key: LiveId) -> ScriptValue {
    if let Some(obj) = value.as_object() {
        let trap = vm.bx.threads.cur().trap.pass();
        let v = vm.bx.heap.value(obj, key.into(), trap);
        if !v.is_err() {
            return v;
        }
    }
    NIL
}

/// Cast any script value to a display string (numbers included).
pub fn to_string_cast(vm: &mut ScriptVm, value: ScriptValue) -> String {
    if value.is_nil() {
        return String::new();
    }
    vm.bx.heap.temp_string_with(|heap, buf| {
        heap.cast_to_string(value, buf);
        buf.to_string()
    })
}

/// Read a script sequence of sequences as rows of numbers:
/// `[[1 2 3], [4 5 6]]`. Non-sequence elements are skipped.
pub fn to_rows(vm: &mut ScriptVm, value: ScriptValue) -> Option<Vec<Vec<f64>>> {
    let items = collect_elements(vm, value)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if let Some(row) = to_f64_vec(vm, item) {
            out.push(row);
        }
    }
    Some(out)
}

/// Build a script array from a slice of numbers (for `data()` getters).
pub fn f64_slice_to_array(vm: &mut ScriptVm, values: &[f64]) -> ScriptValue {
    let arr = vm.bx.heap.new_array();
    for v in values {
        vm.bx.heap.array_push_unchecked(arr, (*v).into());
    }
    arr.into()
}

/// Fetch the nth positional argument of a `script_call` args object.
pub fn arg(vm: &mut ScriptVm, args: ScriptValue, index: usize) -> ScriptValue {
    if let Some(obj) = args.as_object() {
        let trap = vm.bx.threads.cur().trap.pass();
        let v = vm.bx.heap.vec_value(obj, index, trap);
        if !v.is_err() {
            return v;
        }
    }
    NIL
}
