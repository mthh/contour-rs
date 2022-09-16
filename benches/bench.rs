#![feature(test)]
extern crate contour;
extern crate test;

use contour::contour_rings;
use contour::ContourBuilder;
use test::{black_box, Bencher};

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

#[bench]
fn bench_build_geojson_contours_multiple_thresholds(b: &mut Bencher) {
    let c = ContourBuilder::new(14, 17, true);
    b.iter(|| black_box(c.contours(&VALUES2, &[0.5, 1.5, 2.5])));
}

#[bench]
fn bench_build_geojson_contour(b: &mut Bencher) {
    let c = ContourBuilder::new(10, 11, true);
    b.iter(|| black_box(c.contours(&VALUES, &[0.5])));
}

#[bench]
fn bench_build_geojson_contour_no_smoothing(b: &mut Bencher) {
    let c = ContourBuilder::new(10, 11, false);
    b.iter(|| black_box(c.contours(&VALUES, &[0.5])));
}

#[bench]
fn bench_build_isoring(b: &mut Bencher) {
    b.iter(|| black_box(contour_rings(&VALUES, 0.5, 10, 11)));
}

#[bench]
fn bench_build_isoring_values2(b: &mut Bencher) {
    b.iter(|| black_box(contour_rings(&VALUES2, 1.5, 14, 17)));
}
