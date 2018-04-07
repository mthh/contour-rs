extern crate geojson;
extern crate serde_json;

use geojson::Feature;
use serde_json::map::Map;
use std::collections::HashMap;

type Pt = Vec<f64>;
type Ring = Vec<Pt>;


fn contains(ring: &Ring, hole: &Ring) -> i32 {
    let mut i = 0;
    let n = hole.len();
    let mut c;
    while i < n {
        c = ring_contains(ring, &hole[i]);
        if c != 0 {
            return c;
        }
        i += 1;
    }
    return 0;
}

fn ring_contains(ring: &Ring, point: &Pt) -> i32 {
    let x = point[0];
    let y = point[1];
    let n = ring.len();
    let mut contains = -1;
    let mut j = n - 1;
    for i in 0..n {
        let pi = &ring[i];
        let xi = pi[0];
        let yi = pi[1];
        let pj = &ring[j];
        let xj = pj[0];
        let yj = pj[1];
        if segment_contains(&pi, &pj, point) {
            return 0;
        }
        if ((yi > y) != (yj > y)) && ((x < (xj - xi) * (y - yi) / (yj - yi) + xi)) {
            contains = -contains;
        }
        j = i;
    }
    return contains;
}

fn segment_contains(a: &Pt, b: &Pt, c: &Pt) -> bool {
    if collinear(a, b, c) {
        if a[0] == b[0] {
            within(a[1], c[1], b[1])
        } else {
            within(a[0], c[0], b[0])
        }
    } else {
        false
    }
}

fn collinear(a: &Pt, b: &Pt, c: &Pt) -> bool {
    (b[0] - a[0]) * (c[1] - a[1]) == (c[0] - a[0]) * (b[1] - a[1])
}

fn within(p: f64, q: f64, r: f64) -> bool {
    p <= q && q <= r || r <= q && q <= p
}

fn area(ring: &Ring) -> f64 {
    let mut i = 0;
    let n = ring.len() - 1;
    let mut area = ring[n - 1][1] * ring[0][0] - ring[n - 1][0] * ring[0][1];
    while i < n {
        i += 1;
        area += ring[i - 1][1] * ring[i][0] - ring[i - 1][0] * ring[i][1];
    }
    area
}

pub struct ContourBuilder {
    threshold: Vec<f64>,
    dx: u32,
    dy: u32,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Fragment {
    start: usize,
    end: usize,
    ring: Ring,
}

impl ContourBuilder {
    pub fn new(threshold: Vec<f64>, dx: u32, dy: u32) -> Self {
        ContourBuilder {
            threshold: threshold, dx: dx, dy: dy,
        }
    }

    fn smoooth_linear(&self, ring: &mut Ring, values: &Vec<f64>, value: f64) {
        let dx = self.dx;
        let dy = self.dy;
        ring.iter_mut().map(|point| {
            let x = point[0];
            let y = point[1];
            let xt = x.trunc() as u32;
            let yt = y.trunc() as u32;
            let mut v0;
            let v1 = values[(yt * dx + xt) as usize];
            if x > 0.0 && x < (dx as f64) && xt as f64 == x {
                v0 = values[(yt * dx + xt - 1) as usize];
                point[0] = x + (value - v0) / (v1 - v0) - 0.5;
            }
            if y > 0.0 && y < (dy as f64) && yt as f64 == y {
              v0 = values[((yt - 1) * dx + xt) as usize];
              point[1] = y + (value - v0) / (v1 - v0) - 0.5;
            }
        }).collect::<Vec<()>>();
    }

    pub fn contours(&self, values: &Vec<f64>) -> Vec<Feature> {
        self.threshold.iter().map(|value|{
            self.contour(values, *value)
        }).collect::<Vec<Feature>>()
    }

