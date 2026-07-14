# Makepad-D3 Development Plan v2

> Comprehensive development plan based on critical code review findings.
> Addresses 67 identified issues across architecture, D3 API compliance, and abstractions.

**Created**: January 2026
**Target Completion**: Q2 2026

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Testing Strategy](#testing-strategy)
3. [Phase 1: Critical Bug Fixes](#phase-1-critical-bug-fixes-week-1-2)
4. [Phase 2: API Consistency](#phase-2-api-consistency-week-3-4)
5. [Phase 3: Algorithm Corrections](#phase-3-algorithm-corrections-week-5-7)
6. [Phase 4: D3 Feature Parity](#phase-4-d3-feature-parity-week-8-11)
7. [Phase 5: Architecture Improvements](#phase-5-architecture-improvements-week-12-14)
8. [Phase 6: Integration Testing & Documentation](#phase-6-integration-testing--documentation-week-15-16)
9. [Issue Tracking Matrix](#issue-tracking-matrix)
10. [Success Metrics](#success-metrics)

---

## Executive Summary

### Current State
- ✅ Feature coverage: ~95% of D3.js modules implemented
- ⚠️ Correctness issues: 8 critical/high bugs affecting output
- ⚠️ API consistency: Mixed patterns across modules
- ⚠️ D3 compliance: Key features missing or incomplete

### Goals
1. **Zero critical bugs** - All correctness issues resolved
2. **Consistent API** - Unified builder pattern, error handling
3. **D3 compliance** - Match D3.js behavior for all implemented features
4. **Production ready** - Comprehensive tests, documentation

### Effort Estimate
| Phase | Duration | Effort |
|-------|----------|--------|
| Phase 1: Critical Fixes | 2 weeks | ~1,500 LOC |
| Phase 2: API Consistency | 2 weeks | ~2,000 LOC |
| Phase 3: Algorithm Corrections | 3 weeks | ~3,000 LOC |
| Phase 4: D3 Feature Parity | 4 weeks | ~4,000 LOC |
| Phase 5: Architecture | 3 weeks | ~2,500 LOC |
| Phase 6: Testing & Docs | 2 weeks | ~2,000 LOC |
| **Total** | **16 weeks** | **~15,000 LOC** |

---

## Testing Strategy

> **Principle**: Test-Driven Development (TDD) for all changes. Every bug fix and new feature must have tests BEFORE implementation.

### Testing Pyramid

```
                    ┌─────────────────┐
                    │   Integration   │  ← Phase 6: Cross-module tests
                    │     Tests       │     Visual regression, E2E
                    ├─────────────────┤
                    │   D3 Compat     │  ← Continuous: Compare with D3.js
                    │     Tests       │     output for all modules
                    ├─────────────────┤
                    │                 │
                    │   Unit Tests    │  ← Every Phase: Each function
                    │                 │     has dedicated tests
                    └─────────────────┘
```

### Unit Test Requirements by Module

#### Scale Module Tests (`src/scale/`)
```rust
// tests/scale_tests.rs

mod linear_scale {
    use makepad_d3::scale::*;

    #[test]
    fn test_basic_mapping() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert_eq!(scale.scale(0.0), 0.0);
        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(100.0), 500.0);
    }

    #[test]
    fn test_inverted_range() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(500.0, 0.0);  // Y-axis style

        assert_eq!(scale.scale(0.0), 500.0);
        assert_eq!(scale.scale(100.0), 0.0);
    }

    #[test]
    fn test_clamping() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0)
            .with_clamp(true);

        assert_eq!(scale.scale(-50.0), 0.0);   // Clamped to min
        assert_eq!(scale.scale(150.0), 500.0); // Clamped to max
    }

    #[test]
    fn test_nice_bounds() {
        let mut scale = LinearScale::new()
            .with_domain(0.127, 0.942)
            .with_range(0.0, 100.0);

        scale.nice();
        let (min, max) = scale.domain();
        assert_eq!(min, 0.1);
        assert_eq!(max, 1.0);
    }

    #[test]
    fn test_invert() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert_eq!(scale.invert(250.0), 50.0);
    }

    #[test]
    fn test_zero_domain() {
        let scale = LinearScale::new()
            .with_domain(50.0, 50.0)  // Zero-width domain
            .with_range(0.0, 500.0);

        // Should return middle of range
        assert_eq!(scale.scale(50.0), 250.0);
    }

    #[test]
    fn test_nan_handling() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert!(scale.scale(f64::NAN).is_nan());
    }

    #[test]
    fn test_infinity_handling() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert!(scale.scale(f64::INFINITY).is_infinite());
    }

    #[test]
    fn test_ticks_generation() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let ticks = scale.ticks(5);
        assert_eq!(ticks, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);
    }
}

mod log_scale {
    #[test]
    fn test_log_base_10() { /* ... */ }

    #[test]
    fn test_log_base_2() { /* ... */ }

    #[test]
    fn test_negative_domain_error() { /* ... */ }

    #[test]
    fn test_zero_in_domain_error() { /* ... */ }
}

mod time_scale {
    #[test]
    fn test_time_mapping() { /* ... */ }

    #[test]
    fn test_time_ticks_years() { /* ... */ }

    #[test]
    fn test_time_ticks_months() { /* ... */ }

    #[test]
    fn test_time_ticks_days() { /* ... */ }

    #[test]
    fn test_timezone_handling() { /* ... */ }
}

mod band_scale {
    #[test]
    fn test_basic_bands() { /* ... */ }

    #[test]
    fn test_padding_inner() { /* ... */ }

    #[test]
    fn test_padding_outer() { /* ... */ }

    #[test]
    fn test_bandwidth() { /* ... */ }

    #[test]
    fn test_empty_domain() { /* ... */ }

    #[test]
    fn test_single_item_domain() { /* ... */ }
}
```

#### Shape Module Tests (`src/shape/`)
```rust
// tests/shape_tests.rs

mod line_generator {
    use makepad_d3::shape::*;
    use makepad_d3::data::DataPoint;

    #[test]
    fn test_empty_data() {
        let line = LineGenerator::new();
        assert!(line.generate(&[]).is_empty());
    }

    #[test]
    fn test_single_point() {
        let line = LineGenerator::new();
        let path = line.generate(&[DataPoint::from_xy(0.0, 0.0)]);
        assert_eq!(path.len(), 1);
        assert!(matches!(path[0], PathSegment::MoveTo(_)));
    }

    #[test]
    fn test_two_points() {
        let line = LineGenerator::new();
        let path = line.generate(&[
            DataPoint::from_xy(0.0, 0.0),
            DataPoint::from_xy(100.0, 100.0),
        ]);
        assert_eq!(path.len(), 2);
        assert!(matches!(path[0], PathSegment::MoveTo(_)));
        assert!(matches!(path[1], PathSegment::LineTo(_)));
    }

    #[test]
    fn test_defined_function() {
        let line = LineGenerator::new()
            .defined(|d, _| d.y >= 0.0);

        let path = line.generate(&[
            DataPoint::from_xy(0.0, 10.0),
            DataPoint::from_xy(1.0, -5.0),  // Undefined
            DataPoint::from_xy(2.0, 20.0),
        ]);

        // Should create two separate segments
        let move_count = path.iter()
            .filter(|s| matches!(s, PathSegment::MoveTo(_)))
            .count();
        assert_eq!(move_count, 2);
    }

    #[test]
    fn test_custom_accessors() {
        let line = LineGenerator::new()
            .x(|d, _| d.x * 2.0)
            .y(|d, _| d.y * 2.0);

        let path = line.generate(&[
            DataPoint::from_xy(10.0, 20.0),
            DataPoint::from_xy(30.0, 40.0),
        ]);

        if let PathSegment::MoveTo(p) = path[0] {
            assert_eq!(p.x, 20.0);
            assert_eq!(p.y, 40.0);
        }
    }
}

mod curve_tests {
    #[test]
    fn test_linear_curve() { /* ... */ }

    #[test]
    fn test_step_curve_before() { /* ... */ }

    #[test]
    fn test_step_curve_after() { /* ... */ }

    #[test]
    fn test_basis_curve() { /* ... */ }

    #[test]
    fn test_basis_curve_minimum_points() {
        // BasisCurve requires 3+ points
        let curve = BasisCurve::new();

        let two_points = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
        let path = curve.generate(&two_points);

        // Should fall back to linear for insufficient points
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_cardinal_curve_tension() { /* ... */ }

    #[test]
    fn test_catmull_rom_curve() { /* ... */ }

    #[test]
    fn test_monotone_curve_preserves_monotonicity() {
        let curve = MonotoneCurve::new();
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 10.0),
            Point::new(2.0, 5.0),
            Point::new(3.0, 15.0),
        ];

        let path = curve.generate(&points);

        // Verify curve doesn't overshoot between points
        // (monotone X should preserve Y ordering locally)
    }
}

mod pie_layout {
    #[test]
    fn test_basic_pie() { /* ... */ }

    #[test]
    fn test_pie_sorting() { /* ... */ }

    #[test]
    fn test_pie_padding() { /* ... */ }

    #[test]
    fn test_zero_values() { /* ... */ }

    #[test]
    fn test_all_zero_values() { /* ... */ }

    #[test]
    fn test_negative_values() { /* ... */ }
}

mod stack_generator {
    #[test]
    fn test_stack_none_offset() { /* ... */ }

    #[test]
    fn test_stack_expand_offset() { /* ... */ }

    #[test]
    fn test_stack_diverging_offset() { /* ... */ }

    #[test]
    fn test_stack_order_ascending() { /* ... */ }

    #[test]
    fn test_stack_order_descending() { /* ... */ }
}
```

#### Layout Module Tests (`src/layout/`)
```rust
// tests/layout_tests.rs

mod force_simulation {
    use makepad_d3::layout::*;

    #[test]
    fn test_basic_simulation() {
        let nodes: Vec<SimulationNode> = (0..10)
            .map(|i| SimulationNode::new(i))
            .collect();

        let mut sim = ForceSimulation::new(nodes)
            .add_force("center", CenterForce::new(400.0, 300.0));

        sim.tick_n(100);

        // All nodes should be near center
        for node in sim.nodes() {
            assert!((node.x - 400.0).abs() < 50.0);
            assert!((node.y - 300.0).abs() < 50.0);
        }
    }

    #[test]
    fn test_many_body_repulsion() {
        let nodes: Vec<SimulationNode> = (0..5)
            .map(|i| SimulationNode::new(i))
            .collect();

        let mut sim = ForceSimulation::new(nodes)
            .add_force("charge", ManyBodyForce::new().strength(-100.0));

        sim.tick_n(100);

        // Nodes should spread apart
        let min_distance = 50.0;
        for i in 0..sim.nodes().len() {
            for j in (i + 1)..sim.nodes().len() {
                let dx = sim.nodes()[i].x - sim.nodes()[j].x;
                let dy = sim.nodes()[i].y - sim.nodes()[j].y;
                let dist = (dx * dx + dy * dy).sqrt();
                assert!(dist > min_distance);
            }
        }
    }

    #[test]
    fn test_link_force() { /* ... */ }

    #[test]
    fn test_collide_force() { /* ... */ }

    #[test]
    fn test_alpha_decay() { /* ... */ }

    #[test]
    fn test_reheat() { /* ... */ }
}

mod tree_layout {
    #[test]
    fn test_basic_tree() { /* ... */ }

    #[test]
    fn test_tree_node_size() { /* ... */ }

    #[test]
    fn test_tree_separation() { /* ... */ }

    #[test]
    fn test_single_node_tree() { /* ... */ }

    #[test]
    fn test_linear_tree() { /* ... */ }  // All nodes in single branch

    #[test]
    fn test_wide_tree() { /* ... */ }    // Many children per node
}

mod treemap_layout {
    #[test]
    fn test_basic_treemap() { /* ... */ }

    #[test]
    fn test_squarify_tiling() { /* ... */ }

    #[test]
    fn test_slice_tiling() { /* ... */ }

    #[test]
    fn test_dice_tiling() { /* ... */ }

    #[test]
    fn test_zero_value_nodes() {
        let mut root = HierarchyNode::new("root", 0.0);
        root.add_child(HierarchyNode::new("a", 0.0));
        root.add_child(HierarchyNode::new("b", 10.0));

        let layout = TreemapLayout::new().size(100.0, 100.0);
        let result = layout.layout(root);

        // Zero-value node should still have valid coordinates
        assert!(!result.children[0].x.is_nan());
    }

    #[test]
    fn test_aspect_ratio_quality() {
        // Squarify should produce near-square rectangles
    }
}

mod pack_layout {
    #[test]
    fn test_basic_pack() { /* ... */ }

    #[test]
    fn test_no_overlap() {
        let mut root = HierarchyNode::new("root", 0.0);
        for i in 0..20 {
            root.add_child(HierarchyNode::new(format!("c{}", i), (i + 1) as f64));
        }

        let layout = PackLayout::new().size(500.0, 500.0);
        let result = layout.layout(root);

        // Check all pairs for overlap
        for i in 0..result.children.len() {
            for j in (i + 1)..result.children.len() {
                let ci = &result.children[i];
                let cj = &result.children[j];
                let dx = ci.x - cj.x;
                let dy = ci.y - cj.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let min_dist = ci.radius + cj.radius;

                assert!(
                    dist >= min_dist - 0.1,  // Small tolerance
                    "Circles {} and {} overlap", i, j
                );
            }
        }
    }

    #[test]
    fn test_enclosing_circle() { /* ... */ }
}
```

#### Geo Module Tests (`src/geo/`)
```rust
// tests/geo_tests.rs

mod projections {
    use makepad_d3::geo::*;

    #[test]
    fn test_mercator_basic() {
        let proj = MercatorProjection::new()
            .scale(100.0)
            .translate(0.0, 0.0);

        // Equator at prime meridian
        let (x, y) = proj.project(0.0, 0.0);
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_mercator_invert() {
        let proj = MercatorProjection::new()
            .scale(100.0)
            .translate(400.0, 300.0);

        let (x, y) = proj.project(-122.4, 37.8);
        let (lon, lat) = proj.invert(x, y);

        assert!((lon - (-122.4)).abs() < 0.001);
        assert!((lat - 37.8).abs() < 0.001);
    }

    #[test]
    fn test_orthographic_visibility() {
        let proj = OrthographicProjection::new()
            .rotate(0.0, 0.0, 0.0);

        // Point on front of globe
        assert!(proj.is_visible(0.0, 0.0));

        // Point on back of globe
        assert!(!proj.is_visible(180.0, 0.0));
    }

    #[test]
    fn test_orthographic_rotation() { /* ... */ }

    #[test]
    fn test_albers_usa() { /* ... */ }

    #[test]
    fn test_fit_extent() { /* ... */ }
}

mod geojson {
    #[test]
    fn test_parse_point() { /* ... */ }

    #[test]
    fn test_parse_linestring() { /* ... */ }

    #[test]
    fn test_parse_polygon() { /* ... */ }

    #[test]
    fn test_parse_feature_collection() { /* ... */ }

    #[test]
    fn test_parse_invalid_coordinates() {
        let json = r#"{"type":"Point","coordinates":[200, 100]}"#;
        // Should either reject or normalize
    }

    #[test]
    fn test_bbox_calculation() { /* ... */ }
}

mod path_generation {
    #[test]
    fn test_point_to_segments() { /* ... */ }

    #[test]
    fn test_line_to_segments() { /* ... */ }

    #[test]
    fn test_polygon_to_segments() { /* ... */ }

    #[test]
    fn test_spherical_area() { /* ... */ }

    #[test]
    fn test_antimeridian_crossing() { /* ... */ }
}
```

#### Interaction Module Tests (`src/interaction/`)
```rust
// tests/interaction_tests.rs

mod zoom {
    use makepad_d3::interaction::*;

    #[test]
    fn test_zoom_identity() {
        let t = ZoomTransform::identity();
        assert_eq!(t.k, 1.0);
        assert_eq!(t.x, 0.0);
        assert_eq!(t.y, 0.0);
    }

    #[test]
    fn test_zoom_apply() {
        let t = ZoomTransform::new(2.0, 100.0, 50.0);
        let (x, y) = t.apply(10.0, 20.0);
        assert_eq!(x, 2.0 * 10.0 + 100.0);
        assert_eq!(y, 2.0 * 20.0 + 50.0);
    }

    #[test]
    fn test_zoom_invert() {
        let t = ZoomTransform::new(2.0, 100.0, 50.0);
        let (px, py) = t.apply(10.0, 20.0);
        let (x, y) = t.invert(px, py);
        assert!((x - 10.0).abs() < 0.001);
        assert!((y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_zoom_preserves_point_under_cursor() {
        let zoom = ZoomBehavior::new()
            .scale_extent(0.1, 10.0);

        let mut t = ZoomTransform::new(1.0, 100.0, 50.0);
        let cursor = (250.0, 200.0);

        // Get data point under cursor before zoom
        let (px, py) = t.invert(cursor.0, cursor.1);

        // Zoom in
        zoom.handle_wheel(&mut t, 100.0, cursor.0, cursor.1);

        // Data point under cursor should be the same
        let (qx, qy) = t.invert(cursor.0, cursor.1);
        assert!((px - qx).abs() < 0.001);
        assert!((py - qy).abs() < 0.001);
    }

    #[test]
    fn test_scale_extent() { /* ... */ }

    #[test]
    fn test_translate_extent() { /* ... */ }
}

mod brush {
    #[test]
    fn test_brush_selection_normalized() {
        let sel = BrushSelection::new(100.0, 100.0, 10.0, 10.0);
        let norm = sel.normalized();

        assert!(norm.x0 <= norm.x1);
        assert!(norm.y0 <= norm.y1);
        assert_eq!(norm.x0, 10.0);
        assert_eq!(norm.x1, 100.0);
    }

    #[test]
    fn test_brush_contains() { /* ... */ }

    #[test]
    fn test_brush_resize_handles() { /* ... */ }

    #[test]
    fn test_brush_x_constrains_y() { /* ... */ }

    #[test]
    fn test_brush_requires_extent() {
        let brush = BrushBehavior::x();
        // Should panic or error without extent
    }
}
```

#### Color Module Tests (`src/color/`)
```rust
// tests/color_tests.rs

mod rgba {
    use makepad_d3::color::*;

    #[test]
    fn test_from_hex() {
        let c = Rgba::from_hex(0xFF0000);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.0).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_to_hex() {
        let c = Rgba::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(c.to_hex(), 0xFF0000);
    }

    #[test]
    fn test_lerp() {
        let a = Rgba::BLACK;
        let b = Rgba::WHITE;
        let mid = a.lerp(&b, 0.5);

        assert!((mid.r - 0.5).abs() < 0.01);
        assert!((mid.g - 0.5).abs() < 0.01);
        assert!((mid.b - 0.5).abs() < 0.01);
    }
}

mod lab {
    #[test]
    fn test_rgb_to_lab_roundtrip() {
        let original = Rgba::new(0.5, 0.3, 0.8, 1.0);
        let lab = Lab::from_rgba(&original);
        let back = lab.to_rgba();

        assert!((original.r - back.r).abs() < 0.01);
        assert!((original.g - back.g).abs() < 0.01);
        assert!((original.b - back.b).abs() < 0.01);
    }

    #[test]
    fn test_lab_interpolation() { /* ... */ }

    #[test]
    fn test_lab_distance() { /* ... */ }
}

mod hcl {
    #[test]
    fn test_hue_normalization() { /* ... */ }

    #[test]
    fn test_complement() { /* ... */ }

    #[test]
    fn test_hcl_interpolation_shorter() { /* ... */ }

    #[test]
    fn test_hcl_interpolation_longer() { /* ... */ }
}

mod interpolation {
    #[test]
    fn test_interpolate_rgb_preserves_alpha() {
        let a = Rgba::new(1.0, 0.0, 0.0, 0.2);
        let b = Rgba::new(0.0, 0.0, 1.0, 0.8);
        let mid = interpolate_rgb(&a, &b, 0.5);

        assert!((mid.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_hsl_preserves_alpha() {
        let a = Rgba::new(1.0, 0.0, 0.0, 0.2);
        let b = Rgba::new(0.0, 0.0, 1.0, 0.8);
        let mid = interpolate_hsl(&a, &b, 0.5);

        assert!((mid.a - 0.5).abs() < 0.01);  // Currently FAILS
    }

    #[test]
    fn test_interpolate_lab_preserves_alpha() { /* ... */ }
}

mod scales {
    #[test]
    fn test_sequential_scale() { /* ... */ }

    #[test]
    fn test_diverging_scale_midpoint() { /* ... */ }

    #[test]
    fn test_categorical_scale() { /* ... */ }

    #[test]
    fn test_sequential_with_lab_space() { /* ... */ }
}
```

### Test Infrastructure

#### Test Utilities (`tests/common/mod.rs`)
```rust
pub mod common {
    use makepad_d3::shape::{PathSegment, Point};
    use makepad_d3::data::DataPoint;

    /// Assert two floats are approximately equal
    pub fn assert_approx_eq(a: f64, b: f64, tolerance: f64) {
        assert!(
            (a - b).abs() < tolerance,
            "Expected {} ≈ {} (tolerance: {})",
            a, b, tolerance
        );
    }

    /// Assert two points are approximately equal
    pub fn assert_point_eq(a: Point, b: Point, tolerance: f64) {
        assert_approx_eq(a.x, b.x, tolerance);
        assert_approx_eq(a.y, b.y, tolerance);
    }

    /// Generate test data points
    pub fn linear_data(n: usize) -> Vec<DataPoint> {
        (0..n)
            .map(|i| DataPoint::from_xy(i as f64, i as f64 * 2.0))
            .collect()
    }

    /// Generate random data points
    pub fn random_data(n: usize, seed: u64) -> Vec<DataPoint> {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        (0..n)
            .map(|i| DataPoint::from_xy(
                i as f64,
                rng.gen_range(0.0..100.0)
            ))
            .collect()
    }

    /// Count specific path segment types
    pub fn count_segments(path: &[PathSegment]) -> (usize, usize, usize, usize) {
        let mut move_to = 0;
        let mut line_to = 0;
        let mut curve_to = 0;
        let mut close = 0;

        for seg in path {
            match seg {
                PathSegment::MoveTo(_) => move_to += 1,
                PathSegment::LineTo(_) => line_to += 1,
                PathSegment::CurveTo { .. } => curve_to += 1,
                PathSegment::ClosePath => close += 1,
                _ => {}
            }
        }

        (move_to, line_to, curve_to, close)
    }
}
```

#### D3 Reference Test Generator
```rust
// tests/d3_reference/mod.rs

/// Tests that compare our output to D3.js reference output
/// Reference values generated by running D3.js in Node.js

mod d3_scale_reference {
    #[test]
    fn test_linear_scale_d3_reference() {
        // D3: d3.scaleLinear().domain([0, 100]).range([0, 500])
        // Values: [0, 25, 50, 75, 100] => [0, 125, 250, 375, 500]
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let d3_reference = vec![
            (0.0, 0.0),
            (25.0, 125.0),
            (50.0, 250.0),
            (75.0, 375.0),
            (100.0, 500.0),
        ];

        for (input, expected) in d3_reference {
            assert_approx_eq(scale.scale(input), expected, 0.001);
        }
    }

    #[test]
    fn test_log_scale_d3_reference() {
        // Generated from D3.js
        // ...
    }
}
```

### Test Coverage Requirements

| Module | Minimum Coverage | Target Coverage |
|--------|------------------|-----------------|
| Scale | 80% | 95% |
| Shape | 80% | 90% |
| Layout | 75% | 85% |
| Geo | 70% | 85% |
| Interaction | 75% | 90% |
| Color | 85% | 95% |
| Data | 70% | 85% |
| Component | 60% | 80% |

### CI Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: llvm-tools-preview

      - name: Run tests
        run: cargo test --all-features

      - name: Run tests with coverage
        run: |
          cargo install cargo-llvm-cov
          cargo llvm-cov --all-features --lcov --output-path lcov.info

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo llvm-cov --all-features --json | jq '.data[0].totals.lines.percent')
          if (( $(echo "$COVERAGE < 75" | bc -l) )); then
            echo "Coverage $COVERAGE% is below threshold 75%"
            exit 1
          fi
```

### Per-Phase Test Requirements

Each phase must complete these testing milestones:

| Phase | Unit Tests | Integration Tests | Coverage Delta |
|-------|------------|-------------------|----------------|
| Phase 1 | Tests for each bug fix | N/A | +5% |
| Phase 2 | Tests for new trait hierarchy | API compatibility tests | +10% |
| Phase 3 | Algorithm correctness tests | D3 reference comparison | +10% |
| Phase 4 | Feature tests | Cross-module tests | +10% |
| Phase 5 | Refactoring tests | Performance benchmarks | +5% |
| Phase 6 | Edge case tests | Visual regression | +10% |

---

## Phase 1: Critical Bug Fixes (Week 1-2)

> **Priority**: IMMEDIATE
> **Goal**: Fix all correctness bugs that produce wrong output

### 1.1 BasisCurve Start Point Bug
**File**: `src/shape/curve/basis.rs`
**Issue**: Formula uses `p0.x` twice instead of proper basis spline coefficients

```rust
// CURRENT (WRONG)
let start = Point::new(
    (p0.x + 4.0 * p0.x + p1.x) / 6.0,
    (p0.y + 4.0 * p0.y + p1.y) / 6.0,
);

// FIXED
let start = Point::new(
    (p0.x + 4.0 * p1.x + p2.x) / 6.0,
    (p0.y + 4.0 * p1.y + p2.y) / 6.0,
);
```

**Tasks**:
- [ ] Write failing test first (TDD)
- [ ] Fix formula in `basis.rs` line 59-62
- [ ] Add unit test comparing output to D3.js basis curve
- [ ] Add test for minimum points requirement
- [ ] Visual regression test with known data

**Unit Tests Required**:
```rust
#[test]
fn test_basis_curve_formula_correctness() {
    let curve = BasisCurve::new();
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 2.0),
        Point::new(2.0, 1.0),
        Point::new(3.0, 3.0),
    ];

    let path = curve.generate(&points);

    // First control point should use p0, p1, p2 (not p0, p0, p1)
    if let PathSegment::CurveTo { cp1, .. } = &path[1] {
        // D3 reference: first control point for basis
        assert_approx_eq(cp1.x, 0.833, 0.01);  // (0 + 4*1 + 2) / 6
        assert_approx_eq(cp1.y, 1.5, 0.01);    // (0 + 4*2 + 1) / 6
    }
}

#[test]
fn test_basis_curve_minimum_points() {
    let curve = BasisCurve::new();

    // 2 points should fall back to linear
    let two = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];
    let path = curve.generate(&two);
    assert_eq!(path.len(), 2);
    assert!(matches!(path[1], PathSegment::LineTo(_)));
}
```

**Effort**: 3 hours (including tests)

---

### 1.2 Zoom Center Point Calculation
**File**: `src/interaction/zoom.rs`
**Issue**: Current formula assumes symmetric bounds, causing cursor drift

```rust
// CURRENT (WRONG for asymmetric bounds)
transform.x = center_x - (center_x - transform.x) * k1 / k0;

// FIXED (invert → zoom → project)
let (px, py) = transform.invert(center_x, center_y);
transform.k = k1;
let (sx, sy) = transform.apply(px, py);
transform.x += center_x - sx;
transform.y += center_y - sy;
```

**Tasks**:
- [ ] Write failing test first (TDD)
- [ ] Implement `invert()` method on `ZoomTransform`
- [ ] Implement `apply()` method on `ZoomTransform`
- [ ] Refactor `handle_wheel()` to use invert→zoom→project pattern
- [ ] Add test for asymmetric bounds zoom

**Unit Tests Required**:
```rust
#[test]
fn test_zoom_transform_apply_invert_roundtrip() {
    let t = ZoomTransform::new(2.0, 100.0, 50.0);
    let original = (42.0, 73.0);

    let projected = t.apply(original.0, original.1);
    let back = t.invert(projected.0, projected.1);

    assert_approx_eq(back.0, original.0, 0.001);
    assert_approx_eq(back.1, original.1, 0.001);
}

#[test]
fn test_zoom_preserves_cursor_point_asymmetric_bounds() {
    let zoom = ZoomBehavior::new()
        .scale_extent(0.5, 4.0)
        .translate_extent(Extent::new(0.0, 0.0, 1000.0, 500.0));

    let mut t = ZoomTransform::new(1.0, 100.0, 50.0);
    let cursor = (250.0, 200.0);

    let (px, py) = t.invert(cursor.0, cursor.1);
    zoom.handle_wheel(&mut t, 100.0, cursor.0, cursor.1);
    let (qx, qy) = t.invert(cursor.0, cursor.1);

    assert_approx_eq(px, qx, 0.001);
    assert_approx_eq(py, qy, 0.001);
}

#[test]
fn test_zoom_out_preserves_cursor_point() {
    // Same test but zooming out (negative delta)
}
```

**Effort**: 5 hours (including tests)

---

### 1.3 HSL Interpolation Alpha Loss
**File**: `src/color/interpolate.rs`
**Issue**: Alpha channel is lost during HSL interpolation

```rust
// CURRENT (loses alpha)
Hsl::new(h, s, l).to_rgba()

// FIXED
Hsl::new(h, s, l).to_rgba().with_alpha(
    a.a + (b.a - a.a) * t as f32
)
```

**Tasks**:
- [ ] Write failing test first (TDD)
- [ ] Add alpha interpolation to `interpolate_hsl()`
- [ ] Consider adding alpha to `Hsl` struct
- [ ] Add round-trip test for HSL with alpha

**Unit Tests Required**:
```rust
#[test]
fn test_interpolate_hsl_preserves_alpha() {
    let a = Rgba::new(1.0, 0.0, 0.0, 0.2);
    let b = Rgba::new(0.0, 0.0, 1.0, 0.8);

    let mid = interpolate_hsl(&a, &b, 0.5);

    // Alpha should interpolate linearly
    assert_approx_eq(mid.a, 0.5, 0.01);
}

#[test]
fn test_interpolate_hsl_alpha_endpoints() {
    let a = Rgba::new(1.0, 0.0, 0.0, 0.0);
    let b = Rgba::new(0.0, 0.0, 1.0, 1.0);

    let at_a = interpolate_hsl(&a, &b, 0.0);
    let at_b = interpolate_hsl(&a, &b, 1.0);

    assert_approx_eq(at_a.a, 0.0, 0.01);
    assert_approx_eq(at_b.a, 1.0, 0.01);
}

#[test]
fn test_hsl_struct_stores_alpha() {
    let hsl = Hsl::new_with_alpha(0.5, 0.5, 0.5, 0.3);
    assert_approx_eq(hsl.alpha, 0.3, 0.01);

    let rgba = hsl.to_rgba();
    assert_approx_eq(rgba.a, 0.3, 0.01);
}
```

**Effort**: 3 hours (including tests)

---

### 1.4 Brush Extent Validation
**File**: `src/interaction/brush.rs`
**Issue**: X/Y brush without extent creates invalid selections

```rust
// ADD validation
pub fn handle_start(&mut self, x: f64, y: f64) -> bool {
    match self.brush_type {
        BrushType::X | BrushType::Y => {
            if self.extent.is_none() {
                panic!("X and Y brush types require extent to be set");
            }
        }
        _ => {}
    }
    // ... rest of method
}
```

**Tasks**:
- [ ] Write failing test first (TDD)
- [ ] Add validation in `handle_start()`
- [ ] Add validation in `set_selection()`
- [ ] Update documentation with requirement
- [ ] Add test for panic on missing extent

**Unit Tests Required**:
```rust
#[test]
#[should_panic(expected = "X brush requires extent")]
fn test_brush_x_requires_extent() {
    let mut brush = BrushBehavior::x();
    // No extent set
    brush.handle_start(50.0, 50.0);  // Should panic
}

#[test]
#[should_panic(expected = "Y brush requires extent")]
fn test_brush_y_requires_extent() {
    let mut brush = BrushBehavior::y();
    brush.handle_start(50.0, 50.0);  // Should panic
}

#[test]
fn test_brush_xy_works_without_extent() {
    let mut brush = BrushBehavior::xy();
    // Should not panic - XY brush doesn't require extent
    brush.handle_start(50.0, 50.0);
}

#[test]
fn test_brush_x_with_extent_constrains_y() {
    let mut brush = BrushBehavior::x()
        .with_extent(0.0, 0.0, 100.0, 200.0);

    brush.handle_start(50.0, 75.0);
    brush.handle_move(80.0, 150.0);

    let sel = brush.selection().unwrap();
    assert_eq!(sel.y0, 0.0);    // Constrained to extent
    assert_eq!(sel.y1, 200.0);  // Constrained to extent
}
```

**Effort**: 3 hours (including tests)

---

### 1.5 Pack Layout Uninitialized Positions
**File**: `src/layout/hierarchy/pack.rs`
**Issue**: If no valid position found, circles placed at origin

```rust
// ADD fallback handling
if best_dist == f64::INFINITY {
    // No valid position found - use spiral fallback
    let angle = i as f64 * 0.5;
    let radius = 10.0 + i as f64 * 5.0;
    best_x = radius * angle.cos();
    best_y = radius * angle.sin();
    log::warn!("Pack layout: no valid position for circle {}, using fallback", i);
}
```

**Tasks**:
- [ ] Write failing test first (TDD)
- [ ] Add fallback position calculation
- [ ] Add warning log for fallback usage
- [ ] Add test with overlapping circles

**Unit Tests Required**:
```rust
#[test]
fn test_pack_fallback_positions_are_valid() {
    // Create a scenario where normal placement fails
    let mut root = HierarchyNode::new("root", 0.0);

    // Add circles that might cause placement issues
    for i in 0..50 {
        root.add_child(HierarchyNode::new(
            format!("c{}", i),
            (50 - i) as f64  // Decreasing sizes
        ));
    }

    let layout = PackLayout::new().size(100.0, 100.0);  // Small space
    let result = layout.layout(root);

    // All circles should have valid (non-NaN, non-zero) positions
    for child in &result.children {
        assert!(!child.x.is_nan(), "Circle x is NaN");
        assert!(!child.y.is_nan(), "Circle y is NaN");
        assert!(child.radius > 0.0, "Circle radius should be positive");
    }
}

#[test]
fn test_pack_no_circles_at_origin() {
    let mut root = HierarchyNode::new("root", 0.0);
    for i in 0..20 {
        root.add_child(HierarchyNode::new(format!("c{}", i), (i + 1) as f64));
    }

    let layout = PackLayout::new().size(500.0, 500.0);
    let result = layout.layout(root);

    // Count circles at exact origin (indicates fallback wasn't implemented)
    let at_origin = result.children.iter()
        .filter(|c| c.x.abs() < 0.001 && c.y.abs() < 0.001)
        .count();

    // At most one circle should be at origin (the first one)
    assert!(at_origin <= 1, "Multiple circles at origin: {}", at_origin);
}
```

**Effort**: 4 hours (including tests)

---

### 1.6 Orthographic Rotation Matrix
**File**: `src/geo/projection.rs`
**Issue**: Gamma (roll) rotation is incomplete

```rust
// CURRENT (incomplete)
let y2 = y;  // Gamma ignored for Y!

// FIXED (full Euler rotation)
let cos_g = self.rotate_gamma.to_radians().cos();
let sin_g = self.rotate_gamma.to_radians().sin();

// Apply all three rotations
let x2 = x * cos_g - y * sin_g;
let y2 = x * sin_g + y * cos_g;
```

**Tasks**:
- [ ] Write failing test first (TDD)
- [ ] Implement full 3D rotation matrix
- [ ] Extract rotation to reusable function
- [ ] Add test comparing to D3-geo rotation output

**Unit Tests Required**:
```rust
#[test]
fn test_orthographic_rotation_lambda() {
    // Rotate around Z-axis (lambda)
    let proj = OrthographicProjection::new()
        .rotate(90.0, 0.0, 0.0)
        .scale(100.0)
        .translate(0.0, 0.0);

    // Point at (0, 0) should now appear at (-90, 0) after rotation
    let (x, y) = proj.project(0.0, 0.0);
    assert_approx_eq(x, 0.0, 0.1);
    assert_approx_eq(y, 0.0, 0.1);
}

#[test]
fn test_orthographic_rotation_phi() {
    // Rotate around Y-axis (phi)
    let proj = OrthographicProjection::new()
        .rotate(0.0, 90.0, 0.0)
        .scale(100.0)
        .translate(0.0, 0.0);

    // North pole (0, 90) should now be at center
    let (x, y) = proj.project(0.0, 90.0);
    assert_approx_eq(x, 0.0, 0.1);
    assert_approx_eq(y, 0.0, 0.1);
}

#[test]
fn test_orthographic_rotation_gamma() {
    // Rotate around view axis (gamma/roll)
    let proj = OrthographicProjection::new()
        .rotate(0.0, 0.0, 90.0)
        .scale(100.0)
        .translate(0.0, 0.0);

    // Point at (0, 45) should rotate 90 degrees around center
    let (x, y) = proj.project(0.0, 45.0);

    // D3-geo reference values
    assert_approx_eq(x, -70.7, 1.0);
    assert_approx_eq(y, 0.0, 1.0);
}

#[test]
fn test_orthographic_combined_rotation_matches_d3() {
    let proj = OrthographicProjection::new()
        .rotate(30.0, 45.0, 15.0)
        .scale(100.0)
        .translate(0.0, 0.0);

    // D3-geo reference: d3.geoOrthographic().rotate([30, 45, 15])([10, 20])
    let (x, y) = proj.project(10.0, 20.0);

    // These values should match D3-geo output
    assert_approx_eq(x, -32.14, 0.5);  // Update with actual D3 output
    assert_approx_eq(y, 29.87, 0.5);
}
```

**Effort**: 5 hours (including tests)

---

### Phase 1 Deliverables
- [ ] All 6 critical bugs fixed
- [ ] 20+ new unit tests for bug fixes
- [ ] All tests passing (existing + new)
- [ ] Coverage increase: +5%
- [ ] No regressions in existing tests
- [ ] Release patch version (0.x.1)

### Phase 1 Test Summary

| Bug Fix | Tests Added | Test Types |
|---------|-------------|------------|
| BasisCurve formula | 2 | Unit, D3 reference |
| Zoom center point | 3 | Unit, Integration |
| HSL alpha | 3 | Unit, Roundtrip |
| Brush validation | 4 | Unit, Panic |
| Pack fallback | 2 | Unit, Edge case |
| Rotation matrix | 4 | Unit, D3 reference |
| **Total** | **18+** | |

---

## Phase 2: API Consistency (Week 3-4)

> **Priority**: HIGH
> **Goal**: Unify builder patterns, error handling, and trait design

### 2.1 Standardize Builder Pattern

**Current State**: Mixed patterns across modules
- `with_*` prefix (LinearScale, LogScale)
- No prefix (BandScale, SequentialScale)
- Both exist (SequentialScale has `domain()` AND `set_domain()`)

**Target State**: Consistent `with_*` prefix everywhere

**Files to modify**:
- `src/scale/band.rs`
- `src/scale/point.rs`
- `src/scale/sequential.rs`
- `src/scale/time.rs`
- `src/shape/line.rs`
- `src/shape/area.rs`
- `src/shape/arc.rs`

**Tasks**:
- [ ] Audit all builder methods across codebase
- [ ] Rename non-conforming methods to `with_*` pattern
- [ ] Add `#[deprecated]` to old method names
- [ ] Update all examples and tests
- [ ] Document builder pattern in CONTRIBUTING.md

**Effort**: 8 hours

---

### 2.2 Separate Scale Trait Hierarchies

**Current Issue**: `Scale` trait has `set_domain(f64, f64)` that discrete scales ignore

**Solution**: Create proper trait hierarchy

```rust
// Base trait for all scales
pub trait Scale: Send + Sync {
    fn scale(&self, value: f64) -> f64;
    fn scale_type(&self) -> &'static str;
    fn clone_box(&self) -> Box<dyn Scale>;
}

// For continuous scales (Linear, Log, Time, etc.)
pub trait ContinuousScale: Scale {
    fn set_domain(&mut self, min: f64, max: f64);
    fn set_range(&mut self, start: f64, end: f64);
    fn domain(&self) -> (f64, f64);
    fn range(&self) -> (f64, f64);
    fn invert(&self, value: f64) -> f64;
    fn nice(&mut self);
    fn ticks(&self, count: usize) -> Vec<f64>;
}

// For discrete scales (Band, Point, Ordinal)
pub trait DiscreteScale: Scale {
    fn set_domain_values<S: Into<String>>(&mut self, values: Vec<S>);
    fn domain_values(&self) -> &[String];
    fn set_range(&mut self, start: f64, end: f64);
    fn range(&self) -> (f64, f64);
}

// For band/point specific
pub trait BandedScale: DiscreteScale {
    fn bandwidth(&self) -> f64;
    fn step(&self) -> f64;
    fn set_padding(&mut self, inner: f64, outer: f64);
}
```

**Tasks**:
- [ ] Define new trait hierarchy in `src/scale/traits.rs`
- [ ] Implement traits for all scale types
- [ ] Remove no-op implementations
- [ ] Update `ScaleExt` to work with new hierarchy
- [ ] Migration guide for breaking changes

**Effort**: 16 hours

---

### 2.3 Unified Error Handling

**Current State**: Inconsistent error handling
- Silent no-ops (BandScale `set_domain`)
- Panic (some operations)
- `unwrap_or_else` with fallback
- Return NaN

**Solution**: Use `D3Error` and `D3Result` consistently

```rust
// src/error.rs - Extend existing
#[derive(Debug, Clone, thiserror::Error)]
pub enum D3Error {
    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Invalid range: {0}")]
    InvalidRange(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Data error: {0}")]
    DataError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

// Builder methods return Result
impl LogScale {
    pub fn with_base(mut self, base: f64) -> D3Result<Self> {
        if base <= 1.0 {
            return Err(D3Error::InvalidConfig(
                format!("Log base must be > 1.0, got {}", base)
            ));
        }
        self.base = base;
        Ok(self)
    }
}
```

**Tasks**:
- [ ] Extend `D3Error` enum with all error cases
- [ ] Audit all silent failures and panics
- [ ] Convert builder methods to return `Result`
- [ ] Add `try_*` variants for fallible operations
- [ ] Update examples to handle errors

**Effort**: 12 hours

---

### 2.4 Implement ScaleExt Universally

**Current Issue**: Only LinearScale and LogScale implement `ScaleExt`

**Tasks**:
- [ ] Implement `ScaleExt` for TimeScale
- [ ] Implement `ScaleExt` for PowScale
- [ ] Implement `ScaleExt` for SymlogScale
- [ ] Document which scales support `ScaleExt`

**Effort**: 4 hours

---

### Phase 2 Deliverables
- [ ] Unified builder pattern (`with_*` prefix)
- [ ] New scale trait hierarchy
- [ ] Consistent error handling with `D3Result`
- [ ] Migration guide for API changes
- [ ] 40+ new unit tests
- [ ] Coverage increase: +10%
- [ ] Release minor version (0.x+1.0)

### Phase 2 Test Summary

| Task | Tests Added | Test Types |
|------|-------------|------------|
| Builder pattern | 10 | API consistency |
| Scale traits | 15 | Trait compliance, polymorphism |
| Error handling | 10 | Error cases, Result types |
| ScaleExt | 5 | Trait implementation |
| **Total** | **40+** | |

#### Sample Phase 2 Unit Tests

```rust
// Test trait hierarchy
#[test]
fn test_continuous_scale_trait() {
    fn accepts_continuous<S: ContinuousScale>(scale: &S) {
        scale.set_domain(0.0, 100.0);
        scale.set_range(0.0, 500.0);
    }

    let linear = LinearScale::new();
    let log = LogScale::new();
    let time = TimeScale::new();

    accepts_continuous(&linear);
    accepts_continuous(&log);
    accepts_continuous(&time);
}

#[test]
fn test_discrete_scale_trait() {
    fn accepts_discrete<S: DiscreteScale>(scale: &mut S) {
        scale.set_domain_values(vec!["a", "b", "c"]);
    }

    let mut band = BandScale::new();
    let mut point = PointScale::new();

    accepts_discrete(&mut band);
    accepts_discrete(&mut point);
}

#[test]
fn test_error_handling_invalid_log_base() {
    let result = LogScale::new().with_base(0.5);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), D3Error::InvalidConfig(_)));
}

#[test]
fn test_builder_pattern_consistency() {
    // All scales should use with_* prefix
    let _ = LinearScale::new()
        .with_domain(0.0, 100.0)
        .with_range(0.0, 500.0)
        .with_clamp(true);

    let _ = BandScale::new()
        .with_domain(vec!["a", "b", "c"])
        .with_range(0.0, 300.0)
        .with_padding(0.1);

    let _ = TimeScale::new()
        .with_domain_ms(0, 1000000)
        .with_range(0.0, 500.0);
}
```

---

## Phase 3: Algorithm Corrections (Week 5-7)

> **Priority**: HIGH
> **Goal**: Fix incorrect algorithms to match D3.js behavior

### 3.1 Pack Layout Algorithm Rewrite

**Current Issue**: O(n³) complexity, incorrect placement

**Solution**: Implement proper front-chain algorithm

```rust
/// Front-chain circle packing algorithm
/// Based on Wang et al. "Visualization of Large Hierarchical Data by Circle Packing"
pub struct FrontChain {
    nodes: Vec<FrontChainNode>,
    // ... front chain state
}

impl PackLayout {
    fn pack_siblings(&self, circles: &mut [Circle]) {
        if circles.len() < 3 {
            self.pack_few(circles);
            return;
        }

        // Initialize with first 3 circles
        let mut chain = FrontChain::new(&circles[0..3]);

        // Add remaining circles
        for circle in &circles[3..] {
            chain.insert(circle);
        }

        // Extract positions
        chain.finalize(circles);
    }
}
```

**Tasks**:
- [ ] Implement `FrontChain` data structure
- [ ] Implement insertion algorithm
- [ ] Implement tangent circle placement
- [ ] Add comprehensive tests
- [ ] Benchmark against current implementation

**Effort**: 24 hours

---

### 3.2 Treemap Squarify Algorithm Fix

**Current Issue**: `worst_ratio` calculation is incomplete

**Solution**: Implement correct squarification

```rust
fn worst_ratio(&self, row: &[f64], width: f64, height: f64) -> f64 {
    let s = row.iter().sum::<f64>();
    let rmax = row.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let rmin = row.iter().cloned().fold(f64::INFINITY, f64::min);

    let s2 = s * s;
    let w2 = width * width;

    f64::max(
        w2 * rmax / s2,
        s2 / (w2 * rmin)
    )
}

fn squarify(&self, node: &mut HierarchyNode<T>, x0: f64, y0: f64, x1: f64, y1: f64) {
    let values: Vec<f64> = node.children.iter().map(|c| c.value).collect();
    let mut row: Vec<f64> = Vec::new();
    let mut row_sum = 0.0;

    let width = x1 - x0;
    let height = y1 - y0;
    let mut row_width = if width < height { width } else { height };

    for value in values {
        row.push(value);
        row_sum += value;

        let new_ratio = self.worst_ratio(&row, row_width, /* ... */);
        let old_ratio = if row.len() > 1 {
            self.worst_ratio(&row[..row.len()-1], row_width, /* ... */)
        } else {
            f64::INFINITY
        };

        if new_ratio > old_ratio {
            // Adding this rectangle made ratio worse
            row.pop();
            self.layout_row(&row, /* ... */);
            row.clear();
            row.push(value);
            // Update remaining space
        }
    }

    if !row.is_empty() {
        self.layout_row(&row, /* ... */);
    }
}
```

**Tasks**:
- [ ] Fix `worst_ratio` calculation
- [ ] Implement proper row layout
- [ ] Handle remaining space correctly
- [ ] Compare output to D3.js treemap
- [ ] Add visual regression tests

**Effort**: 16 hours

---

### 3.3 Geographic Area Calculation

**Current Issue**: Uses pixel-space shoelace formula instead of spherical geodesy

**Solution**: Implement spherical excess formula

```rust
/// Calculate geographic area using spherical excess
/// Returns area in steradians (multiply by R² for square meters)
pub fn spherical_area(&self, coordinates: &[Vec<Position>]) -> f64 {
    let mut total = 0.0;

    for ring in coordinates {
        total += self.ring_spherical_area(ring);
    }

    total.abs()
}

fn ring_spherical_area(&self, ring: &[Position]) -> f64 {
    let n = ring.len();
    if n < 3 { return 0.0; }

    let mut sum = 0.0;

    for i in 0..n {
        let j = (i + 1) % n;
        let k = (i + 2) % n;

        let (lon1, lat1) = (ring[i][0].to_radians(), ring[i][1].to_radians());
        let (lon2, lat2) = (ring[j][0].to_radians(), ring[j][1].to_radians());
        let (lon3, lat3) = (ring[k][0].to_radians(), ring[k][1].to_radians());

        // Spherical excess calculation
        sum += (lon2 - lon1) * (2.0 + lat1.sin() + lat2.sin());
    }

    sum.abs() / 2.0
}
```

**Tasks**:
- [ ] Implement spherical excess formula
- [ ] Add `spherical_area()` method to `GeoPath`
- [ ] Keep pixel area as `projected_area()` for compatibility
- [ ] Test against known geographic areas
- [ ] Document the difference between methods

**Effort**: 8 hours

---

### 3.4 Tree Layout Depth Calculation

**Current Issue**: Redundant depth recalculation during normalization

**Solution**: Use pre-computed height from `each_before()`

```rust
fn second_walk<T>(&self, node: &mut HierarchyNode<T>, min_x: f64, max_x: f64) {
    // Use pre-computed height instead of recalculating
    let height = node.height as f64;  // Already set in each_before()
    let x_range = (max_x - min_x).max(1.0);

    self.normalize_coords(node, min_x, x_range, height);
}
```

**Tasks**:
- [ ] Remove `find_max_depth()` call
- [ ] Use `node.height` directly
- [ ] Verify `each_before()` sets height correctly
- [ ] Add assertion for height validity

**Effort**: 2 hours

---

### 3.5 Force Simulation Alpha Decay

**Current Issue**: Uses additive blending instead of exponential decay

**Solution**: Match D3's multiplicative decay

```rust
pub fn tick(&mut self) {
    // Exponential decay (D3-compatible)
    self.alpha *= 1.0 - self.alpha_decay;

    // Check for convergence
    if self.alpha < self.alpha_min {
        self.alpha = self.alpha_min;
    }

    // Apply forces
    for force in self.forces.values() {
        force.apply(&mut self.nodes, self.alpha);
    }

    // Update positions
    self.apply_velocities();
}

/// Reheat the simulation (e.g., after topology change)
pub fn reheat(&mut self) {
    self.alpha = 1.0;
}
```

**Tasks**:
- [ ] Change alpha decay to multiplicative
- [ ] Add `reheat()` method
- [ ] Update `alpha_decay` default value
- [ ] Document convergence behavior

**Effort**: 4 hours

---

### Phase 3 Deliverables
- [ ] Correct pack layout algorithm (O(n²))
- [ ] Correct treemap squarification
- [ ] Spherical area calculation
- [ ] Optimized tree layout
- [ ] D3-compatible force decay
- [ ] 50+ new unit tests
- [ ] Coverage increase: +10%
- [ ] Visual regression test suite

### Phase 3 Test Summary

| Task | Tests Added | Test Types |
|------|-------------|------------|
| Pack layout | 15 | Correctness, overlap, performance |
| Treemap squarify | 12 | Aspect ratio, D3 reference |
| Spherical area | 8 | Known areas, D3 comparison |
| Tree layout | 5 | Depth, positioning |
| Force decay | 10 | Convergence, alpha |
| **Total** | **50+** | |

#### Sample Phase 3 Unit Tests

```rust
// Pack layout tests
#[test]
fn test_pack_layout_no_overlap() {
    let mut root = create_test_hierarchy(100);
    let layout = PackLayout::new().size(1000.0, 1000.0);
    let result = layout.layout(root);

    for i in 0..result.children.len() {
        for j in (i + 1)..result.children.len() {
            let ci = &result.children[i];
            let cj = &result.children[j];
            let dist = ((ci.x - cj.x).powi(2) + (ci.y - cj.y).powi(2)).sqrt();
            let min_dist = ci.radius + cj.radius;

            assert!(
                dist >= min_dist - 0.01,
                "Overlap between {} and {}: dist={}, min={}",
                i, j, dist, min_dist
            );
        }
    }
}

#[test]
fn test_pack_layout_performance() {
    use std::time::Instant;

    let sizes = [100, 500, 1000];
    let mut times = Vec::new();

    for &n in &sizes {
        let root = create_test_hierarchy(n);
        let layout = PackLayout::new().size(1000.0, 1000.0);

        let start = Instant::now();
        let _ = layout.layout(root);
        times.push(start.elapsed());
    }

    // O(n²) should show quadratic growth
    let ratio_1 = times[1].as_micros() as f64 / times[0].as_micros() as f64;
    let ratio_2 = times[2].as_micros() as f64 / times[1].as_micros() as f64;

    // Should be roughly (500/100)² = 25x and (1000/500)² = 4x
    assert!(ratio_1 < 50.0, "Pack layout worse than O(n²)");
    assert!(ratio_2 < 10.0, "Pack layout worse than O(n²)");
}

// Treemap tests
#[test]
fn test_treemap_squarify_aspect_ratio() {
    let mut root = HierarchyNode::new("root", 0.0);
    for i in 1..=10 {
        root.add_child(HierarchyNode::new(format!("c{}", i), i as f64 * 10.0));
    }

    let layout = TreemapLayout::new()
        .size(400.0, 400.0)
        .tiling(TilingMethod::Squarify);

    let result = layout.layout(root);

    // Check aspect ratios are close to 1
    for child in &result.children {
        let aspect = (child.x1 - child.x0) / (child.y1 - child.y0);
        let ratio = aspect.max(1.0 / aspect);
        assert!(ratio < 3.0, "Bad aspect ratio: {}", ratio);
    }
}

// Spherical area tests
#[test]
fn test_spherical_area_known_values() {
    let geo_path = GeoPath::new(MercatorProjection::new());

    // Earth's total surface area ≈ 510.1 million km²
    // One hemisphere = 255 million km²
    let hemisphere = Geometry::Polygon {
        coordinates: vec![vec![
            [-180.0, 0.0], [180.0, 0.0], [180.0, 90.0], [-180.0, 90.0], [-180.0, 0.0]
        ]]
    };

    let area_steradians = geo_path.spherical_area(&hemisphere);
    let earth_radius_km = 6371.0;
    let area_km2 = area_steradians * earth_radius_km * earth_radius_km;

    assert_approx_eq(area_km2, 255_000_000.0, 5_000_000.0);  // Within 2%
}

// Force simulation tests
#[test]
fn test_force_simulation_converges() {
    let nodes: Vec<_> = (0..20).map(|i| SimulationNode::new(i)).collect();
    let mut sim = ForceSimulation::new(nodes)
        .add_force("center", CenterForce::new(0.0, 0.0))
        .add_force("charge", ManyBodyForce::new().strength(-30.0));

    // Run until convergence
    let mut iterations = 0;
    while sim.alpha() > sim.alpha_min() && iterations < 1000 {
        sim.tick();
        iterations += 1;
    }

    assert!(iterations < 500, "Simulation should converge in ~300 iterations");
    assert!(sim.alpha() < 0.01, "Alpha should be near zero");
}
```

---

## Phase 4: D3 Feature Parity (Week 8-11)

> **Priority**: MEDIUM
> **Goal**: Implement missing D3 features for full compatibility

### 4.1 Adaptive Resampling for Projections

**Missing Feature**: `precision()` parameter is ignored

```rust
pub trait Projection: Send + Sync {
    fn project(&self, lon: f64, lat: f64) -> (f64, f64);
    fn precision(&self) -> f64;

    /// Project a line segment with adaptive resampling
    fn project_line(&self, coords: &[Position]) -> Vec<(f64, f64)> {
        let mut result = Vec::new();

        for i in 0..coords.len() - 1 {
            self.resample_segment(
                coords[i], coords[i + 1],
                &mut result
            );
        }

        result
    }

    fn resample_segment(&self, a: Position, b: Position, out: &mut Vec<(f64, f64)>) {
        let (ax, ay) = self.project(a[0], a[1]);
        let (bx, by) = self.project(b[0], b[1]);

        // Check if segment needs subdivision
        let mid_lon = (a[0] + b[0]) / 2.0;
        let mid_lat = (a[1] + b[1]) / 2.0;
        let (mx, my) = self.project(mid_lon, mid_lat);

        // Linear interpolation of projected points
        let expected_x = (ax + bx) / 2.0;
        let expected_y = (ay + by) / 2.0;

        let error = ((mx - expected_x).powi(2) + (my - expected_y).powi(2)).sqrt();

        if error > self.precision() {
            // Subdivide
            self.resample_segment(a, [mid_lon, mid_lat], out);
            self.resample_segment([mid_lon, mid_lat], b, out);
        } else {
            out.push((ax, ay));
        }
    }
}
```

**Tasks**:
- [ ] Add `precision` field to all projections
- [ ] Implement `project_line()` with adaptive resampling
- [ ] Update `GeoPath` to use `project_line()`
- [ ] Add tests for curved projection accuracy

**Effort**: 12 hours

---

### 4.2 fitExtent / fitSize for Projections

**Missing Feature**: Cannot auto-fit GeoJSON to viewport

```rust
impl<P: Projection + ProjectionBuilder> P {
    /// Fit projection to bounds, preserving aspect ratio
    pub fn fit_extent(
        self,
        extent: [[f64; 2]; 2],  // [[x0, y0], [x1, y1]]
        geometry: &Geometry
    ) -> Self {
        let bounds = self.compute_bounds(geometry);

        let width = extent[1][0] - extent[0][0];
        let height = extent[1][1] - extent[0][1];

        let geo_width = bounds.east - bounds.west;
        let geo_height = bounds.north - bounds.south;

        let scale = 0.95 * f64::min(
            width / geo_width,
            height / geo_height
        );

        let center_lon = (bounds.west + bounds.east) / 2.0;
        let center_lat = (bounds.south + bounds.north) / 2.0;

        let (cx, cy) = self.project(center_lon, center_lat);

        let translate_x = (extent[0][0] + extent[1][0]) / 2.0 - cx * scale;
        let translate_y = (extent[0][1] + extent[1][1]) / 2.0 - cy * scale;

        self.scale(scale).translate(translate_x, translate_y)
    }

    /// Fit projection to size, centered
    pub fn fit_size(self, size: [f64; 2], geometry: &Geometry) -> Self {
        self.fit_extent([[0.0, 0.0], size], geometry)
    }
}
```

**Tasks**:
- [ ] Implement `compute_bounds()` for all geometry types
- [ ] Implement `fit_extent()`
- [ ] Implement `fit_size()`
- [ ] Handle edge cases (empty geometry, single point)
- [ ] Add examples to documentation

**Effort**: 8 hours

---

### 4.3 Antimeridian Handling

**Missing Feature**: Lines crossing ±180° are not cut

```rust
impl GeoPath {
    fn clip_antimeridian(&self, coords: &[Position]) -> Vec<Vec<Position>> {
        let mut segments: Vec<Vec<Position>> = Vec::new();
        let mut current: Vec<Position> = Vec::new();

        for i in 0..coords.len() {
            let p = coords[i];

            if i > 0 {
                let prev = coords[i - 1];

                // Check for antimeridian crossing
                if (p[0] - prev[0]).abs() > 180.0 {
                    // Interpolate crossing point
                    let (cross1, cross2) = self.antimeridian_crossing(prev, p);

                    current.push(cross1);
                    segments.push(std::mem::take(&mut current));

                    current.push(cross2);
                }
            }

            current.push(p);
        }

        if !current.is_empty() {
            segments.push(current);
        }

        segments
    }

    fn antimeridian_crossing(&self, a: Position, b: Position) -> (Position, Position) {
        let t = (180.0 - a[0].abs()) / (b[0].abs() + a[0].abs() - 360.0);
        let lat = a[1] + t * (b[1] - a[1]);

        if a[0] > 0.0 {
            ([180.0, lat], [-180.0, lat])
        } else {
            ([-180.0, lat], [180.0, lat])
        }
    }
}
```

**Tasks**:
- [ ] Implement `clip_antimeridian()` for LineString
- [ ] Implement for Polygon (split into MultiPolygon)
- [ ] Handle MultiLineString and MultiPolygon
- [ ] Add tests with Pacific-crossing data

**Effort**: 12 hours

---

### 4.4 Color Scale with Color Space Parameter

**Missing Feature**: Scales always use RGB interpolation

```rust
pub trait ColorScale: Send + Sync {
    fn color(&self, t: f64) -> Rgba;
    fn scale_type(&self) -> &'static str;

    /// Create a new scale with different interpolation space
    fn with_color_space(&self, space: ColorSpace) -> Box<dyn ColorScale>;
}

impl SequentialScale {
    pub fn new_with_space(colors: Vec<Rgba>, space: ColorSpace) -> Self {
        Self {
            colors,
            color_space: space,
            ..Default::default()
        }
    }
}

impl ColorScale for SequentialScale {
    fn color(&self, t: f64) -> Rgba {
        // ... get i and local_t ...

        match self.color_space {
            ColorSpace::Rgb => self.colors[i].lerp(&self.colors[i + 1], local_t),
            ColorSpace::Lab => interpolate_lab(&self.colors[i], &self.colors[i + 1], local_t),
            ColorSpace::Hcl => interpolate_hcl(&self.colors[i], &self.colors[i + 1], local_t),
            ColorSpace::Hsl => interpolate_hsl(&self.colors[i], &self.colors[i + 1], local_t),
        }
    }
}
```

**Tasks**:
- [ ] Add `ColorSpace` field to `SequentialScale`
- [ ] Add `ColorSpace` field to `DivergingScale`
- [ ] Implement `with_color_space()` method
- [ ] Update constructors with space parameter
- [ ] Add Lab-based scale examples

**Effort**: 8 hours

---

### 4.5 ArcGenerator with Accessor Functions

**Missing Feature**: ArcGenerator uses static values, not D3-style accessors

```rust
pub struct ArcGenerator<D = ()> {
    inner_radius_fn: Box<dyn Fn(&D, usize) -> f64 + Send + Sync>,
    outer_radius_fn: Box<dyn Fn(&D, usize) -> f64 + Send + Sync>,
    start_angle_fn: Box<dyn Fn(&D, usize) -> f64 + Send + Sync>,
    end_angle_fn: Box<dyn Fn(&D, usize) -> f64 + Send + Sync>,
    corner_radius: f64,
    pad_angle: f64,
    _phantom: PhantomData<D>,
}

impl<D> ArcGenerator<D> {
    pub fn inner_radius<F>(mut self, f: F) -> Self
    where F: Fn(&D, usize) -> f64 + Send + Sync + 'static
    {
        self.inner_radius_fn = Box::new(f);
        self
    }

    pub fn generate(&self, data: &[D]) -> Vec<Vec<PathSegment>> {
        data.iter()
            .enumerate()
            .map(|(i, d)| {
                let inner = (self.inner_radius_fn)(d, i);
                let outer = (self.outer_radius_fn)(d, i);
                let start = (self.start_angle_fn)(d, i);
                let end = (self.end_angle_fn)(d, i);

                self.generate_arc(inner, outer, start, end)
            })
            .collect()
    }
}

// Maintain backward compatibility
impl ArcGenerator<()> {
    /// Create arc with static values (backward compatible)
    pub fn static_arc() -> StaticArcGenerator {
        StaticArcGenerator::new()
    }
}
```

**Tasks**:
- [ ] Create generic `ArcGenerator<D>`
- [ ] Implement accessor function setters
- [ ] Maintain `StaticArcGenerator` for compatibility
- [ ] Update PieChart examples
- [ ] Add donut chart example with data-driven radii

**Effort**: 12 hours

---

### 4.6 Graticule Generator

**Missing Feature**: Cannot draw map grid lines

```rust
/// Generates a graticule (grid of meridians and parallels)
pub struct Graticule {
    /// Step size for major parallels/meridians (degrees)
    pub step: [f64; 2],
    /// Step size for minor lines (degrees)
    pub step_minor: [f64; 2],
    /// Extent of graticule [west, south, east, north]
    pub extent: [f64; 4],
    /// Precision for line interpolation
    pub precision: f64,
}

impl Graticule {
    pub fn new() -> Self {
        Self {
            step: [10.0, 10.0],
            step_minor: [2.5, 2.5],
            extent: [-180.0, -90.0, 180.0, 90.0],
            precision: 2.5,
        }
    }

    /// Generate all graticule lines
    pub fn lines(&self) -> Vec<Geometry> {
        let mut lines = Vec::new();

        // Meridians (vertical lines)
        let mut lon = self.extent[0];
        while lon <= self.extent[2] {
            lines.push(self.meridian(lon));
            lon += self.step[0];
        }

        // Parallels (horizontal lines)
        let mut lat = self.extent[1];
        while lat <= self.extent[3] {
            lines.push(self.parallel(lat));
            lat += self.step[1];
        }

        lines
    }

    fn meridian(&self, lon: f64) -> Geometry {
        let mut coords = Vec::new();
        let mut lat = self.extent[1];

        while lat <= self.extent[3] {
            coords.push([lon, lat]);
            lat += self.precision;
        }

        Geometry::LineString { coordinates: coords }
    }

    fn parallel(&self, lat: f64) -> Geometry {
        let mut coords = Vec::new();
        let mut lon = self.extent[0];

        while lon <= self.extent[2] {
            coords.push([lon, lat]);
            lon += self.precision;
        }

        Geometry::LineString { coordinates: coords }
    }

    /// Generate outline of graticule extent
    pub fn outline(&self) -> Geometry {
        Geometry::Polygon {
            coordinates: vec![vec![
                [self.extent[0], self.extent[1]],
                [self.extent[2], self.extent[1]],
                [self.extent[2], self.extent[3]],
                [self.extent[0], self.extent[3]],
                [self.extent[0], self.extent[1]],
            ]]
        }
    }
}
```

**Tasks**:
- [ ] Implement `Graticule` struct
- [ ] Add `lines()` method
- [ ] Add `outline()` method
- [ ] Add step configuration
- [ ] Add globe example with graticule

**Effort**: 8 hours

---

### 4.7 Oklab/Oklch Color Spaces

**Missing Feature**: Modern perceptually uniform color spaces

```rust
/// Oklab color space - more perceptually uniform than CIELAB
#[derive(Clone, Copy, Debug)]
pub struct Oklab {
    pub l: f32,  // Lightness [0, 1]
    pub a: f32,  // Green-red axis
    pub b: f32,  // Blue-yellow axis
    pub alpha: f32,
}

impl Oklab {
    pub fn from_rgba(rgba: &Rgba) -> Self {
        // Convert sRGB to linear RGB
        let r = srgb_to_linear(rgba.r);
        let g = srgb_to_linear(rgba.g);
        let b = srgb_to_linear(rgba.b);

        // Linear RGB to LMS
        let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
        let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
        let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

        // LMS to Oklab
        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        Self {
            l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
            alpha: rgba.a,
        }
    }

    pub fn to_rgba(&self) -> Rgba {
        // Inverse transformation
        // ... (implementation)
    }

    pub fn lerp(&self, other: &Oklab, t: f32) -> Oklab {
        Oklab {
            l: self.l + (other.l - self.l) * t,
            a: self.a + (other.a - self.a) * t,
            b: self.b + (other.b - self.b) * t,
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }
}

/// Oklch - cylindrical form of Oklab (like HCL is to Lab)
#[derive(Clone, Copy, Debug)]
pub struct Oklch {
    pub l: f32,  // Lightness
    pub c: f32,  // Chroma
    pub h: f32,  // Hue (degrees)
    pub alpha: f32,
}
```

**Tasks**:
- [ ] Implement `Oklab` struct with conversions
- [ ] Implement `Oklch` struct with conversions
- [ ] Add `interpolate_oklab()` function
- [ ] Add Oklab to `ColorSpace` enum
- [ ] Compare gradients with Lab

**Effort**: 8 hours

---

### Phase 4 Deliverables
- [ ] Adaptive resampling in projections
- [ ] fitExtent/fitSize methods
- [ ] Antimeridian handling
- [ ] Color space parameter for scales
- [ ] Generic ArcGenerator
- [ ] Graticule generator
- [ ] Oklab/Oklch color spaces
- [ ] 60+ new unit tests
- [ ] Coverage increase: +10%
- [ ] All D3 examples reproducible

### Phase 4 Test Summary

| Task | Tests Added | Test Types |
|------|-------------|------------|
| Adaptive resampling | 8 | Precision, curve quality |
| fitExtent/fitSize | 6 | Bounds, centering |
| Antimeridian | 10 | Crossing detection, splitting |
| Color space scales | 8 | Lab interpolation, perceptual |
| ArcGenerator | 10 | Accessor functions, D3 compat |
| Graticule | 8 | Line generation, extent |
| Oklab/Oklch | 10 | Conversion, roundtrip |
| **Total** | **60+** | |

#### Sample Phase 4 Unit Tests

```rust
// Adaptive resampling tests
#[test]
fn test_adaptive_resampling_precision() {
    let proj = MercatorProjection::new()
        .precision(0.5)
        .scale(100.0);

    let line = vec![[0.0, 0.0], [90.0, 0.0]];  // 90° arc
    let segments = proj.project_line(&line);

    // With precision 0.5, should subdivide significantly
    assert!(segments.len() > 10, "Should resample long arc");
}

// fitExtent tests
#[test]
fn test_fit_extent_centers_geometry() {
    let geojson = GeoJson::parse(r#"{
        "type": "Polygon",
        "coordinates": [[[-10, -10], [10, -10], [10, 10], [-10, 10], [-10, -10]]]
    }"#).unwrap();

    let proj = MercatorProjection::new()
        .fit_extent([[0.0, 0.0], [800.0, 600.0]], &geojson.geometry());

    // Center of geometry should map to center of extent
    let (cx, cy) = proj.project(0.0, 0.0);
    assert_approx_eq(cx, 400.0, 10.0);
    assert_approx_eq(cy, 300.0, 10.0);
}

// Antimeridian tests
#[test]
fn test_antimeridian_crossing_detection() {
    let geo_path = GeoPath::new(MercatorProjection::new());

    let line = Geometry::LineString {
        coordinates: vec![[170.0, 0.0], [-170.0, 0.0]]
    };

    let segments = geo_path.to_segments(&line);

    // Should split into two segments
    let move_count = segments.iter()
        .filter(|s| matches!(s, GeoPathSegment::MoveTo(_, _)))
        .count();

    assert_eq!(move_count, 2, "Should split at antimeridian");
}

// Oklab tests
#[test]
fn test_oklab_perceptual_uniformity() {
    // Oklab should have more uniform perceptual steps than sRGB
    let colors: Vec<Rgba> = (0..=10)
        .map(|i| {
            let t = i as f64 / 10.0;
            interpolate_oklab(&Rgba::BLACK, &Rgba::WHITE, t)
        })
        .collect();

    // Check that lightness steps are roughly equal
    let oklabs: Vec<Oklab> = colors.iter().map(|c| Oklab::from_rgba(c)).collect();

    for i in 1..oklabs.len() {
        let step = oklabs[i].l - oklabs[i - 1].l;
        assert_approx_eq(step, 0.1, 0.02);  // Should be ~0.1 each
    }
}
```

---

## Phase 5: Architecture Improvements (Week 12-14)

> **Priority**: MEDIUM
> **Goal**: Improve internal architecture for maintainability and performance

### 5.1 Force Simulation Performance

**Current Issue**: HashMap lookup per force per tick

```rust
// CURRENT
let force_names: Vec<String> = self.forces.keys().cloned().collect();
for name in force_names {
    if let Some(force) = self.forces.get(&name) {
        force.apply(&mut self.nodes, alpha);
    }
}

// IMPROVED
struct ForceSimulation {
    forces: Vec<(String, Box<dyn Force>)>,  // Vec for O(1) iteration
    force_index: HashMap<String, usize>,    // For named lookup
}

impl ForceSimulation {
    pub fn tick(&mut self) {
        for (_, force) in &self.forces {
            force.apply(&mut self.nodes, self.alpha);
        }
    }

    pub fn get_force(&self, name: &str) -> Option<&dyn Force> {
        self.force_index.get(name)
            .and_then(|&i| self.forces.get(i))
            .map(|(_, f)| f.as_ref())
    }
}
```

**Tasks**:
- [ ] Refactor to use `Vec` for iteration
- [ ] Add `HashMap` index for named access
- [ ] Benchmark improvement
- [ ] Update documentation

**Effort**: 4 hours

---

### 5.2 Layout-Specific Result Types

**Current Issue**: `HierarchyNode` has mixed properties for all layouts

```rust
// CURRENT
pub struct HierarchyNode<T> {
    pub x: f64,
    pub y: f64,
    pub width: f64,       // Only treemap
    pub rect_height: f64, // Only treemap
    pub radius: f64,      // Only pack
}

// IMPROVED
pub struct TreeLayoutNode<T> {
    pub data: T,
    pub x: f64,
    pub y: f64,
    pub depth: usize,
    pub height: usize,
    pub children: Vec<TreeLayoutNode<T>>,
}

pub struct TreemapLayoutNode<T> {
    pub data: T,
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub children: Vec<TreemapLayoutNode<T>>,
}

pub struct PackLayoutNode<T> {
    pub data: T,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub children: Vec<PackLayoutNode<T>>,
}

// Generic hierarchy for input
pub struct HierarchyNode<T> {
    pub data: T,
    pub value: f64,
    pub children: Vec<HierarchyNode<T>>,
}

// Layout methods return specific types
impl TreeLayout {
    pub fn layout<T: Clone>(&self, root: HierarchyNode<T>) -> TreeLayoutNode<T>;
}

impl TreemapLayout {
    pub fn layout<T: Clone>(&self, root: HierarchyNode<T>) -> TreemapLayoutNode<T>;
}
```

**Tasks**:
- [ ] Define layout-specific output types
- [ ] Update layout methods to return specific types
- [ ] Add conversion methods between types
- [ ] Update chart examples
- [ ] Migration guide for breaking change

**Effort**: 16 hours

---

### 5.3 Event Response Types

**Current Issue**: Interaction methods return only `bool`

```rust
// CURRENT
pub fn handle_wheel(&self, ...) -> bool

// IMPROVED
#[derive(Debug, Clone)]
pub enum EventResponse {
    /// Event was ignored (not consumed)
    Ignored,

    /// Event was consumed, no visual update needed
    Consumed,

    /// Event was consumed, visual update needed
    ConsumedWithUpdate,

    /// Event consumed, animation started
    ConsumedWithAnimation {
        duration_ms: u64,
    },

    /// Event partially handled, propagate to parent
    Partial,
}

impl ZoomBehavior {
    pub fn handle_wheel(&self, transform: &mut ZoomTransform,
        delta: f64, center_x: f64, center_y: f64) -> EventResponse
    {
        let old_k = transform.k;
        // ... apply zoom ...

        if (transform.k - old_k).abs() < f64::EPSILON {
            EventResponse::Ignored
        } else {
            EventResponse::ConsumedWithUpdate
        }
    }
}
```

**Tasks**:
- [ ] Define `EventResponse` enum
- [ ] Update all interaction methods
- [ ] Add animation support infrastructure
- [ ] Document event handling patterns

**Effort**: 8 hours

---

### 5.4 Remove Dead Code

**Identified Dead Code**:
1. `HierarchyNode.parent` - never set
2. `compute_order_from_values()` - duplicates `compute_order()`
3. Various unused imports and variables

**Tasks**:
- [ ] Run `cargo clippy` with `--warn dead_code`
- [ ] Remove or fix `parent` tracking
- [ ] Consolidate duplicated methods
- [ ] Document intentionally unused code

**Effort**: 4 hours

---

### 5.5 Precision Consistency

**Current Issue**: Mixed f32/f64 usage causes confusion

**Solution**: Define clear boundaries

```rust
// Public API: f64 for precision
pub trait Scale {
    fn scale(&self, value: f64) -> f64;
}

// GPU/Color: f32 for performance
pub struct Rgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// Internal: Document conversions
impl Rgba {
    /// Convert from f64 coordinates (scale output) to f32 for GPU
    #[inline]
    pub fn from_f64(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }
}
```

**Tasks**:
- [ ] Audit all f32/f64 conversions
- [ ] Document precision policy
- [ ] Add conversion helpers
- [ ] Remove unnecessary casts

**Effort**: 6 hours

---

### Phase 5 Deliverables
- [ ] Optimized force simulation
- [ ] Type-safe layout results
- [ ] Rich event responses
- [ ] No dead code
- [ ] Consistent precision handling
- [ ] 30+ new unit tests
- [ ] Coverage increase: +5%
- [ ] All refactoring covered by tests

### Phase 5 Test Summary

| Task | Tests Added | Test Types |
|------|-------------|------------|
| Force performance | 5 | Benchmark, comparison |
| Layout result types | 10 | Type safety, compilation |
| Event responses | 10 | State machine, behavior |
| Dead code removal | 0 | Covered by existing tests |
| Precision | 5 | f32/f64 boundary |
| **Total** | **30+** | |

#### Sample Phase 5 Unit Tests

```rust
// Layout type safety tests
#[test]
fn test_tree_layout_returns_tree_node() {
    let root = create_test_hierarchy(10);
    let layout = TreeLayout::new().size(800.0, 600.0);

    let result: TreeLayoutNode<String> = layout.layout(root);

    // TreeLayoutNode should only have x, y (not width, radius)
    assert!(result.x >= 0.0);
    assert!(result.y >= 0.0);
    // These shouldn't compile:
    // let _ = result.width;   // No such field
    // let _ = result.radius;  // No such field
}

#[test]
fn test_treemap_layout_returns_treemap_node() {
    let root = create_test_hierarchy(10);
    let layout = TreemapLayout::new().size(800.0, 600.0);

    let result: TreemapLayoutNode<String> = layout.layout(root);

    // TreemapLayoutNode should have bounds
    assert!(result.x0 <= result.x1);
    assert!(result.y0 <= result.y1);
}

// Event response tests
#[test]
fn test_event_response_consumed_with_update() {
    let zoom = ZoomBehavior::new();
    let mut t = ZoomTransform::identity();

    let response = zoom.handle_wheel(&mut t, 100.0, 400.0, 300.0);

    assert!(matches!(response, EventResponse::ConsumedWithUpdate));
}

#[test]
fn test_event_response_ignored() {
    let zoom = ZoomBehavior::new()
        .scale_extent(1.0, 1.0);  // Can't zoom

    let mut t = ZoomTransform::identity();
    let response = zoom.handle_wheel(&mut t, 100.0, 400.0, 300.0);

    assert!(matches!(response, EventResponse::Ignored));
}

// Performance tests
#[test]
fn test_force_simulation_vec_faster_than_hashmap() {
    use std::time::Instant;

    let nodes: Vec<_> = (0..1000).map(|i| SimulationNode::new(i)).collect();
    let mut sim = ForceSimulation::new(nodes.clone())
        .add_force("center", CenterForce::new(0.0, 0.0))
        .add_force("charge", ManyBodyForce::new());

    let start = Instant::now();
    sim.tick_n(100);
    let elapsed = start.elapsed();

    // Should complete 100 ticks of 1000 nodes in < 100ms
    assert!(elapsed.as_millis() < 100, "Force simulation too slow");
}
```

---

## Phase 6: Integration Testing & Documentation (Week 15-16)

> **Priority**: MEDIUM
> **Goal**: Integration tests, visual regression, benchmarks, and documentation
>
> Note: Unit tests are written in Phases 1-5. This phase focuses on integration testing, visual regression, and documentation.

### 6.1 Visual Regression Tests

```rust
// tests/visual_regression.rs

/// Generate SVG output for visual comparison with D3.js
fn render_to_svg<F: Fn(&mut SvgRenderer)>(name: &str, render: F) {
    let mut svg = SvgRenderer::new(800, 600);
    render(&mut svg);

    let output = svg.to_string();
    let path = format!("tests/snapshots/{}.svg", name);

    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(&path, &output).unwrap();
    } else {
        let expected = std::fs::read_to_string(&path).unwrap();
        assert_eq!(output, expected, "Visual regression for {}", name);
    }
}

#[test]
fn test_line_chart_visual() {
    render_to_svg("line_chart", |svg| {
        let data = vec![/* ... */];
        let line = LineGenerator::new()
            .curve(MonotoneCurve::new());
        // ... render
    });
}
```

**Tasks**:
- [ ] Create `SvgRenderer` for test output
- [ ] Add snapshot tests for all chart types
- [ ] Compare with D3.js reference output
- [ ] CI integration for regression detection

**Effort**: 16 hours

---

### 6.2 D3 API Compatibility Tests

```rust
// tests/d3_compatibility.rs

/// Tests that verify exact D3.js behavior match
mod d3_compat {
    use makepad_d3::scale::*;

    #[test]
    fn linear_scale_matches_d3() {
        // D3: d3.scaleLinear().domain([0, 100]).range([0, 500])(50) => 250
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(0.0), 0.0);
        assert_eq!(scale.scale(100.0), 500.0);
    }

    #[test]
    fn band_scale_matches_d3() {
        // D3: d3.scaleBand().domain(['a','b','c']).range([0,300]).bandwidth()
        let scale = BandScale::new()
            .domain(vec!["a", "b", "c"])
            .range(0.0, 300.0);

        assert_eq!(scale.bandwidth(), 100.0);
        assert_eq!(scale.scale_discrete("a"), Some(0.0));
        assert_eq!(scale.scale_discrete("b"), Some(100.0));
    }
}
```

**Tasks**:
- [ ] Create D3 reference test suite
- [ ] Cover all scale types
- [ ] Cover shape generators
- [ ] Cover layout algorithms
- [ ] Document known differences

**Effort**: 12 hours

---

### 6.3 Edge Case Tests

```rust
#[test]
fn test_empty_data() {
    let line = LineGenerator::new();
    assert!(line.generate(&[]).is_empty());

    let pie = PieLayout::new();
    assert!(pie.compute(&[]).is_empty());
}

#[test]
fn test_single_point() {
    let line = LineGenerator::new();
    let path = line.generate(&[DataPoint::from_xy(0.0, 0.0)]);
    assert_eq!(path.len(), 1);  // Just MoveTo
}

#[test]
fn test_nan_handling() {
    let scale = LinearScale::new()
        .with_domain(0.0, 100.0)
        .with_range(0.0, 500.0);

    assert!(scale.scale(f64::NAN).is_nan());
}

#[test]
fn test_infinite_values() {
    let scale = LinearScale::new()
        .with_domain(0.0, 100.0)
        .with_range(0.0, 500.0);

    assert_eq!(scale.scale(f64::INFINITY), f64::INFINITY);
}
```

**Tasks**:
- [ ] Test empty data for all generators
- [ ] Test single-element data
- [ ] Test NaN and infinity
- [ ] Test negative values where unexpected
- [ ] Test very large/small values

**Effort**: 8 hours

---

### 6.4 Documentation Updates

**Tasks**:
- [ ] Add D3 API comparison table to README
- [ ] Document all breaking changes
- [ ] Add migration guide from v1 to v2
- [ ] Add cookbook with common patterns
- [ ] Add performance guide

**Effort**: 8 hours

---

### 6.5 Benchmark Suite

```rust
// benches/scales.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_linear_scale(c: &mut Criterion) {
    let scale = LinearScale::new()
        .with_domain(0.0, 1000.0)
        .with_range(0.0, 800.0);

    c.bench_function("linear_scale_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(scale.scale(i as f64));
            }
        })
    });
}

fn bench_pack_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("pack_layout");

    for size in [10, 100, 500, 1000].iter() {
        let data = generate_hierarchy(*size);
        let layout = PackLayout::new().size(800.0, 600.0);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| b.iter(|| layout.layout(black_box(data.clone())))
        );
    }
}
```

**Tasks**:
- [ ] Add benchmarks for all scale types
- [ ] Add benchmarks for layout algorithms
- [ ] Add benchmarks for path generation
- [ ] Compare with baseline (before optimizations)
- [ ] Add to CI for regression detection

**Effort**: 8 hours

---

### Phase 6 Deliverables
- [ ] Visual regression test suite (50+ snapshots)
- [ ] D3 compatibility test suite (100+ reference tests)
- [ ] Cross-module integration tests (30+)
- [ ] Updated documentation with examples
- [ ] Benchmark suite with CI integration
- [ ] Final coverage target: 85%+

### Phase 6 Test Summary

| Task | Tests Added | Test Types |
|------|-------------|------------|
| Visual regression | 50 | Snapshot comparison |
| D3 reference | 100 | Output matching |
| Integration | 30 | Cross-module |
| Edge cases | 20 | Boundary conditions |
| **Total** | **200+** | |

### Complete Test Count Summary

| Phase | Unit Tests | Integration | Total |
|-------|------------|-------------|-------|
| Phase 1 | 18 | 0 | 18 |
| Phase 2 | 40 | 5 | 45 |
| Phase 3 | 50 | 10 | 60 |
| Phase 4 | 60 | 10 | 70 |
| Phase 5 | 30 | 5 | 35 |
| Phase 6 | 20 | 200 | 220 |
| **Total** | **218** | **230** | **448+** |

---

## Issue Tracking Matrix

| ID | Module | Issue | Severity | Phase | Status |
|----|--------|-------|----------|-------|--------|
| S-01 | Scale | Trait contract violations | Critical | 2 | ⬜ |
| S-02 | Scale | SequentialScale type mismatch | Critical | 2 | ⬜ |
| S-03 | Scale | Inconsistent builder pattern | High | 2 | ⬜ |
| S-04 | Scale | Silent error handling | High | 2 | ⬜ |
| S-05 | Scale | ScaleExt not universal | Medium | 2 | ⬜ |
| SH-01 | Shape | BasisCurve formula bug | Critical | 1 | ⬜ |
| SH-02 | Shape | ArcGenerator breaks pattern | High | 4 | ⬜ |
| SH-03 | Shape | Area path merging fragile | High | 3 | ⬜ |
| SH-04 | Shape | Inconsistent edge cases | Medium | 3 | ⬜ |
| L-01 | Layout | Pack O(n³) complexity | Critical | 3 | ⬜ |
| L-02 | Layout | Squarify incomplete | Critical | 3 | ⬜ |
| L-03 | Layout | Uninitialized positions | Critical | 1 | ⬜ |
| L-04 | Layout | Force HashMap lookup | High | 5 | ⬜ |
| L-05 | Layout | Mixed layout properties | High | 5 | ⬜ |
| L-06 | Layout | Parent field never set | Medium | 5 | ⬜ |
| G-01 | Geo | No adaptive resampling | Critical | 4 | ⬜ |
| G-02 | Geo | Pixel area calculation | Critical | 3 | ⬜ |
| G-03 | Geo | No antimeridian handling | High | 4 | ⬜ |
| G-04 | Geo | Naive line clipping | High | 4 | ⬜ |
| G-05 | Geo | Position missing altitude | High | 4 | ⬜ |
| G-06 | Geo | Incomplete rotation matrix | High | 1 | ⬜ |
| G-07 | Geo | Missing fitExtent | High | 4 | ⬜ |
| I-01 | Interaction | Zoom center bug | Critical | 1 | ⬜ |
| I-02 | Interaction | No event binding | High | 5 | ⬜ |
| I-03 | Interaction | Handle detection scale | High | 3 | ⬜ |
| I-04 | Interaction | Brush extent validation | High | 1 | ⬜ |
| I-05 | Interaction | Only bool return | Medium | 5 | ⬜ |
| C-01 | Color | HSL alpha loss | Critical | 1 | ⬜ |
| C-02 | Color | Scale ignores color space | High | 4 | ⬜ |
| C-03 | Color | Floating-point comparison | Medium | 2 | ⬜ |
| C-04 | Color | Missing Oklab | Medium | 4 | ⬜ |

---

## Success Metrics

### Code Quality
- [ ] Zero critical bugs
- [ ] Zero high severity bugs
- [ ] All `cargo clippy` warnings resolved
- [ ] No `unsafe` code without documentation
- [ ] Zero `unwrap()` in library code (use `expect()` or `?`)

### Test Coverage
| Metric | Target | Measurement |
|--------|--------|-------------|
| Line coverage | >85% | `cargo llvm-cov` |
| Branch coverage | >75% | `cargo llvm-cov` |
| Unit tests | 218+ | Test count |
| Integration tests | 230+ | Test count |
| Visual regression | 50+ | Snapshot count |
| D3 reference tests | 100+ | Reference comparisons |

### Test Quality Requirements
- [ ] All public APIs have tests
- [ ] All error paths have tests
- [ ] Edge cases: empty, single, NaN, infinity
- [ ] All bug fixes have regression tests
- [ ] D3 reference values verified against actual D3.js

### Documentation
- [ ] All public items documented
- [ ] D3 comparison table complete
- [ ] Migration guide published
- [ ] Cookbook with 20+ examples
- [ ] Test documentation explaining test patterns

### Performance
- [ ] Pack layout O(n²) or better (benchmarked)
- [ ] Force simulation 60fps with 1000 nodes
- [ ] No regressions in benchmarks
- [ ] CI blocks on performance regression >10%

### API Quality
- [ ] Consistent builder pattern
- [ ] Unified error handling
- [ ] Type-safe layout results
- [ ] Rich event responses

### CI/CD Requirements
- [ ] All tests pass on every PR
- [ ] Coverage cannot decrease
- [ ] Benchmarks run on merge to main
- [ ] Visual regression on every PR
- [ ] D3 reference tests updated quarterly

---

## Appendix: Breaking Changes

### Scale Module
- `Scale` trait split into `ContinuousScale` and `DiscreteScale`
- Builder methods renamed to `with_*` pattern
- Error-returning methods added

### Layout Module
- Layout methods return specific types (`TreeLayoutNode`, etc.)
- `HierarchyNode` simplified to input-only

### Interaction Module
- Methods return `EventResponse` instead of `bool`

### Migration Path
1. Update trait implementations
2. Rename builder method calls
3. Handle new Result types
4. Update layout result usage
5. Handle EventResponse

---

*Plan created: January 2026*
*Last updated: January 2026*
