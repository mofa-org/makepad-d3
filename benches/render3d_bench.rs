//! Benchmarks for 3D rendering operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use makepad_d3::render3d::{
    Surface3D, Scatter3D, ScatterPoint3D, Bar3D,
    Camera3D, Colormap, Mat4, Transform3D,
    types::Vec3,
};

fn surface_mesh_build_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("surface_mesh_build");

    for resolution in [25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(resolution),
            resolution,
            |b, &res| {
                b.iter(|| {
                    let mut surface = Surface3D::new();
                    surface.set_function(res, (-2.0, 2.0), (-2.0, 2.0), |x, z| {
                        (x * x + z * z).sqrt().sin()
                    });
                    surface.rebuild_mesh();
                    black_box(&surface);
                })
            },
        );
    }

    group.finish();
}

fn surface_face_sorting_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("surface_face_sorting");

    for resolution in [25, 50, 100].iter() {
        let mut surface = Surface3D::new();
        surface.set_function(*resolution, (-2.0, 2.0), (-2.0, 2.0), |x, z| {
            (x * x + z * z).sqrt().sin()
        });
        surface.rebuild_mesh();

        group.bench_with_input(
            BenchmarkId::from_parameter(resolution),
            resolution,
            |b, _| {
                b.iter(|| {
                    let faces = surface.get_sorted_faces(800.0, 600.0);
                    black_box(faces.len());
                })
            },
        );
    }

    group.finish();
}

fn camera_projection_benchmark(c: &mut Criterion) {
    let camera = Camera3D::new()
        .with_distance(5.0)
        .with_yaw(0.3)
        .with_pitch(0.5);

    c.bench_function("camera_view_projection", |b| {
        b.iter(|| {
            let transform = camera.view_projection(1.33);
            black_box(transform);
        })
    });

    c.bench_function("camera_view_projection_matrix", |b| {
        b.iter(|| {
            let matrix = camera.view_projection_matrix(1.33);
            black_box(matrix);
        })
    });

    c.bench_function("camera_orbital_transform", |b| {
        b.iter(|| {
            let transform = camera.orbital_transform();
            black_box(transform);
        })
    });
}

fn colormap_sampling_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("colormap_sample");

    for colormap in [
        Colormap::Viridis,
        Colormap::Plasma,
        Colormap::Turbo,
        Colormap::CoolWarm,
    ].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", colormap)),
            colormap,
            |b, &cm| {
                b.iter(|| {
                    for i in 0..1000 {
                        let t = i as f32 / 1000.0;
                        black_box(cm.sample(t));
                    }
                })
            },
        );
    }

    group.finish();
}

fn scatter_projection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("scatter_projection");

    for point_count in [100, 500, 1000].iter() {
        let mut scatter = Scatter3D::new();
        let points: Vec<ScatterPoint3D> = (0..*point_count)
            .map(|i| {
                let t = i as f64 / *point_count as f64;
                ScatterPoint3D::new(
                    (t * 6.28).cos(),
                    t * 2.0 - 1.0,
                    (t * 6.28).sin(),
                ).with_value(t)
            })
            .collect();
        scatter.set_points(points);

        group.bench_with_input(
            BenchmarkId::from_parameter(point_count),
            point_count,
            |b, _| {
                b.iter(|| {
                    let projected = scatter.get_projected_points(800.0, 600.0);
                    black_box(projected.len());
                })
            },
        );
    }

    group.finish();
}

fn bar3d_face_generation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bar3d_faces");

    for grid_size in [5, 10, 20].iter() {
        let mut bar_chart = Bar3D::new();
        let data: Vec<Vec<f64>> = (0..*grid_size)
            .map(|i| {
                (0..*grid_size)
                    .map(|j| (i + j) as f64 / (*grid_size * 2) as f64)
                    .collect()
            })
            .collect();
        bar_chart.set_data(data);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}", grid_size, grid_size)),
            grid_size,
            |b, _| {
                b.iter(|| {
                    let faces = bar_chart.get_sorted_faces(800.0, 600.0);
                    black_box(faces.len());
                })
            },
        );
    }

    group.finish();
}

fn matrix_operations_benchmark(c: &mut Criterion) {
    let m1 = Mat4::from_rotation_y(0.5);
    let m2 = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));

    c.bench_function("mat4_multiply", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(m1 * m2);
            }
        })
    });

    let v = Vec3::new(1.0, 2.0, 3.0);
    c.bench_function("vec3_normalize_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(v.normalize());
            }
        })
    });

    c.bench_function("transform3d_compose", |b| {
        let t1 = Transform3D::from_mat4(&m1);
        let t2 = Transform3D::from_mat4(&m2);
        b.iter(|| {
            for _ in 0..1000 {
                black_box(t1.to_columns());
                black_box(t2.to_columns());
            }
        })
    });
}

criterion_group!(
    benches,
    surface_mesh_build_benchmark,
    surface_face_sorting_benchmark,
    camera_projection_benchmark,
    colormap_sampling_benchmark,
    scatter_projection_benchmark,
    bar3d_face_generation_benchmark,
    matrix_operations_benchmark,
);
criterion_main!(benches);