    fn contour(&self, values: &Vec<f64>, value: f64) -> Feature {
        let mut polygons = Vec::new();
        let mut holes = Vec::new();
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        isoring.compute(values, value);
        let mut rings = isoring.result;
        rings.iter_mut().map(|ring|{
            self.smoooth_linear(ring, values, value);
            if area(&ring) > 0.0 {
                polygons.push(vec![ring.clone()]);
            } else {
                holes.push(ring.clone());
            }
        }).collect::<Vec<()>>();
        holes.iter().map(|hole|{
            for i in 0..polygons.len() {
                let polygon = &mut polygons[i];
                if contains(&polygon[0], hole) != -1 {
                    polygon.push(hole.clone());
                    return;
                }
            }
        }).collect::<Vec<()>>();
        let mut properties = Map::new();
        properties.insert(String::from("value"), serde_json::to_value(value).unwrap());
        Feature {
            geometry: Some(geojson::Geometry {
                value: geojson::Value::MultiPolygon(polygons),
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


struct IsoRingBuilder {
    fragment_by_start: HashMap<usize, Fragment>,
    fragment_by_end: HashMap<usize, Fragment>,
    dx: u32,
    dy: u32,
    result: Vec<Ring>,
}

impl IsoRingBuilder {
    fn new(dx: u32, dy: u32) -> Self {
       IsoRingBuilder {
           fragment_by_start: HashMap::new(),
           fragment_by_end: HashMap::new(),
           dx: dx,
           dy :dy,
           result: Vec::new(),
       }
    }

    fn compute(&mut self, values: &Vec<f64>, value: f64) {
        let cases = vec![
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
        let dx = self.dx;
        let dy = self.dy;
        let mut x = -1;
        let mut y = -1;
        let mut t0;
        let mut t1;
        let mut t2;
        let mut t3;
        // Special case for the first row (y = -1, t2 = t3 = 0).
        t1 = values[0] >= value;
        cases[((t1 as u32) << 1) as usize].iter().map(|ring| {
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();
        x += 1;
        while x < dx as i32 - 1 {
            t0 = t1;
            t1 = values[(x + 1) as usize] >= value;
            cases[((t0 as u32) | (t1 as u32) << 1) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y);
            }).collect::<Vec<()>>();
            x += 1;
        }
        cases[((t1 as u32) << 0) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();
        // General case for the intermediate rows.
        y += 1;
        while y < dy as i32 - 1 {
            x = -1;
            t1 = values[(y * dx as i32 + dx as i32) as usize] >= value;
            t2 = values[(y * dx as i32) as usize] >= value;
            cases[((t1 as u32) << 1 | (t2 as u32) << 2) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y);
            }).collect::<Vec<()>>();
            x += 1;
            while x < dx as i32 - 1 {
               t0 = t1;
               t1 = values[(y * dx as i32 + dx as i32 + x + 1) as usize] >= value;
               t3 = t2;
               t2 = values[(y * dx as i32 + x + 1) as usize] >= value;
               cases[((t0 as u32) | (t1 as u32) << 1 | (t2 as u32) << 2 | (t3 as u32) << 3) as usize].iter().map(|ring| {
                   self.stitch(&ring, x, y);
               }).collect::<Vec<()>>();
               x += 1;
           }
           cases[((t1 as u32) | (t2 as u32) << 3) as usize].iter().map(|ring|{
               self.stitch(&ring, x, y);
           }).collect::<Vec<()>>();
           y += 1;
        }
        // Special case for the last row (y = dy - 1, t0 = t1 = 0).
        x = -1;
        t2 = values[(y * dx as i32) as usize] >= value;
        cases[((t2 as u32) << 2) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();
        x += 1;
        while x < dx as i32 - 1 {
            t3 = t2;
            t2 = values[(y * dx as i32 + x + 1) as usize] >= value;
            cases[((t2 as u32) << 2 | (t3 as u32) << 3) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y);
            }).collect::<Vec<()>>();
            x += 1;
        }
        cases[((t2 as u32) << 3) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();
    }

    fn index(&self, point: &Pt) -> usize {
      return (point[0] * 2.0 + point[1] * (self.dx as f64 + 1.) * 4.) as usize;
    }

    fn stitch(&mut self, line: &Vec<Vec<f64>>, x: i32, y: i32) {
        let start = vec![line[0][0] + x as f64, line[0][1] + y as f64];
        let end = vec![line[1][0] + x as f64, line[1][1] + y as f64];
        let start_index = self.index(&start);
        let end_index = self.index(&end);
        let mut cfbs1 = self.fragment_by_start.clone();
        let mut cfbe1 = self.fragment_by_end.clone();
        let mut cfbs2 = self.fragment_by_start.clone();
        let mut cfbe2 = self.fragment_by_end.clone();
        if let Some(f) = cfbe1.get_mut(&start_index) {
            if let Some(g) = cfbs1.get_mut(&end_index) {
                self.fragment_by_end.remove(&f.end);
                self.fragment_by_start.remove(&g.start);
                if f.end == g.end && f.start == g.start {
                    f.ring.push(end);
                    self.result.push(f.ring.clone());
                } else {
                    let mut temp = f.ring.clone();
                    temp.extend(g.ring.iter().cloned());
                    f.ring = temp.clone();
                    self.fragment_by_start.insert(f.start, Fragment { start: f.start, end: g.end, ring: temp });
                    self.fragment_by_end.insert(g.end, self.fragment_by_start[&f.start].clone());
                    self.fragment_by_end.insert(start_index, f.clone());
                }
            } else {
                self.fragment_by_end.remove(&f.end);
                f.ring.push(end.clone());
                if let Some(a) = self.fragment_by_start.get_mut(&f.start) {
                    a.end = end_index;
                    a.ring = f.ring.clone();
                }
                f.end = end_index;
                self.fragment_by_end.insert(end_index, f.clone());
                self.fragment_by_end.insert(start_index, f.clone());

            }

        } else if let Some(f) = cfbs2.get_mut(&end_index) {
            if let Some(g) = cfbe2.get_mut(&start_index) {
                self.fragment_by_start.remove(&f.start);
                self.fragment_by_end.remove(&g.end);
                if f.end == g.end && f.start == g.start {
                    f.ring.push(end);
                    self.result.push(f.ring.clone());
                } else {
                    let mut temp = g.ring.clone();
                    temp.extend(f.ring.iter().cloned());
                    g.ring = temp.clone();
                    self.fragment_by_start.insert(g.start, Fragment { start: g.start, end: f.end, ring: temp });
                    self.fragment_by_end.insert(f.end, self.fragment_by_start[&g.start].clone());
                    self.fragment_by_end.insert(start_index, self.fragment_by_start[&g.start].clone());
                }
            } else {
                self.fragment_by_start.remove(&f.start);
                f.ring.insert(0, start);
                if let Some(a) = self.fragment_by_end.get_mut(&f.end) {
                    a.start = start_index;
                    a.ring = f.ring.clone();
                }
                f.start = start_index;
                self.fragment_by_start.insert(start_index, f.clone());
                self.fragment_by_start.insert(end_index, f.clone());
            }
        } else {
            self.fragment_by_start.insert(start_index, Fragment { start: start_index, end: end_index, ring: vec![start, end] });
            self.fragment_by_end.insert(end_index, self.fragment_by_start[&start_index].clone());
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use ::ContourBuilder;
        use geojson;
        let c = ContourBuilder::new(vec![0.5], 10, 10);
        let res = c.contours(&vec![
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
        ]);
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
}
