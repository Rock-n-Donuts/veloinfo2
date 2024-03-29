use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cycleway {
    pub name: Option<String>,
    pub way_id: i64,
    pub geom: Vec<[f64; 2]>,
    pub source: i64,
    pub target: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct CyclewayDb {
    name: Option<String>,
    way_id: i64,
    geom: String,
    source: i64,
    target: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct NodeDb {
    pub way_id: i64,
    pub geom: String,
    pub node_id: i64,
    pub lng: f64,
    pub lat: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Node {
    pub way_id: i64,
    pub geom: Vec<[f64; 2]>,
    pub node_id: i64,
    pub lng: f64,
    pub lat: f64,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct RouteDB {
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

impl Cycleway {
    pub async fn get(way_id: &i64, conn: sqlx::Pool<Postgres>) -> Result<Cycleway> {
        let response: CyclewayDb = sqlx::query_as(
            r#"select
                name,  
                way_id,
                source,
                target,
                ST_AsText(ST_Transform(geom, 4326)) as geom  
               from cycleway_way where way_id = $1"#,
        )
        .bind(way_id)
        .fetch_one(&conn)
        .await?;
        Ok(response.into())
    }

    pub async fn get_by_score_id(
        score_id: &i32,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<Cycleway>> {
        let responses: Vec<CyclewayDb> = sqlx::query_as(
            r#"select
                c.name,  
                c.way_id,
                c.source,
                c.target,
                ST_AsText(ST_Transform(c.geom, 4326)) as geom  
               from cycleway_way c
               join cyclability_score cs on c.way_id = any(cs.way_ids)
               where cs.id = $1
               "#,
        )
        .bind(score_id)
        .fetch_all(&conn)
        .await?;
        Ok(responses.iter().map(|response| response.into()).collect())
    }

    pub async fn find(
        lng: &f64,
        lat: &f64,
        conn: sqlx::Pool<Postgres>,
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
                FROM cycleway_way
                WHERE ST_DWithin(geom, ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857), 1000)
            ) as subquery
            ORDER BY (dp).geom <-> ST_Transform(ST_SetSRID(ST_MakePoint($1, $2), 4326), 3857)
            LIMIT 1"#,
        )
        .bind(lng)
        .bind(lat)
        .fetch_one(&conn)
        .await
        {
            Ok(response) => response,
            Err(e) => return Err(e),
        };
        Ok(response.into())
    }

    pub async fn route(source: &i64, target: &i64, conn: sqlx::Pool<Postgres>) -> Vec<RouteDB> {
        let responses: Vec<RouteDB> = match sqlx::query_as(
            r#"SELECT 
                    pa.*,
                    x1,
                    y1,
                    way_id         
                FROM pgr_bdastar(
                    FORMAT(
                        $FORMAT$
                        SELECT *,
                        st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) as cost,
                        st_length(ST_MakeLine(ST_Point(x1, y2), ST_Point(x2, y2))) as reverse_cost
                        from edge 
                        where target is not null                         
                        $FORMAT$
                    )
                , 
                $1, 
                $2
                ) as pa
                left join edge on node = source and target is not null 
            ORDER BY pa.path_seq ASC"#,
        )
        .bind(source)
        .bind(target)
        .fetch_all(&conn)
        .await
        {
            Ok(response) => response,
            Err(e) => {
                eprintln!("could not make route {:?}", e);
                vec![]
            }
        };
        responses
    }
}

impl From<CyclewayDb> for Cycleway {
    fn from(response: CyclewayDb) -> Self {
        Cycleway::from(&response)
    }
}

impl From<&CyclewayDb> for Cycleway {
    fn from(response: &CyclewayDb) -> Self {
        let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
        let points = re
            .captures_iter(response.geom.as_str())
            .map(|cap| {
                let x = cap[1].parse::<f64>().unwrap();
                let y = cap[2].parse::<f64>().unwrap();

                [x, y]
            })
            .collect::<Vec<[f64; 2]>>();
        Cycleway {
            name: response.name.clone(),
            way_id: response.way_id,
            geom: points,
            source: response.source,
            target: response.target,
        }
    }
}

impl From<NodeDb> for Node {
    fn from(response: NodeDb) -> Self {
        let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
        let points = re
            .captures_iter(response.geom.as_str())
            .map(|cap| {
                let x = cap[1].parse::<f64>().unwrap();
                let y = cap[2].parse::<f64>().unwrap();

                [x, y]
            })
            .collect::<Vec<[f64; 2]>>();
        Node {
            node_id: response.node_id,
            way_id: response.way_id,
            geom: points,
            lng: response.lng,
            lat: response.lat,
        }
    }
}
