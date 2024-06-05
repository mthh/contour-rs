use crate::{Float, GridValue};
use geo_types::MultiLineString;

/// A line has the geometry and threshold of a contour ring, built by [ContourBuilder](`crate::contourbuilder::ContourBuilder`).
#[derive(Debug, Clone)]
pub struct Line<V: GridValue> {
    pub(crate) geometry: MultiLineString<Float>,
    pub(crate) threshold: V,
}

impl<V: GridValue> Line<V> {
    /// Borrow the [`MultiLineString`](geo_types::MultiLineString) geometry of this contour.
    pub fn geometry(&self) -> &MultiLineString<Float> {
        &self.geometry
    }

    /// Get the owned lines and threshold of this contour.
    pub fn into_inner(self) -> (MultiLineString<Float>, V) {
        (self.geometry, self.threshold)
    }

    /// Get the threshold used to construct this isoline.
    pub fn threshold(&self) -> V {
        self.threshold
    }
}

#[cfg(feature = "geojson")]
impl<V: GridValue + serde::Serialize> Line<V> {
    /// Convert the line to a struct from the `geojson` crate.
    ///
    /// To get a string representation, call to_geojson().to_string().
    /// ```
    /// use contour::ContourBuilder;
    ///
    /// let builder = ContourBuilder::new(10, 10, false);
    /// # #[rustfmt::skip]
    /// let contours = builder.lines(&[
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
    /// let geojson_string = contours[0].to_geojson().unwrap().to_string();
    ///
    /// assert_eq!(&geojson_string[0..27], r#"{"geometry":{"coordinates":"#);
    /// ```
    pub fn to_geojson(&self) -> crate::Result<geojson::Feature> {
        let mut properties = geojson::JsonObject::with_capacity(1);
        properties.insert(
            "threshold".to_string(),
            serde_json::to_value(self.threshold)?,
        );

        Ok(geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::from(self.geometry())),
            id: None,
            properties: Some(properties),
            foreign_members: None,
        })
    }
}
