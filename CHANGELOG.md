## Changelog

### 0.9.0 (2023-xx-xx)

- Add support for building isolines as MultiLineStrings (instead of solely building contour polygons as MultiPolygons).

- Improve some minor details in the documentation.

### 0.8.0 (2023-02-21)

- Be less restrictive about the geo-types version and use geo_types::Coord instead of deprecated geo_types::Coordinate.

- Update to Rust 2021 edition.

### 0.7.0 (2022-09-23)

- BREAKING: Make geojson optional, use geo-types for geometry representation
  (thanks to @michaelkirk, see #5 and #6 for details).

- BREAKING: Rename the "value" field to "threshold" in the GeoJSON representation.

- Add `x_origin`, `y_origin`, `x_step` and `y_step`
  attributes to `ContourBuilder` struct. They can be set using the *builder pattern*, before calling
  the `contours` method.

- Create this changelog and complete it retroactively.

### 0.6.0 (2022-09-15)

- Bump maximum supported geojson version to 0.24.

### 0.5.0 (2022-06-25)

- Bump maximum supported geojson version to 0.23.

### 0.4.0 (2021-02-09)

- Bump maximum supported geojson version to 0.22.

### 0.3.0 (2020-12-08)

- Support a range of geojson crate versions instead of a specific one.

### 0.2.0 (2020-07-25)

- Modernize error handling.
- Bump supported geojson version to 0.19.

### 0.1.0 (2019-01-21)

- First version