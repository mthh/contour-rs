use crate::area::{area, contains};
use crate::error::{new_error, ErrorKind, Result};
use crate::isoringbuilder::IsoRingBuilder;
use crate::{Band, Contour, Float, Line, Ring};
use geo_types::{LineString, MultiLineString, MultiPolygon, Polygon};
use rustc_hash::FxHashMap;

/// Contours generator, using builder pattern, to
/// be used on a rectangular `Slice` of values to
/// get a `Vec` of [`Contour`] (uses [`contour_rings`] internally).
///
/// [`contour_rings`]: fn.contour_rings.html
pub struct ContourBuilder {
    /// The number of columns in the grid
    dx: usize,
    /// The number of rows in the grid
    dy: usize,
    /// Whether to smooth the contours
    smooth: bool,
    /// The horizontal coordinate for the origin of the grid.
    x_origin: Float,
    /// The vertical coordinate for the origin of the grid.
    y_origin: Float,
    /// The horizontal step for the grid
    x_step: Float,
    /// The vertical step for the grid
    y_step: Float,
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
    pub fn new(dx: usize, dy: usize, smooth: bool) -> Self {
        ContourBuilder {
            dx,
            dy,
            smooth,
            x_origin: 0.,
            y_origin: 0.,
            x_step: 1.,
            y_step: 1.,
        }
    }

    /// Sets the x origin of the grid.
    pub fn x_origin(mut self, x_origin: impl Into<Float>) -> Self {
        self.x_origin = x_origin.into();
        self
    }

    /// Sets the y origin of the grid.
    pub fn y_origin(mut self, y_origin: impl Into<Float>) -> Self {
        self.y_origin = y_origin.into();
        self
    }

    /// Sets the x step of the grid.
    pub fn x_step(mut self, x_step: impl Into<Float>) -> Self {
        self.x_step = x_step.into();
        self
    }

    /// Sets the y step of the grid.
    pub fn y_step(mut self, y_step: impl Into<Float>) -> Self {
        self.y_step = y_step.into();
        self
    }

