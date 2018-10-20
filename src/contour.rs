use geojson::{Feature, Geometry};
use geojson::Value::MultiPolygon;
use serde_json::map::Map;
use serde_json::to_value;
use rustc_hash::FxHashMap;
use ::area::{contains, area};

pub type Pt = Vec<f64>;
pub type Ring = Vec<Pt>;

lazy_static! {
    static ref CASES: Vec<Vec<Vec<Vec<f64>>>> = vec![
      vec![],
      vec![vec![vec![1.0, 1.5], vec![0.5, 1.0]]],
      vec![vec![vec![1.5, 1.0], vec![1.0, 1.5]]],
      vec![vec![vec![1.5, 1.0], vec![0.5, 1.0]]],
      vec![vec![vec![1.0, 0.5], vec![1.5, 1.0]]],
      vec![vec![vec![1.0, 1.5], vec![0.5, 1.0]], vec![vec![1.0, 0.5], vec![1.5, 1.0]]],
      vec![vec![vec![1.0, 0.5], vec![1.0, 1.5]]],
      vec![vec![vec![1.0, 0.5], vec![0.5, 1.0]]],
      vec![vec![vec![0.5, 1.0], vec![1.0, 0.5]]],
      vec![vec![vec![1.0, 1.5], vec![1.0, 0.5]]],
      vec![vec![vec![0.5, 1.0], vec![1.0, 0.5]], vec![vec![1.5, 1.0], vec![1.0, 1.5]]],
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
        ContourBuilder {
            dx: dx, dy: dy, smooth: smooth,
        }
    }

    fn smoooth_linear(&self, ring: &mut Ring, values: &[f64], value: f64) {
        let dx = self.dx;
        let dy = self.dy;
        let len_values = values.len();
        ring.iter_mut().map(|point| {
            let x = point[0];
            let y = point[1];
            let xt = x.trunc() as u32;
            let yt = y.trunc() as u32;
            let mut v0;
            let ix = (yt * dx + xt) as usize;
            if ix < len_values {
                let v1 = values[ix];
                if x > 0.0 && x < (dx as f64) && xt as f64 == x {
                    v0 = values[(yt * dx + xt - 1) as usize];
                    point[0] = x + (value - v0) / (v1 - v0) - 0.5;
                }
                if y > 0.0 && y < (dy as f64) && yt as f64 == y {
                    v0 = values[((yt - 1) * dx + xt) as usize];
                    point[1] = y + (value - v0) / (v1 - v0) - 0.5;
                }
            }
        }).for_each(drop);
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
        thresholds.iter().map(|value|{
            self.contour(values, *value)
        }).collect::<Vec<Feature>>()
    }

    fn contour(&self, values: &[f64], threshold: f64) -> Feature {
        let mut polygons = Vec::new();
        let mut holes = Vec::new();
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        let mut result = isoring.compute(values, threshold);

        result.drain(..).map(|mut ring|{
            if self.smooth {
                self.smoooth_linear(&mut ring, values, threshold);
            }
            if area(&ring) > 0.0 {
                polygons.push(vec![ring]);
            } else {
                holes.push(ring);
            }
        }).for_each(drop);

        holes.drain(..).map(|hole|{
            for i in 0..polygons.len() {
                let polygon = &mut polygons[i];
                if contains(&polygon[0], &hole) != -1 {
                    polygon.push(hole);
                    return;
                }
            }
        }).for_each(drop);

        let mut properties = Map::new();
        properties.insert(String::from("value"), to_value(threshold).unwrap());
        Feature {
            geometry: Some(Geometry {
                value: MultiPolygon(polygons),
                bbox: None,
                foreign_members: None
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
    fragment_by_start: FxHashMap<usize, Fragment>,
    fragment_by_end: FxHashMap<usize, Fragment>,
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
           dx: dx,
           dy :dy,
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
        CASES[(t1 << 1) as usize].iter().map(|ring| {
            self.stitch(&ring, x, y, &mut result);
        }).for_each(drop);
        x += 1;
        while x < dx - 1 {
            t0 = t1;
            t1 = (values[(x + 1) as usize] >= threshold) as u32;
            CASES[(t0 | t1 << 1) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y, &mut result);
            }).for_each(drop);
            x += 1;
        }
        CASES[(t1 << 0) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y, &mut result);
        }).for_each(drop);

        // General case for the intermediate rows.
        y += 1;
        while y < dy - 1 {
            x = -1;
            t1 = (values[(y * dx + dx) as usize] >= threshold) as u32;
            t2 = (values[(y * dx) as usize] >= threshold) as u32;
            CASES[(t1 << 1 | t2 << 2) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y, &mut result);
            }).for_each(drop);
            x += 1;
            while x < dx - 1 {
               t0 = t1;
               t1 = (values[(y * dx + dx + x + 1) as usize] >= threshold) as u32;
               t3 = t2;
               t2 = (values[(y * dx + x + 1) as usize] >= threshold) as u32;
               CASES[(t0 | t1 << 1 | t2 << 2 | t3 << 3) as usize].iter().map(|ring| {
                   self.stitch(&ring, x, y, &mut result);
               }).for_each(drop);
               x += 1;
           }
           CASES[(t1 | t2 << 3) as usize].iter().map(|ring|{
               self.stitch(&ring, x, y, &mut result);
           }).for_each(drop);
           y += 1;
        }

        // Special case for the last row (y = dy - 1, t0 = t1 = 0).
        x = -1;
        t2 = (values[(y * dx) as usize] >= threshold) as u32;
        CASES[(t2 << 2) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y, &mut result);
        }).for_each(drop);
        x += 1;
        while x < dx - 1 {
            t3 = t2;
            t2 = (values[(y * dx + x + 1) as usize] >= threshold) as u32;
            CASES[(t2 << 2 | t3 << 3) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y, &mut result);
            }).for_each(drop);
            x += 1;
        }
        CASES[(t2 << 3) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y, &mut result);
        }).for_each(drop);

        result
    }

    fn index(&self, point: &Pt) -> usize {
      return (point[0] * 2.0 + point[1] * (self.dx as f64 + 1.) * 4.) as usize;
    }

    fn stitch(&mut self, line: &Vec<Vec<f64>>, x: i32, y: i32, result: &mut Vec<Ring>) {
        let start = vec![line[0][0] + x as f64, line[0][1] + y as f64];
        let end = vec![line[1][0] + x as f64, line[1][1] + y as f64];
        let start_index = self.index(&start);
        let end_index = self.index(&end);
        if self.fragment_by_end.contains_key(&start_index){
            let mut f = self.fragment_by_end.remove(&start_index).unwrap();
            if self.fragment_by_start.contains_key(&end_index) {
                let (g_start, g_end) = get_start_end(&self.fragment_by_start, end_index);
                if f.end == g_end && f.start == g_start {
                    f.ring.push(end);
                    result.push(f.ring);
                } else {
                    if g_start != end_index {
                        let g = self.fragment_by_start.remove(&end_index).unwrap();
                        f.ring.extend(g.ring);
                    } else if let Some(_t) = self.fragment_by_start.remove(&g_start) {
                        f.ring.extend(_t.ring);
                    }
                    self.fragment_by_start.insert(f.start, Fragment { start: f.start, end: g_end, ring: f.ring.clone() });
                    self.fragment_by_end.insert(g_end, Fragment { start: f.start, end: g_end, ring: f.ring });
                }
            } else {
                if let Some(a) = self.fragment_by_start.get_mut(&f.start) {
                    a.end = end_index;
                    a.ring.push(end.clone());
                }
                f.ring.push(end);
                f.end = end_index;
                self.fragment_by_end.insert(end_index, f);
            }
        } else if self.fragment_by_start.contains_key(&end_index) {
            let mut f = self.fragment_by_start.remove(&end_index).unwrap();
            if self.fragment_by_end.contains_key(&start_index) {
                let (g_start, g_end) = get_start_end(&self.fragment_by_end, start_index);
                if f.end == g_end && f.start == g_start {
                    f.ring.push(end);
                    result.push(f.ring);
                } else {
                    if start_index != g_end {
                        let g = self.fragment_by_end.remove(&start_index).unwrap();
                        f.ring.extend(g.ring);
                    } else if let Some(_t) = self.fragment_by_end.remove(&g_end) {
                        f.ring.extend(_t.ring);
                    }
                    self.fragment_by_start.insert(g_start, Fragment { start: g_start, end: f.end, ring: f.ring.clone() });
                    self.fragment_by_end.insert(f.end, Fragment { start: f.start, end: g_end, ring: f.ring });
                }
            } else {
                if let Some(a) = self.fragment_by_end.get_mut(&f.end) {
                    a.start = start_index;
                    a.ring.insert(0, start.clone());
                }
                f.ring.insert(0, start);
                f.start = start_index;
                self.fragment_by_start.insert(start_index, f);
            }
        } else {
            let a = vec![start, end];
            self.fragment_by_start.insert(start_index, Fragment { start: start_index, end: end_index, ring: a.clone() });
            self.fragment_by_end.insert(end_index, Fragment { start: start_index, end: end_index, ring: a });
        }
    }
}

fn get_start_end(map: &FxHashMap<usize, Fragment>, ix: usize) -> (usize, usize) {
    let frag = map.get(&ix).unwrap();
    (frag.start, frag.end)
}
