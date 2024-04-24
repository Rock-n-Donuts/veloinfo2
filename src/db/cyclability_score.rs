use chrono::{DateTime, Local};
use regex::Regex;
use sqlx::{Postgres, Row};

#[derive(sqlx::FromRow, Debug)]
pub struct CyclabilityScore {
    pub id: i32,
    pub name: Option<String>,
    pub score: f64,
    pub comment: Option<String>,
    pub way_ids: Vec<i64>,
    pub created_at: DateTime<Local>,
    pub photo_path: Option<String>,
    pub photo_path_thumbnail: Option<String>,
    pub geom: Vec<[f64; 2]>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CyclabilityScoreDb {
    pub id: i32,
    pub name: Option<String>,
    pub score: f64,
    pub comment: Option<String>,
    pub way_ids: Vec<i64>,
    pub created_at: DateTime<Local>,
    pub photo_path: Option<String>,
    pub photo_path_thumbnail: Option<String>,
    pub geom: String,
}

impl CyclabilityScore {
    pub async fn get_recents(
        lng1: f64,
        lat1: f64,
        lng2: f64,
        lat2: f64,
        conn: &sqlx::Pool<Postgres>,
    ) -> Result<Vec<CyclabilityScore>, sqlx::Error> {
        let cs: Vec<CyclabilityScoreDb> = sqlx::query_as(
            r#"select DISTINCT ON (cs.created_at) cs.id, 
                        cs.score, 
                        c.name,
                        cs.comment, 
                        cs.way_ids, 
                        cs.created_at, 
                        cs.photo_path, 
                        cs.photo_path_thumbnail,
                        ST_AsText(ST_Transform(c.geom, 4326)) as geom
               from cyclability_score cs
               join (
                    select * from cycleway_way 
                    where geom && ST_Transform(st_makeenvelope($1, $2, $3, $4, 4326), 3857)
               ) c on c.way_id = any(cs.way_ids) 
               order by cs.created_at desc
               limit 100"#,
        )
        .bind(lng1)
        .bind(lat1)
        .bind(lng2)
        .bind(lat2)
        .fetch_all(conn)
        .await?;

        Ok(cs.iter().map(|c| c.into()).collect())
    }

    pub async fn get_history(
        way_ids: &Vec<i64>,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<CyclabilityScore> {
        let cs: Vec<CyclabilityScoreDb> = match sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at, photo_path, photo_path_thumbnail
               from cyclability_score
               where way_ids = $1
               order by created_at desc
               limit 100"#,
        )
        .bind(way_ids)
        .fetch_all(conn)
        .await
        {
            Ok(cs) => cs,
            Err(e) => {
                eprintln!("Error while fetching history: {}", e);
                vec![]
            }
        };

        cs.iter().map(|c| c.into()).collect()
    }

    pub async fn get_by_id(
        id: i32,
        conn: &sqlx::Pool<Postgres>,
    ) -> Result<CyclabilityScore, sqlx::Error> {
        let cs: CyclabilityScoreDb = sqlx::query_as(
            r#"select id, 
                      name, 
                      ST_AsText(ST_Transform(geom, 4326)) as geom, 
                      score, 
                      comment, 
                      way_ids, 
                      created_at, 
                      photo_path, 
                      photo_path_thumbnail
               from cyclability_score
               join cycleway_way on way_id = any(way_ids)
               where id = $1"#,
        )
        .bind(id)
        .fetch_one(conn)
        .await?;
        Ok(cs.into())
    }

    pub async fn get_by_way_ids(
        way_ids: &Vec<i64>,
        conn: &sqlx::Pool<Postgres>,
    ) -> Result<Vec<CyclabilityScore>, sqlx::Error> {
        let cs: Vec<CyclabilityScoreDb> = sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at, photo_path, photo_path_thumbnail
               from cyclability_score
               where way_ids = $1
               order by created_at desc"#,
        )
        .bind(way_ids)
        .fetch_all(conn)
        .await?;

        Ok(cs.iter().map(|c| c.into()).collect())
    }

    pub async fn get_photo_by_way_ids(way_ids: &Vec<i64>, conn: &sqlx::Pool<Postgres>) -> Vec<i32> {
        let result = sqlx::query(
            r#"select id
               from cyclability_score
               where way_ids && $1
               and photo_path_thumbnail is not null
               order by created_at desc"#,
        )
        .bind(way_ids)
        .fetch_all(conn)
        .await
        .unwrap();

        result.iter().map(|photo| photo.get(0)).collect()
    }

    pub async fn insert(
        score: &f64,
        comment: &Option<String>,
        way_ids: &Vec<i64>,
        photo_path: &Option<String>,
        photo_path_thumbnail: &Option<String>,
        conn: &sqlx::Pool<Postgres>,
    ) -> Result<i32, sqlx::Error> {
        let id: i32 = sqlx::query(
            r#"INSERT INTO cyclability_score 
                    (way_ids, score, comment, photo_path, photo_path_thumbnail) 
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING id"#,
        )
        .bind(way_ids)
        .bind(score)
        .bind(comment)
        .bind(&photo_path)
        .bind(&photo_path_thumbnail)
        .fetch_one(conn)
        .await?
        .get(0);

        if let Some(photo_path) = photo_path {
            sqlx::query(
                r#"UPDATE cyclability_score 
                        SET photo_path = $1,
                        photo_path_thumbnail = $2
                        WHERE id = $3"#,
            )
            .bind(photo_path.replace("{}", id.to_string().as_str()))
            .bind(match photo_path_thumbnail {
                Some(p) => Some(p.replace("{}", id.to_string().as_str())),
                None => None,
            })
            .bind(id)
            .execute(conn)
            .await?;
        };

        sqlx::query(r#"REFRESH MATERIALIZED VIEW bike_path"#)
            .execute(conn)
            .await?;

        Ok(id)
    }
}

impl From<&CyclabilityScoreDb> for CyclabilityScore {
    fn from(response: &CyclabilityScoreDb) -> Self {
        let re = Regex::new(r"(-?\d+\.*\d*) (-?\d+\.*\d*)").unwrap();
        let points = re
            .captures_iter(response.geom.as_str())
            .map(|cap| {
                let x = cap[1].parse::<f64>().unwrap();
                let y = cap[2].parse::<f64>().unwrap();

                [x, y]
            })
            .collect::<Vec<[f64; 2]>>();
        CyclabilityScore {
            id: response.id,
            name: response.name.clone(),
            score: response.score,
            comment: response.comment.clone(),
            way_ids: response.way_ids.clone(),
            created_at: response.created_at,
            photo_path: response.photo_path.clone(),
            photo_path_thumbnail: response.photo_path_thumbnail.clone(),
            geom: points,
        }
    }
}

impl From<CyclabilityScoreDb> for CyclabilityScore {
    fn from(response: CyclabilityScoreDb) -> Self {
        CyclabilityScore::from(&response)
    }
}
