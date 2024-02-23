use crate::component::index_js::indexjs;
use crate::component::info_panel::info_panel_down;
use crate::component::info_panel::info_panel_up;
use crate::component::photo_scroll::photo_scroll;
use crate::component::segment_panel::segment_panel;
use crate::component::segment_panel::segment_panel_edit;
use crate::component::segment_panel::segment_panel_post;
use crate::component::segment_panel::select_score_id;
use crate::score_selector_controler::score_bounds_controler;
use anyhow::Result;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::Response;
use axum::routing::post;
use axum::routing::{get, Router};
use component::style::style;
use score_selector_controler::score_selector_controler;
use segment::route;
use segment::select;
use sqlx::PgPool;
use std::env;
use std::process::Command;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use lazy_static::lazy_static;

mod component;
mod db;
mod score_selector_controler;
mod segment;

lazy_static! {
    static ref IMAGE_DIR: String = env::var("IMAGE_DIR").unwrap();
}

#[derive(Clone)]
struct VeloinfoState {
    conn: PgPool,
}

#[tokio::main]
async fn main() {
    let dev = env::var("ENV").unwrap().as_str().contains("dev");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conn = PgPool::connect(format!("{}", env::var("DATABASE_URL").unwrap()).as_str())
        .await
        .unwrap();
    let state = VeloinfoState { conn: conn.clone() };

    sqlx::migrate!().run(&conn).await.unwrap();

    println!("Starting cron scheduler");
    let sched = JobScheduler::new().await.unwrap();
    sched
        .add(
            Job::new("0 0 0 * * *", |_uuid, _l| {
                println!("Importing data");
                let output = Command::new("./import.sh")
                    .output()
                    .expect("failed to execute process");
                println!("status: {}", output.status);
            })
            .unwrap(),
        )
        .await
        .unwrap();
    sched.start().await.unwrap();

    let mut app = Router::new()
        .route("/", get(index))
        .route("/segment_panel/id/:id", get(select_score_id))
        .route(
            "/segment_panel/ways/:way_ids",
            get(segment_panel),
        )
        .route(
            "/segment_panel/edit/ways/:way_ids",
            get(segment_panel_edit),
        )
        .route("/segment_panel", post(segment_panel_post))
        .route("/segment/select/:way_id", get(select))
        .route("/segment/route/:way_id1/:way_ids", get(route))
        .route(
            "/cyclability_score/geom/:cyclability_score_id",
            get(score_bounds_controler),
        )
        .route("/info_panel/down", get(info_panel_down))
        .route("/info_panel/up", post(info_panel_up))
        .route("/score_selector/:score", get(score_selector_controler))
        .route("/photo_scroll/:photo/:way_ids", get(photo_scroll))
        .route("/style.json", get(style))
        .route("/index.js", get(indexjs))
        .nest_service("/pub/", ServeDir::new("pub"))
        .nest_service("/images/", ServeDir::new(IMAGE_DIR.as_str()))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(1024 * 1024 * 10));

    if dev {
        let livereload = LiveReloadLayer::new();   
        app = app.layer(livereload.request_predicate(not_htmx_predicate));
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn not_htmx_predicate<T>(req: &Request<T>) -> bool {
    !req.headers().contains_key("hx-request")
}
struct VIError(anyhow::Error);

impl From<anyhow::Error> for VIError {
    fn from(error: anyhow::Error) -> Self {
        VIError(error)
    }
}

impl From<askama::Error> for VIError {
    fn from(error: askama::Error) -> Self {
        VIError(anyhow::Error::from(error))
    }
}

impl From<regex::Error> for VIError {
    fn from(error: regex::Error) -> Self {
        VIError(anyhow::Error::from(error))
    }
}

impl IntoResponse for VIError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate {}

async fn index() -> Result<impl IntoResponse, VIError> {
    let template = IndexTemplate {};
    let body = template.render()?;
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    Ok((headers, Html(body)))
}

pub struct VeloInfoError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for VeloInfoError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl From<regex::Error> for VeloInfoError {
    fn from(error: regex::Error) -> Self {
        VeloInfoError(anyhow::Error::from(error))
    }
}

impl From<anyhow::Error> for VeloInfoError {
    fn from(error: anyhow::Error) -> Self {
        VeloInfoError(error)
    }
}
impl From<sqlx::Error> for VeloInfoError {
    fn from(error: sqlx::Error) -> Self {
        VeloInfoError(anyhow::Error::from(error))
    }
}
