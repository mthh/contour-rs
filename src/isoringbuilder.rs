use crate::error::{new_error, ErrorKind, Result};
use crate::{Float, Pt, Ring};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use slab::Slab;

lazy_static! {
    #[rustfmt::skip]
    static ref CASES: Vec<Vec<Vec<Vec<Float>>>> = vec![
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
pub fn contour_rings(values: &[Float], threshold: Float, dx: usize, dy: usize) -> Result<Vec<Ring>> {
    let mut isoring = IsoRingBuilder::new(dx, dy);
    isoring.compute(values, threshold)
}

/// Isoring generator to compute marching squares with isolines stitched into rings.
pub struct IsoRingBuilder {
    fragment_by_start: FxHashMap<usize, usize>,
    fragment_by_end: FxHashMap<usize, usize>,
    f: Slab<Fragment>,
    dx: usize,
    dy: usize,
    is_empty: bool,
}

impl IsoRingBuilder {
    /// Constructs a new IsoRing generator for a grid with `dx` * `dy` dimension.
    /// # Arguments
    ///
    /// * `dx` - The number of columns in the grid.
    /// * `dy` - The number of rows in the grid.
    pub fn new(dx: usize, dy: usize) -> Self {
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
    pub fn compute(&mut self, values: &[Float], threshold: Float) -> Result<Vec<Ring>> {
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
        let dx = self.dx as i64;
        let dy = self.dy as i64;
        let mut x = -1;
        let mut y = -1;
        let mut t0;
        let mut t1;
        let mut t2;
        let mut t3;

        // Special case for the first row (y = -1, t2 = t3 = 0).
        t1 = (values[0] >= threshold) as usize;
        case_stitch!(t1 << 1, x, y, &mut result);
        x += 1;
        while x < dx - 1 {
            t0 = t1;
            t1 = (values[(x + 1) as usize] >= threshold) as usize;
            case_stitch!(t0 | t1 << 1, x, y, &mut result);
            x += 1;
        }
        case_stitch!(t1, x, y, &mut result);

        // General case for the intermediate rows.
        y += 1;
        while y < dy - 1 {
            x = -1;
            t1 = (values[(y * dx + dx) as usize] >= threshold) as usize;
            t2 = (values[(y * dx) as usize] >= threshold) as usize;
            case_stitch!(t1 << 1 | t2 << 2, x, y, &mut result);
            x += 1;
            while x < dx - 1 {
                t0 = t1;
                t1 = (values[(y * dx + dx + x + 1) as usize] >= threshold) as usize;
                t3 = t2;
                t2 = (values[(y * dx + x + 1) as usize] >= threshold) as usize;
                case_stitch!(
                    t0 | t1 << 1 | t2 << 2 | t3 << 3,
                    x,
                    y,
                    &mut result
                );
                x += 1;
            }
            case_stitch!(t1 | t2 << 3, x, y, &mut result);
            y += 1;
        }

        // Special case for the last row (y = dy - 1, t0 = t1 = 0).
        x = -1;
        t2 = (values[(y * dx) as usize] >= threshold) as usize;
        case_stitch!(t2 << 2, x, y, &mut result);
        x += 1;
        while x < dx - 1 {
            t3 = t2;
            t2 = (values[(y * dx + x + 1) as usize] >= threshold) as usize;
            case_stitch!(t2 << 2 | t3 << 3, x, y, &mut result);
            x += 1;
        }
        case_stitch!(t2 << 3, x, y, &mut result);
        self.is_empty = false;
        Ok(result)
    }

    fn index(&self, point: &Pt) -> usize {
        (point.x * 2.0 + point.y * (self.dx as Float + 1.) * 4.) as usize
    }

    // Stitchs segments to rings.
    fn stitch(
        &mut self,
        line: &[Vec<Float>],
        x: i64,
        y: i64,
        result: &mut Vec<Ring>,
    ) -> Result<()> {
        let start = Pt {
            x: line[0][0] + x as Float,
            y: line[0][1] + y as Float,
        };
        let end = Pt {
            x: line[1][0] + x as Float,
            y: line[1][1] + y as Float,
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
                let f = self
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
                let f = self
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
