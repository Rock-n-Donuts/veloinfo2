use axum::routing::{get, Router};
use askama::Template;
use tower_livereload::LiveReloadLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_http_proxy=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/", get(root))
        .layer(LiveReloadLayer::new())
        .nest_service("/pub/", ServeDir::new("pub"));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
}

async fn root() -> axum::response::Html<String> {
    let template = IndexTemplate {};
    let body = template.render().unwrap();
    axum::response::Html(body)
}

