use askama::Template;
use axum::extract::Path;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;

lazy_static! {
    static ref KEYCLOAK_BROWSER_URL: String =
        env::var("KEYCLOAK_BROWSER_URL").expect("KEYCLOAK_BROWSER_URL must be set");
    static ref VELOINFO_URL: String = env::var("VELOINFO_URL").expect("VELOINFO_URL must be set");
}

#[derive(Template)]
#[template(path = "menu.html", escape = "none")]
pub struct Menu {
    open: bool,
    lat: f64,
    lng: f64,
    zoom: i32,
}

#[derive(Debug, Deserialize)]
pub struct Position {
    lat: f64,
    lng: f64,
    zoom: f64,
}

pub async fn menu_open(Path(position): Path<Position>) -> Menu {
    let lat = position.lat;
    let lng = position.lng;
    let zoom = position.zoom.floor() as i32;
    Menu {
        open: true,
        lat,
        lng,
        zoom,
    }
}

pub async fn menu_close() -> Menu {
    let lat = 0.0;
    let lng = 0.0;
    let zoom = 0;
    Menu {
        open: false,
        lat,
        lng,
        zoom,
    }
}
