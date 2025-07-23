use crate::Float;
use geo_types::MultiPolygon;

/// A contour has the geometry and threshold of a contour ring, built by [ContourBuilder](`crate::contourbuilder::ContourBuilder`).
#[derive(Debug, Clone)]
pub struct Contour {
    pub(crate) geometry: MultiPolygon<Float>,
    pub(crate) threshold: Float,
}

impl Contour {
    /// Borrow the [`MultiPolygon`](geo_types::MultiPolygon) geometry of this contour.
    pub fn geometry(&self) -> &MultiPolygon<Float> {
        &self.geometry
    }

    /// Get the owned polygons and threshold of this contour.
    pub fn into_inner(self) -> (MultiPolygon<Float>, Float) {
        (self.geometry, self.threshold)
    }

    /// Get the threshold used to construct this contour.
    pub fn threshold(&self) -> Float {
        self.threshold
    }

    #[cfg(feature = "geojson")]
    /// Convert the contour to a struct from the `geojson` crate.
    ///
    /// To get a string representation, call to_geojson().to_string().
    /// ```
    /// use contour::ContourBuilder;
    ///
    /// let builder = ContourBuilder::new(10, 10, false);
    /// # #[rustfmt::skip]
    /// let contours = builder.contours(&[
    /// // ...ellided for brevity
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 2., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 2., 1., 2., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
    /// ], &[0.5]).unwrap();
    ///
    /// let geojson_string = contours[0].to_geojson().to_string();
    ///
    /// assert_eq!(&geojson_string[0..27], r#"{"type":"Feature","geometry"#);
    /// ```
    pub fn to_geojson(&self) -> geojson::Feature {
        let mut properties = geojson::JsonObject::with_capacity(1);
        properties.insert("threshold".to_string(), self.threshold.into());

        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(self.geometry())),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        }
    }
}
