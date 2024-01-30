use sqlx::Postgres;
use sqlx::types::chrono::NaiveDateTime;

#[derive(sqlx::FromRow)]
pub struct CyclabilityScore{
    pub id: i32,
    pub score: f64,
    pub comment: Option<String>,
    pub way_ids: Vec<i64>,
    pub created_at: NaiveDateTime,
}

impl CyclabilityScore {
    pub async fn get(id: i64, conn: sqlx::Pool<Postgres>) -> Result<CyclabilityScore, sqlx::Error> {
        sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at
               from cyclability_score
               where $1 = id"#,
        )
        .bind(id)
        .fetch_one(&conn)
        .await
    }

    pub async fn get_recents(conn: sqlx::Pool<Postgres>) -> Result<Vec<CyclabilityScore>, sqlx::Error> {
        sqlx::query_as(
            r#"select id, score, comment, way_ids, created_at
               from cyclability_score
               order by created_at desc"#,
        )
        .fetch_all(&conn)
        .await
    }
}