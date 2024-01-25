use askama::Template;
use std::env;

#[derive(Template)]
#[template(path = "style.json", escape = "none")]
struct Style {
    martin_url: String,
}

pub async fn style() -> String {
    let martin_url = env::var("MARTIN_URL").unwrap();

    Style {
        martin_url,
    }
    .render().unwrap()
}
