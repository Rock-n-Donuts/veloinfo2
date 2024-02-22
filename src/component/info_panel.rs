use super::score_circle::ScoreCircle;
use crate::db::{cyclability_score::CyclabilityScore, cycleway::Cycleway};
use crate::VeloinfoState;
use anyhow::Ok;
use anyhow::Result;
use askama::Template;
use axum::extract::State;
use axum::Json;
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
        bounds: Bounds,
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<InfopanelContribution>> {
        let scores = CyclabilityScore::get_recents(bounds, conn.clone()).await?;

        let r: Vec<std::prelude::v1::Result<InfopanelContribution, _>> =
            join_all(scores.iter().map(|score| async {
                Ok(InfopanelContribution {
                    created_at: score
                        .created_at
                        .with_timezone(&Montreal)
                        .format_localized("%H:%M - %d %B", Locale::fr_CA)
                        .to_string(),
                    timeago: timeago::Formatter::with_language(French)
                        .convert_chrono(score.created_at, Local::now()),
                    score_circle: ScoreCircle { score: score.score },
                    name: get_name(score.way_ids.as_ref(), conn.clone()).await,
                    comment: score.comment.clone().unwrap_or("".to_string()),
                    score_id: score.id,
                    photo_path_thumbnail: score.photo_path_thumbnail.clone(),
                })
            }))
            .await;

        Ok(r.iter()
            .filter(
                |result: &&std::prelude::v1::Result<InfopanelContribution, _>| match result {
                    Result::Ok(_) => true,
                    Err(_) => false,
                },
            )
            .map(
                |result: &std::prelude::v1::Result<InfopanelContribution, _>| {
                    result.as_ref().unwrap()
                },
            )
            .cloned()
            .collect::<Vec<InfopanelContribution>>())
    }

    pub async fn get_history(
        way_ids: &Vec<i64>,
        conn: sqlx::Pool<Postgres>,
    ) -> Vec<InfopanelContribution> {
        let scores = CyclabilityScore::get_history(way_ids, conn.clone()).await;

        join_all(scores.iter().map(|score| async {
            InfopanelContribution {
                created_at: score
                    .created_at
                    .format_localized("%H:%M - %d %B", Locale::fr_CA)
                    .to_string(),
                timeago: timeago::Formatter::with_language(French)
                    .convert_chrono(score.created_at, Local::now()),
                score_circle: ScoreCircle { score: score.score },
                name: get_name(score.way_ids.as_ref(), conn.clone()).await,
                comment: score.comment.clone().unwrap_or("".to_string()),
                score_id: score.id,
                photo_path_thumbnail: score.photo_path_thumbnail.clone(),
            }
        }))
        .await
    }
}

async fn get_name(way_ids: &Vec<i64>, conn: sqlx::Pool<Postgres>) -> String {
    join_all(way_ids.iter().map(|way_id| async {
        Ok(Cycleway::get(*way_id, conn.clone())
            .await?
            .name
            .unwrap_or("Non inconnu".to_string()))
    }))
    .await
    .iter()
    .fold(
        "".to_string(),
        |acc, name: &std::prelude::v1::Result<String, _>| {
            let erreur = "erreur".to_string();
            let name = name.as_ref().unwrap_or(&erreur);
            if acc.find(name.as_str()) != None {
                return acc;
            }
            format!("{} {}", acc, name)
        },
    )
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

#[derive(Deserialize, Debug)]
pub struct Bounds {
    pub _ne: LatLng,
    pub _sw: LatLng,
}

pub async fn info_panel_up(
    State(state): State<VeloinfoState>,
    Json(bounds): Json<Bounds>,
) -> InfoPanelTemplate {
    let contributions = match InfopanelContribution::get(bounds, state.conn).await{
        Result::Ok(c) => c,
        Err(e) => {
            eprintln!("Error getting contributions {:?}", e);
            Vec::new()},
    };
    InfoPanelTemplate {
        arrow: "▼".to_string(),
        contributions: contributions,
    }
}
