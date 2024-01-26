use askama::Template;
use std::env;

#[derive(Template)]
#[template(path = "style.json", escape = "none")]
struct Style {
    martin_url: String,
}

pub async fn style() -> String {
    let martin_url = env::var("MARTIN_URL").unwrap();

    Style { martin_url }.render().unwrap()
}

#[derive(Template)]
#[template(path = "index.js", escape = "none")]
struct IndexJs {
    martin_url: String,
}

pub async fn indexjs() -> String {
    let martin_url = env::var("MARTIN_URL").unwrap();

    IndexJs { martin_url }.render().unwrap()
}
