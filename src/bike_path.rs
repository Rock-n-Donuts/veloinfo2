use sqlx::Postgres;
use anyhow::Result;

pub async fn prepare_bp(conn: sqlx::Pool<Postgres>) -> Result<()> {
    sqlx::query(r#"create or replace view bike_path as
    select cycleway.*, recent_cyclability_score.score 
    from cycleway 
    left join (
        SELECT *
        FROM cyclability_score
        WHERE (way_id, created_at) IN (
            SELECT way_id, MAX(created_at)
            FROM cyclability_score
            GROUP BY way_id
        )
        order by created_at desc
    )AS recent_cyclability_score ON cycleway.way_id = recent_cyclability_score.way_id;"#).execute(&conn).await?;
    Ok(())
}