use anyhow::Result;
use axum::extract::State;
use axum::{extract::Path, Json};
use futures::future::join_all;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

use crate::{VeloInfoError, VeloinfoState};

#[derive(Debug, sqlx::FromRow)]
struct ResponseDb {
    name: Option<String>,
    way_id: Option<i64>,
    geom: Option<String>,
    source: Option<i64>,
    target: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub name: Option<String>,
    pub way_id: Option<i64>,
    pub geom: Option<Vec<[f64; 2]>>,
    pub source: Option<i64>,
    pub target: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct RouteDB {
    way_id: i64,
    source: i64,
    target: i64,
    geom: String,
}

impl From<&RouteDB> for Route {
    fn from(response: &RouteDB) -> Self {
        let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
        let points = re
            .captures_iter(response.geom.as_str())
            .map(|cap| {
                let x = cap[1].parse::<f64>().unwrap();
                let y = cap[2].parse::<f64>().unwrap();
                [x, y]
            })
            .collect::<Vec<[f64; 2]>>();
        Route {
            way_ids: vec![response.way_id],
            geom: points,
            source: response.source,
            target: response.target,
        }
    }
}

impl Segment {
    pub async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<Segment> {
        let response: ResponseDb = sqlx::query_as(
            r#"select
                name,  
                way_id,
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

    async fn route(name: &str, source: i64, target: i64, conn: sqlx::Pool<Postgres>) -> Result<Route> {
        println!("name: {}, source: {}, target: {}",name, source, target);
        let responses: Vec<RouteDB> = sqlx::query_as(
            r#"select   way_id,
                        $2 as source,
                        $3 as target, 
                        ST_AsText(ST_Transform(geom, 4326)) as geom,
                        path_seq
                from pgr_bdastar(
                    FORMAT(
                        $FORMAT$
                        select  way_id as id, 
                            source, 
                            target, 
                            st_length(geom) as cost, 
                            st_length(geom) as reverse_cost, 
                            st_x(st_startpoint(geom)) as x1,
                            st_y(st_startpoint(geom)) as y1,
                            st_x(st_endpoint(geom)) as x2,
                            st_y(st_endpoint(geom)) as y2
                        FROM cycleway
                        $FORMAT$,
                        $1
                    )
                , 
                $2, 
                $3
                ) as pa join cycleway c on pa.edge = c.way_id
                order by path_seq asc"#,
        )
        .bind(name)
        .bind(source)
        .bind(target)
        .fetch_all(&conn)
        .await
        .unwrap();
        println!("responses: {:?}", responses);
        let segment: Route = responses.iter().fold(
            Route {
                way_ids: Vec::new(),
                geom: vec![],
                source: source,
                target: target,
            },
            |mut acc, response| {
                let this_merge: Route = response.into();
                acc.way_ids.extend(this_merge.way_ids);
                acc.geom.extend(this_merge.geom);
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
                    name: response.name.clone(),
                    way_id: response.way_id,
                    geom: Some(points),
                    source: response.source,
                    target: response.target,
                }
            }
            None => Segment {
                name: None,
                way_id: None,
                geom: None,
                source: None,
                target: None,
            },
        }
    }
}

pub async fn select(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
) -> Result<Json<Segment>, VeloInfoError> {
    let conn = state.conn;
    let searched_segment: Segment = Segment::get(way_id, conn.clone()).await?;

    Ok(Json(searched_segment))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route {
    pub way_ids: Vec<i64>,
    pub geom: Vec<[f64; 2]>,
    pub source: i64,
    pub target: i64,
}

pub async fn route(
    State(state): State<VeloinfoState>,
    Path((way_id1, way_ids)): Path<(i64, String)>,
) -> Result<Json<Route>, VeloInfoError> {
    let re = Regex::new(r"\d+").unwrap();
    let way_ids_i64 = re.find_iter(&way_ids)
        .map(|m| m.as_str().parse::<i64>().unwrap()).collect::<Vec<i64>>();
    println!("way_ids_i64: {:?}", way_ids_i64);
    let conn = state.conn;
    let start_segment: Segment = Segment::get(way_id1, conn.clone()).await?;
    let searched_segments: Route = join_all(way_ids_i64
        .iter()
        .map(|way_id| { 
            Segment::get(*way_id, conn.clone())
        })).await.iter()
        .fold(
            Route {
                way_ids: vec![],
                geom: vec![],
                source: 0,
                target: 0,
            },
            |acc, segment| {
                let segment = segment.as_ref().unwrap();
                let mut acc = acc;
                acc.way_ids.push(segment.way_id.unwrap());
                acc.geom.extend(segment.geom.as_ref().unwrap());
                if acc.source == 0 {
                    acc.source = segment.source.unwrap();
                }
                acc.target = segment.target.unwrap();
                acc
            },
        );
    println!("start_segment: {:?}", start_segment);
    println!("searched_segment: {:?}", searched_segments);
    let name = start_segment.name.unwrap_or("Non inconnu".to_string());

    let mut routes: Vec<Route> = vec![];
    // We try to find the longest path between the 4 possible combinations
    // It is not the best way to do it, but it is the simplest
    routes.push(
        Segment::route(
            name.as_str(),
            start_segment.source.unwrap(),
            searched_segments.target,
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Segment::route(
            name.as_str(),
            start_segment.target.unwrap(),
            searched_segments.source,
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Segment::route(
            name.as_str(),
            start_segment.source.unwrap(),
            searched_segments.source,
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Segment::route(
            name.as_str(),
            start_segment.target.unwrap(),
            searched_segments.target,
            conn.clone(),
        )
        .await?,
    );
    println!("segments: {:?}", routes);

    // We keep the longest segment
    let merge = routes
        .iter()
        .max_by(|x, y| x.geom.len().cmp(&y.geom.len()))
        .expect("no bigger segment")
        .to_owned();
    Ok(Json(merge))
}
