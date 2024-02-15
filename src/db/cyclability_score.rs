use chrono::{DateTime, Local};
use sqlx::Postgres;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CyclabilityScore {
    pub id: i32,
    pub score: f64,
    pub comment: Option<String>,
    pub way_ids: Vec<i64>,
    pub created_at: DateTime<Local>,
}

impl CyclabilityScore {
    pub async fn get_recents(
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<CyclabilityScore>, sqlx::Error> {
        sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at
               from cyclability_score
               order by created_at desc
               limit 100"#,
        )
        .fetch_all(&conn)
        .await
    }

    pub async fn get_by_id(
        id: i32,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<CyclabilityScore, sqlx::Error> {
        sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at
               from cyclability_score
               where id = $1"#,
        )
        .bind(id)
        .fetch_one(&conn)
        .await
    }

    pub async fn insert(
        score: f64,
        comment: Option<String>,
        way_ids: Vec<i64>,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO cyclability_score 
                    (way_ids, score, comment) 
                    VALUES ($1, $2, $3)"#,
        )
        .bind(way_ids)
        .bind(score)
        .bind(comment)
        .execute(&conn)
        .await?;
        Ok(())
    }
}
