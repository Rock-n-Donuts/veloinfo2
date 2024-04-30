use super::score_circle::ScoreCircle;
use crate::db::cyclability_score::CyclabilityScore;
use crate::VeloinfoState;
use askama::Template;
use axum::extract::{Path, State};
use chrono::Locale;
use chrono_tz::America::Montreal;
use futures::future::join_all;
use serde::Deserialize;
use sqlx::types::chrono::Local;
use sqlx::Postgres;
use timeago;
use timeago::languages::french::French;

#[derive(Template)]
#[template(path = "info_panel.html", escape = "none")]
pub struct InfoPanelTemplate {
    pub arrow: String,
    pub contributions: Vec<InfopanelContribution>,
}

#[derive(Template, Clone)]
#[template(path = "info_panel_contribution.html", escape = "none")]
pub struct InfopanelContribution {
    created_at: String,
    timeago: String,
    score_circle: ScoreCircle,
    name: String,
    comment: String,
    score_id: i32,
    photo_path_thumbnail: Option<String>,
}

impl InfopanelContribution {
    pub async fn get(
        lng1: f64,
        lat1: f64,
        lng2: f64,
        lat2: f64,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<InfopanelContribution> {
        let scores = match CyclabilityScore::get_recents(lng1, lat1, lng2, lat2, conn).await {
            Result::Ok(cs) => cs,
            Err(e) => {
                eprintln!("Error getting contributions {:?}", e);
                Vec::new()
            }
        };

        join_all(scores.iter().map(|score| async {
            InfopanelContribution {
                created_at: score
                    .created_at
                    .with_timezone(&Montreal)
                    .format_localized("%H:%M - %d %B", Locale::fr_CA)
                    .to_string(),
                timeago: timeago::Formatter::with_language(French)
                    .convert_chrono(score.created_at, Local::now()),
                score_circle: ScoreCircle { score: score.score },
                name: get_name(&score.name).await,
                comment: score.comment.clone().unwrap_or("".to_string()),
                score_id: score.id,
                photo_path_thumbnail: score.photo_path_thumbnail.clone(),
            }
        }))
        .await
    }

    pub async fn get_history(
        way_ids: &Vec<i64>,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<InfopanelContribution> {
        let scores = CyclabilityScore::get_history(way_ids, conn).await;

        join_all(scores.iter().map(|score| async {
            InfopanelContribution {
                created_at: score
                    .created_at
                    .with_timezone(&Montreal)
                    .format_localized("%H:%M - %d %B", Locale::fr_CA)
                    .to_string(),
                timeago: timeago::Formatter::with_language(French)
                    .convert_chrono(score.created_at, Local::now()),
                score_circle: ScoreCircle { score: score.score },
                name: get_name(&score.name).await,
                comment: score.comment.clone().unwrap_or("".to_string()),
                score_id: score.id,
                photo_path_thumbnail: score.photo_path_thumbnail.clone(),
            }
        }))
        .await
    }

    pub async fn get_history_by_way_id(
        way_id: i64,
        conn: &sqlx::Pool<Postgres>,
    ) -> Vec<InfopanelContribution> {
        let scores = match CyclabilityScore::get_by_way_ids(&vec![way_id], &conn).await {
            Ok(cs) => cs,
            Err(e) => {
                eprintln!("Error getting contributions get_history_by_way_id {:?}", e);
                Vec::new()
            }
        };

        join_all(scores.iter().map(|score| async {
            InfopanelContribution {
                created_at: score
                    .created_at
                    .with_timezone(&Montreal)
                    .format_localized("%H:%M - %d %B", Locale::fr_CA)
                    .to_string(),
                timeago: timeago::Formatter::with_language(French)
                    .convert_chrono(score.created_at, Local::now()),
                score_circle: ScoreCircle { score: score.score },
                name: get_name(&score.name).await,
                comment: score.comment.clone().unwrap_or("".to_string()),
                score_id: score.id,
                photo_path_thumbnail: score.photo_path_thumbnail.clone(),
            }
        }))
        .await
    }
}

async fn get_name(names: &Vec<Option<String>>) -> String {
    names.iter().fold("".to_string(), |acc, name| {
        let blank_name = "non inconnu".to_string();
        let name = match name {
            Some(name) => name,
            None => &blank_name,
        };
        if acc.find(name) != None {
            return acc;
        }
        format!("{} {}", acc, name)
    })
}

pub async fn info_panel_down() -> InfoPanelTemplate {
    InfoPanelTemplate {
        arrow: "▲".to_string(),
        contributions: Vec::new(),
    }
}

#[derive(Deserialize, Debug)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

pub async fn info_panel_up(
    State(state): State<VeloinfoState>,
    Path((lng1, lat1, lng2, lat2)): Path<(f64, f64, f64, f64)>,
) -> InfoPanelTemplate {
    let contributions = InfopanelContribution::get(lng1, lat1, lng2, lat2, &state.conn).await;
    InfoPanelTemplate {
        arrow: "▼".to_string(),
        contributions: contributions,
    }
}
