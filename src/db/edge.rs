use serde::{Deserialize, Serialize};
use sqlx::Postgres;

use super::cycleway::Node;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
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
        conn: sqlx::Pool<Postgres>,
    ) -> Vec<Point> {
        let biggest_lng = start_node.lng.max(end_node.lng) + 0.02;
        let biggest_lat = start_node.lat.max(end_node.lat) + 0.02;
        let smallest_lng = start_node.lng.min(end_node.lng) - 0.02;
        let smallest_lat = start_node.lat.min(end_node.lat) - 0.02;

        println!("start: {}", start_node.node_id);
        println!("end: {}", end_node.node_id);

        println!(
            "big: {}, {}, small: {}, {}",
            biggest_lng, biggest_lat, smallest_lng, smallest_lat
        );

        let response: Vec<Point> = match sqlx::query_as(
            r#"SELECT distinct on (pa.path_seq)
                x1 as x,
                y1 as y
                    FROM pgr_bdastar(
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
                                        case
                                            when aw.tags->>'highway' = 'cycleway' then 1 / 1
                                            when aw.tags->>'cycleway' = 'lane' then 1 / 1
                                            when aw.tags->>'bicycle' = 'designated' then 1 / 1
                                            when aw.tags->>'cycleway:right' = 'lane' then 1 / 1
                                            when aw.tags->>'cycleway:both' = 'lane' then 1 / 1
                                            when aw.tags->>'cycleway:left' = 'lane' then 1 / 1
                                            when aw.tags->>'cycleway' = 'shared_lane' then 1 / 0.5
                                            when aw.tags->>'bicycle' = 'yes' then 1 / 0.5
                                            when aw.tags->>'highway' = 'residential' then 1 / 0.5
                                            when aw.tags->>'highway' = 'tertiary' then 1 / 0.5
                                            when aw.tags->>'highway' = 'secondary' then 1 / 0.25
                                            when aw.tags->>'highway' = 'footway' then 1 / 0.25
                                            when aw.tags->>'highway' = 'proposed' then 100
                                            when aw.tags->>'highway' = 'steps' then 30
                                            when aw.tags->>'highway' is not null then 1 / 0.25
                                            else 1 / 0.25
                                        end
                                    WHEN cs.score = 0 THEN 100
                                    ELSE 1 / cs.score
                                END as cost,
                                st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) * 
                                CASE
                                    when aw.tags->>'oneway:bicycle' = 'no' and cs.score is not null and cs.score != 0 then 1 / cs.score
                                    when aw.tags->>'oneway:bicycle' = 'yes' then 100
                                    when aw.tags->>'oneway' = 'yes' then 100
                                    WHEN cs.score IS NULL THEN
                                        case
                                            when aw.tags->>'highway' = 'cycleway' then 1 / 1
                                            when aw.tags->>'cycleway' = 'lane' then 1 / 1
                                            when aw.tags->>'bicycle' = 'designated' then 1 / 1
                                            when aw.tags->>'cycleway:right' = 'lane' then 1 / 1
                                            when aw.tags->>'cycleway:both' = 'lane' then 1 / 1
                                            when aw.tags->>'cycleway:left' = 'lane' then 1 / 1
                                            when aw.tags->>'cycleway' = 'shared_lane' then 1 / 0.5
                                            when aw.tags->>'bicycle' = 'yes' then 1 / 0.5
                                            when aw.tags->>'highway' = 'residential' then 1 / 0.5
                                            when aw.tags->>'highway' = 'tertiary' then 1 / 0.5
                                            when aw.tags->>'highway' = 'secondary' then 1 / 0.25
                                            when aw.tags->>'highway' = 'footway' then 1 / 0.25
                                            when aw.tags->>'highway' = 'proposed' then 100
                                            when aw.tags->>'highway' = 'steps' then 30
                                            when aw.tags->>'highway' is not null then 1 / 0.25
                                            else 1 / 0.25
                                        end
                                    WHEN cs.score = 0 THEN 100
                                    ELSE 1 / cs.score
                                END as reverse_cost
                                from edge e
                                left join (
		                            select unnest(way_ids) as way_id, avg(score) as score
		                            from cyclability_score
		                            group by way_id
		                        ) cs on e.way_id = cs.way_id
                                left join all_way aw on e.way_id = aw.way_id
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
                left join edge on node = source and source is not null 
                ORDER BY pa.path_seq ASC"#,
        )
        .bind(start_node.node_id)
        .bind(end_node.node_id)
        .bind(biggest_lng)
        .bind(biggest_lat)
        .bind(smallest_lng)
        .bind(smallest_lat)
        .fetch_all(&conn)
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
}
