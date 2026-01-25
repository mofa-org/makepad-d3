# Quick Comparison: makepad-d3 (Claude) vs makepad-d3-glm (GLM)

## At a Glance

| | makepad-d3 (Claude) | makepad-d3-glm (GLM) |
|--|---------------------|----------------------|
| **Lines of Code** | ~81,000 | ~31,500 |
| **Overall Score** | 8.5/10 (A-) | 6.6/10 (B-) |
| **Strength** | Architecture, Docs, Features | Pragmatism, Stats, Animation |
| **Weakness** | Over-engineering | Less extensible |

---

## Winner by Category

| Category | Winner | Why |
|----------|--------|-----|
| Architecture | Claude | Multi-trait hierarchy, clean modules |
| API Design | Claude | D3.js-faithful naming |
| Documentation | Claude | Comprehensive module/function docs |
| Feature Breadth | Claude | 3D, Geo, Streaming, Components |
| Code Simplicity | GLM | Less abstraction |
| Statistics | GLM | Dedicated comprehensive module |
| Animation | GLM | Full tween system with 18 easings |
| Hit Testing | GLM | Dedicated module with Bezier support |

---

## Key Architectural Differences

### Polymorphism Approach

```rust
// Claude: Trait objects (open, extensible)
pub struct ForceSimulation {
    forces: HashMap<String, Box<dyn Force>>,
}

// GLM: Enums (closed, simpler)
pub struct ForceSimulation {
    forces: Vec<Force>,  // Force is enum
}
```

### Trait Design

```rust
// Claude: 4 traits with hierarchy
trait Scale { ... }
trait ContinuousScale: Scale { ... }
trait DiscreteScale: Scale { ... }
trait ScaleExt: Scale { ... }

// GLM: 1 trait
trait Scale { ... }
```

### API Naming

| Concept | Claude | GLM |
|---------|--------|-----|
| Input space | `domain()` | `get_data_range()` |
| Map value | `scale(v)` | `get_pixel_for_value(v)` |

---

## Unique Features

### Only in Claude
- 3D GPU Rendering (Surface, Scatter, Bar)
- Geographic Projections (4 types)
- Perceptual Color Spaces (Lab, HCL, OKLab)
- UI Components (Legend, Tooltip, Crosshair)
- Streaming Data Sources
- Pack/Partition Layouts
- Voronoi Diagrams

### Only in GLM
- Statistics Module (mean, median, variance, correlation, regression)
- Animation System (Tween, Timer, Transition, 18 easing functions)
- Dedicated Hit Testing (polygon, bezier, nearest point)
- Sample Examples Module
- Link Generator for trees/networks

---

## Issues Found

### GLM Issues
1. **NaN panic risk** in stats.rs:69 - `partial_cmp().unwrap()`
2. **Unused dependencies** - serde declared but never used
3. **Force data cloning** - clones all forces every tick
4. **Magic numbers** - undocumented tick limit of 1000

### Claude Issues
1. **No statistics module** - missing basic stats
2. **No animation system** - relies on Makepad
3. **O(n²) force simulation** - same as GLM

---

## Recommendations

### For GLM to improve:
1. Train on D3.js source code for API naming
2. Learn Rust trait hierarchy patterns
3. Add thiserror for error handling
4. Split large files (shape/mod.rs is 1900 LOC)

### For Claude to improve:
1. Add statistics module (copy GLM's)
2. Add animation/tween system
3. Constrain scope to prevent over-building

---

## When to Use Each

| Use Case | Recommendation |
|----------|----------------|
| Production viz library | Claude |
| Need custom extensions | Claude |
| D3.js migration | Claude |
| Simple focused tool | GLM |
| Need built-in stats | GLM |
| Need animation | GLM |

---

## AI Training Insights

### Claude appears trained on:
- D3.js source code
- Large Rust projects (trait patterns)
- API design guidelines
- Documentation best practices

### GLM appears trained on:
- Rust fundamentals
- Functional programming (enums)
- Statistical computing
- Practical implementation

### Key difference:
- **Claude**: Optimized for architecture and documentation
- **GLM**: Optimized for practical, working code

---

*See [AI_MODEL_COMPARISON.md](./AI_MODEL_COMPARISON.md) for full analysis*
