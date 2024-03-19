use crate::db::cycleway::Cycleway;
use crate::{VeloInfoError, VeloinfoState};
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn select(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
) -> Result<Json<Cycleway>, VeloInfoError> {
    let conn = state.conn;
    let searched_segment: Cycleway = Cycleway::get(&way_id, conn.clone()).await?;

    Ok(Json(searched_segment))
}
