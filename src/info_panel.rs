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
    way_ids: String,
    status: String,
    segment_name: String,
    options: String,
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
    }
}

pub async fn get_panel() -> String {
    InfoPanel {
        way_ids: "".to_string(),
        status: "none".to_string(),
        segment_name: "".to_string(),
        options: "".to_string(),
    }
    .render()
    .unwrap()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostValue {
    pub score: Option<f64>,
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
    join_all(way_ids_i64.iter().map(|way_id| {
        sqlx::query(
            r#"INSERT INTO cyclability_score 
                    (way_id, score) 
                    VALUES ($1, $2)"#,
        )
        .bind(way_id)
        .bind(post.score)
        .execute(&conn)
    }))
    .await;

    info_panel(State(state), Path(way_ids)).await
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
    let s = vec![
        (-1., "⚪ inconnu", "disabled"),
        (0.2, "🔴 Impossible", ""),
        (0.4, "🟠 mauvais", ""),
        (0.6, "🟡 difficile", ""),
        (0.8, "🟢 bon", ""),
        (1., "🔵 excellent", ""),
    ];
    let way_score = match way.score {
        Some(score) => score,
        None => 0.8,
    };
    let options = s
        .iter()
        .map(|(score, color, disabled)| {
            ScoreOption {
                score: *score,
                selected: if way_score == *score {
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
        .join(" ");
    let segment_name =
        way.name.unwrap_or("nom inconnu".to_string()) + format!(" ({})", way_ids).as_str();

    let info_panel = InfoPanel {
        way_ids,
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
