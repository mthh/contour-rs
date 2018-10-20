//! Computes isorings and contour polygons by applying
//! [marching squares](https://en.wikipedia.org/wiki/Marching_squares)
//! to a rectangular array of numeric values.
//! Outputs ring coordinates or polygons contours as a Vec of GeoJSON Feature.
//! This is a port of [d3-contour](https://github.com/d3/d3-contour/).

#[macro_use] extern crate lazy_static;
extern crate geojson;
extern crate serde_json;
extern crate rustc_hash;

mod area;
mod contour;

pub use contour::{ContourBuilder, IsoRingBuilder};

#[cfg(test)]
mod tests {
    use ::ContourBuilder;
    use geojson;

    #[test]
    fn test_empty_polygons() {
        let c = ContourBuilder::new(10, 10, true);
        let res = c.contours(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert!(p.is_empty());
            }
            _ => panic!(""),
        };
    }

    #[test]
    fn test_simple_polygon() {
        let c = ContourBuilder::new(10, 10, true);
        let res = c.contours(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![vec![vec![
                        vec![6., 7.5], vec![6., 6.5], vec![6., 5.5], vec![6., 4.5],
                        vec![6., 3.5], vec![5.5, 3.], vec![4.5, 3.], vec![3.5, 3.],
                        vec![3., 3.5], vec![3., 4.5], vec![3., 5.5], vec![3., 6.5],
                        vec![3., 7.5], vec![3.5, 8.], vec![4.5, 8.], vec![5.5, 8.],
                        vec![6., 7.5]]]]);
            }
            _ => panic!(""),
        };
    }

    #[test]
    fn test_polygon_with_hole() {
        let c = ContourBuilder::new(10, 10, true);
        let res = c.contours(&[
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
        ], &[0.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![vec![
                        vec![vec![6., 7.5], vec![6., 6.5], vec![6., 5.5], vec![6., 4.5], vec![6., 3.5], vec![5.5, 3.], vec![4.5, 3.],
                               vec![3.5, 3.], vec![3., 3.5], vec![3., 4.5], vec![3., 5.5], vec![3., 6.5], vec![3., 7.5], vec![3.5, 8.],
                               vec![4.5, 8.], vec![5.5, 8.], vec![6., 7.5]],
                      vec![vec![4.5, 7.], vec![4., 6.5], vec![4., 5.5], vec![4., 4.5], vec![4.5, 4.], vec![5., 4.5], vec![5., 5.5],
                       vec![5., 6.5],vec![4.5, 7.]]
                            ]]);
            }
            _ => panic!(""),
        };
    }

    #[test]
    fn test_multipolygon() {
        let c = ContourBuilder::new(10, 10, true);
        let res = c.contours(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 0., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 0., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 0., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 0., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 0., 1., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![
                        vec![
                          vec![vec![5., 7.5], vec![5., 6.5], vec![5., 5.5], vec![5., 4.5], vec![5., 3.5], vec![4.5, 3.], vec![3.5, 3.],
                           vec![3., 3.5], vec![3., 4.5], vec![3., 5.5], vec![3., 6.5], vec![3., 7.5], vec![3.5, 8.], vec![4.5, 8.],
                           vec![5., 7.5]]],
                        vec![
                          vec![vec![7., 7.5], vec![7., 6.5], vec![7., 5.5], vec![7., 4.5], vec![7., 3.5], vec![6.5, 3.], vec![6., 3.5],
                           vec![6., 4.5], vec![6., 5.5], vec![6., 6.5], vec![6., 7.5], vec![6.5, 8.], vec![7., 7.5]]
                        ]
                ]);
            }
            _ => panic!(""),
        };
    }


    #[test]
    fn test_multipolygon_with_hole() {
        let c = ContourBuilder::new(10, 10, true);
        let res = c.contours(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 1., 1., 1., 0., 1., 1., 1., 0., 0.,
            0., 1., 0., 1., 0., 1., 0., 1., 0., 0.,
            0., 1., 1., 1., 0., 1., 1., 1., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![
                        vec![
                          vec![vec![4., 5.5], vec![4., 4.5], vec![4., 3.5], vec![3.5, 3.], vec![2.5, 3.], vec![1.5, 3.], vec![1., 3.5],
                           vec![1., 4.5], vec![1., 5.5], vec![1.5, 6.], vec![2.5, 6.], vec![3.5, 6.], vec![4., 5.5]],
                          vec![vec![2.5, 5.], vec![2., 4.5], vec![2.5, 4.], vec![3., 4.5], vec![2.5, 5.]]
                        ],
                        vec![
                          vec![vec![8., 5.5], vec![8., 4.5], vec![8., 3.5], vec![7.5, 3.], vec![6.5, 3.], vec![5.5, 3.], vec![5., 3.5],
                           vec![5., 4.5], vec![5., 5.5], vec![5.5, 6.], vec![6.5, 6.], vec![7.5, 6.], vec![8., 5.5]],
                          vec![vec![6.5, 5.], vec![6., 4.5], vec![6.5, 4.], vec![7., 4.5], vec![6.5, 5.]]
                        ]
                ]);
            }
            _ => panic!(""),
        };
    }

    #[test]
    fn test_simple_polygon_no_smoothing() {
        let c = ContourBuilder::new(10, 10, false);
        let res = c.contours(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
            0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
            0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
            0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
            0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![
                        vec![
                          vec![vec![6., 7.5], vec![6., 6.5], vec![6., 5.5], vec![6., 4.5], vec![6., 3.5], vec![5.5, 3.], vec![4.5, 3.],
                           vec![3.5, 3.], vec![3., 3.5], vec![3., 4.5], vec![3., 5.5], vec![3., 6.5], vec![3., 7.5], vec![3.5, 8.],
                           vec![4.5, 8.], vec![5.5, 8.], vec![6., 7.5]]
                        ]
                ]);
            }
            _ => panic!(""),
        };
    }

    #[test]
    fn test_multiple_thresholds() {
        let c = ContourBuilder::new(10, 10, true);
        let res = c.contours(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 1., 0., 0., 0.,
            0., 0., 0., 1., 2., 2., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 2., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 1., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 1., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5, 1.5]);
        match res[0].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![vec![vec![
                        vec![7.,8.5],vec![7.,7.5],vec![7.,6.5],vec![7.,5.5],vec![7.,4.5],
                        vec![7.,3.5],vec![6.5,3.],vec![5.5,3.],vec![4.5,3.],vec![3.5,3.],
                        vec![3.,3.5],vec![3.,4.5],vec![3.,5.5],vec![3.,6.5],vec![3.,7.5],
                        vec![3.,8.5],vec![3.5,9.],vec![4.5,9.],vec![5.5,9.],vec![6.5,9.],
                        vec![7.,8.5]]
                    ]
                ]);
            }
            _ => panic!(""),
        };
        match res[1].clone().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![vec![vec![
                        vec![6.,6.5],vec![6.,5.5],vec![5.5,5.],vec![4.5,5.],
                        vec![4.,5.5],vec![4.5,6.],vec![5.,6.5],vec![5.5,7.],
                        vec![6.,6.5]
                    ]]
                ]);
            }
            _ => panic!(""),
        };
    }
}
