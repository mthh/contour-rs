#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! Computes isorings and contour polygons by applying
//! [marching squares](https://en.wikipedia.org/wiki/Marching_squares)
//! to a rectangular array of numeric values.
//!
//! Use the [`ContourBuilder`]) to compute for a given set of values and thresholds:
//! - isolines, as a Vec of [`Line`],
//! - contour polygons, as a Vec of [`Contour`],
//! - isobands, as a Vec of [`Band`].
//!
//! The [`contour_rings`] function is a convenience function to compute ring (isoline) coordinates
//! for a single threshold.
//!
//! While contour polygons ([`Contour`]) enclose all the values above a given threshold,
//! isobands ([`Band`]) are polygons that enclose all the values between two thresholds.
//!
//! The core of the algorithm is ported from [d3-contour](https://github.com/d3/d3-contour/).
//!
//! #### Example:
#![cfg_attr(feature = "geojson", doc = "```")]
#![cfg_attr(not(feature = "geojson"), doc = "```ignore")]
//! # use contour::ContourBuilder;
//! let c = ContourBuilder::new(10, 10, false); // x dim., y dim., smoothing
//! let res = c.contours(&vec![
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
//!     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
//! ], &[0.5]).unwrap(); // values, thresholds
//!
//! let output = serde_json::json!({
//!   "type": "Feature",
//!   "geometry": {
//!     "type": "MultiPolygon",
//!     "coordinates": [[[
//!       [6., 7.5], [6., 6.5], [6., 5.5], [6., 4.5],
//!       [6., 3.5], [5.5, 3.], [4.5, 3.], [3.5, 3.],
//!       [3., 3.5], [3., 4.5], [3., 5.5], [3., 6.5],
//!       [3., 7.5], [3.5, 8.], [4.5, 8.], [5.5, 8.],
//!       [6., 7.5]
//!     ]]],
//!   },
//!   "properties": {"threshold": 0.5},
//! });
//!
//! assert_eq!(res[0].to_geojson(), std::convert::TryFrom::try_from(output).unwrap());
//! ```
//!
//! [`contour_rings`]: fn.contour_rings.html
//! [`ContourBuilder`]: struct.ContourBuilder.html

mod area;
mod band;
mod contour;
mod contourbuilder;
mod error;
mod isoringbuilder;
mod line;

#[cfg(feature = "f32")]
pub type Float = f32;
#[cfg(not(feature = "f32"))]
pub type Float = f64;
#[cfg(feature = "f32")]
pub type Pt = geo_types::Coord<f32>;
#[cfg(not(feature = "f32"))]
pub type Pt = geo_types::Coord;

pub type Ring = Vec<Pt>;

pub use crate::band::Band;
pub use crate::contour::Contour;
pub use crate::contourbuilder::ContourBuilder;
pub use crate::isoringbuilder::contour_rings;
pub use crate::line::Line;
pub use crate::error::{Error, ErrorKind, Result};

#[cfg(test)]
mod tests {
    use crate::{ContourBuilder, Float};
    use geo_types::{line_string, polygon, MultiLineString, MultiPolygon};

