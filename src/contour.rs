use crate::area::{area, contains};
use crate::error::{new_error, ErrorKind, Result};
use geo_types::{LineString, MultiLineString, MultiPolygon, Polygon};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use slab::Slab;

pub type Pt = geo_types::Coord;
pub type Ring = Vec<Pt>;

lazy_static! {
    #[rustfmt::skip]
    static ref CASES: Vec<Vec<Vec<Vec<f64>>>> = vec![
        vec![],
        vec![vec![vec![1.0, 1.5], vec![0.5, 1.0]]],
        vec![vec![vec![1.5, 1.0], vec![1.0, 1.5]]],
        vec![vec![vec![1.5, 1.0], vec![0.5, 1.0]]],
        vec![vec![vec![1.0, 0.5], vec![1.5, 1.0]]],
        vec![
            vec![vec![1.0, 1.5], vec![0.5, 1.0]],
            vec![vec![1.0, 0.5], vec![1.5, 1.0]]
        ],
        vec![vec![vec![1.0, 0.5], vec![1.0, 1.5]]],
        vec![vec![vec![1.0, 0.5], vec![0.5, 1.0]]],
        vec![vec![vec![0.5, 1.0], vec![1.0, 0.5]]],
        vec![vec![vec![1.0, 1.5], vec![1.0, 0.5]]],
        vec![
            vec![vec![0.5, 1.0], vec![1.0, 0.5]],
            vec![vec![1.5, 1.0], vec![1.0, 1.5]]
        ],
        vec![vec![vec![1.5, 1.0], vec![1.0, 0.5]]],
        vec![vec![vec![0.5, 1.0], vec![1.5, 1.0]]],
        vec![vec![vec![1.0, 1.5], vec![1.5, 1.0]]],
        vec![vec![vec![0.5, 1.0], vec![1.0, 1.5]]],
        vec![]
    ];
}

#[derive(Clone, Debug)]
struct Fragment {
    start: usize,
    end: usize,
    ring: Ring,
}

/// Contours generator, using builder pattern, to
/// be used on a rectangular `Slice` of values to
/// get a `Vec` of [`Contour`] (uses [`contour_rings`] internally).
///
/// [`contour_rings`]: fn.contour_rings.html
pub struct ContourBuilder {
    /// The number of columns in the grid
    dx: u32,
    /// The number of rows in the grid
    dy: u32,
    /// Whether to smooth the contours
    smooth: bool,
    /// The horizontal coordinate for the origin of the grid.
    x_origin: f64,
    /// The vertical coordinate for the origin of the grid.
    y_origin: f64,
    /// The horizontal step for the grid
    x_step: f64,
    /// The vertical step for the grid
    y_step: f64,
}

impl ContourBuilder {
    /// Constructs a new contours generator for a grid with `dx` * `dy` dimension.
    /// Set `smooth` to true to smooth the contour lines.
    ///
    /// By default, `x_origin` and `y_origin` are set to `0.0`, and `x_step` and `y_step` to `1.0`.
    ///
    /// # Arguments
    ///
    /// * `dx` - The number of columns in the grid.
    /// * `dy` - The number of rows in the grid.
    /// * `smooth` - Whether or not the generated rings will be smoothed using linear interpolation.
    pub fn new(dx: u32, dy: u32, smooth: bool) -> Self {
        ContourBuilder {
            dx,
            dy,
            smooth,
            x_origin: 0f64,
            y_origin: 0f64,
            x_step: 1f64,
            y_step: 1f64,
        }
    }

    /// Sets the x origin of the grid.
    pub fn x_origin(mut self, x_origin: impl Into<f64>) -> Self {
        self.x_origin = x_origin.into();
        self
    }

    /// Sets the y origin of the grid.
    pub fn y_origin(mut self, y_origin: impl Into<f64>) -> Self {
        self.y_origin = y_origin.into();
        self
    }

    /// Sets the x step of the grid.
    pub fn x_step(mut self, x_step: impl Into<f64>) -> Self {
        self.x_step = x_step.into();
        self
    }

