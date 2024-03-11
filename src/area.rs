use crate::{Float, Pt};

pub fn area(ring: &[Pt]) -> Float {
    let n = ring.len();
    let mut area = ring[n - 1].y * ring[0].x - ring[n - 1].x * ring[0].y;
    for i in 1..n {
        area += ring[i - 1].y * ring[i].x - ring[i - 1].x * ring[i].y;
    }
    // Note that in the shoelace formula you need to divide this result by 2 to get the actual area.
    // Here we skip this division because we only use this area formula to calculate the winding
    // order of polygons and to compare their relative sizes.
    area
}

pub fn contains(ring: &[Pt], hole: &[Pt]) -> i32 {
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
    0
}

fn ring_contains(ring: &[Pt], point: &Pt) -> i32 {
    let x = point.x;
    let y = point.y;
    let n = ring.len();
    let mut contains = -1;
    let mut j = n - 1;
    for i in 0..n {
        let pi = &ring[i];
        let xi = pi.x;
        let yi = pi.y;
        let pj = &ring[j];
        let xj = pj.x;
        let yj = pj.y;
        if segment_contains(pi, pj, point) {
            return 0;
        }
        if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
            contains = -contains;
        }
        j = i;
    }
    contains
}

fn segment_contains(a: &Pt, b: &Pt, c: &Pt) -> bool {
    if collinear(a, b, c) {
        if (a.x - b.x).abs() < Float::EPSILON {
            within(a.y, c.y, b.y)
        } else {
            within(a.x, c.x, b.x)
        }
    } else {
        false
    }
}

fn collinear(a: &Pt, b: &Pt, c: &Pt) -> bool {
    ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs() < Float::EPSILON
}

fn within(p: Float, q: Float, r: Float) -> bool {
    p <= q && q <= r || r <= q && q <= p
}
