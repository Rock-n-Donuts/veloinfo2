use askama::Template;
use askama_axum::IntoResponse;
use axum::http::{HeaderMap, HeaderValue};
use std::env;

#[derive(Template)]
#[template(path = "index.js", escape = "none")]
struct IndexJs {
    martin_url: String,
}

pub async fn indexjs() -> impl IntoResponse {
    let martin_url = env::var("MARTIN_URL").expect("MARTIN_URL must be set");

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/javascript"),
    );
    let resp = IndexJs { martin_url };
    (headers, resp)
}