    /// Sets the y step of the grid.
    pub fn y_step(mut self, y_step: impl Into<f64>) -> Self {
        self.y_step = y_step.into();
        self
    }

    fn smoooth_linear(&self, ring: &mut Ring, values: &[f64], value: f64) {
        let dx = self.dx;
        let dy = self.dy;
        let len_values = values.len();

        ring.iter_mut()
            .map(|point| {
                let x = point.x;
                let y = point.y;
                let xt = x.trunc() as u32;
                let yt = y.trunc() as u32;
                let mut v0;
                let ix = (yt * dx + xt) as usize;
                if ix < len_values {
                    let v1 = values[ix];
                    if x > 0.0 && x < (dx as f64) && (xt as f64 - x).abs() < std::f64::EPSILON {
                        v0 = values[(yt * dx + xt - 1) as usize];
                        point.x = x + (value - v0) / (v1 - v0) - 0.5;
                    }
                    if y > 0.0 && y < (dy as f64) && (yt as f64 - y).abs() < std::f64::EPSILON {
                        v0 = values[((yt - 1) * dx + xt) as usize];
                        point.y = y + (value - v0) / (v1 - v0) - 0.5;
                    }
                }
            })
            .for_each(drop);
    }

    /// Computes isolines according the given input `values` and the given `thresholds`.
    /// Returns a `Vec` of [`Line`] (that can easily be transformed
    /// to GeoJSON Features of MultiLineString).
    /// The threshold value of each Feature is stored in its `value` property.
    ///
    /// # Arguments
    ///
    /// * `values` - The slice of values to be used.
    /// * `thresholds` - The slice of thresholds values to be used.
    pub fn lines(&self, values: &[f64], thresholds: &[f64]) -> Result<Vec<Line>> {
        if values.len() as u32 != self.dx * self.dy {
            return Err(new_error(ErrorKind::BadDimension));
        }
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        thresholds
            .iter()
            .map(|threshold| self.line(values, *threshold, &mut isoring))
            .collect()
    }

    fn line(&self, values: &[f64], threshold: f64, isoring: &mut IsoRingBuilder) -> Result<Line> {
        let mut result = isoring.compute(values, threshold)?;
        let mut linestrings = Vec::new();

        result
            .drain(..)
            .map(|mut ring| {
                // Smooth the ring if needed
                if self.smooth {
                    self.smoooth_linear(&mut ring, values, threshold);
                }
                // Compute the polygon coordinates according to the grid properties if needed
                if (self.x_origin, self.y_origin) != (0f64, 0f64)
                    || (self.x_step, self.y_step) != (1f64, 1f64)
                {
                    ring.iter_mut()
                        .map(|point| {
                            point.x = point.x * self.x_step + self.x_origin;
                            point.y = point.y * self.y_step + self.y_origin;
                        })
                        .for_each(drop);
                }
                linestrings.push(LineString(ring));
            }).for_each(drop);
        Ok(Line {
            geometry: MultiLineString(linestrings),
            threshold,
        })
    }
    /// Computes contours according the given input `values` and the given `thresholds`.
    /// Returns a `Vec` of [`Contour`] (that can easily be transformed
    /// to GeoJSON Features of MultiPolygon).
    /// The threshold value of each Feature is stored in its `value` property.
    ///
    /// # Arguments
    ///
    /// * `values` - The slice of values to be used.
    /// * `thresholds` - The slice of thresholds values to be used.
    pub fn contours(&self, values: &[f64], thresholds: &[f64]) -> Result<Vec<Contour>> {
        if values.len() as u32 != self.dx * self.dy {
            return Err(new_error(ErrorKind::BadDimension));
        }
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        thresholds
            .iter()
            .map(|threshold| self.contour(values, *threshold, &mut isoring))
            .collect()
    }

