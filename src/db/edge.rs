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
                                                SELECT *,
                                                cost,
                                                reverse_cost
                                                from (
                                                    select e.*, 
                                                    st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) * 
                                                    CASE
                                                        WHEN cs.score IS NULL THEN 
                                                            score_road
                                                        WHEN cs.score = 0 THEN 1 / 0.001
                                                        ELSE score_road * cs.score
                                                    END as cost,
                                                    st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) * 
                                                    CASE
                                                        when aw.tags->>'oneway:bicycle' = 'no' and cs.score is not null and cs.score != 0 then score_road * cs.score
                                                        when aw.tags->>'oneway' = 'no' and cs.score is not null and cs.score != 0 then score_road * cs.score
                                                        when aw.tags->>'oneway:bicycle' = 'yes' then 1 / 0.001
                                                        when aw.tags->>'oneway' = 'yes' then 1 / 0.001
                                                        WHEN cs.score IS NULL THEN
                                                            score_road
                                                        WHEN cs.score = 0 THEN 1 / 0.001
                                                        ELSE score_road * cs.score
                                                    END as reverse_cost
                                                    from edge e
                                                    left join (
                                                        select 
                                                            unnest(way_ids) as way_id, 
                                                            avg(score) as score
                                                        from cyclability_score
                                                        group by way_id
                                                    ) cs on e.way_id = cs.way_id
                                                    join (
                                                        select 
                                                            *,
                                                            case
                                                                when tags->>'bicycle' = 'no' then 1 / 0.0001
                                                                when tags->>'highway' = 'cycleway' then 1 / 1
                                                                when tags->>'bicycle' = 'designated' then 1 / 1
                                                                when tags->>'cycleway' = 'track' then 1 / 1
                                                                when tags->>'cycleway:both' = 'track' then 1 / 1
                                                                when tags->>'cycleway:left' = 'track' then 1 / 1
                                                                when tags->>'cycleway:right' = 'track' then 1 / 1
                                                                when tags->>'cycleway' = 'lane' then 1 / 0.75
                                                                when tags->>'cycleway:both' = 'lane' then 1 / 0.75
                                                                when tags->>'cycleway:left' = 'lane' then 1 / 0.75
                                                                when tags->>'cycleway:right' = 'lane' then 1 / 0.75
                                                                when tags->>'cycleway:both' = 'shared_lane' then 1 / 0.50
                                                                when tags->>'cycleway:left' = 'shared_lane' then 1 / 0.50
                                                                when tags->>'cycleway:right' = 'shared_lane' then 1 / 0.50
                                                                when tags->>'cycleway' = 'shared_lane' then 1 / 0.50
                                                                when tags->>'highway' = 'residential' then 1 / 0.50
                                                                when tags->>'highway' = 'tertiary' then 1 / 0.33
                                                                when tags->>'highway' = 'secondary' then 1 / 0.2
                                                                when tags->>'highway' = 'service' then 1 / 0.2
                                                                when tags->>'bicycle' = 'yes' then 1 / 1
                                                                when tags->>'highway' = 'primary' then 1 / 0.1
                                                                when tags->>'highway' = 'footway' then 1 / 0.1
                                                                when tags->>'highway' = 'steps' then 1 / 0.05
                                                                when tags->>'highway' = 'proposed' then 1 / 0.001
                                                                when tags->>'highway' is not null then 1 / 0.25
                                                                else 1 / 0.25
                                                            end as score_road
                                                        from all_way
                                                    ) aw on e.way_id = aw.way_id
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
                                                    select e.*, 
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
                geom,
                unnest(nodes) as node_id,
                ST_X(st_transform((dp).geom, 4326)) as lng,
                ST_Y(st_transform((dp).geom, 4326)) as lat
            FROM (  
                SELECT (ST_DumpPoints(geom)) as dp, 
                        way_id,
                        ST_AsText(ST_Transform(geom, 4326)) as geom,
                        nodes
                FROM all_way
                WHERE ST_DWithin(geom, ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857), 1000)
            ) as subquery
            ORDER BY (dp).geom <-> ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857)
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
