use crate::auth::auth;
use crate::auth::logout;
use crate::component::index_js::indexjs;
use crate::component::info_panel::info_panel_down;
use crate::component::info_panel::info_panel_up;
use crate::component::menu::{menu_close, menu_open};
use crate::component::photo_scroll::photo_scroll;
use crate::component::point_panel::PointPanel;
use crate::component::segment_panel::segment_panel_bigger;
use crate::component::segment_panel::segment_panel_bigger_route;
use crate::component::segment_panel::segment_panel_edit;
use crate::component::segment_panel::segment_panel_lng_lat;
use crate::component::segment_panel::segment_panel_post;
use crate::component::segment_panel::select_score_id;
use crate::node::route;
use crate::node::select_nodes;
use crate::score_selector_controler::score_bounds_controler;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::post;
use axum::routing::{get, Router};
use component::style::style;
use lazy_static::lazy_static;
use score_selector_controler::score_selector_controler;
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

mod auth;
mod component;
mod db;
mod node;
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
            Job::new("0 0 8 * * *", |_uuid, _l| {
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
        .route("/auth", get(auth))
        .route("/logout", get(logout))
        .route(
            "/select/nodes/:start_lng/:start_lat/:end_lng/:end_lat",
            get(select_nodes),
        )
        .route("/segment_panel/id/:id", get(select_score_id))
        .route(
            "/segment_panel_lng_lat/:lng/:lat",
            get(segment_panel_lng_lat),
        )
        .route("/segment_panel/edit/ways/:way_ids", get(segment_panel_edit))
        .route("/segment_panel", post(segment_panel_post))
        .route("/segment_panel_bigger", get(segment_panel_bigger))
        .route(
            "/segment_panel_bigger/:start_lng/:start_lat/:end_lng/:end_lat",
            get(segment_panel_bigger_route),
        )
        .route("/menu/open", get(menu_open))
        .route("/menu/closed", get(menu_close))
        .route("/segment/select/:way_id", get(select))
        .route("/point/select/:lng/:lat", get(PointPanel::select))
        .route("/route/:start_lng/:start_lat/:end_lgt/:end_lat", get(route))
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

#[derive(Template)]
#[template(path = "index.html", escape = "none")]
pub struct IndexTemplate {}

pub async fn index() -> (HeaderMap, IndexTemplate) {
    let template = IndexTemplate {};
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    (headers, template)
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
