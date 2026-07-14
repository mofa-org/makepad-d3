# d3.* Splash DSL Guide

How to use makepad-d3's chart widgets from Splash DSL — written in the same
style as makepad's `splash.md` so it can be handed to humans or AI agents
generating `runsplash`-style mini apps.

## Availability

- **In an app**: the app must call `makepad_d3::script_mod(vm)` in
  `AppMain::script_mod` after `makepad_widgets::script_mod(vm)`. After that,
  `d3.` is in scope everywhere the widgets prelude is imported
  (`use mod.prelude.widgets.*`) — no extra imports.
- **In a sandboxed body** (`runsplash`-style): the host must render the body
  with the `d3.Splash` widget (an isolated VM with `d3.*` registered). Stock
  `Splash`/`runsplash` sandboxes do NOT see `d3.*`.

## Key rules (inherit all of splash.md, plus)

- **Nested array literals need commas.** `[[0 5], [1 18]]` works;
  `[[0 5] [1 18]]` parses as indexing and corrupts the data.
- **Fixed-size charts go before `Fill` siblings** in a `flow: Right` row —
  a `Fill` child is deferred and a fixed child written after it will draw
  into the same space.
- Every chart renders a **built-in demo dataset when `data:` is omitted** —
  a bare `d3.BarChart{}` always shows something.
- Charts default to `width: Fill height: 300` — inside a `Fit`-height parent
  give the row or the chart an explicit height.
- **Axis/label text is currently invisible** (upstream `DrawText::draw_abs`
  gap on the makepad dev tip) — grids, marks, and `Label` widgets render
  fine. Don't rely on tick labels yet.
- Charts addressed via `ui.` need a `:=` id, like any widget.

## Widget catalog

### Basic charts

| Widget | `data:` shape | Notable props | Methods | Events |
|---|---|---|---|---|
| `d3.BarChart` | `[v v ...]` + `labels: ["a" ...]` | `bar_color` `hover_color` | `set_data` `set_labels` `set_domain` `data()` | `on_click(i)` `on_hover(i)` |
| `d3.LineChart` | numbers, `[[x y], ...]`, or `[{x:.. y:..}, ...]` | `line_color` `line_width` `dot_color` `dot_radius` | `set_data` `set_domain` `data()` | `on_click(i)` `on_hover(i)` |
| `d3.AreaChart` | same as LineChart | `fill_color` `line_color` `line_width` | `set_data` `set_domain` `data()` | `on_click(i)` |
| `d3.ScatterChart` | same as LineChart | `dot_color` `hover_color` `dot_radius` | `set_data` `set_domain` `data()` | `on_click(i)` `on_hover(i)` |
| `d3.PieChart` | `[v v ...]` + `labels:` | `inner_radius` (0–0.95 donut hole) | `set_data` `set_labels` `data()` | `on_click(i)` `on_hover(i)` |

### Statistical

| Widget | `data:` shape | Notable props | Methods | Events |
|---|---|---|---|---|
| `d3.Histogram` | raw sample values `[v v ...]` | `bins` `bar_color` `hover_color` | `set_data` `set_bins` `data()` | `on_click(bin)` `on_hover(bin)` |
| `d3.Heatmap` | rows: `[[..], [..]]` | `colormap` `cell_gap` | `set_data` | `on_click(row, col)` `on_hover(row, col)` |
| `d3.RadarChart` | one value per axis + `labels:` | `line_color` `fill_color` `rings` | `set_data` `set_labels` `data()` | `on_click(axis)` |
| `d3.BoxPlot` | array of distributions `[[..], [..]]` + `labels:` | `box_color` `median_color` `whisker_color` `outlier_color` | `set_data` `set_labels` | `on_click(series)` |

`colormap:` accepts `"viridis"` (default), `"plasma"`, `"inferno"`,
`"magma"`, `"coolwarm"`, `"turbo"`, `"gray"`.

### Hierarchies

All take the same nested `data:` object (or a flat number array as
single-level shorthand). Leaves are colored by top-level branch
(d3 Category10):

```splash
data: {name: "root" children: [
    {name: "a" value: 8},
    {name: "b" children: [{name: "b1" value: 3}, {name: "b2" value: 5}]}
]}
```

| Widget | Renders | Extra props | Events |
|---|---|---|---|
| `d3.Treemap` | squarified rectangles | — | `on_click(leaf)` |
| `d3.Sunburst` | radial partition rings | — | — |
| `d3.CirclePack` | nested circles | `branch_color` | — |
| `d3.TreeChart` | tidy node-link tree (horizontal) | `link_color` `node_color` `leaf_color` `node_radius` | — |