    fn contour(
        &self,
        values: &[f64],
        threshold: f64,
        isoring: &mut IsoRingBuilder,
    ) -> Result<Contour> {
        let (mut polygons, mut holes) = (Vec::new(), Vec::new());
        let mut result = isoring.compute(values, threshold)?;

        result
            .drain(..)
            .map(|mut ring| {
                // Smooth the ring if needed
                if self.smooth {
                    self.smoooth_linear(&mut ring, values, threshold);
                }
                // Compute the polygon coordinates according to the grid properties if needed
                if (self.x_origin, self.y_origin) != (0f64, 0f64)
                    || (self.x_step, self.y_step) != (1f64, 1f64)
                {
                    ring.iter_mut()
                        .map(|point| {
                            point.x = point.x * self.x_step + self.x_origin;
                            point.y = point.y * self.y_step + self.y_origin;
                        })
                        .for_each(drop);
                }
                if area(&ring) > 0.0 {
                    polygons.push(Polygon::new(LineString::new(ring), vec![]))
                } else {
                    holes.push(LineString::new(ring));
                }
            })
            .for_each(drop);

        holes
            .drain(..)
            .map(|hole| {
                for polygon in &mut polygons {
                    if contains(&polygon.exterior().0, &hole.0) != -1 {
                        polygon.interiors_push(hole);
                        return;
                    }
                }
            })
            .for_each(drop);

        Ok(Contour {
            geometry: MultiPolygon(polygons),
            threshold,
        })
    }
}

/// A line has the geometry and threshold of a contour ring, built by [`ContourBuilder`].
#[derive(Debug, Clone)]
pub struct Line {
    geometry: MultiLineString,
    threshold: f64,
}

impl Line {
    /// Borrow the [`MultiPolygon`](geo_types::MultiPolygon) geometry of this contour.
    pub fn geometry(&self) -> &MultiLineString {
        &self.geometry
    }

    /// Get the owned polygons and threshold of this countour.
    pub fn into_inner(self) -> (MultiLineString, f64) {
        (self.geometry, self.threshold)
    }

    /// Get the threshold used to construct this contour.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    #[cfg(feature = "geojson")]
    /// Convert the line to a struct from the `geojson` crate.
    ///
    /// To get a string representation, call to_geojson().to_string().
    /// ```
    /// use contour::ContourBuilder;
    ///
    /// let builder = ContourBuilder::new(10, 10, false);
    /// # #[rustfmt::skip]
    /// let contours = builder.lines(&[
    /// // ...ellided for brevity
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
    /// ], &[0.5]).unwrap();
    ///
    /// let geojson_string = contours[0].to_geojson().to_string();
    ///
    /// assert_eq!(&geojson_string[0..27], r#"{"geometry":{"coordinates":"#);
    /// ```
    pub fn to_geojson(&self) -> geojson::Feature {
        let mut properties = geojson::JsonObject::with_capacity(1);
        properties.insert("threshold".to_string(), self.threshold.into());

        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(self.geometry())),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        }
    }

}

/// A contour has the geometry and threshold of a contour ring, built by [`ContourBuilder`].
#[derive(Debug, Clone)]
pub struct Contour {
    geometry: MultiPolygon,
    threshold: f64,
}

impl Contour {
    /// Borrow the [`MultiPolygon`](geo_types::MultiPolygon) geometry of this contour.
    pub fn geometry(&self) -> &MultiPolygon {
        &self.geometry
    }

    /// Get the owned polygons and threshold of this countour.
    pub fn into_inner(self) -> (MultiPolygon, f64) {
        (self.geometry, self.threshold)
    }

    /// Get the threshold used to construct this contour.
    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    #[cfg(feature = "geojson")]
    /// Convert the contour to a struct from the `geojson` crate.
    ///
    /// To get a string representation, call to_geojson().to_string().
    /// ```
    /// use contour::ContourBuilder;
    ///
    /// let builder = ContourBuilder::new(10, 10, false);
    /// # #[rustfmt::skip]
    /// let contours = builder.contours(&[
    /// // ...ellided for brevity
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
    /// ], &[0.5]).unwrap();
    ///
    /// let geojson_string = contours[0].to_geojson().to_string();
    ///
    /// assert_eq!(&geojson_string[0..27], r#"{"geometry":{"coordinates":"#);
    /// ```
    pub fn to_geojson(&self) -> geojson::Feature {
        let mut properties = geojson::JsonObject::with_capacity(1);
        properties.insert("threshold".to_string(), self.threshold.into());

        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(self.geometry())),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        }
    }
}

