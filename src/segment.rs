use axum::{extract::{Path, State}, Json};
use crate::db::cycleway::{Cycleway, Route};
use futures::future::join_all;
use regex::Regex;
use crate::{VeloInfoError, VeloinfoState};

pub async fn select(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
) -> Result<Json<Cycleway>, VeloInfoError> {
    let conn = state.conn;
    let searched_segment: Cycleway = Cycleway::get(way_id, conn.clone()).await?;

    Ok(Json(searched_segment))
}

pub async fn route(
    State(state): State<VeloinfoState>,
    Path((way_id1, way_ids)): Path<(i64, String)>,
) -> Result<Json<Route>, VeloInfoError> {
    let re = Regex::new(r"\d+").unwrap();
    let way_ids_i64 = re.find_iter(&way_ids)
        .map(|m| m.as_str().parse::<i64>().unwrap()).collect::<Vec<i64>>();
    let conn = state.conn;
    let start_segment: Cycleway = Cycleway::get(way_id1, conn.clone()).await?;
    let searched_segments: Route = join_all(way_ids_i64
        .iter()
        .map(|way_id| { 
            Cycleway::get(*way_id, conn.clone())
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
    let name = start_segment.name.unwrap_or("Non inconnu".to_string());

    let mut routes: Vec<Route> = vec![];
    // We try to find the longest path between the 4 possible combinations
    // It is not the best way to do it, but it is the simplest
    routes.push(
        Cycleway::route(
            name.as_str(),
            start_segment.source.unwrap(),
            searched_segments.target,
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            name.as_str(),
            start_segment.target.unwrap(),
            searched_segments.source,
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            name.as_str(),
            start_segment.source.unwrap(),
            searched_segments.source,
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
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
