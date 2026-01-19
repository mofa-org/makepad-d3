# Makepad Stack Overflow Issue: Large Widget Instances

## Summary

When creating multiple instances of widgets that contain several draw primitives (DrawTriangle, DrawChartLine, DrawPoint, etc.), the app crashes with a stack overflow during startup. This appears to be a limitation in how `live_design!` macro constructs widget hierarchies.

## Environment

- Platform: macOS (Darwin 24.6.0)
- Default stack size: 8MB
- Makepad: Latest

## Minimal Reproduction

### Widget Definition

A widget containing multiple draw primitives:

```rust
#[derive(Live, LiveHook, Widget)]
pub struct TreemapWidget {
    #[redraw]
    #[live]
    draw_rect: DrawTriangle,    // Large - inherits DrawQuad

    #[redraw]
    #[live]
    draw_line: DrawChartLine,   // Large - inherits DrawQuad

    #[redraw]
    #[live]
    draw_point: DrawPoint,      // Large - inherits DrawQuad

    #[walk]
    walk: Walk,

    #[rust]
    tree: Option<HierarchyNode<TreeNode>>,

    #[rust]
    positioned_tree: Option<HierarchyNode<TreeNode>>,

    #[rust]
    colors: Vec<Vec4>,

    #[rust]
    animator: ChartAnimator,

    // ... other fields
}
```

### Live Design with Multiple Instances

```rust
live_design! {
    import makepad_widgets::base::*;

    App = {{App}} {
        ui: <Root> {
            main_page = <View> {
                // Main page instance
                treemap_card = <ChartCard> {
                    treemap = <TreemapWidget> { width: Fill, height: Fill }
                }
            }

            // Detail page with 5 more instances
            treemap_detail_page = <View> {
                treemap_binary = <TreemapWidget> { width: Fill, height: Fill }
                treemap_squarify = <TreemapWidget> { width: Fill, height: Fill }
                treemap_slicedice = <TreemapWidget> { width: Fill, height: Fill }
                treemap_slice = <TreemapWidget> { width: Fill, height: Fill }
                treemap_dice = <TreemapWidget> { width: Fill, height: Fill }
            }
        }
    }
}
```

### Result

App crashes immediately on startup:

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

Exit code: 134 (SIGABRT)

## Analysis

### 1. Draw Primitives Are Large

Each `DrawQuad`-based primitive contains shader uniforms, geometry data, and other fields. Estimated size: ~500+ bytes each.

### 2. Widget Size Compounds

```
3 draw primitives × ~500 bytes = ~1.5KB per TreemapWidget
6 instances = ~9KB just for draw primitives
Plus recursive construction overhead in macro-generated code
```

### 3. Stack Exhaustion During Construction

The `live_design!` macro generates nested function calls to construct the entire widget hierarchy. With large widgets, this recursive construction exceeds the default 8MB stack limit.

### 4. Observed Threshold

| Instance Count | Result |
|----------------|--------|
| 1 instance | Works |
| 2 instances | Works |
| 3+ instances | Stack overflow |

### 5. Cumulative Effect

Interestingly, 5 TreemapWidgets on the detail page work fine **when the main page treemap is removed**. This suggests it's the cumulative stack usage across the entire `live_design!` hierarchy, not just the TreemapWidget count.

## Workaround Applied

### 1. Reduce Widget Size

Remove unused draw primitives:

```rust
// Before: 3 draw primitives
pub struct TreemapWidget {
    #[live] draw_rect: DrawTriangle,
    #[live] draw_line: DrawChartLine,  // Remove if not essential
    #[live] draw_point: DrawPoint,     // Remove if not essential
}

// After: 1 draw primitive
pub struct TreemapWidget {
    #[live] draw_rect: DrawTriangle,   // Keep only what's needed
}
```

### 2. Disable Auto-Initialization in draw_walk

Prevent initialization during the draw cycle to reduce stack depth:

```rust
fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
    let rect = cx.walk_turtle_with_area(&mut self.area, walk);

    if rect.size.x > 0.0 && rect.size.y > 0.0 {
        self.chart_rect = rect;

        // Only draw if already initialized externally
        // Avoids stack overflow when many treemap widgets are used
        if self.initialized && self.tree.is_some() {
            self.draw_treemap(cx, rect);
        }
    }

    DrawStep::done()
}
```

### 3. Lazy Initialization on First User Action

Defer initialization to separate it from the startup stack:

```rust
pub struct App {
    #[live]
    ui: WidgetRef,

    #[rust]
    current_page: CurrentPage,

    #[rust]
    front_treemap_initialized: bool,  // Track initialization state
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Initialize on first action, not during startup
        if !self.front_treemap_initialized {
            self.ui.treemap_widget(id!(treemap)).initialize_default(cx);
            self.front_treemap_initialized = true;
        }

        // ... rest of event handling
    }
}
```

### 4. Add External Initialization Method

```rust
impl TreemapWidgetRef {
    /// Initialize with default demo data
    pub fn initialize_default(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.initialize_data();
            inner.start_animation(cx);
            inner.initialized = true;
        }
    }
}
```

## Suggested Makepad Improvements

### 1. Heap-Allocate Draw Primitives

```rust
// Current: inline allocation (on stack during construction)
#[live]
draw_rect: DrawTriangle,

// Suggested: heap allocation
#[live]
draw_rect: Box<DrawTriangle>,
```

### 2. Lazy Widget Construction

Only instantiate widgets when first accessed or made visible, rather than constructing all widgets in `live_design!` at startup.

### 3. Iterative Macro Expansion

Modify the `live_design!` macro to use iterative construction instead of recursive function calls to reduce stack depth.

### 4. Documentation / Compile-Time Warning

- Document widget size limits in Makepad documentation
- Consider adding compile-time warnings when widget hierarchies exceed safe thresholds
- Provide guidance on widget design for large applications

### 5. Smaller DrawQuad Base

If shader uniforms can be shared or lazily loaded, reduce the base primitive size to allow more widget instances.

## Files Modified in Workaround

1. `examples/chart_zoo/src/charts/treemap_chart.rs`
   - Removed `DrawPoint` field
   - Removed `DrawChartLine` field
   - Modified `draw_walk()` to skip auto-initialization
   - Added `initialize_default()` method

2. `examples/chart_zoo/src/main.rs`
   - Added `front_treemap_initialized` flag
   - Added lazy initialization in `handle_actions()`

## Conclusion

This is a scalability limitation in Makepad's widget construction mechanism. The framework works correctly for typical use cases but hits system stack limits when applications have many large widgets. The workaround involves reducing widget size and deferring initialization, but a framework-level solution would benefit all Makepad users building complex applications.