    #[test]
    fn test_empty_polygons() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
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
        ], &[0.5]).unwrap();
        assert!(res[0].geometry().0.is_empty());
    }

    #[test]
    fn test_empty_isoline() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
            let res = c.lines(&[
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
        ], &[0.5]).unwrap();
        assert!(res[0].geometry().0.is_empty());
    }

    #[test]
    fn test_simple_polygon() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![polygon![
                (x: 6.,  y: 7.5),
                (x: 6.,  y: 6.5),
                (x: 6.,  y: 5.5),
                (x: 6.,  y: 4.5),
                (x: 6.,  y: 3.5),
                (x: 5.5, y:  3.),
                (x: 4.5, y:  3.),
                (x: 3.5, y:  3.),
                (x: 3.,  y: 3.5),
                (x: 3.,  y: 4.5),
                (x: 3.,  y: 5.5),
                (x: 3.,  y: 6.5),
                (x: 3.,  y: 7.5),
                (x: 3.5, y:  8.),
                (x: 4.5, y:  8.),
                (x: 5.5, y:  8.),
                (x: 6.,  y: 7.5)
            ]])
        );
    }

    #[test]
    fn test_simple_isoline() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
            let res = c.lines(&[
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiLineString::<Float>(vec![line_string![
                (x: 6.,  y: 7.5),
                (x: 6.,  y: 6.5),
                (x: 6.,  y: 5.5),
                (x: 6.,  y: 4.5),
                (x: 6.,  y: 3.5),
                (x: 5.5, y:  3.),
                (x: 4.5, y:  3.),
                (x: 3.5, y:  3.),
                (x: 3.,  y: 3.5),
                (x: 3.,  y: 4.5),
                (x: 3.,  y: 5.5),
                (x: 3.,  y: 6.5),
                (x: 3.,  y: 7.5),
                (x: 3.5, y:  8.),
                (x: 4.5, y:  8.),
                (x: 5.5, y:  8.),
                (x: 6.,  y: 7.5)
            ]])
        );
    }

    #[test]
    fn test_polygon_with_hole() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![polygon! {
                exterior: [
                    (x: 6., y: 7.5),
                    (x: 6., y: 6.5),
                    (x: 6., y: 5.5),
                    (x: 6., y: 4.5),
                    (x: 6., y: 3.5),
                    (x: 5.5,y:  3.),
                    (x: 4.5,y:  3.),
                    (x: 3.5,y:  3.),
                    (x: 3., y: 3.5),
                    (x: 3., y: 4.5),
                    (x: 3., y: 5.5),
                    (x: 3., y: 6.5),
                    (x: 3., y: 7.5),
                    (x: 3.5,y:  8.),
                    (x: 4.5,y:  8.),
                    (x: 5.5,y:  8.),
                    (x: 6., y: 7.5),
                ],
                interiors: [[
                    (x: 4.5,y:  7.),
                    (x: 4., y: 6.5),
                    (x: 4., y: 5.5),
                    (x: 4., y: 4.5),
                    (x: 4.5,y:  4.),
                    (x: 5., y: 4.5),
                    (x: 5., y: 5.5),
                    (x: 5., y: 6.5),
                    (x: 4.5,y:  7.),
                ]]
            }])
        );
    }

    #[test]
    fn test_multipolygon() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![
                polygon![
                    (x: 5., y: 7.5),
                    (x: 5., y: 6.5),
                    (x: 5., y: 5.5),
                    (x: 5., y: 4.5),
                    (x: 5., y: 3.5),
                    (x: 4.5,y:  3.),
                    (x: 3.5,y:  3.),
                    (x: 3., y: 3.5),
                    (x: 3., y: 4.5),
                    (x: 3., y: 5.5),
                    (x: 3., y: 6.5),
                    (x: 3., y: 7.5),
                    (x: 3.5,y:  8.),
                    (x: 4.5,y:  8.),
                    (x: 5., y: 7.5),
                ],
                polygon![
                    (x: 7., y: 7.5),
                    (x: 7., y: 6.5),
                    (x: 7., y: 5.5),
                    (x: 7., y: 4.5),
                    (x: 7., y: 3.5),
                    (x: 6.5,y:  3.),
                    (x: 6., y: 3.5),
                    (x: 6., y: 4.5),
                    (x: 6., y: 5.5),
                    (x: 6., y: 6.5),
                    (x: 6., y: 7.5),
                    (x: 6.5,y:  8.),
                    (x: 7., y: 7.5),
                ],
            ])
        );
    }

    #[test]
    fn test_multipolygon_with_hole() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![
                polygon! {
                     exterior: [
                             (x: 4., y: 5.5),
                             (x: 4., y: 4.5),
                             (x: 4., y: 3.5),
                             (x: 3.5,y:  3.),
                             (x: 2.5,y:  3.),
                             (x: 1.5,y:  3.),
                             (x: 1., y: 3.5),
                             (x: 1., y: 4.5),
                             (x: 1., y: 5.5),
                             (x: 1.5,y:  6.),
                             (x: 2.5,y:  6.),
                             (x: 3.5,y:  6.),
                             (x: 4., y: 5.5),
                     ],
                     interiors: [[
                         (x: 2.5, y:  5.),
                         (x: 2.,  y: 4.5),
                         (x: 2.5, y:  4.),
                         (x: 3.,  y: 4.5),
                         (x: 2.5, y:  5.),
                     ]]
                },
                polygon! {
                    exterior: [
                        (x: 8., y: 5.5),
                        (x: 8., y: 4.5),
                        (x: 8., y: 3.5),
                        (x: 7.5,y:  3.),
                        (x: 6.5,y:  3.),
                        (x: 5.5,y:  3.),
                        (x: 5., y: 3.5),
                        (x: 5., y: 4.5),
                        (x: 5., y: 5.5),
                        (x: 5.5,y:  6.),
                        (x: 6.5,y:  6.),
                        (x: 7.5,y:  6.),
                        (x: 8., y: 5.5),
                    ],
                    interiors: [[
                        (x: 6.5, y: 5.),
                        (x: 6.,  y:4.5),
                        (x: 6.5, y: 4.),
                        (x: 7.,  y:4.5),
                        (x: 6.5, y: 5.),
                    ]],
                },
            ])
        );
    }

    #[test]
    fn test_simple_polygon_no_smoothing() {
        let c = ContourBuilder::new(10, 10, false);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![polygon![
                            (x: 6.,  y: 7.5),
                            (x: 6.,  y: 6.5),
                            (x: 6.,  y: 5.5),
                            (x: 6.,  y: 4.5),
                            (x: 6.,  y: 3.5),
                            (x: 5.5, y:  3.),
                            (x: 4.5, y:  3.),
                            (x: 3.5, y:  3.),
                            (x: 3.,  y: 3.5),
                            (x: 3.,  y: 4.5),
                            (x: 3.,  y: 5.5),
                            (x: 3.,  y: 6.5),
                            (x: 3.,  y: 7.5),
                            (x: 3.5, y:  8.),
                            (x: 4.5, y:  8.),
                            (x: 5.5, y:  8.),
                            (x: 6.,  y: 7.5),

            ]])
        );
    }

    #[test]
    fn test_multiple_thresholds() {
        let c = ContourBuilder::new(10, 10, true);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5, 1.5]).unwrap();
        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![polygon![
            (x: 7., y: 8.5),
            (x: 7., y: 7.5),
            (x: 7., y: 6.5),
            (x: 7., y: 5.5),
            (x: 7., y: 4.5),
            (x: 7., y: 3.5),
            (x: 6.5,y:  3.),
            (x: 5.5,y:  3.),
            (x: 4.5,y:  3.),
            (x: 3.5,y:  3.),
            (x: 3., y: 3.5),
            (x: 3., y: 4.5),
            (x: 3., y: 5.5),
            (x: 3., y: 6.5),
            (x: 3., y: 7.5),
            (x: 3., y: 8.5),
            (x: 3.5,y:  9.),
            (x: 4.5,y:  9.),
            (x: 5.5,y:  9.),
            (x: 6.5,y:  9.),
            (x: 7., y: 8.5)
                ]])
        );
        assert_eq!(
            res[1].geometry(),
            &MultiPolygon::<Float>(vec![polygon![
                (x: 6.,  y: 6.5),
                (x: 6.,  y: 5.5),
                (x: 5.5, y:  5.),
                (x: 4.5, y:  5.),
                (x: 4.,  y: 5.5),
                (x: 4.5, y:  6.),
                (x: 5.,  y: 6.5),
                (x: 5.5, y:  7.),
                (x: 6.,  y: 6.5)
            ]])
        );
    }

    #[test]
    fn test_multipolygon_with_x_y_steps() {
        let c = ContourBuilder::new(10, 10, true)
            .x_step(2.0)
            .y_step(2.0)
            .x_origin(100.0)
            .y_origin(200.0);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();

        assert_eq!(
            res[0].geometry(),
            &MultiPolygon::<Float>(vec![
                polygon![
                    (x: 110.0, y: 215.0),
                    (x: 110.0, y: 213.0),
                    (x: 110.0, y: 211.0),
                    (x: 110.0, y: 209.0),
                    (x: 110.0, y: 207.0),
                    (x: 109.0, y: 206.0),
                    (x: 107.0, y: 206.0),
                    (x: 106.0, y: 207.0),
                    (x: 106.0, y: 209.0),
                    (x: 106.0, y: 211.0),
                    (x: 106.0, y: 213.0),
                    (x: 106.0, y: 215.0),
                    (x: 107.0, y: 216.0),
                    (x: 109.0, y: 216.0),
                    (x: 110.0, y: 215.0)
                ],
                polygon![
                    (x: 114.0, y: 215.0),
                    (x: 114.0, y: 213.0),
                    (x: 114.0, y: 211.0),
                    (x: 114.0, y: 209.0),
                    (x: 114.0, y: 207.0),
                    (x: 113.0, y: 206.0),
                    (x: 112.0, y: 207.0),
                    (x: 112.0, y: 209.0),
                    (x: 112.0, y: 211.0),
                    (x: 112.0, y: 213.0),
                    (x: 112.0, y: 215.0),
                    (x: 113.0, y: 216.0),
                    (x: 114.0, y: 215.0)
                ]
            ])
        );
    }

    #[cfg(feature = "geojson")]
    #[test]
    fn test_simple_polygon_no_smoothing_geojson() {
        let c = ContourBuilder::new(10, 10, false);
        #[rustfmt::skip]
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
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
        ], &[0.5]).unwrap();
        match res[0].to_geojson().geometry.unwrap().value {
            geojson::Value::MultiPolygon(p) => {
                assert_eq!(
                    p,
                    vec![vec![vec![
                        vec![6., 7.5],
                        vec![6., 6.5],
                        vec![6., 5.5],
                        vec![6., 4.5],
                        vec![6., 3.5],
                        vec![5.5, 3.],
                        vec![4.5, 3.],
                        vec![3.5, 3.],
                        vec![3., 3.5],
                        vec![3., 4.5],
                        vec![3., 5.5],
                        vec![3., 6.5],
                        vec![3., 7.5],
                        vec![3.5, 8.],
                        vec![4.5, 8.],
                        vec![5.5, 8.],
                        vec![6., 7.5],
                    ]]]
                );
            }
            _ => panic!(""),
        };
    }
}