### Flows & networks

`d3.Sankey`, `d3.ArcDiagram`, and `d3.ForceGraph` take a node/link graph:

```splash
data: {
    nodes: [{name: "Coal"}, {name: "Gas"}, {name: "Power"}, {name: "Homes"}]
    links: [
        {source: 0 target: 2 value: 10},
        {source: 1 target: 2 value: 6},
        {source: 2 target: 3 value: 14}
    ]
}
```

| Widget | `data:` | Notable props | Events |
|---|---|---|---|
| `d3.Sankey` | graph (directed, acyclic) | `node_width` `node_padding` `link_alpha` | `on_click(node)` |
| `d3.ChordDiagram` | square flow matrix `[[..], ..]` | `ribbon_alpha` | — |
| `d3.ArcDiagram` | graph | `node_radius` `arc_alpha` | `on_click(node)` |
| `d3.ForceGraph` | graph (simulation runs on data set, drawn converged) | `link_color` `node_radius` | `on_click(node)` `on_hover(node)` |

### Densities

| Widget | `data:` | Notable props |
|---|---|---|
| `d3.Hexbin` | x/y points (pairs or `{x y}` objects) | `hex_radius` `colormap` |
| `d3.Ridgeline` | array of series `[[..], [..]]` | `overlap` |
| `d3.Horizon` | array of series (non-negative) | `bands` `band_color` |
| `d3.Contour` | value grid rows (same as Heatmap) | `thresholds` `colormap` `line_width` |

### Geographic

| Widget | `data:` | Notable props | Methods | Events |
|---|---|---|---|---|
| `d3.Globe` | `[{lon: 116.4 lat: 39.9}, ...]` markers | `rotation_lon` `rotation_lat` `sphere_color` `graticule_color` `point_color` | `set_rotation(lon, lat)` | `on_click(marker)` |

Drag rotates the globe.

### 3D (drag to orbit, scroll to zoom)

| Widget | `data:` | Notable props |
|---|---|---|
| `d3.Surface3D` | height grid rows `[[..], [..]]` | `colormap` |
| `d3.Scatter3D` | `[{x y z value?}, ...]` or `[[x y z], ...]` | `colormap` |
| `d3.Bar3D` | value grid rows (one bar per cell) | `colormap` |

### Sandbox host

| Widget | Purpose |
|---|---|
| `d3.Splash` | Evaluates a Splash body string in an **isolated VM with `d3.*` registered** — the runsplash path. Feed with `set_text` (Rust: `D3SplashRef::set_text` / script: `ui.host.set_text("...")`). No networking; body-level helper `fn`s have no `ui` global (inline handlers do). |

## Patterns

### Live-updating chart

```splash
View{ width: Fill height: Fit flow: Down spacing: 10
    chart := d3.BarChart{
        height: 300
        data: [30 86 168 281 303 365]
        labels: ["Jan" "Feb" "Mar" "Apr" "May" "Jun"]
        on_click: |i| ui.status.set_text("clicked " + i)
    }
    View{ width: Fill height: Fit flow: Right spacing: 10
        Button{text: "Update" on_click: || ui.chart.set_data([2 7 1 8 2 8])}
        Button{text: "Pin 0..400" on_click: || ui.chart.set_domain(0, 400)}
        status := Label{text: "-"}
    }
}
```

### Dashboard row (fixed before Fill)

```splash
View{ width: Fill height: 300 flow: Right spacing: 12
    d3.PieChart{ width: 300 height: Fill data: [35 25 18 12 10] }
    d3.LineChart{ width: Fill height: Fill data: [[0 5], [1 18], [2 12], [3 40]] }
}
```

### Hierarchy + flow

```splash
View{ width: Fill height: 280 flow: Right spacing: 12
    d3.Treemap{ width: Fill height: Fill data: {name: "fs" children: [
        {name: "src" children: [{name: "a.rs" value: 40}, {name: "b.rs" value: 25}]},
        {name: "docs" value: 30}
    ]}}
    d3.Sankey{ width: Fill height: Fill data: {
        nodes: [{name: "In"}, {name: "Mid"}, {name: "Out"}]
        links: [{source: 0 target: 1 value: 8}, {source: 1 target: 2 value: 8}]
    }}
}
```

### Heat + contour from the same grid

```splash
let grid = [[0 1 2 1], [1 3 4 2], [0 2 3 1]]
View{ width: Fill height: 240 flow: Right spacing: 12
    d3.Heatmap{ width: Fill height: Fill data: grid colormap: "plasma" }
    d3.Contour{ width: Fill height: Fill data: grid thresholds: 6 }
}
```