/// Computes isoring for the given `Slice` of `values` according to the `threshold` value
/// (the inside of the isoring is the surface where input `values` are greater than or equal
/// to the given threshold value).
///
/// # Arguments
///
/// * `values` - The slice of values to be used.
/// * `threshold` - The threshold value.
/// * `dx` - The number of columns in the grid.
/// * `dy` - The number of rows in the grid.
pub fn contour_rings(values: &[f64], threshold: f64, dx: u32, dy: u32) -> Result<Vec<Ring>> {
    let mut isoring = IsoRingBuilder::new(dx, dy);
    isoring.compute(values, threshold)
}

/// Isoring generator to compute marching squares with isolines stitched into rings.
struct IsoRingBuilder {
    fragment_by_start: FxHashMap<usize, usize>,
    fragment_by_end: FxHashMap<usize, usize>,
    f: Slab<Fragment>,
    dx: u32,
    dy: u32,
    is_empty: bool,
}

impl IsoRingBuilder {
    /// Constructs a new IsoRing generator for a grid with `dx` * `dy` dimension.
    /// # Arguments
    ///
    /// * `dx` - The number of columns in the grid.
    /// * `dy` - The number of rows in the grid.
    pub fn new(dx: u32, dy: u32) -> Self {
        IsoRingBuilder {
            fragment_by_start: FxHashMap::default(),
            fragment_by_end: FxHashMap::default(),
            f: Slab::new(),
            dx,
            dy,
            is_empty: true,
        }
    }

    /// Computes isoring for the given slice of `values` according to the `threshold` value
    /// (the inside of the isoring is the surface where input `values` are greater than or equal
    /// to the given threshold value).
    ///
    /// # Arguments
    ///
    /// * `values` - The slice of values to be used.
    /// * `threshold` - The threshold value to use.
    pub fn compute(&mut self, values: &[f64], threshold: f64) -> Result<Vec<Ring>> {
        macro_rules! case_stitch {
            ($ix:expr, $x:ident, $y:ident, $result:expr) => {
                CASES[$ix]
                    .iter()
                    .map(|ring| self.stitch(&ring, $x, $y, $result))
                    .collect::<Result<Vec<()>>>()?;
            };
        }

        if !self.is_empty {
            self.clear();
        }
        let mut result = Vec::new();
        let dx = self.dx as i32;
        let dy = self.dy as i32;
        let mut x = -1;
        let mut y = -1;
        let mut t0;
        let mut t1;
        let mut t2;
        let mut t3;

        // Special case for the first row (y = -1, t2 = t3 = 0).
        t1 = (values[0] >= threshold) as u32;
        case_stitch!((t1 << 1) as usize, x, y, &mut result);
        x += 1;
        while x < dx - 1 {
            t0 = t1;
            t1 = (values[(x + 1) as usize] >= threshold) as u32;
            case_stitch!((t0 | t1 << 1) as usize, x, y, &mut result);
            x += 1;
        }
        case_stitch!(t1 as usize, x, y, &mut result);

        // General case for the intermediate rows.
        y += 1;
        while y < dy - 1 {
            x = -1;
            t1 = (values[(y * dx + dx) as usize] >= threshold) as u32;
            t2 = (values[(y * dx) as usize] >= threshold) as u32;
            case_stitch!((t1 << 1 | t2 << 2) as usize, x, y, &mut result);
            x += 1;
            while x < dx - 1 {
                t0 = t1;
                t1 = (values[(y * dx + dx + x + 1) as usize] >= threshold) as u32;
                t3 = t2;
                t2 = (values[(y * dx + x + 1) as usize] >= threshold) as u32;
                case_stitch!(
                    (t0 | t1 << 1 | t2 << 2 | t3 << 3) as usize,
                    x,
                    y,
                    &mut result
                );
                x += 1;
            }
            case_stitch!((t1 | t2 << 3) as usize, x, y, &mut result);
            y += 1;
        }

        // Special case for the last row (y = dy - 1, t0 = t1 = 0).
        x = -1;
        t2 = (values[(y * dx) as usize] >= threshold) as u32;
        case_stitch!((t2 << 2) as usize, x, y, &mut result);
        x += 1;
        while x < dx - 1 {
            t3 = t2;
            t2 = (values[(y * dx + x + 1) as usize] >= threshold) as u32;
            case_stitch!((t2 << 2 | t3 << 3) as usize, x, y, &mut result);
            x += 1;
        }
        case_stitch!((t2 << 3) as usize, x, y, &mut result);
        self.is_empty = false;
        Ok(result)
    }

