use anyhow::Result;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::Response;
use axum::routing::{get, Router};
use info_panel::info_panel_down;
use info_panel::info_panel_up;
use public::indexjs;
use public::style;
use segment::route;
use segment::select;
use segment_panel::{
    get_empty_segment_panel, segment_panel, segment_panel_post, segment_panel_score_id,
};
use sqlx::PgPool;
use std::env;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod bike_path;
mod info_panel;
mod public;
mod segment_panel;
mod db;
mod segment;

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

    // Prepare bike path because is is destroyed by the import
    bike_path::prepare_bp(conn.clone()).await.unwrap();

    let mut app = Router::new()
        .route("/", get(index))
        .route("/segment_panel", get(get_empty_segment_panel))
        .route(
            "/segment_panel/:way_ids",
            get(segment_panel).post(segment_panel_post),
        )
        .route(
            "/segment_panel_score/:score_id",
            get(segment_panel_score_id),
        )
        .route("/segment/select/:way_id", get(select))
        .route("/segment/route/:way_id1/:way_ids", get(route))
        .route("/info_panel/down", get(info_panel_down))
        .route("/info_panel/up", get(info_panel_up))
        .route("/style.json", get(style))
        .route("/index.js", get(indexjs))
        .nest_service("/pub/", ServeDir::new("pub"))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    if dev {
        app = app.layer(LiveReloadLayer::new());
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
struct IndexTemplate {
    segment_panel: String,
    info_panel: String,
}

async fn index(State(state): State<VeloinfoState>) -> Result<Html<String>, VIError> {
    let segment_panel = get_empty_segment_panel().await;
    let info_panel = info_panel::get_empty_info_panel(state.conn).await;
    let template = IndexTemplate {
        segment_panel,
        info_panel,
    };
    let body = template.render()?;
    Ok(Html(body))
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
