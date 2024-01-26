use anyhow::Result;
use askama::Template;
use askama_axum::IntoResponse;
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::Response;
use axum::routing::{get, Router};
use public::indexjs;
use public::style;
use segment::route;
use segment::select;
use segment_panel::{get_panel, segment_panel, segment_panel_post, segment_panel_score_id};
use sqlx::PgPool;
use std::env;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_livereload::LiveReloadLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod bike_path;
mod public;
mod segment;
mod segment_panel;

#[derive(Clone)]
struct VeloinfoState {
    conn: PgPool,
}

#[tokio::main]
async fn main() {
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

    let app = Router::new()
        .route("/", get(index))
        .route("/segment_panel", get(get_panel))
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
        .route("/style.json", get(style))
        .route("/index.js", get(indexjs))
        .nest_service("/pub/", ServeDir::new("pub"))
        .with_state(state)
        .layer(LiveReloadLayer::new())
        .layer(TraceLayer::new_for_http());

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
}

async fn index() -> Result<Html<String>, VIError> {
    let segment_panel = get_panel().await;
    let template = IndexTemplate {
        segment_panel: segment_panel,
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
        // Votre logique de conversion ici
        // Par exemple, si SegmentError est une énumération avec une variante Sqlx :
        VeloInfoError(anyhow::Error::from(error))
    }
}
