use axum::extract::{Path, State};
use axum::Json;

use crate::db::cycleway::Cycleway;
use crate::VeloinfoState;
use crate::{component::score_selector::ScoreSelector, VIError};

pub async fn score_selector_controler(Path(score): Path<f64>) -> Result<ScoreSelector, VIError> {
    let score_selector = ScoreSelector::get_score_selector(score);
    Ok(score_selector)
}

pub async fn score_bounds_controler(State(state): State<VeloinfoState>, Path(score): Path<i32>) -> Result<Json<Vec<Cycleway>>, VIError> {
    let geom = Cycleway::get_by_score_id(score, state.conn.clone()).await?;
    Ok(Json(geom))
}