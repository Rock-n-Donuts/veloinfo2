use crate::db::cycleway::{Cycleway, Node};
use crate::db::edge::{Edge, Point};
use crate::VeloinfoState;
use axum::{
    extract::{Path, State},
    Json,
};
use axum_macros::debug_handler;
use futures::future::join_all;

#[debug_handler]
pub async fn select_node(
    State(state): State<VeloinfoState>,
    Path((lng, lat)): Path<(f64, f64)>,
) -> Json<Node> {
    let conn = state.conn;
    match Cycleway::find(&lng, &lat, conn.clone()).await {
        Ok(response) => Json(response),
        Err(e) => {
            eprintln!("Error while fetching node: {}", e);
            Json(Node {
                way_id: 0,
                geom: vec![],
                node_id: 0,
                lng: 0.,
                lat: 0.,
            })
        }
    }
}

#[debug_handler]
pub async fn select_nodes(
    State(state): State<VeloinfoState>,
    Path((start_lng, start_lat, end_lng, end_lat)): Path<(f64, f64, f64, f64)>,
) -> Json<Vec<Cycleway>> {
    let conn = state.conn;
    let end = match Cycleway::find(&end_lng, &end_lat, conn.clone()).await {
        Ok(end) => end,
        Err(e) => {
            eprintln!("Error while fetching end node: {}", e);
            Node {
                way_id: 0,
                geom: vec![],
                node_id: 0,
                lng: 0.,
                lat: 0.,
            }
        }
    };
    let start = match Cycleway::find(&start_lng, &start_lat, conn.clone()).await {
        Ok(start) => start,
        Err(e) => {
            eprintln!("Error while fetching start node: {}", e);
            Node {
                way_id: 0,
                geom: vec![],
                node_id: 0,
                lng: 0.,
                lat: 0.,
            }
        }
    };

    let route = Cycleway::route(&start.node_id, &end.node_id, conn.clone()).await;
    let segments = join_all(route.iter().map(|r| async {
        match Cycleway::get(&r.way_id, conn.clone()).await {
            Ok(cycleway) => cycleway,
            Err(e) => {
                eprintln!("Error while fetching cycleway: {}", e);
                Cycleway {
                    name: Some("".to_string()),
                    way_id: 0,
                    geom: vec![],
                    source: 0,
                    target: 0,
                }
            }
        }
    }))
    .await;

    Json(segments)
}

pub async fn route(
    State(state): State<VeloinfoState>,
    Path((start_lng, start_lat, end_lng, end_lat)): Path<(f64, f64, f64, f64)>,
) -> Json<Vec<Point>> {
    println!("start: {}, {}", start_lng, start_lat);
    let start = match Cycleway::find(&start_lng, &start_lat, state.conn.clone()).await {
        Ok(start) => start,
        Err(e) => {
            eprintln!("Error while fetching start node: {}", e);
            Node {
                way_id: 0,
                geom: vec![],
                node_id: 0,
                lng: 0.,
                lat: 0.,
            }
        }
    };
    println!("start: {:?}", start);
    let end = match Cycleway::find(&end_lng, &end_lat, state.conn.clone()).await {
        Ok(end) => end,
        Err(e) => {
            eprintln!("Error while fetching end node: {}", e);
            Node {
                way_id: 0,
                geom: vec![],
                node_id: 0,
                lng: 0.,
                lat: 0.,
            }
        }
    };
    println!("end: {:?}", end);
    Json(Edge::route(&start, &end, state.conn.clone()).await)
}