    fn index(&self, point: &Pt) -> usize {
        (point.x * 2.0 + point.y * (self.dx as f64 + 1.) * 4.) as usize
    }

    // Stitchs segments to rings.
    fn stitch(&mut self, line: &[Vec<f64>], x: i32, y: i32, result: &mut Vec<Ring>) -> Result<()> {
        let start = Pt {
            x: line[0][0] + x as f64,
            y: line[0][1] + y as f64,
        };
        let end = Pt {
            x: line[1][0] + x as f64,
            y: line[1][1] + y as f64,
        };
        let start_index = self.index(&start);
        let end_index = self.index(&end);
        if self.fragment_by_end.contains_key(&start_index) {
            if self.fragment_by_start.contains_key(&end_index) {
                let f_ix = self
                    .fragment_by_end
                    .remove(&start_index)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                let g_ix = self
                    .fragment_by_start
                    .remove(&end_index)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                if f_ix == g_ix {
                    let mut f = self.f.remove(f_ix);
                    f.ring.push(end);
                    result.push(f.ring);
                } else {
                    let mut f = self.f.remove(f_ix);
                    let g = self.f.remove(g_ix);
                    f.ring.extend(g.ring);
                    let ix = self.f.insert(Fragment {
                        start: f.start,
                        end: g.end,
                        ring: f.ring,
                    });
                    self.fragment_by_start.insert(f.start, ix);
                    self.fragment_by_end.insert(g.end, ix);
                }
            } else {
                let f_ix = self
                    .fragment_by_end
                    .remove(&start_index)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                let mut f = self
                    .f
                    .get_mut(f_ix)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                f.ring.push(end);
                f.end = end_index;
                self.fragment_by_end.insert(end_index, f_ix);
            }
        } else if self.fragment_by_start.contains_key(&end_index) {
            if self.fragment_by_end.contains_key(&start_index) {
                let f_ix = self
                    .fragment_by_start
                    .remove(&end_index)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                let g_ix = self
                    .fragment_by_end
                    .remove(&start_index)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                if f_ix == g_ix {
                    let mut f = self.f.remove(f_ix);
                    f.ring.push(end);
                    result.push(f.ring);
                } else {
                    let f = self.f.remove(f_ix);
                    let mut g = self.f.remove(g_ix);
                    g.ring.extend(f.ring);
                    let ix = self.f.insert(Fragment {
                        start: g.start,
                        end: f.end,
                        ring: g.ring,
                    });
                    self.fragment_by_start.insert(g.start, ix);
                    self.fragment_by_end.insert(f.end, ix);
                }
            } else {
                let f_ix = self
                    .fragment_by_start
                    .remove(&end_index)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                let mut f = self
                    .f
                    .get_mut(f_ix)
                    .ok_or_else(|| new_error(ErrorKind::Unexpected))?;
                f.ring.insert(0, start);
                f.start = start_index;
                self.fragment_by_start.insert(start_index, f_ix);
            }
        } else {
            let ix = self.f.insert(Fragment {
                start: start_index,
                end: end_index,
                ring: vec![start, end],
            });
            self.fragment_by_start.insert(start_index, ix);
            self.fragment_by_end.insert(end_index, ix);
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.f.clear();
        self.fragment_by_end.clear();
        self.fragment_by_start.clear();
        self.is_empty = true;
    }
}
