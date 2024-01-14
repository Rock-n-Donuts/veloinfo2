use std::env;

use axum::http::response;
use axum::{extract::Path, http::StatusCode, Json};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::{Decode, PgPool};
use std::f64;

#[derive(Debug, sqlx::FromRow)]
struct Response {
    geom: Option<String>,
    source: Option<i64>,
    target: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response2 {
    geom: Option<Vec<[f64; 2]>>,
    source: Option<i64>,
    target: Option<i64>,
}

pub async fn cycleway(
    Path(way_id): Path<i64>,
    Json(coordinate): Json<Response2>,
) -> Result<Json<Response2>, StatusCode> {
    let conn = PgPool::connect(format!("{}", env::var("DATABASE_URL").unwrap()).as_str())
        .await
        .unwrap();

    let response: Response = sqlx::query_as(
        r#"select  
            source,
            target,
            ST_AsText(ST_Transform(geom, 4326)) as geom  
            from cycleway where way_id = $1"#,
    )
    .bind(way_id)
    .fetch_one(&conn)
    .await
    .unwrap();
    let response = from_response_to_response2(response);
    if coordinate.geom.is_none() {
        Ok(Json(response))
    } else {
        println!("coordinate {:?}", coordinate);
        let response: Vec<Response> = sqlx::query_as(
            r#"select source, target, ST_AsText(ST_Transform(geom, 4326)) as geom from pgr_bdastar(
                'select  way_id as id, 
                        source, 
                        target, 
                        st_length(geom) as cost, 
                        st_length(geom) as reverse_cost, 
                        st_x(st_startpoint(geom)) as x1,
                        st_y(st_startpoint(geom)) as y1,
                        st_x(st_endpoint(geom)) as x2,
                        st_y(st_endpoint(geom)) as y2
                FROM cycleway ', 
                $1, 
                $2
                ) as pa join cycleway c on pa.edge = c.way_id"#,
        )
        .bind(coordinate.clone().source.unwrap())
        .bind(response.target.unwrap())
        .fetch_all(&conn)
        .await
        .unwrap();
        let source = coordinate.clone().source;
        let target = coordinate.target;
        println!("response {:?}", response);
        let response = response
            .into_iter()
            .map(from_response_to_response2)
            .fold(
                Response2 {
                    geom: Some(vec![]),
                    source: source,
                    target: target,
                } , | mut acc, response | {
                    acc.geom.as_mut().unwrap().append(&mut response.geom.unwrap().clone());
                    acc
                },
            );

        Ok(Json(response))
    }
}

fn from_response_to_response2<'a>(response: Response) -> Response2 {
    match response.geom {
        Some(str) => {
            let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
            let points = re
                .captures_iter(str.as_str())
                .map(|cap| {
                    let x = cap[1].parse::<f64>().unwrap();
                    let y = cap[2].parse::<f64>().unwrap();

                    [x, y]
                })
                .collect::<Vec<[f64; 2]>>();
            Response2 {
                geom: Some(points),
                source: response.source,
                target: response.target,
            }
        }
        None => Response2 {
            geom: None,
            source: None,
            target: None,
        },
    }
}
