use geojson::{Feature, Geometry};
use geojson::Value::MultiPolygon;
use serde_json::map::Map;
use serde_json::to_value;
use std::collections::BTreeMap;

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

#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct Fragment {
    start: usize,
    end: usize,
    ring: Ring,
}

pub struct ContourBuilder {
    threshold: Vec<f64>,
    dx: u32,
    dy: u32,
    smooth: bool,
}

impl ContourBuilder {
    pub fn new(threshold: Vec<f64>, dx: u32, dy: u32, smooth: bool) -> Self {
        ContourBuilder {
            threshold: threshold, dx: dx, dy: dy, smooth: smooth,
        }
    }

    fn smoooth_linear(&self, ring: &mut Ring, values: &[f64], value: f64) {
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

    pub fn contours(&self, values: &[f64]) -> Vec<Feature> {
        self.threshold.iter().map(|value|{
            self.contour(values, *value)
        }).collect::<Vec<Feature>>()
    }

    fn contour(&self, values: &[f64], value: f64) -> Feature {
        let mut polygons = Vec::new();
        let mut holes = Vec::new();
        let mut isoring = IsoRingBuilder::new(self.dx, self.dy);
        isoring.compute(values, value);

        isoring.result.drain(..).map(|mut ring|{
            if self.smooth {
                self.smoooth_linear(&mut ring, values, value);
            }
            if area(&ring) > 0.0 {
                polygons.push(vec![ring]);
            } else {
                holes.push(ring);
            }
        }).collect::<Vec<()>>();

        holes.drain(..).map(|hole|{
            for i in 0..polygons.len() {
                let polygon = &mut polygons[i];
                if contains(&polygon[0], &hole) != -1 {
                    polygon.push(hole);
                    return;
                }
            }
        }).collect::<Vec<()>>();

        let mut properties = Map::new();
        properties.insert(String::from("value"), to_value(value).unwrap());
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


pub struct IsoRingBuilder {
    fragment_by_start: BTreeMap<usize, Fragment>,
    fragment_by_end: BTreeMap<usize, Fragment>,
    dx: u32,
    dy: u32,
    result: Vec<Ring>,
}

impl IsoRingBuilder {
    pub fn new(dx: u32, dy: u32) -> Self {
       IsoRingBuilder {
           fragment_by_start: BTreeMap::new(),
           fragment_by_end: BTreeMap::new(),
           dx: dx,
           dy :dy,
           result: Vec::new(),
       }
    }

    pub fn compute(&mut self, values: &[f64], value: f64) {
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
        CASES[((t1 as u32) << 1) as usize].iter().map(|ring| {
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();
        x += 1;
        while x < dx as i32 - 1 {
            t0 = t1;
            t1 = values[(x + 1) as usize] >= value;
            CASES[((t0 as u32) | (t1 as u32) << 1) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y);
            }).collect::<Vec<()>>();
            x += 1;
        }
        CASES[((t1 as u32) << 0) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();

        // General case for the intermediate rows.
        y += 1;
        while y < dy as i32 - 1 {
            x = -1;
            t1 = values[(y * dx as i32 + dx as i32) as usize] >= value;
            t2 = values[(y * dx as i32) as usize] >= value;
            CASES[((t1 as u32) << 1 | (t2 as u32) << 2) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y);
            }).collect::<Vec<()>>();
            x += 1;
            while x < dx as i32 - 1 {
               t0 = t1;
               t1 = values[(y * dx as i32 + dx as i32 + x + 1) as usize] >= value;
               t3 = t2;
               t2 = values[(y * dx as i32 + x + 1) as usize] >= value;
               CASES[((t0 as u32) | (t1 as u32) << 1 | (t2 as u32) << 2 | (t3 as u32) << 3) as usize].iter().map(|ring| {
                   self.stitch(&ring, x, y);
               }).collect::<Vec<()>>();
               x += 1;
           }
           CASES[((t1 as u32) | (t2 as u32) << 3) as usize].iter().map(|ring|{
               self.stitch(&ring, x, y);
           }).collect::<Vec<()>>();
           y += 1;
        }

        // Special case for the last row (y = dy - 1, t0 = t1 = 0).
        x = -1;
        t2 = values[(y * dx as i32) as usize] >= value;
        CASES[((t2 as u32) << 2) as usize].iter().map(|ring|{
            self.stitch(&ring, x, y);
        }).collect::<Vec<()>>();
        x += 1;
        while x < dx as i32 - 1 {
            t3 = t2;
            t2 = values[(y * dx as i32 + x + 1) as usize] >= value;
            CASES[((t2 as u32) << 2 | (t3 as u32) << 3) as usize].iter().map(|ring|{
                self.stitch(&ring, x, y);
            }).collect::<Vec<()>>();
            x += 1;
        }
        CASES[((t2 as u32) << 3) as usize].iter().map(|ring|{
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
        if self.fragment_by_end.contains_key(&start_index){
            // let mut f = self.fragment_by_end.get(&start_index).unwrap().clone();
            let mut f = self.fragment_by_end.remove(&start_index).unwrap();
            if self.fragment_by_start.contains_key(&end_index) {
                let (g_start, g_end) = get_start_end(&self.fragment_by_start, end_index);
                // self.fragment_by_end.remove(&f.end);
                self.fragment_by_start.remove(&g_start);
                if f.end == g_end && f.start == g_start {
                    f.ring.push(end);
                    self.result.push(f.ring.drain(..).collect());
                } else {
                    let temp;
                    {
                        let g = self.fragment_by_start.get(&end_index).unwrap();
                        f.ring.extend(g.ring.iter().cloned());
                        temp = f.ring.drain(..).collect();
                    }
                    self.fragment_by_start.insert(f.start, Fragment { start: f.start, end: g_end, ring: temp });
                    self.fragment_by_end.insert(g_end, self.fragment_by_start[&f.start].clone());
                }
            } else {
                // self.fragment_by_end.remove(&f.end);
                if let Some(a) = self.fragment_by_start.get_mut(&f.start) {
                    a.end = end_index;
                    a.ring.push(end.clone()); // a.ring = f.ring.clone();
                }
                f.ring.push(end);
                f.end = end_index;
                self.fragment_by_end.insert(end_index, f);
            }
        } else if self.fragment_by_start.contains_key(&end_index) {
            // let mut f = self.fragment_by_start.get(&end_index).unwrap().clone();
            let mut f = self.fragment_by_start.remove(&end_index).unwrap();
            if self.fragment_by_end.contains_key(&start_index) {
                let (g_start, g_end) = get_start_end(&self.fragment_by_end, start_index);
                // self.fragment_by_start.remove(&f.start);
                self.fragment_by_end.remove(&g_end);
                if f.end == g_end && f.start == g_start {
                    f.ring.push(end);
                    self.result.push(f.ring.drain(..).collect());
                } else {
                    let temp;
                    {
                        let g = self.fragment_by_end.get(&start_index).unwrap();
                        f.ring.extend(g.ring.iter().cloned());
                        temp = f.ring.drain(..).collect();
                    }
                    self.fragment_by_start.insert(g_start, Fragment { start: g_start, end: f.end, ring: temp });
                    self.fragment_by_end.insert(f.end, self.fragment_by_start[&g_start].clone());
                }
            } else {
                // self.fragment_by_start.remove(&f.start);
                if let Some(a) = self.fragment_by_end.get_mut(&f.end) {
                    a.start = start_index;
                    a.ring.insert(0, start.clone()); // a.ring = f.ring.clone();
                }
                f.ring.insert(0, start);
                f.start = start_index;
                self.fragment_by_start.insert(start_index, f);
            }
        } else {
            self.fragment_by_start.insert(start_index, Fragment { start: start_index, end: end_index, ring: vec![start, end] });
            self.fragment_by_end.insert(end_index, self.fragment_by_start[&start_index].clone());
        }
    }
}

fn get_start_end(map: &BTreeMap<usize, Fragment>, ix: usize) -> (usize, usize) {
    let frag = map.get(&ix).unwrap();
    (frag.start, frag.end)
}