    fn smooth_linear(&self, ring: &mut Ring, values: &[Float], value: Float) {
        let dx = self.dx;
        let dy = self.dy;
        let len_values = values.len();

        ring.iter_mut()
            .map(|point| {
                let x = point.x;
                let y = point.y;
                let xt = x.trunc() as usize;
                let yt = y.trunc() as usize;
                let mut v0;
                let ix = yt * dx + xt;
                if ix < len_values {
                    let v1 = values[ix];
                    if x > 0.0 && x < (dx as Float) && (xt as Float - x).abs() < Float::EPSILON {
                        v0 = values[yt * dx + xt - 1];
                        point.x = x + (value - v0) / (v1 - v0) - 0.5;
                    }
                    if y > 0.0 && y < (dy as Float) && (yt as Float - y).abs() < Float::EPSILON {
                        v0 = values[(yt - 1) * dx + xt];
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
    pub fn lines(&self, values: &[Float], thresholds: &[Float]) -> Result<Vec<Line>> {
        if values.len() != self.dx * self.dy {
            return Err(new_error(ErrorKind::BadDimension));
        }
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        thresholds
            .iter()
            .map(|threshold| self.line(values, *threshold, &mut isoring))
            .collect()
    }

    fn line(
        &self,
        values: &[Float],
        threshold: Float,
        isoring: &mut IsoRingBuilder,
    ) -> Result<Line> {
        let mut result = isoring.compute(values, threshold)?;
        let mut linestrings = Vec::new();

        result.drain(..).for_each(|mut ring| {
            // Smooth the ring if needed
            if self.smooth {
                self.smooth_linear(&mut ring, values, threshold);
            }
            // Compute the polygon coordinates according to the grid properties if needed
            if (self.x_origin, self.y_origin) != (0.0, 0.0)
                || (self.x_step, self.y_step) != (1.0, 1.0)
            {
                ring.iter_mut().for_each(|point| {
                    point.x = point.x * self.x_step + self.x_origin;
                    point.y = point.y * self.y_step + self.y_origin;
                });
            }
            linestrings.push(LineString(ring));
        });
        Ok(Line {
            geometry: MultiLineString::<Float>(linestrings),
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
    pub fn contours(&self, values: &[Float], thresholds: &[Float]) -> Result<Vec<Contour>> {
        if values.len() != self.dx * self.dy {
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
        values: &[Float],
        threshold: Float,
        isoring: &mut IsoRingBuilder,
    ) -> Result<Contour> {
        let (mut polygons, mut holes) = (Vec::new(), Vec::new());
        let mut result = isoring.compute(values, threshold)?;

        result.drain(..).for_each(|mut ring| {
            // Smooth the ring if needed
            if self.smooth {
                self.smooth_linear(&mut ring, values, threshold);
            }
            // Compute the polygon coordinates according to the grid properties if needed
            if (self.x_origin, self.y_origin) != (0.0, 0.0)
                || (self.x_step, self.y_step) != (1.0, 1.0)
            {
                ring.iter_mut().for_each(|point| {
                    point.x = point.x * self.x_step + self.x_origin;
                    point.y = point.y * self.y_step + self.y_origin;
                });
            }
            if area(&ring) > 0.0 {
                polygons.push(Polygon::<Float>::new(LineString::new(ring), vec![]))
            } else {
                holes.push(LineString::new(ring));
            }
        });

        holes.drain(..).for_each(|hole| {
            for polygon in &mut polygons {
                if contains(&polygon.exterior().0, &hole.0) != -1 {
                    polygon.interiors_push(hole);
                    return;
                }
            }
        });

        Ok(Contour {
            geometry: MultiPolygon::<Float>(polygons),
            threshold,
        })
    }

    /// Computes isobands according the given input `values` and the given `thresholds`.
    /// Returns a `Vec` of [`Band`] (that can easily be transformed
    /// to GeoJSON Features of MultiPolygon).
    /// The threshold value of each Feature is stored in its `value` property.
    ///
    /// # Arguments
    ///
    /// * `values` - The slice of values to be used.
    /// * `thresholds` - The slice of thresholds values to be used
    ///                  (have to be equal to or greater than 2).
    pub fn isobands(&self, values: &[Float], thresholds: &[Float]) -> Result<Vec<Band>> {
        // We will compute rings as previously, but we will
        // iterate over the contours in pairs and use the paths from the lower threshold
        // and the path from the upper threshold to create the isoband.
        if values.len() != self.dx * self.dy {
            return Err(new_error(ErrorKind::BadDimension));
        }
        if thresholds.len() < 2 {
            return Err(new_error(ErrorKind::Unexpected));
        }
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);

        let rings = thresholds
            .iter()
            .map(|threshold| {
                // Compute the rings for the current threshold
                let rings = isoring.compute(values, *threshold)?;
                let rings = rings
                    .into_iter()
                    .map(|mut ring| {
                        // Smooth the ring if needed
                        if self.smooth {
                            self.smooth_linear(&mut ring, values, *threshold);
                        }
                        ring.dedup();
                        // Compute the polygon coordinates according to the grid properties if needed
                        if (self.x_origin, self.y_origin) != (0.0, 0.0)
                            || (self.x_step, self.y_step) != (1.0, 1.0)
                        {
                            ring.iter_mut().for_each(|point| {
                                point.x = point.x * self.x_step + self.x_origin;
                                point.y = point.y * self.y_step + self.y_origin;
                            });
                        }
                        ring
                    })
                    .filter(|ring| ring.len() > 3)
                    .collect::<Vec<Ring>>();
                Ok((rings, *threshold))
            })
            .collect::<Result<Vec<(Vec<Ring>, Float)>>>()?;

        // We now have the rings for each isolines for all the given thresholds,
        // we can iterate over them in pairs to compute the isobands.
        let b = rings
            .windows(2)
            .map(|rings| {
                let ((lower_path, min_v), (upper_path, max_v)) = (&rings[0], &rings[1]);
                let concatenated = [&lower_path[..], &upper_path[..]].concat();
                (concatenated, min_v, max_v)
            })
            .collect::<Vec<_>>();

        let mut bands: Vec<Band> = Vec::new();
        // Reconstruction of the polygons
        b.into_iter().for_each(|(rings, min_v, max_v)| {
            let mut rings_and_area = rings
                .into_iter()
                .map(|ring| {
                    let area = area(&ring);
                    (ring, area)
                })
                .collect::<Vec<_>>();

            rings_and_area.sort_by_key(|(_, area)| area.abs() as u64);

            let mut enclosed_by_n = FxHashMap::default();

            for (i, (ring, _)) in rings_and_area.iter().enumerate() {
                let mut enclosed_by_j = 0;
                for (j, (ring_test, _)) in rings_and_area.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    if contains(ring_test, ring) != -1 {
                        enclosed_by_j += 1;
                    }
                }
                enclosed_by_n.insert(i, enclosed_by_j);
            }

            let mut polygons: Vec<Polygon<Float>> = Vec::new();
            let mut interior_rings: Vec<LineString<Float>> = Vec::new();

            for (i, (ring, _)) in rings_and_area.into_iter().enumerate() {
                if *enclosed_by_n.get(&i).unwrap() % 2 == 0 {
                    polygons.push(Polygon::<Float>::new(ring.into(), vec![]));
                } else {
                    interior_rings.push(ring.into());
                }
            }
            for interior_ring in interior_rings.into_iter() {
                for polygon in polygons.iter_mut() {
                    if contains(&polygon.exterior().0, &interior_ring.0) != -1 {
                        polygon.interiors_push(interior_ring);
                        break;
                    }
                }
            }

            polygons.reverse();

            bands.push(Band {
                geometry: MultiPolygon::<Float>(polygons),
                min_v: *min_v,
                max_v: *max_v,
            });
        });

        Ok(bands)
    }
}
