use crate::db::cycleway::{Cycleway, Route};
use crate::{VeloInfoError, VeloinfoState};
use axum::{
    extract::{Path, State},
    Json,
};
use futures::future::join_all;
use regex::Regex;

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
    let re = Regex::new(r"\d+")?;
    let conn = state.conn;
    let way_ids_i64 = re
        .find_iter(&way_ids)
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    let start_segment: Cycleway = Cycleway::get(way_id1, conn.clone()).await?;
    let cycleways = join_all(
        way_ids_i64
            .iter()
            .map(|way_id| async { Cycleway::get(*way_id, conn.clone()).await.unwrap() }),
    )
    .await;

    let mut routes: Vec<Route> = vec![];
    // We try to find the longest path between the 4 possible combinations
    // It is not the best way to do it, but it is the simplest
    routes.push(
        Cycleway::route(
            &start_segment.source.unwrap(),
            &cycleways.first().unwrap().target.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.source.unwrap(),
            &cycleways.first().unwrap().source.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.source.unwrap(),
            &cycleways.last().unwrap().target.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.source.unwrap(),
            &cycleways.last().unwrap().source.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.target.unwrap(),
            &cycleways.first().unwrap().target.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.target.unwrap(),
            &cycleways.first().unwrap().source.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.target.unwrap(),
            &cycleways.last().unwrap().target.unwrap(),
            conn.clone(),
        )
        .await?,
    );
    routes.push(
        Cycleway::route(
            &start_segment.target.unwrap(),
            &cycleways.last().unwrap().source.unwrap(),
            conn.clone(),
        )
        .await?,
    );

    // We keep the longest segment
    let merge = routes
        .iter()
        .max_by(|x, y| x.geom.len().cmp(&y.geom.len()))
        .expect("no bigger segment")
        .to_owned();
    Ok(Json(merge))
}
