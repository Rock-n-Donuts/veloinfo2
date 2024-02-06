use axum::extract::Path;

use crate::{component::score_selector::ScoreSelector, VIError};

pub async fn score_selector_controler(Path(score): Path<f64>) -> Result<ScoreSelector, VIError> {
    let score_selector = ScoreSelector::get_score_selector(score);
    Ok(score_selector)
}
