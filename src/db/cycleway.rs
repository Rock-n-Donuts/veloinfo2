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

#[derive(Debug, Serialize, Clone)]
pub struct Route {
    pub way_ids: Vec<i64>,
    pub geom: Vec<[f64; 2]>,
    pub source: i64,
    pub target: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct RouteDB {
    way_id: i64,
    source: i64,
    target: i64,
    geom: String,
}

impl Cycleway {
    pub async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<Cycleway> {
        let response: CyclewayDb = sqlx::query_as(
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

    pub async fn get_by_score_id(
        score_id: i32,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<Cycleway>> {
        let responses: Vec<CyclewayDb> = sqlx::query_as(
            r#"select
                c.name,  
                c.way_id,
                c.source,
                c.target,
                ST_AsText(ST_Transform(c.geom, 4326)) as geom  
               from cycleway c
               join cyclability_score cs on c.way_id = any(cs.way_ids)
               where cs.id = $1
               "#,
        )
        .bind(score_id)
        .fetch_all(&conn)
        .await?;
        Ok(responses.iter().map(|response| response.into()).collect())
    }

    pub async fn route(source: &i64, target: &i64, conn: sqlx::Pool<Postgres>) -> Result<Route> {
        let responses: Vec<RouteDB> = sqlx::query_as(
            r#"select   way_id,
                        $1 as source,
                        $2 as target, 
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
                        from cycleway                        
                        $FORMAT$
                    )
                , 
                $1, 
                $2
                ) as pa join cycleway c on pa.edge = c.way_id
                order by path_seq asc"#,
        )
        .bind(source)
        .bind(target)
        .fetch_all(&conn)
        .await
        .unwrap();
        let segment: Route = responses.iter().fold(
            Route {
                way_ids: Vec::new(),
                geom: vec![],
                source: *source,
                target: *target,
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

    // todo: Finish to have a route from node to node
    #[allow(dead_code)]
    pub async fn route2(source: &i64, target: &i64, conn: sqlx::Pool<Postgres>) -> Result<Route> {
        let responses: Vec<RouteDB> = sqlx::query_as(
            r#"select   way_id,
                        $1 as source,
                        $2 as target, 
                        ST_AsText(ST_Transform(geom, 4326)) as geom,
                        path_seq
                from pgr_bdastar(
                    FORMAT(
                        $FORMAT$
                        WITH c_points AS (
                            SELECT 
                                (ST_DumpPoints(geom)).geom AS point, 
                                (ST_DumpPoints(geom)).path[1] AS ord,
                                way_id
                            FROM cycleway
                        ),
                        cycleway_pairs AS (
                            SELECT 
                                way_id as id, 
                                point AS point1, 
                                LAG(point) OVER (PARTITION BY way_id ORDER BY ord) AS point2
                            FROM c_points
                        )
                        SELECT cpa.id,
                            cp1.node_id as source,
                            cp2.node_id as target,
                            ST_Length(st_makeline(cpa.point1, cpa.point2)) as cost,
                            ST_Length(st_makeline(cpa.point1, cpa.point2)) as reverse_cost,
                            ST_X(cpa.point1) AS x1, 
                            ST_Y(cpa.point1) AS y1, 
                            ST_X(cpa.point2) AS x2, 
                            ST_Y(cpa.point2) AS y2
                        FROM cycleway_pairs cpa
                        join cycleway_point cp1 on cp1.geom = point1
                        join cycleway_point cp2 on cp2.geom = point2
                        WHERE point2 IS NOT NULL;                        
                        $FORMAT$
                    )
                , 
                $1, 
                $2
                ) as pa join cycleway c on pa.edge = c.way_id
                order by path_seq asc"#,
        )
        .bind(source)
        .bind(target)
        .fetch_all(&conn)
        .await
        .unwrap();
        let segment: Route = responses.iter().fold(
            Route {
                way_ids: Vec::new(),
                geom: vec![],
                source: *source,
                target: *target,
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
