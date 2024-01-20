use anyhow::Result;
use askama::Template;
use axum::{extract::{Path, State}, Form};
use serde::{Serialize, Deserialize};
use sqlx::Postgres;

use crate::{VeloInfoError, VeloinfoState};

#[derive(Template)]
#[template(path = "info_panel.html", escape = "none")]
pub struct InfoPanel {
    way_id: i64,
    status: String,
    segment_name: String,
    options: String,
}

#[derive(Debug, sqlx::FromRow)]
struct WayInfo {
    way_id: i64,
    name: Option<String>,
    score: Option<f64>,
}

impl WayInfo {
    pub async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<WayInfo> {
        let response: WayInfo = sqlx::query_as(
            r#"select  
                c.way_id,
                c.name,
                cs.score
               from cycleway c
               left join cyclability_score cs on cs.way_id = c.way_id
               where c.way_id = $1
               order by created_at desc"#,
        )
        .bind(way_id)
        .fetch_one(&conn)
        .await
        .unwrap_or(WayInfo {
            way_id: 0,
            name: None,
            score: None,
        });
        Ok(response.into())
    }
}

pub async fn get_panel() -> String {
    InfoPanel {
        way_id: 0,
        status: "none".to_string(),
        segment_name: "".to_string(),
        options: "".to_string(),
    }.render().unwrap()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostValue {
    pub score: Option<f64>,
}

pub async fn info_panel_post(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
    Form(post): Form<PostValue>,
) -> Result<String, VeloInfoError> {
    println!("post: {:?}", post);
    let conn = state.conn.clone();
    sqlx::query(
        r#"INSERT INTO cyclability_score 
            (way_id, score) 
            VALUES ($1, $2)"#,
    ).bind(way_id).bind(post.score).execute(&conn).await?;

    info_panel(State(state), Path(way_id)).await
}

pub async fn info_panel(
    State(state): State<VeloinfoState>,
    Path(way_id): Path<i64>,
) -> Result<String, VeloInfoError> {
    let way = WayInfo::get(way_id, state.conn.clone()).await?;
    let s = vec![
        (0.2, "🔴 Impossible"),
        (0.4, "🟠 mauvais"),
        (0.6, "🟡 difficile"),
        (0.8, "🟢 bon"),
        (1., "🔵 excellent"),
    ];
    println!("way: {:?}", way);
    let options = s
        .iter()
        .map(|(score, color)| {
            ScoreOption {
                score: *score,
                good_score: way.score.unwrap_or(0.8),
                color: color.to_string(),
            }
            .render()
            .unwrap()
        })
        .collect::<Vec<String>>()
        .join(" ");

    let segment_name =
        way.name.unwrap_or("nom inconnu".to_string()) + format!(" ({})", way.way_id).as_str();

    let info_panel = InfoPanel {
        way_id,
        status: "segment".to_string(),
        segment_name,
        options,
    }
    .render()
    .unwrap()
    .to_string();

    Ok(info_panel)
}

#[derive(Template)]
#[template(path = "score_option.html", escape = "none")]
struct ScoreOption {
    score: f64,
    good_score: f64,
    color: String,
}
