use std::env;

use askama::Template;
use askama_axum::IntoResponse;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::Html;
use axum::routing::{get, post, Router};
use edit_buttons::{get_edit_buttons, get_start_buttons};
use segment::merge;
use segment::select;
use sqlx::PgPool;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use anyhow::Result;

mod edit_buttons;
mod segment;

#[derive(Clone)]
struct VeloinfoState {
    conn: PgPool,
}

#[tokio::main]
async fn main() {
    let conn = PgPool::connect(format!("{}", env::var("DATABASE_URL").unwrap()).as_str())
        .await
        .unwrap();

    sqlx::migrate!().run(&conn).await.unwrap();
    let state = VeloinfoState { conn: conn.clone() };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(index))
        .route("/edit_buttons/:edit", get(get_edit_buttons)) // Fix: Call get_edit_buttons() inside get()
        .route("/cycleway/select/:way_id", get(select))
        .route("/cycleway/merge/:way_id", post(merge))
        .with_state(state)
        .layer(LiveReloadLayer::new())
        .nest_service("/pub/", ServeDir::new("pub"));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate {
    edit_buttons: String,
}

struct VIError(anyhow::Error);

impl From<anyhow::Error> for VIError {
    fn from(error: anyhow::Error) -> Self {
        VIError(error)
    }
}

impl From <askama::Error> for VIError {
    fn from(error: askama::Error) -> Self {
        VIError(anyhow::Error::from(error))
    }
}

impl IntoResponse for VIError {
    fn into_response(self) -> Response{
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}



async fn index() -> Result<Html<String>, VIError> {
    let edit_buttons = get_start_buttons();
    let template = IndexTemplate {
        edit_buttons: edit_buttons.render()?.to_string(),
    };
    let body = template.render()?;
    Ok(Html(body))
}
