use askama::Template;
use axum::response::Html;
use axum::routing::{get, Router};
use cycleway::cycleway;
use edit_buttons::{get_edit_buttons, get_start_buttons};
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod cycleway;
mod edit_buttons;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/edit_buttons/:edit", get(get_edit_buttons)) // Fix: Call get_edit_buttons() inside get()
        .route("/cycleway/:way_id", get(cycleway))
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

async fn root() -> Html<String> {
    let edit_buttons = get_start_buttons();
    let template = IndexTemplate {
        edit_buttons: edit_buttons.render().unwrap().to_string(),
    };
    let body = template.render().unwrap();
    Html(body)
}
