use anyhow::Ok;
use anyhow::Result;
use crate::db::{cyclability_score::CyclabilityScore, cycleway::Cycleway};
use askama::Template;
use axum::extract::State;
use futures::future::join_all;
use sqlx::Postgres;
use crate::VeloinfoState;
use chrono::Locale;
use super::score_circle::ScoreCircle;

#[derive(Template)]
#[template(path = "info_panel.html", escape = "none")]
pub struct InfoPanelTemplate {
    pub arrow: String,
    pub direction: String,
    pub contributions: Vec<InfopanelContribution>,
}


#[derive(Template, Clone)]
#[template(path = "info_panel_contribution.html", escape = "none")]
pub struct InfopanelContribution {
    created_at: String,
    score_circle: ScoreCircle,
    comment: String,
    name: String,
    score_id: i32,
}

impl InfopanelContribution {
    pub async fn get(
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<InfopanelContribution>> {
        let scores = CyclabilityScore::get_recents(conn.clone()).await?;

        let r: Vec<std::prelude::v1::Result<InfopanelContribution, _>> = join_all(scores.iter().map(|score| async {
            Ok(InfopanelContribution {
                created_at: score.created_at.format_localized("%H:%M - %d %B", Locale::fr_CA).to_string(),
                score_circle: ScoreCircle {
                    score: score.score,
                },
                comment: score.comment.clone().unwrap_or("rien a dire".to_string()),
                name: get_name(score.way_ids.as_ref(), conn.clone()).await,
                score_id: score.id,
            })
        }))
        .await;

        Ok(r.iter().filter(|result: &&std::prelude::v1::Result<InfopanelContribution, _>| match result {
            Result::Ok(_) => true,
            Err(_) => false,
        }).map(|result: &std::prelude::v1::Result<InfopanelContribution, _>| result.as_ref().unwrap()).cloned().collect::<Vec<InfopanelContribution>>())
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
    .fold("".to_string(), |acc, name: &std::prelude::v1::Result<String, _>| {
        let erreur = "erreur".to_string();
        let name = name.as_ref().unwrap_or(&erreur);
        if acc.find(name.as_str()) != None {
            return acc;
        }
        format!("{} {}", acc, name)
    })
}

pub async fn info_panel_down() -> String {
    let template = InfoPanelTemplate {
        arrow: "▲".to_string(),
        direction: "up".to_string(),
        contributions: Vec::new(),
    };
    template.render().unwrap()
}

pub async fn info_panel_up(State(state): State<VeloinfoState>) -> InfoPanelTemplate {
    let contributions = InfopanelContribution::get(state.conn).await.unwrap();

    InfoPanelTemplate {
        arrow: "▼".to_string(),
        direction: "down".to_string(),
        contributions: contributions,
    }
}
