use super::{info_panel::InfoPanelTemplate, score_selector::ScoreSelector};
use crate::{db::cyclability_score::CyclabilityScore, VeloInfoError, VeloinfoState};
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
#[template(path = "segment_panel.html", escape = "none")]
pub struct SegmentPanel {
    status: String,
    way_ids: String,
    segment_name: String,
    score_selector: ScoreSelector,
    comment: String,
    info_panel_template: InfoPanelTemplate,
    edit: bool,
}

#[derive(Debug, sqlx::FromRow, Clone)]
struct WayInfo {
    way_id: i64,
    name: Option<String>,
    score: Option<f64>,
}

impl WayInfo {
    pub async fn get(way_id: i64, conn: sqlx::Pool<Postgres>) -> Result<WayInfo, sqlx::Error> {
        sqlx::query_as(
            r#"select  
                c.way_id,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostValue {
    pub score: Option<f64>,
    pub comment: Option<String>,
}

pub async fn segment_panel_post(
    State(state): State<VeloinfoState>,
    Path(way_ids): Path<String>,
    Form(post): Form<PostValue>,
) -> Result<SegmentPanel, VeloInfoError> {
    let re = Regex::new(r"\d*").unwrap();
    let way_ids_i64 = re
        .find_iter(way_ids.as_str())
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
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

    segment_panel(State(state), Path(way_ids)).await
}

pub async fn segment_panel_edit(
    State(state): State<VeloinfoState>,
    Path(way_ids): Path<String>,
) -> Result<String, VeloInfoError> {
    let re = Regex::new(r"\d+").unwrap();
    let way_ids_i64 = re
        .find_iter(way_ids.as_str())
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    let conn = state.conn.clone();
    let ways = join_all(
        way_ids_i64
            .iter()
            .map(|way_id| async { WayInfo::get(*way_id, conn.clone()).await.unwrap() }),
    )
    .await;
    let all_same_score = ways.iter().all(|way| way.score == ways[0].score);
    let mut way = ways[0].clone();
    if !all_same_score {
        way.score = Some(-1.);
    }
    let segment_name = ways
        .iter()
        .fold("".to_string(), |acc, way| match way.name.as_ref() {
            Some(name) => {
                if acc.find(name) != None {
                    return acc;
                }
                format!("{} {}", acc, name)
            }
            None => acc,
        });
    let info_panel = SegmentPanel {
        status: "segment".to_string(),
        way_ids: way_ids.clone(),
        segment_name,
        score_selector: ScoreSelector::get_score_selector(way.score.unwrap_or(-1.), true),
        comment: "".to_string(),
        info_panel_template: InfoPanelTemplate {
            arrow: "▲".to_string(),
            direction: "up".to_string(),
            contributions: Vec::new(),
        },
        edit: true,
    }
    .render()
    .unwrap()
    .to_string();

    Ok(info_panel)
}

pub async fn segment_panel(
    State(state): State<VeloinfoState>,
    Path(way_ids): Path<String>,
) -> Result<SegmentPanel, VeloInfoError> {
    let re = Regex::new(r"\d+").unwrap();
    let way_ids_i64 = re
        .find_iter(way_ids.as_str())
        .map(|cap| cap.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    let ways = join_all(way_ids_i64.iter().map(|way_id| async {
        let conn = state.conn.clone();
        WayInfo::get(*way_id, conn).await.unwrap()
    }))
    .await;
    let all_same_score = ways.iter().all(|way| way.score == ways[0].score);
    let mut way = ways[0].clone();
    if !all_same_score {
        way.score = Some(-1.);
    }
    let segment_name = ways
        .iter()
        .fold("".to_string(), |acc, way| match way.name.as_ref() {
            Some(name) => {
                if acc.find(name) != None {
                    return acc;
                }
                format!("{} {}", acc, name)
            }
            None => acc,
        });
    let info_panel = SegmentPanel {
        status: "segment".to_string(),
        way_ids: way_ids.clone(),
        segment_name,
        score_selector: ScoreSelector::get_score_selector(way.score.unwrap_or(-1.), false),
        comment: "".to_string(),
        info_panel_template: InfoPanelTemplate {
            arrow: "▲".to_string(),
            direction: "up".to_string(),
            contributions: Vec::new(),
        },
        edit: false,
    };

    Ok(info_panel)
}

pub async fn select_score_id(
    State(state): State<VeloinfoState>,
    Path(id): Path<i32>,
) -> Result<String, VeloInfoError> {
    let score = CyclabilityScore::get_by_id(id, state.conn.clone())
        .await
        .unwrap();
    let (segment_name, way_ids) = join_all(score.way_ids.iter().map(|way_id| async {
        let conn = state.conn.clone();
        WayInfo::get(*way_id, conn).await.unwrap()
    }))
    .await
    .iter()
    .fold(
        ("".to_string(), "".to_string()),
        |(names, ways), way| match way.name.as_ref() {
            Some(name) => {
                if names.find(name) != None {
                    return (names, ways);
                }
                (
                    format!("{} {}", names, name),
                    format!("{} {}", ways, way.way_id),
                )
            }
            None => (names, ways),
        },
    );
    let panel = SegmentPanel {
        status: "segment".to_string(),
        way_ids,
        segment_name,
        score_selector: ScoreSelector::get_score_selector(score.score, false),
        comment: score.comment.unwrap_or("".to_string()),
        info_panel_template: InfoPanelTemplate {
            arrow: "▲".to_string(),
            direction: "up".to_string(),
            contributions: Vec::new(),
        },
        edit: false,
    };

    Ok(panel.render().unwrap())
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
