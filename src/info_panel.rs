use crate::{VeloInfoError, VeloinfoState};
use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    Form,
};
use futures::future::join_all;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

#[derive(Template)]
#[template(path = "info_panel.html", escape = "none")]
pub struct InfoPanel {
    score_id: String,
    way_ids: String,
    status: String,
    segment_name: String,
    options: String,
    comment: String,
}

#[derive(Debug, sqlx::FromRow, Clone)]
struct WayInfo {
    name: Option<String>,
    score: Option<f64>,
}

impl WayInfo {
    pub async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<WayInfo, sqlx::Error> {
        println!("get way_id: {}", way_id);
        sqlx::query_as(
            r#"select  
                c.name,
                cs.score,
                cs.comment
               from cycleway c
               left join cyclability_score cs on c.way_id = any(cs.way_ids)
               where c.way_id = $1
               order by created_at desc"#,
        )
        .bind(way_id)
        .fetch_one(&conn)
        .await
    }
}

#[derive(Debug, sqlx::FromRow, Clone)]
struct Score {
    id: i64,
    score: f64,
    comment: String,
    created_at: String,
    way_ids: Vec<i64>,
}

impl Score {
    pub async fn get(id: i64, conn: sqlx::Pool<Postgres>) -> Result<Score, sqlx::Error> {
        sqlx::query_as(
            r#"select *
               from cyclability_score
               where $1 = id"#,
        )
        .bind(id)
        .fetch_one(&conn)
        .await
    }
}

pub async fn get_panel() -> String {
    InfoPanel {
        score_id: "".to_string(),
        way_ids: "".to_string(),
        status: "none".to_string(),
        segment_name: "".to_string(),
        options: "".to_string(),
        comment: "".to_string(),
    }
    .render()
    .unwrap()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostValue {
    pub score: Option<f64>,
    pub comment: Option<String>,
}

pub async fn info_panel_post(
    State(state): State<VeloinfoState>,
    Path(way_ids): Path<String>,
    Form(post): Form<PostValue>,
) -> Result<String, VeloInfoError> {
    let re = Regex::new(r"\d*").unwrap();
    let way_ids_i64 = re
        .find_iter(way_ids.as_str())
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    println!("post: {:?}", post);
    let conn = state.conn.clone();

    sqlx::query(
        r#"INSERT INTO cyclability_score 
                    (way_ids, score, comment) 
                    VALUES ($1, $2, $3)"#,
    )
    .bind(way_ids_i64)
    .bind(post.score)
    .bind(post.comment)
    .execute(&conn)
    .await?;

    info_panel(State(state), Path(way_ids)).await
}

pub async fn info_panel_score_id(
    State(state): State<VeloinfoState>,
    Path(score_id): Path<String>,
) -> Result<String, VeloInfoError> {
    let score = Score::get(score_id.parse::<i64>().unwrap(), state.conn.clone()).await?;
    let way_info = WayInfo::get(score.way_ids[0], state.conn.clone()).await?;
    let segment_name = way_info.name.unwrap_or("nom inconnu".to_string())
        + format!(
            " ({})",
            score
                .way_ids
                .iter()
                .fold("".to_string(), |acc, id| acc.to_string()
                    + id.to_string().as_str())
        )
        .as_str();

    let info_panel = InfoPanel {
        score_id: score.id.to_string(),
        way_ids: score
            .way_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(" "),
        status: "segment".to_string(),
        segment_name,
        options: get_options(score.score),
        comment: score.comment,
    };
    Ok(info_panel.render().unwrap().to_string())
}

fn get_options(score: f64) -> String {
    let s = vec![
        (-1., "⚪ inconnu", "disabled"),
        (0.2, "🔴 Impossible", ""),
        (0.4, "🟠 mauvais", ""),
        (0.6, "🟡 difficile", ""),
        (0.8, "🟢 bon", ""),
        (1., "🔵 excellent", ""),
    ];
    s.iter()
        .map(|(s, color, disabled)| {
            ScoreOption {
                score: *s,
                selected: if *s == score {
                    "selected".to_string()
                } else {
                    "".to_string()
                },
                color: color.to_string(),
                disabled: disabled.to_string(),
            }
            .render()
            .unwrap()
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub async fn info_panel(
    State(state): State<VeloinfoState>,
    Path(way_ids): Path<String>,
) -> Result<String, VeloInfoError> {
    let re = Regex::new(r"\d+").unwrap();
    let way_ids_i64 = re
        .find_iter(way_ids.as_str())
        .map(|cap| cap.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    let ways = join_all(way_ids_i64.iter().map(|way_id| {
        let conn = state.conn.clone();
        let a = WayInfo::get(*way_id, conn);
        a
    }))
    .await;
    let all_same_score = ways
        .iter()
        .all(|way| way.as_ref().unwrap().score == ways[0].as_ref().unwrap().score);
    let mut way = ways[0].as_ref().unwrap().clone();
    println!("way: {:?}", way);
    if !all_same_score {
        way.score = Some(-1.);
    }
    let way_score = match way.score {
        Some(score) => score,
        None => 0.8,
    };
    let options = get_options(way_score);
    let segment_name =
        way.name.unwrap_or("nom inconnu".to_string()) + format!(" ({})", way_ids).as_str();

    let info_panel = InfoPanel {
        score_id: "".to_string(),
        way_ids,
        status: "segment".to_string(),
        segment_name,
        options,
        comment: "".to_string(),
    }
    .render()
    .unwrap()
    .to_string();

    Ok(info_panel)
}

#[derive(Template)]
#[template(
    source = r#"<option value="{{score}}" {{selected}} {{disabled}}>{{color}}</option>"#,
    escape = "none",
    ext = "txt"
)]
struct ScoreOption {
    score: f64,
    selected: String,
    color: String,
    disabled: String,
}
