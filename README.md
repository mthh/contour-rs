# contour-rs

Warning: work in progress  

[![Build Status](https://travis-ci.org/mthh/contour-rs.svg?branch=master)](https://travis-ci.org/mthh/contour-rs)  

Computes *isorings* and __*contour polygons*__ by applying [marching squares](https://en.wikipedia.org/wiki/Marching_squares) to a rectangular array of numeric values.  
This is a port of [d3-contour](https://github.com/d3/d3-contour).  
Outputs ring coordinates or polygons contours as a `Vec` of [GeoJSON](https://github.com/georust/rust-geojson) [Feature](https://docs.rs/geojson/0.9.1/geojson/struct.Feature.html).  


The API exposes:
- an `IsoRingBuilder` struct, which computes isorings coordinates for a threshold value (-> returns a `Vec` of rings coordinates).
- a `ContourBuilder` struct, which computes isorings coordinates for a `Vec` of threshold values and transform them in `MultiPolygon`s (-> returns a `Vec` of GeoJSON Features).


#### Simple example:

```rust
let c = ContourBuilder::new(vec![0.5], 10, 10); // Vec of thresholds, x dim., y dim.
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
```
__*Output:*__
```rust
[Feature {
  bbox: None,
  geometry: Some(Geometry {
    bbox: None,
    value: MultiPolygon([[[[6, 7.5], [6, 6.5], [6, 5.5], [6, 4.5], [6, 3.5], [5.5, 3], [4.5, 3], [3.5, 3], [3, 3.5], [3, 4.5], [3, 5.5], [3, 6.5], [3, 7.5], [3.5, 8], [4.5, 8], [5.5, 8], [6, 7.5]]]]),
    foreign_members: None
    }),
   id: None,
   properties: Some({"value": Number(0.5)}),
   foreign_members: None
   }]
```


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
