use std::env;

use axum::{extract::Path, Json};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::f64;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Cycleway {
    way_id: i64,
    name: Option<String>,
    winter_service: Option<String>,
    line_string: LineString,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct LineString {
    points: Vec<(f64, f64)>,
}

impl From<Option<std::string::String>> for LineString {
    fn from(str: Option<std::string::String>) -> Self {
        match str {
            Some(str) => {
                let e = 2.71828182845904523536028747135266249775724709369995f64;
                let x1 = 20037508.34;
                let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
                let points = re
                    .captures_iter(str.as_str())
                    .map(|cap| {
                        let x = cap[1].parse::<f64>().unwrap();
                        let y = cap[2].parse::<f64>().unwrap();

                        // Tranform to from EPSG:3857 to EPSG:4326 (lat,long)
                        let x = x / x1 * 180.0;
                        let y = y / (x1 / 180.0);
                        let exponent = y * (std::f64::consts::PI / 180.0);
                        let y = e.powf(exponent).atan();
                        let y = y / (std::f64::consts::PI / 360.0);
                        let y = y - 90.;

                        (x, y)
                    })
                    .collect::<Vec<(f64, f64)>>();
                LineString { points }
            }
            None => LineString { points: vec![] },
        }
    }
}

pub async fn cycleway(Path(way_id): Path<i64>) -> Json<Cycleway> {
    let conn = PgPool::connect(format!("{}", env::var("DATABASE_URL").unwrap()).as_str())
        .await
        .unwrap();

    let response = sqlx::query_as!(
        Cycleway,
        r#"select way_id,
                                  name, 
                                  winter_service, 
                                  ST_AsText(geom) as line_string  
                                  from cycleway where way_id = $1"#,
        way_id
    )
    .fetch_one(&conn)
    .await;

    Json(response.unwrap())
}
