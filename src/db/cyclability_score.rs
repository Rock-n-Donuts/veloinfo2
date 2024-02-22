use crate::component::info_panel::Bounds;
use chrono::{DateTime, Local};
use sqlx::{Postgres, Row};

#[derive(sqlx::FromRow, Debug)]
pub struct CyclabilityScore {
    pub id: i32,
    pub score: f64,
    pub comment: Option<String>,
    pub way_ids: Vec<i64>,
    pub created_at: DateTime<Local>,
    pub photo_path: Option<String>,
    pub photo_path_thumbnail: Option<String>,
}

impl CyclabilityScore {
    pub async fn get_recents(
        bounds: &Bounds,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<CyclabilityScore>, sqlx::Error> {
        sqlx::query_as(
            r#"select DISTINCT ON (cs.created_at) cs.id, cs.score, cs.comment, cs.way_ids, cs.created_at, cs.photo_path, cs.photo_path_thumbnail
               from cyclability_score cs
               join cycleway c on c.way_id = any(cs.way_ids) 
               where
               c.geom && ST_Transform(st_makeenvelope($1, $2, $3, $4, 4326), 3857)
               order by cs.created_at desc
               limit 100"#,
        )
        .bind(bounds._ne.lng)
        .bind(bounds._ne.lat)
        .bind(bounds._sw.lng)
        .bind(bounds._sw.lat)
        .fetch_all(&conn)
        .await
    }

    pub async fn get_history(
        way_ids: &Vec<i64>,
        conn: sqlx::Pool<Postgres>,
    ) -> Vec<CyclabilityScore> {
        let result = sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at, photo_path, photo_path_thumbnail
               from cyclability_score
               where way_ids = $1
               order by created_at desc
               limit 100"#,
        )
        .bind(way_ids)
        .fetch_all(&conn)
        .await;

        match result {
            Ok(scores) => scores,
            Err(_) => vec![],
        }
    }

    pub async fn get_by_id(
        id: i32,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<CyclabilityScore, sqlx::Error> {
        sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at, photo_path, photo_path_thumbnail
               from cyclability_score
               where id = $1"#,
        )
        .bind(id)
        .fetch_one(&conn)
        .await
    }

    pub async fn get_by_way_ids(
        way_ids: &Vec<i64>,
        conn: sqlx::Pool<Postgres>,
    ) -> Option<CyclabilityScore> {
        sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at, photo_path, photo_path_thumbnail
               from cyclability_score
               where way_ids = $1
               order by created_at desc"#,
        )
        .bind(way_ids)
        .fetch_one(&conn)
        .await
        .ok()
    }

    pub async fn get_photo_by_way_ids(
        way_ids: &Vec<i64>,
        conn: sqlx::Pool<Postgres>,
    ) -> Vec<i32> {
        let result = sqlx::query(
            r#"select id
               from cyclability_score
               where way_ids && $1
               and photo_path_thumbnail is not null
               order by created_at desc"#,
        )
        .bind(way_ids)
        .fetch_all(&conn)
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
        conn: sqlx::Pool<Postgres>,
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
        .fetch_one(&conn)
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
            .execute(&conn)
            .await?;
        };
        Ok(id)
    }
}
