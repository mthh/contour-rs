use contour::{ContourBuilder, Float};
use geojson::{FeatureCollection, GeoJson};
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let pot_pop_fr = include_str!("../tests/fixtures/pot_pop_fr.json");
    let raw_data: serde_json::Value = serde_json::from_str(pot_pop_fr).unwrap();
    let matrix: Vec<Float> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            #[cfg(not(feature = "f32"))]
            {
                x.as_f64().unwrap()
            }
            #[cfg(feature = "f32")]
            {
                x.as_f64().unwrap() as f32
            }
        })
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    let x_origin = -6.144721171428571;
    let y_origin = 51.78171334283718;
    let x_step = 0.11875873095057177;
    let y_step = -0.08993203637245273;

    let contours = ContourBuilder::new(w, h, true)
        .x_step(x_step)
        .y_step(y_step)
        .x_origin(x_origin)
        .y_origin(y_origin)
        .isobands(
            &matrix,
            &[
                0., 105483.25, 310000., 527416.25, 850000., 1054832.5, 2109665., 3164497.5,
                4219330., 5274162.5, 6328995., 7383827.5, 8438660., 9704459., 10548326.,
            ],
        )
        .unwrap();

    let features = contours
        .iter()
        .map(|contour| contour.to_geojson())
        .collect::<Vec<geojson::Feature>>();

    let geojson_str = GeoJson::from(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
    .to_string();

    let mut file_writer = BufWriter::new(File::create("/tmp/example-output.geojson").unwrap());
    file_writer.write(&geojson_str.as_bytes()).unwrap();

    let volcano = include_str!("../tests/fixtures/volcano.json");
    let raw_data: serde_json::Value = serde_json::from_str(volcano).unwrap();
    let matrix: Vec<Float> = raw_data["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| {
            #[cfg(not(feature = "f32"))]
            {
                x.as_f64().unwrap()
            }
            #[cfg(feature = "f32")]
            {
                x.as_f64().unwrap() as f32
            }
        })
        .collect();
    let h = raw_data["height"].as_u64().unwrap() as usize;
    let w = raw_data["width"].as_u64().unwrap() as usize;

    let contours = ContourBuilder::new(w, h, true)
        .isobands(
            &matrix,
            &[
                90., 95., 100., 105., 110., 115., 120., 125., 130., 135., 140., 145., 150., 155.,
                160., 165., 170., 175., 180., 185., 190., 195., 200.,
            ],
        )
        .unwrap();

    let features = contours
        .iter()
        .map(|contour| contour.to_geojson())
        .collect::<Vec<geojson::Feature>>();

    let geojson_str = GeoJson::from(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
    .to_string();

    let mut file_writer = BufWriter::new(File::create("/tmp/example-output2.geojson").unwrap());
    file_writer.write(&geojson_str.as_bytes()).unwrap();
}
