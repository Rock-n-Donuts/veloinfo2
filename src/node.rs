use crate::component::route_panel::RoutePanel;
use crate::db::cycleway::Node;
use crate::db::edge::{Edge, Point};
use crate::VeloinfoState;
use axum::extract::{Path, State};

pub async fn route(
    State(state): State<VeloinfoState>,
    Path((start_lng, start_lat, end_lng, end_lat)): Path<(f64, f64, f64, f64)>,
) -> RoutePanel {
    let start = match Edge::find_closest_node(&start_lng, &start_lat, state.conn.clone()).await {
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
    let end = match Edge::find_closest_node(&end_lng, &end_lat, state.conn.clone()).await {
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
    let mut edges = Edge::route(&start, &end, state.conn.clone()).await;
    edges.insert(
        0,
        Point {
            x: start_lng,
            y: start_lat,
        },
    );
    edges.push(Point {
        x: end_lng,
        y: end_lat,
    });
    println!("edges: {:?}", edges);
    let edges_coordinate: Vec<(f64, f64)> = edges.iter().map(|edge| (edge.x, edge.y)).collect();
    let route_json = match serde_json::to_string(&edges_coordinate) {
        Ok(edges_coordinate) => edges_coordinate,
        Err(e) => {
            eprintln!("Error while serializing edges: {}", e);
            "[]".to_string()
        }
    };
    RoutePanel { route_json }
}
