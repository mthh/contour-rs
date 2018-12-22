use crate::area::{area, contains};
use geojson::Value::MultiPolygon;
use geojson::{Feature, Geometry};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use serde_json::map::Map;
use serde_json::to_value;
use slab::Slab;

pub type Pt = Vec<f64>;
pub type Ring = Vec<Pt>;

lazy_static! {
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

/// Contours generator to
/// be used on a rectangular `Vec` of values to
/// get a `Vec` of Features of MultiPolygon (use [`IsoRingBuilder`] internally).
///
/// [`IsoRingBuilder`]: struct.IsoRingBuilder.html
pub struct ContourBuilder {
    dx: u32,
    dy: u32,
    smooth: bool,
}

impl ContourBuilder {
    /// Constructs a new contours generator for a grid with `dx` * `dy` dimension.
    ///
    /// # Arguments
    ///
    /// * `dx` - The number of columns in the grid.
    /// * `dy` - The number of rows in the grid.
    /// * `smooth` - Whether or not the generated rings will be smoothed using linear interpolation.
    pub fn new(dx: u32, dy: u32, smooth: bool) -> Self {
        ContourBuilder { dx, dy, smooth }
    }

    fn smoooth_linear(&self, ring: &mut Ring, values: &[f64], value: f64) {
        let dx = self.dx;
        let dy = self.dy;
        let len_values = values.len();
        ring.iter_mut()
            .map(|point| {
                let x = point[0];
                let y = point[1];
                let xt = x.trunc() as u32;
                let yt = y.trunc() as u32;
                let mut v0;
                let ix = (yt * dx + xt) as usize;
                if ix < len_values {
                    let v1 = values[ix];
                    if x > 0.0 && x < (dx as f64) && (xt as f64 - x).abs() < std::f64::EPSILON {
                        v0 = values[(yt * dx + xt - 1) as usize];
                        point[0] = x + (value - v0) / (v1 - v0) - 0.5;
                    }
                    if y > 0.0 && y < (dy as f64) && (yt as f64 - y).abs() < std::f64::EPSILON {
                        v0 = values[((yt - 1) * dx + xt) as usize];
                        point[1] = y + (value - v0) / (v1 - v0) - 0.5;
                    }
                }
            })
            .for_each(drop);
    }

    /// Computes contours according the given input `values` and the given `thresholds`.
    /// Returns a `Vec` of Features of MultiPolygon.
    /// The threshold value of each Feature is stored in its `value` property.
    ///
    /// # Arguments
    ///
    /// * `values` - ...
    /// * `thresholds` - ...
    pub fn contours(&self, values: &[f64], thresholds: &[f64]) -> Vec<Feature> {
        thresholds
            .iter()
            .map(|value| self.contour(values, *value))
            .collect::<Vec<Feature>>()
    }

    fn contour(&self, values: &[f64], threshold: f64) -> Feature {
        let mut polygons = Vec::new();
        let mut holes = Vec::new();
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        let mut result = isoring.compute(values, threshold);

        result
            .drain(..)
            .map(|mut ring| {
                if self.smooth {
                    self.smoooth_linear(&mut ring, values, threshold);
                }
                if area(&ring) > 0.0 {
                    polygons.push(vec![ring]);
                } else {
                    holes.push(ring);
                }
            })
            .for_each(drop);

        holes
            .drain(..)
            .map(|hole| {
                for polygon in &mut polygons {
                    if contains(&polygon[0], &hole) != -1 {
                        polygon.push(hole);
                        return;
                    }
                }
            })
            .for_each(drop);

        let mut properties = Map::with_capacity(1);
        properties.insert(String::from("value"), to_value(threshold).unwrap());
        Feature {
            geometry: Some(Geometry {
                value: MultiPolygon(polygons),
                bbox: None,
                foreign_members: None,
            }),
            properties: Some(properties),
            bbox: None,
            id: None,
            foreign_members: None,
        }
    }
}

/// Isoring generator to compute marching squares with isolines stitched into rings.
pub struct IsoRingBuilder {
    fragment_by_start: FxHashMap<usize, usize>,
    fragment_by_end: FxHashMap<usize, usize>,
    f: Slab<Fragment>,
    dx: u32,
    dy: u32,
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
        }
    }

    /// Computes isoring for the given slice of `values` according to the `threshold` value
    /// (the inside of the isoring is the surface where input `values` are greater than or equal
    /// to the given threshold value).
    ///
    /// # Arguments
    ///
    /// * `values` - The number of columns in the grid.
    /// * `threshold` - The number of rows in the grid.
    pub fn compute(&mut self, values: &[f64], threshold: f64) -> Vec<Ring> {
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
        CASES[(t1 << 1) as usize]
            .iter()
            .map(|ring| {
                self.stitch(&ring, x, y, &mut result);
            })
            .for_each(drop);
        x += 1;
        while x < dx - 1 {
            t0 = t1;
            t1 = (values[(x + 1) as usize] >= threshold) as u32;
            CASES[(t0 | t1 << 1) as usize]
                .iter()
                .map(|ring| {
                    self.stitch(&ring, x, y, &mut result);
                })
                .for_each(drop);
            x += 1;
        }
        CASES[(t1 << 0) as usize]
            .iter()
            .map(|ring| {
                self.stitch(&ring, x, y, &mut result);
            })
            .for_each(drop);

        // General case for the intermediate rows.
        y += 1;
        while y < dy - 1 {
            x = -1;
            t1 = (values[(y * dx + dx) as usize] >= threshold) as u32;
            t2 = (values[(y * dx) as usize] >= threshold) as u32;
            CASES[(t1 << 1 | t2 << 2) as usize]
                .iter()
                .map(|ring| {
                    self.stitch(&ring, x, y, &mut result);
                })
                .for_each(drop);
            x += 1;
            while x < dx - 1 {
                t0 = t1;
                t1 = (values[(y * dx + dx + x + 1) as usize] >= threshold) as u32;
                t3 = t2;
                t2 = (values[(y * dx + x + 1) as usize] >= threshold) as u32;
                CASES[(t0 | t1 << 1 | t2 << 2 | t3 << 3) as usize]
                    .iter()
                    .map(|ring| {
                        self.stitch(&ring, x, y, &mut result);
                    })
                    .for_each(drop);
                x += 1;
            }
            CASES[(t1 | t2 << 3) as usize]
                .iter()
                .map(|ring| {
                    self.stitch(&ring, x, y, &mut result);
                })
                .for_each(drop);
            y += 1;
        }

        // Special case for the last row (y = dy - 1, t0 = t1 = 0).
        x = -1;
        t2 = (values[(y * dx) as usize] >= threshold) as u32;
        CASES[(t2 << 2) as usize]
            .iter()
            .map(|ring| {
                self.stitch(&ring, x, y, &mut result);
            })
            .for_each(drop);
        x += 1;
        while x < dx - 1 {
            t3 = t2;
            t2 = (values[(y * dx + x + 1) as usize] >= threshold) as u32;
            CASES[(t2 << 2 | t3 << 3) as usize]
                .iter()
                .map(|ring| {
                    self.stitch(&ring, x, y, &mut result);
                })
                .for_each(drop);
            x += 1;
        }
        CASES[(t2 << 3) as usize]
            .iter()
            .map(|ring| {
                self.stitch(&ring, x, y, &mut result);
            })
            .for_each(drop);

        result
    }

    fn index(&self, point: &[f64]) -> usize {
        (point[0] * 2.0 + point[1] * (self.dx as f64 + 1.) * 4.) as usize
    }

    fn stitch(&mut self, line: &[Vec<f64>], x: i32, y: i32, result: &mut Vec<Ring>) {
        let start = vec![line[0][0] + x as f64, line[0][1] + y as f64];
        let end = vec![line[1][0] + x as f64, line[1][1] + y as f64];
        let start_index = self.index(&start);
        let end_index = self.index(&end);
        if self.fragment_by_end.contains_key(&start_index) {
            if self.fragment_by_start.contains_key(&end_index) {
                let f_ix = self.fragment_by_end.remove(&start_index).unwrap();
                let g_ix = self.fragment_by_start.remove(&end_index).unwrap();
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
                    self.fragment_by_start.insert(
                        f.start,
                        ix,
                    );
                    self.fragment_by_end.insert(
                        g.end,
                        ix,
                    );
                }
            } else {
                let f_ix = self.fragment_by_end.remove(&start_index).unwrap();
                let mut f = self.f.get_mut(f_ix).unwrap();
                f.ring.push(end);
                f.end = end_index;
                self.fragment_by_end.insert(
                    end_index,
                    f_ix,
                );
            }
        } else if self.fragment_by_start.contains_key(&end_index) {
            if self.fragment_by_end.contains_key(&start_index) {
                let f_ix = self.fragment_by_start.remove(&end_index).unwrap();
                let g_ix = self.fragment_by_end.remove(&start_index).unwrap();
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
                    self.fragment_by_start.insert(
                        g.start,
                        ix,
                    );
                    self.fragment_by_end.insert(
                        f.end,
                        ix,
                    );
                }
            } else {
                let f_ix = self.fragment_by_start.remove(&end_index).unwrap();
                let mut f = self.f.get_mut(f_ix).unwrap();
                f.ring.insert(0, start);
                f.start = start_index;
                self.fragment_by_start.insert(
                    start_index,
                    f_ix,
                );
            }
        } else {
            let ix = self.f.insert(Fragment {
                start: start_index,
                end: end_index,
                ring: vec![start, end],
            });
            self.fragment_by_start.insert(
                start_index,
                ix,
            );
            self.fragment_by_end.insert(
                end_index,
                ix,
            );
        }
    }
}
