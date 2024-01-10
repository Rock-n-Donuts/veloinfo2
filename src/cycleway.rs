use std::env;

use axum::{extract::Path, Json};
use regex::Regex;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Cycleway {
    way_id: i64,
    name: Option<String>,
    winter_service: Option<String>,
    geom: LineString,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct LineString {
    points: Vec<(f64, f64)>,
}

impl From<Option<std::string::String>> for LineString {
    fn from(str: Option<std::string::String>) -> Self {
        match str {
            Some(str) => {
                let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
                let points = re.captures_iter(str.as_str()).map(|cap| {
                    let x = cap[1].parse::<f64>().unwrap();
                    let y = cap[2].parse::<f64>().unwrap();
                    (x, y)
                }).collect::<Vec<(f64, f64)>>();
                LineString { points }
            }
            None => LineString { points: vec![] }
            
        }
    }
}

pub async fn cycleway(Path(way_id): Path<i64>) -> Json<Cycleway> {
    let conn = PgPool::connect(format!("{}", env::var("DATABASE_URL").unwrap()).as_str())
        .await
        .unwrap();

    let response = sqlx::query_as!(Cycleway, r#"select way_id,
                                  name, 
                                  winter_service, 
                                  ST_AsText(geom) as geom  
                                  from cycleway where way_id = $1"#, way_id)
        .fetch_one(&conn)
        .await;

    println!("response: {:?}", response);
    Json(response.unwrap())
}
