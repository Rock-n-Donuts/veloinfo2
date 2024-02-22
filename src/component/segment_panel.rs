use std::env;

use super::{
    info_panel::InfopanelContribution, score_circle::ScoreCircle, score_selector::ScoreSelector,
};
use crate::{db::cyclability_score::CyclabilityScore, VeloInfoError, VeloinfoState};
use anyhow::Result;
use askama::Template;
use axum::extract::multipart::Multipart;
use axum::extract::{Path, State};
use futures::future::join_all;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Postgres;

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
    photo_ids: Vec<i32>,
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
    pub photo: Option<i64>,
}

lazy_static! {
    static ref RE_NUMBER: Regex = Regex::new(r"\d+").unwrap();
}

lazy_static! {
    static ref IMAGE_DIR: String = env::var("IMAGE_DIR").unwrap();
}

pub async fn segment_panel_post(
    State(state): State<VeloinfoState>,
    mut multipart: Multipart,
) -> Result<SegmentPanel, VeloInfoError> {
    let mut score = -1.;
    let mut comment = "".to_string();
    let mut way_ids = "".to_string();
    let mut photo = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();
        match name {
            "score" => score = field.text().await.unwrap().parse::<f64>().unwrap(),
            "comment" => comment = field.text().await.unwrap(),
            "way_ids" => way_ids = field.text().await.unwrap(),
            "photo" => photo = Some(field.bytes().await.unwrap()),
            _ => (),
        }
    }
    let way_ids_i64 = RE_NUMBER
        .find_iter(way_ids.as_str())
        .map(|m| m.as_str().parse::<i64>().unwrap())
        .collect::<Vec<i64>>();

    let id = CyclabilityScore::insert(
        score,
        Some(comment),
        way_ids_i64,
        match photo.as_ref() {
            Some(_photo) => Some(IMAGE_DIR.to_string() + "/{}.jpeg"),
            None => None,
        },
        match photo.as_ref() {
            Some(_photo) => Some(IMAGE_DIR.to_string() + "/{}_thumbnail.jpeg"),
            None => None,
        },
        state.conn.clone(),
    )
    .await?;

    if let Some(photo) = photo {
        let img = image::load_from_memory(&photo).unwrap();
        let img = img.resize(1500, 1500, image::imageops::FilterType::Lanczos3);  
        img.save(IMAGE_DIR.to_string() + "/"+id.to_string().as_str()+".jpeg").unwrap();
        let img = img.resize(300, 300, image::imageops::FilterType::Lanczos3);
        img.save(IMAGE_DIR.to_string() + "/"+id.to_string().as_str()+"_thumbnail.jpeg").unwrap();
    }

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

    let photo_ids = CyclabilityScore::get_photo_by_way_ids(&way_ids_i64, conn.clone()).await;
    let history = InfopanelContribution::get_history(&way_ids_i64, conn).await;
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
        photo_ids,
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
    let history = InfopanelContribution::get_history(&way_ids_i64, state.conn.clone()).await;
    let photo_ids = CyclabilityScore::get_photo_by_way_ids(&way_ids_i64, state.conn.clone()).await; 
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
        photo_ids,
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
    let history = InfopanelContribution::get_history(&score.way_ids, conn.clone()).await;
    let photo_ids = CyclabilityScore::get_photo_by_way_ids(&score.way_ids, conn.clone()).await;
    Ok(SegmentPanel {
        way_ids,
        score_circle: ScoreCircle { score: score.score },
        segment_name,
        score_selector: ScoreSelector::get_score_selector(score.score),
        comment: score.comment.unwrap_or("".to_string()),
        edit,
        history,
        photo_ids,
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
