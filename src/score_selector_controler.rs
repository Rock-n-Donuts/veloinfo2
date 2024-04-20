use axum::extract::{Path, State};
use axum::Json;

use crate::component::score_selector::ScoreSelector;
use crate::db::cycleway::Cycleway;
use crate::VeloinfoState;

pub async fn score_selector_controler(Path(score): Path<f64>) -> ScoreSelector {
    let score_selector = ScoreSelector::get_score_selector(score);
    score_selector
}

pub async fn score_bounds_controler(
    State(state): State<VeloinfoState>,
    Path(score): Path<i32>,
) -> Json<Vec<Cycleway>> {
    let geom = match Cycleway::get_by_score_id(&score, &state.conn).await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error while fetching cycleways: {}", e);
            vec![]
        }
    };
    Json(geom)
}
