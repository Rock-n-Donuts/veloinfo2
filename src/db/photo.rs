use chrono::{DateTime, Local};

#[derive(Debug, sqlx::FromRow)]
pub struct Photo {
    timestamp: DateTime<Local>,
    path: String,
    cyclability_score_id: i32,
}

impl Photo {
    pub async fn get(photo_id: i32, conn: sqlx::PgPool) -> Result<Photo, sqlx::Error> {
        sqlx::query_as(
            r#"SELECT * FROM photo WHERE photo_id = $1"#
        )
        .bind(photo_id)
        .fetch_one(&conn)
        .await
    }
}