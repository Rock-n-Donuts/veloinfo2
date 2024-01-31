use crate::db::{cyclability_score::CyclabilityScore, cycleway::Cycleway};
use askama::Template;
use axum::extract::State;
use futures::future::join_all;
use sqlx::Postgres;

use crate::VeloinfoState;

#[derive(Template)]
#[template(path = "info_panel.html", escape = "none")]
pub struct InfoPanelTemplate {
    pub arrow: String,
    pub direction: String,
    pub contributions: Vec<InfopanelContribution>,
}

#[derive(Template)]
#[template(path = "info_panel_contribution.html")]
pub struct InfopanelContribution {
    created_at: String,
    score: String,
    comment: String,
    name: String,
}

impl InfopanelContribution {
    pub async fn get(
        conn: sqlx::Pool<Postgres>,
    ) -> Result<Vec<InfopanelContribution>, sqlx::Error> {
        let scores = CyclabilityScore::get_recents(conn.clone()).await?;

        Ok(join_all(scores.iter().map(|score| async {
            InfopanelContribution {
                created_at: score.created_at.format("%d/%m/%Y").to_string(),
                score: get_score_string(score.score),
                comment: score.comment.clone().unwrap_or("rien a dire".to_string()),
                name: get_name(score.way_ids.as_ref(), conn.clone()).await,
            }
        }))
        .await)
    }
}

async fn get_name(way_ids: &Vec<i64>, conn: sqlx::Pool<Postgres>) -> String {
    join_all(way_ids.iter().map(|way_id| async {
        Cycleway::get(*way_id, conn.clone())
            .await
            .unwrap()
            .name
            .unwrap_or("Non inconnu".to_string())
    }))
    .await
    .iter()
    .fold("".to_string(), |acc, name| {
        if acc.find(name) != None {
            return acc;
        }
        format!("{} {}", acc, name)
    })
}

fn get_score_string(score: f64) -> String {
    if score < 0.3 {
        "🔴".to_string()
    } else if score < 0.5 {
        "🟠".to_string()
    } else if score < 0.7 {
        "🟡".to_string()
    } else if score < 0.9 {
        "🟢".to_string()
    } else {
        "🔵".to_string()
    } 
}

pub async fn info_panel_down() -> String {
    let template = InfoPanelTemplate {
        arrow: "▲".to_string(),
        direction: "up".to_string(),
        contributions: Vec::new(),
    };
    template.render().unwrap()
}

pub async fn info_panel_up(State(state): State<VeloinfoState>) -> String {
    let contributions = InfopanelContribution::get(state.conn).await.unwrap();

    let template = InfoPanelTemplate {
        arrow: "▼".to_string(),
        direction: "down".to_string(),
        contributions: contributions,
    };
    template.render().unwrap()
}
