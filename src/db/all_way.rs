use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllWay {
    pub name: Option<String>,
    pub way_id: i64,
    pub geom: Vec<[f64; 2]>,
    pub source: i64,
    pub target: i64,
    pub score_id: Option<i32>,
    pub score: Option<f64>,
}

#[derive(Debug, sqlx::FromRow)]
struct AllWayDb {
    name: Option<String>,
    way_id: i64,
    geom: String,
    source: i64,
    target: i64,
    score_id: Option<i32>,
    score: Option<f64>,
}

impl AllWay {
    pub async fn get(way_id: &i64, conn: &sqlx::Pool<Postgres>) -> Result<AllWay> {
        let response: Result<AllWayDb, sqlx::Error> = sqlx::query_as(
            r#"select
                name,  
                way_id,
                source,
                target,
                ST_AsText(ST_Transform(geom, 4326)) as geom,  
                score,
                cs.id as score_id
               from all_way 
               left join (
                    select *
                    from cyclability_score 
                    where $1 = any(way_ids)
                    order by created_at desc
                    limit 1
               ) cs on way_id = any(cs.way_ids)
               where 
                cs.id is not null and
                way_id = $1"#,
        )
        .bind(way_id)
        .fetch_one(conn)
        .await;

        match response {
            Ok(response) => Ok(response.into()),
            Err(e) => {
                eprintln!("Error getting cycleway {:?} {:?}", way_id, e);
                Err(e.into())
            }
        }
    }
}

impl From<AllWayDb> for AllWay {
    fn from(response: AllWayDb) -> Self {
        AllWay::from(&response)
    }
}

impl From<&AllWayDb> for AllWay {
    fn from(response: &AllWayDb) -> Self {
        let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
        let points = re
            .captures_iter(response.geom.as_str())
            .map(|cap| {
                let x = cap[1].parse::<f64>().unwrap();
                let y = cap[2].parse::<f64>().unwrap();

                [x, y]
            })
            .collect::<Vec<[f64; 2]>>();
        AllWay {
            name: response.name.clone(),
            way_id: response.way_id,
            geom: points,
            source: response.source,
            target: response.target,
            score: response.score,
            score_id: response.score_id,
        }
    }
}
