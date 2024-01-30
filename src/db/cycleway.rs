use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;


#[derive(Debug, sqlx::FromRow)]
struct ResponseDb {
    name: Option<String>,
    way_id: Option<i64>,
    geom: Option<String>,
    source: Option<i64>,
    target: Option<i64>,
}


#[derive(Debug, Serialize, Clone)]
pub struct Route {
    pub way_ids: Vec<i64>,
    pub geom: Vec<[f64; 2]>,
    pub source: i64,
    pub target: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cycleway {
    pub name: Option<String>,
    pub way_id: Option<i64>,
    pub geom: Option<Vec<[f64; 2]>>,
    pub source: Option<i64>,
    pub target: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct RouteDB {
    way_id: i64,
    source: i64,
    target: i64,
    geom: String,
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

impl Cycleway {
    pub async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<Cycleway> {
        let response: ResponseDb = sqlx::query_as(
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

    pub async fn route(name: &str, source: i64, target: i64, conn: sqlx::Pool<Postgres>) -> Result<Route> {
        let responses: Vec<RouteDB> = sqlx::query_as(
            r#"select   way_id,
                        $2 as source,
                        $3 as target, 
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
                        FROM cycleway
                        $FORMAT$,
                        $1
                    )
                , 
                $2, 
                $3
                ) as pa join cycleway c on pa.edge = c.way_id
                order by path_seq asc"#,
        )
        .bind(name)
        .bind(source)
        .bind(target)
        .fetch_all(&conn)
        .await
        .unwrap();
        let segment: Route = responses.iter().fold(
            Route {
                way_ids: Vec::new(),
                geom: vec![],
                source: source,
                target: target,
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

impl From<ResponseDb> for Cycleway {
    fn from(response: ResponseDb) -> Self {
        Cycleway::from(&response)
    }
}

impl From<&ResponseDb> for Cycleway {
    fn from(response: &ResponseDb) -> Self {
        match response.geom.as_ref() {
            Some(str) => {
                let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
                let points = re
                    .captures_iter(str.as_str())
                    .map(|cap| {
                        let x = cap[1].parse::<f64>().unwrap();
                        let y = cap[2].parse::<f64>().unwrap();

                        [x, y]
                    })
                    .collect::<Vec<[f64; 2]>>();
                Cycleway {
                    name: response.name.clone(),
                    way_id: response.way_id,
                    geom: Some(points),
                    source: response.source,
                    target: response.target,
                }
            }
            None => Cycleway {
                name: None,
                way_id: None,
                geom: None,
                source: None,
                target: None,
            },
        }
    }
}

