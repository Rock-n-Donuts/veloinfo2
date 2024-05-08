use serde::{Deserialize, Serialize};
use sqlx::Postgres;

use super::cycleway::{Node, NodeDb};

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub way_id: i64,
    pub node_id: i64,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Edge {
    seq: i32,
    path_seq: i32,
    node: i64,
    edge: i64,
    cost: f64,
    agg_cost: f64,
    pub x1: f64,
    pub y1: f64,
    pub way_id: i64,
}

impl Edge {
    pub async fn route(
        start_node: &Node,
        end_node: &Node,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<Point> {
        let biggest_lng = start_node.lng.max(end_node.lng) + 0.16;
        let biggest_lat = start_node.lat.max(end_node.lat) + 0.16;
        let smallest_lng = start_node.lng.min(end_node.lng) - 0.16;
        let smallest_lat = start_node.lat.min(end_node.lat) - 0.16;

        let request = r#"SELECT distinct on (pa.path_seq)
                                    e.x1 as x,
                                    e.y1 as y,
                                    way_id,
                                    node as node_id
                                        FROM pgr_astar(
                                            FORMAT(
                                                $FORMAT$
                                                SELECT *
                                                from edge
                                                where target is not null
                                                and cost is not null
                                                and geom && ST_Transform(ST_MakeEnvelope(%s, %s, %s, %s, 4326), 3857)            
                                                $FORMAT$,
                                                $3, $4, $5, $6
                                            )
                                        , 
                                        $1, 
                                        $2,
                                        heuristic => 5,
                                        epsilon => 1
                                        ) as pa
                                    join edge e on pa.edge = e.id 
                                    ORDER BY pa.path_seq ASC"#;

        let response: Vec<Point> = match sqlx::query_as(request)
            .bind(start_node.node_id)
            .bind(end_node.node_id)
            .bind(biggest_lng)
            .bind(biggest_lat)
            .bind(smallest_lng)
            .bind(smallest_lat)
            .fetch_all(conn)
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error while fetching route: {}", e);
                vec![]
            }
        };
        response
    }

    pub async fn route_without_score(
        start_node: &Node,
        end_node: &Node,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<Point> {
        let biggest_lng = start_node.lng.max(end_node.lng) + 0.02;
        let biggest_lat = start_node.lat.max(end_node.lat) + 0.02;
        let smallest_lng = start_node.lng.min(end_node.lng) - 0.02;
        let smallest_lat = start_node.lat.min(end_node.lat) - 0.02;

        let request = r#"SELECT distinct on (pa.path_seq)
                                    e.x1 as x,
                                    e.y1 as y,
                                    way_id,
                                    node as node_id
                                        FROM pgr_bdastar(
                                            FORMAT(
                                                $FORMAT$
                                                SELECT *,
                                                cost,
                                                reverse_cost
                                                from (
                                                    select e.id,
                                                    e.source,
                                                    e.target, 
                                                    e.x1,
                                                    e.y1,
                                                    e.x2,
                                                    e.y2,
                                                    st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) * 
                                                    1 / 0.25 * 2 as cost,
                                                    st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) * 
                                                    1 / 0.25 * 2 as reverse_cost
                                                    from edge e
                                                    where e.target is not null and
                                                            x1 <= %s and
                                                            y1 <= %s and
                                                            x1 >= %s and
                                                            y1 >= %s 
                                                    )as sub                    
                                                $FORMAT$,
                                                $3, $4, $5, $6
                                            )
                                        , 
                                        $1, 
                                        $2
                                        ) as pa
                                    join edge e on pa.edge = e.id 
                                    ORDER BY pa.path_seq ASC"#;

        let response: Vec<Point> = match sqlx::query_as(request)
            .bind(start_node.node_id)
            .bind(end_node.node_id)
            .bind(biggest_lng)
            .bind(biggest_lat)
            .bind(smallest_lng)
            .bind(smallest_lat)
            .fetch_all(conn)
            .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error while fetching route: {}", e);
                vec![]
            }
        };
        response
    }

    pub async fn find_closest_node(
        lng: &f64,
        lat: &f64,
        conn: &sqlx::Pool<Postgres>,
    ) -> Result<Node, sqlx::Error> {
        let response: NodeDb = match sqlx::query_as(
            r#"        
            SELECT
                way_id,
                ST_AsText(ST_Transform(geom, 4326)) as geom,
                node_id,
                ST_X(st_transform(geom, 4326)) as lng,
                ST_Y(st_transform(geom, 4326)) as lat
            FROM (  
                SELECT 
                        way_id,
                        source as node_id,
                        source_geom as geom
                FROM edge e
	            WHERE 
                    ST_DWithin(geom, ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857), 1000) and
                    cost_road < 20
                union
                SELECT  
                        way_id,
                        target as node_id,
                        target_geom as geom
                FROM edge e
	            WHERE 
                    ST_DWithin(geom, ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857), 1000) and
                    cost_road < 20
            ) as subquery
            ORDER BY geom <-> ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857)
            LIMIT 1"#,
        )
        .bind(lng)
        .bind(lat)
        .fetch_one(conn)
        .await
        {
            Ok(response) => response,
            Err(e) => return Err(e),
        };
        Ok(response.into())
    }
}
