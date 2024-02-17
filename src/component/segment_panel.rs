use super::{
    info_panel::InfopanelContribution, score_circle::ScoreCircle, score_selector::ScoreSelector,
};
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
use lazy_static::lazy_static;

#[derive(Template)]
#[template(path = "segment_panel.html", escape = "none")]
pub struct SegmentPanel {
    way_ids: String,
    score_circle: ScoreCircle,
    segment_name: String,
    score_selector: ScoreSelector,
    comment: String,
    edit: bool,
    history: Vec<InfopanelContribution>,
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
    pub score: f64,
    pub comment: String,
    pub way_ids: String,
}

lazy_static! {
    static ref RE_NUMBER: Regex = Regex::new(r"\d+").unwrap();
}

pub async fn segment_panel_post(
    State(state): State<VeloinfoState>,
    Form(post): Form<PostValue>,
) -> Result<SegmentPanel, VeloInfoError> {
    let way_ids = post.way_ids;
    let way_ids_i64 = RE_NUMBER
        .find_iter(way_ids.as_str())
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();

    CyclabilityScore::insert(
        post.score,
        Some(post.comment.clone()),
        way_ids_i64.clone(),
        state.conn.clone(),
    )
    .await?;
    segment_panel(State(state), Path(way_ids)).await
}

pub async fn segment_panel_edit(
    State(state): State<VeloinfoState>,
    Path(way_ids): Path<String>,
) -> Result<SegmentPanel, VeloInfoError> {
    let way_ids_i64 = RE_NUMBER
        .find_iter(way_ids.as_str())
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();
    let cyclability_score =
        CyclabilityScore::get_by_way_ids(way_ids_i64.clone(), state.conn.clone()).await;
    if let Some(score) = cyclability_score {
        println!("score: {:?}", score);
        return segment_panel_score_id(state.conn.clone(), score.id, true).await;
    }

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

    let history = InfopanelContribution::get_history(way_ids_i64, conn).await;
    let segment_panel = SegmentPanel {
        way_ids: way_ids.clone(),
        score_circle: ScoreCircle {
            score: way.score.unwrap_or(-1.),
        },
        segment_name,
        score_selector: ScoreSelector::get_score_selector(way.score.unwrap_or(-1.)),
        comment: "".to_string(),
        edit: true,
        history,
    };

    Ok(segment_panel)
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
    let cyclability_score =
        CyclabilityScore::get_by_way_ids(way_ids_i64.clone(), state.conn.clone()).await;
    if let Some(score) = cyclability_score {
        return segment_panel_score_id(state.conn.clone(), score.id, false).await;
    }

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
    let history = InfopanelContribution::get_history(way_ids_i64, state.conn).await;

    let info_panel = SegmentPanel {
        way_ids: way_ids.clone(),
        score_circle: ScoreCircle {
            score: way.score.unwrap_or(-1.),
        },
        segment_name,
        score_selector: ScoreSelector::get_score_selector(way.score.unwrap_or(-1.)),
        comment: "".to_string(),
        edit: false,
        history,
    };

    Ok(info_panel)
}

async fn segment_panel_score_id(
    conn: sqlx::Pool<Postgres>,
    id: i32,
    edit: bool,
) -> Result<SegmentPanel, VeloInfoError> {
    let score = CyclabilityScore::get_by_id(id, conn.clone()).await.unwrap();
    let (segment_name, way_ids) = join_all(score.way_ids.iter().map(|way_id| async {
        let conn = conn.clone();
        WayInfo::get(*way_id, conn).await
    }))
    .await
    .iter()
    .fold(("".to_string(), "".to_string()), |(names, ways), way| {
        let way = match way {
            Ok(way) => way,
            _ => return (names, ways),
        };
        match way.name.as_ref() {
            Some(name) => {
                if names.find(name) != None {
                    return (names, format!("{} {}", ways, way.way_id));
                }
                (
                    format!("{} {}", names, name),
                    format!("{} {}", ways, way.way_id),
                )
            }
            None => (names, format!("{} {}", ways, way.way_id)),
        }
    });
    let history = InfopanelContribution::get_history(score.way_ids, conn).await;
    Ok(SegmentPanel {
        way_ids,
        score_circle: ScoreCircle { score: score.score },
        segment_name,
        score_selector: ScoreSelector::get_score_selector(score.score),
        comment: score.comment.unwrap_or("".to_string()),
        edit,
        history,
    })
}

pub async fn select_score_id(
    State(state): State<VeloinfoState>,
    Path(id): Path<i32>,
) -> Result<SegmentPanel, VeloInfoError> {
    segment_panel_score_id(state.conn.clone(), id, false).await
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
