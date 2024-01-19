use askama_axum::IntoResponse;
use axum::extract::State;
use axum::{extract::Path, http::StatusCode, Json};
use axum::response::Response;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;
use anyhow::Result;

use crate::VeloinfoState;

#[derive(Debug, sqlx::FromRow)]
struct ResponseDb {
    geom: Option<String>,
    source: Option<i64>,
    target: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    geom: Option<Vec<[f64; 2]>>,
    source: Option<i64>,
    target: Option<i64>,
}

impl Segment {
    async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<Segment> {
        let response: ResponseDb = sqlx::query_as(
            r#"select  
                source,
                target,
                ST_AsText(ST_Transform(geom, 4326)) as geom  
               from cycleway where way_id = $1"#,
        )
        .bind(way_id)
        .fetch_one(&conn)
        .await?;
        Ok(response.into())
    }

    async fn route(source: i64, target: i64, conn: sqlx::Pool<Postgres>) -> Result<Segment> {
        let responses: Vec<ResponseDb> = sqlx::query_as(
            r#"select $1 as source,
                        $2 as target, 
                        ST_AsText(ST_Transform(geom, 4326)) as geom 
                from pgr_bdastar(
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
        .bind(source)
        .bind(target)
        .fetch_all(&conn)
        .await?;
        let segment: Segment= responses.iter().fold(
            Segment {
                geom: Some(vec![]),
                source: Some(source),
                target: Some(target),
            },
            |mut acc, response| {
                let this_segement: Segment = response.into();
                acc.geom
                    .as_mut()
                    .unwrap()
                    .extend(this_segement.geom.unwrap());
                acc
            },
        );
        Ok(segment)
    }
}

impl From<ResponseDb> for Segment {
    fn from(response: ResponseDb) -> Self {
        Segment::from(&response)
    }
}

impl From<&ResponseDb> for Segment {
    fn from(response: &ResponseDb) -> Self {
        match response.geom.as_ref() {
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
                Segment {
                    geom: Some(points),
                    source: response.source,
                    target: response.target,
                }
            }
            None => Segment {
                geom: None,
                source: None,
                target: None,
            },
        }
    }
}

pub struct SegmentError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for SegmentError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for SegmentError {
    fn from(error: anyhow::Error) -> Self {
        SegmentError(error)
    }
}
impl From<sqlx::Error> for SegmentError {
    fn from(error: sqlx::Error) -> Self {
        // Votre logique de conversion ici
        // Par exemple, si SegmentError est une énumération avec une variante Sqlx :
        SegmentError(anyhow::Error::from(error))
    }
}
pub async fn select(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>
) -> Result<Json<Segment>, SegmentError> {
    let conn = state.conn;
    let searched_segment: Segment = Segment::get(way_id, conn.clone()).await?;

        Ok(Json(searched_segment))
}

pub async fn merge(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
    Json(start_segment): Json<Segment>,
) -> Result<Json<Segment>, SegmentError> {
    let conn = state.conn;
    let searched_segment: Segment = Segment::get(way_id, conn.clone()).await?;

    if start_segment.geom.is_none() {
        Ok(Json(searched_segment))
    } else {
        let mut segments: Vec<Segment> = vec![];
        // We try to find the longest path between the 4 possible combinations
        // It is not the best way to do it, but it is the simplest
        segments.push(
            Segment::route(
                start_segment.source.unwrap(),
                searched_segment.target.unwrap(),
                conn.clone(),
            )
            .await?,
        );
        segments.push(
            Segment::route(
                start_segment.target.unwrap(),
                searched_segment.source.unwrap(),
                conn.clone(),
            )
            .await?,
        );
        segments.push(
            Segment::route(
                start_segment.source.unwrap(),
                searched_segment.source.unwrap(),
                conn.clone(),
            )
            .await?,
        );
        segments.push(
            Segment::route(
                start_segment.target.unwrap(),
                searched_segment.target.unwrap(),
                conn.clone(),
            )
            .await?,
        );
        segments.iter().for_each(|segment| {
            println!("{:?}", segment);
        });

        // We keep the longest segment
        let segment = segments
            .iter()
            .max_by(|x, y| {
                x.geom
                    .as_ref()
                    .unwrap()
                    .len()
                    .cmp(&y.geom.as_ref().unwrap().len())
            }).expect("no bigger segment").to_owned();
        Ok(Json(segment))
    }
}