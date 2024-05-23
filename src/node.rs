use crate::component::route_panel::RoutePanel;
use crate::db::edge::{Edge, Point};
use crate::VeloinfoState;
use axum::extract::{Path, State};

pub async fn route(
    State(state): State<VeloinfoState>,
    Path((start_lng, start_lat, end_lng, end_lat)): Path<(f64, f64, f64, f64)>,
) -> RoutePanel {
    let start = match Edge::find_closest_node(&start_lng, &start_lat, &state.conn).await {
        Ok(start) => start,
        Err(e) => {
            return RoutePanel {
                route_json: "[]".to_string(),
                total_length: 0.0,
                error: format!("Error while fetching start node: {}", e),
            };
        }
    };
    let end = match Edge::find_closest_node(&end_lng, &end_lat, &state.conn).await {
        Ok(end) => end,
        Err(e) => {
            return RoutePanel {
                route_json: "[]".to_string(),
                total_length: 0.0,
                error: format!("Error while fetching end node: {}", e),
            };
        }
    };
    let mut edges = Edge::route(&start, &end, &state.conn).await;
    if let 0 = edges.len() {
        println!("No route found");
        return RoutePanel {
            route_json: "[]".to_string(),
            total_length: 0.0,
            error: format!("No route found from {start:?} to {end:?}"),
        };
    };
    edges.insert(
        0,
        Point {
            x: start_lng,
            y: start_lat,
            length: 0.0,
            way_id: 0,
            node_id: 0,
        },
    );
    edges.push(Point {
        x: end_lng,
        y: end_lat,
        length: 0.0,
        way_id: 0,
        node_id: 0,
    });
    let edges_coordinate: Vec<(f64, f64)> = edges.iter().map(|edge| (edge.x, edge.y)).collect();
    let total_length: f64 = edges.iter().map(|edge| edge.length).sum();
    let route_json = match serde_json::to_string(&edges_coordinate) {
        Ok(edges_coordinate) => edges_coordinate,
        Err(e) => {
            return RoutePanel {
                route_json: "[]".to_string(),
                total_length: 0.0,
                error: format!("Error while serializing edges: {}", e),
            };
        }
    };
    RoutePanel {
        route_json,
        total_length: (total_length / 10.0).round() / 100.0,
        error: "".to_string(),
    }
}
