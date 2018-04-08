#![feature(test)]
extern crate contour;
extern crate test;

use test::{Bencher, black_box};
use contour::ContourBuilder;
use contour::IsoRingBuilder;

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
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
];

#[bench]
fn bench_build_geojson_contour(b: &mut Bencher) {
    let c = ContourBuilder::new(vec![0.5], 10, 10, true);
    b.iter(||
        black_box(
            c.contours(&VALUES)
        )
    );
}

#[bench]
fn bench_build_geojson_contour_no_smoothing(b: &mut Bencher) {
    let c = ContourBuilder::new(vec![0.5], 10, 10, false);
    b.iter(||
        black_box(
            c.contours(&VALUES)
        )
    );
}

#[bench]
fn bench_isoring(b: &mut Bencher) {
    let mut i = IsoRingBuilder::new(10, 10);
    b.iter(||
        black_box(
            i.compute(&VALUES, 0.5)
        )
    );
}
