extern crate contour;

use contour::{contour_rings, ContourBuilder};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[rustfmt::skip]
static VALUES: [f64; 110] = [
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
    0., 0., 0., 1., 0., 1., 0., 0., 0., 0.,
    0., 0., 0., 1., 0., 1., 0., 0., 0., 0.,
    0., 0., 0., 1., 0., 1., 0., 0., 0., 0.,
    0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
];

#[rustfmt::skip]
static VALUES2: [f64; 238] = [
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 3., 3., 0., 0.,
    0., 0., 0., 1., 1., 1., 1., 0., 0., 0., 3., 3., 0., 0.,
    0., 0., 0., 1., 1., 1., 1., 1., 0., 0., 3., 3., 0., 0.,
    0., 0., 0., 1., 2., 2., 1., 1., 0., 0., 3., 3., 0., 0.,
    0., 0., 0., 1., 2., 2., 1., 1., 0., 0., 3., 3., 0., 0.,
    0., 0., 0., 1., 2., 2., 1., 1., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 1., 1., 1., 1., 1., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 1., 1., 1., 1., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 2., 2., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 2., 2., 2., 2., 0., 0.,
    0., 0., 1., 1., 0., 0., 0., 0., 2., 2., 2., 2., 0., 0.,
    0., 1., 1., 1., 0., 0., 0., 0., 0., 2., 2., 0., 0., 0.,
    0., 1., 1., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 1., 1., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
];

criterion_group!(
    benches,
    bench_build_contours_multiple_thresholds,
    bench_build_contours_multiple_thresholds_and_x_y_steps_and_origins,
    bench_build_geojson_contour,
    bench_build_geojson_contour_no_smoothing,
    bench_build_isoring,
    bench_build_isoring_values2,
    bench_contourbuilder_isobands_volcano_without_xy_step_xy_origin,
    bench_contourbuilder_isobands_pot_pop_fr_without_xy_step_xy_origin
);
criterion_main!(benches);

fn bench_build_contours_multiple_thresholds(c: &mut Criterion) {
    let cb = ContourBuilder::new(14, 17, true);
    c.bench_function("build_contours_multiple_thresholds", |b| {
        b.iter(|| black_box(cb.contours(&VALUES2, &[0.5, 1.5, 2.5])))
    });
}

fn bench_build_contours_multiple_thresholds_and_x_y_steps_and_origins(c: &mut Criterion) {
    let cb = ContourBuilder::new(14, 17, true)
        .x_step(0.5)
        .y_step(0.5)
        .x_origin(0.25)
        .y_origin(0.25);
    c.bench_function(
        "build_contours_multiple_thresholds_and_x_y_steps_and_origins",
        |b| b.iter(|| black_box(cb.contours(&VALUES2, &[0.5, 1.5, 2.5]))),
    );
}

fn bench_build_geojson_contour(c: &mut Criterion) {
    let cb = ContourBuilder::new(10, 11, true);
    c.bench_function("build_geojson_contour", |b| {
        b.iter(|| black_box(cb.contours(&VALUES, &[0.5])))
    });
}

fn bench_build_geojson_contour_no_smoothing(c: &mut Criterion) {
    let cb = ContourBuilder::new(10, 11, false);
    c.bench_function("build_geojson_contour_no_smoothing", |b| {
        b.iter(|| black_box(cb.contours(&VALUES, &[0.5])))
    });
}

fn bench_build_isoring(c: &mut Criterion) {
    c.bench_function("build_isoring", |b| {
        b.iter(|| black_box(contour_rings(&VALUES, 0.5, 10, 11)))
    });
}

fn bench_build_isoring_values2(c: &mut Criterion) {
    c.bench_function("build_isoring_values2", |b| {
        b.iter(|| black_box(contour_rings(&VALUES2, 1.5, 14, 17)))
    });
}

fn bench_contourbuilder_isobands_volcano_without_xy_step_xy_origin(c: &mut Criterion) {
    let data_str = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    c.bench_function(
        "contourbuilder_isobands_volcano_without_xy_step_xy_origin",
        |b| {
            b.iter(|| {
                black_box(
                    ContourBuilder::new(w, h, true)
                        .isobands(
                            &matrix,
                            &[
                                90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140.,
                                145., 150., 155., 160., 165., 170., 175., 180., 185., 190., 195.,
                                200.,
                            ],
                        )
                        .unwrap(),
                )
            })
        },
    );
}

fn bench_contourbuilder_isobands_pot_pop_fr_without_xy_step_xy_origin(c: &mut Criterion) {
    let data_str = include_str!("../tests/fixtures/pot_pop_fr.json");
    let raw_data: serde_json::Value = serde_json::from_str(data_str).unwrap();
    let matrix: Vec<f64> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    c.bench_function(
        "contourbuilder_isobands_pot_pop_fr_without_xy_step_xy_origin",
        |b| {
            b.iter(|| {
                black_box(
                    ContourBuilder::new(w, h, true)
                        .isobands(
                            &matrix,
                            &[
                                0.001, 105483.25, 527416.25, 1054832.5, 2109665., 3164497.5,
                                4219330., 5274162.5, 6328995., 7383827.5, 8438660., 9704459.,
                                10548326.,
                            ],
                        )
                        .unwrap(),
                )
            })
        },
    );
}
