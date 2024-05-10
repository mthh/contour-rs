use crate::Float;
use geo_types::MultiPolygon;

/// An isoband has the geometry and min / max values of a contour ring, built by [ContourBuilder](`crate::contourbuilder::ContourBuilder`).
#[derive(Debug, Clone)]
pub struct Band {
    pub(crate) geometry: MultiPolygon<Float>,
    pub(crate) min_v: Float,
    pub(crate) max_v: Float,
}

impl Band {
    /// Borrow the [`MultiPolygon`](geo_types::MultiPolygon) geometry of this contour.
    pub fn geometry(&self) -> &MultiPolygon<Float> {
        &self.geometry
    }

    /// Get the owned polygons and thresholds (min and max) of this band.
    pub fn into_inner(self) -> (MultiPolygon<Float>, Float, Float) {
        (self.geometry, self.min_v, self.max_v)
    }

    /// Get the minimum value used to construct this band.
    pub fn min_v(&self) -> Float {
        self.min_v
    }

    /// Get the maximum value used to construct this band.
    pub fn max_v(&self) -> Float {
        self.max_v
    }

    #[cfg(feature = "geojson")]
    /// Convert the band to a struct from the `geojson` crate.
    ///
    /// To get a string representation, call to_geojson().to_string().
    /// ```
    /// use contour::ContourBuilder;
    ///
    /// let builder = ContourBuilder::new(10, 10, false);
    /// # #[rustfmt::skip]
    /// let contours = builder.isobands(&[
    /// // ...ellided for brevity
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 2., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 1., 1., 1., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    /// #     0., 0., 0., 0., 0., 0., 0., 0., 0., 0.
    /// ], &[0.5, 1.5, 2.5]).unwrap();
    ///
    /// let geojson_string = contours[0].to_geojson().to_string();
    ///
    /// assert_eq!(&geojson_string[0..27], r#"{"geometry":{"coordinates":"#);
    /// ```
    pub fn to_geojson(&self) -> geojson::Feature {
        let mut properties = geojson::JsonObject::with_capacity(2);
        properties.insert("min_v".to_string(), self.min_v.into());
        properties.insert("max_v".to_string(), self.max_v.into());

        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(self.geometry())),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        }
    }
}
