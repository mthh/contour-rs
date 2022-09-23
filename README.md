# contour-rs

[![Build status GitHub Actions](https://github.com/mthh/contour-rs/actions/workflows/build_test_ubuntu.yml/badge.svg)](https://github.com/mthh/contour-rs/actions/workflows/build_test_ubuntu.yml)
[![Build status Appveyor](https://ci.appveyor.com/api/projects/status/uemh49tq7vy4uke6?svg=true)](https://ci.appveyor.com/project/mthh/contour-rs)
[![Docs.rs version](https://docs.rs/contour/badge.svg)](https://docs.rs/contour/)

Computes *isorings* and __*contour polygons*__ by applying [marching squares](https://en.wikipedia.org/wiki/Marching_squares) to a rectangular array of numeric values.  
Outputs ring coordinates or polygons contours (represented using geo-types [MultiPolygon](https://docs.rs/geo-types/latest/geo_types/geometry/struct.MultiPolygon.html)s).
The generated contours can also easily be serialised to GeoJSON.

*Note : This is a port of [d3-contour](https://github.com/d3/d3-contour).*  

<div style="text-align:center"><a href="https://mthh.github.io/wasm_demo_contour/"><img src ="https://raw.githubusercontent.com/mthh/contour-rs/master/illustration.png" /></a></div><br>

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
contour = "0.7.0"
```

and this to your crate root:

```rust
extern crate contour;
```

The API exposes:
- a `contour_rings` function, which computes isorings coordinates for one threshold value (*returns a `Vec` of rings coordinates*).
- a `ContourBuilder` struct, which computes isorings coordinates for a `Vec` of threshold values and transform them in `Contour`s (a type containing the threshold value and the geometry as a MultiPolygon, easily serializable to GeoJSON).


### Example:

**Without defining origin and step:**

```rust
let c = ContourBuilder::new(10, 10, false); // x dim., y dim., smoothing
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
    0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
], &[0.5])?; // values, thresholds
```

**With origin and step**

```rust
let c = ContourBuilder::new(10, 10, true) // x dim., y dim., smoothing
    .x_step(2) // The horizontal coordinate for the origin of the grid.
    .y_step(2) // The vertical coordinate for the origin of the grid.
    .x_origin(100) // The horizontal step for the grid
    .y_origin(200); // The vertical step for the grid

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
], &[0.5]).unwrap(); // values, thresholds

```

**Using the `geojson` feature**

The `geojson` feature is not enabled by default, so you need to specify it in your `Cargo.toml`:

```toml
[dependencies]
contour = { version = "0.7.0", features = ["geojson"] }
```

```rust
let c = ContourBuilder::new(10, 10, true) // x dim., y dim., smoothing
    .x_step(2) // The horizontal coordinate for the origin of the grid.
    .y_step(2) // The vertical coordinate for the origin of the grid.
    .x_origin(100) // The horizontal step for the grid
    .y_origin(200); // The vertical step for the grid

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
], &[0.5]).unwrap(); // values, thresholds
println!("{:?}", res[0].to_geojson()); // prints the GeoJSON representation of the first contour
```

```rust
__*Output:*__
```rust
Feature {
    bbox: None,
    geometry: Some(Geometry {
        bbox: None,
        value: MultiPolygon([
            [[
                [110.0, 215.0], [110.0, 213.0], [110.0, 211.0], [110.0, 209.0],
                [110.0, 207.0], [109.0, 206.0], [107.0, 206.0], [106.0, 207.0],
                [106.0, 209.0], [106.0, 211.0], [106.0, 213.0], [106.0, 215.0],
                [107.0, 216.0], [109.0, 216.0], [110.0, 215.0]
            ]],
            [[
                [114.0, 215.0], [114.0, 213.0], [114.0, 211.0], [114.0, 209.0],
                [114.0, 207.0], [113.0, 206.0], [112.0, 207.0], [112.0, 209.0],
                [112.0, 211.0], [112.0, 213.0], [112.0, 215.0], [113.0, 216.0],
                [114.0, 215.0]
            ]]
        ]),
        foreign_members: None
    }),
    id: None,
    properties: Some({"threshold": Number(0.5)}),
    foreign_members: None
}
```

### Demo

Demo of this crate compiled to WebAssembly and used from JavaScript : [wasm_demo_contour](https://mthh.github.io/wasm_demo_contour/).

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
