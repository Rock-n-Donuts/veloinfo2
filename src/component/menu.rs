use askama::Template;
use axum_extra::extract::CookieJar;


#[derive(Template)]
#[template(path = "menu.html", escape = "none")]
pub struct Menu{
    open: bool,
    lat: f64,
    lng: f64,
    zoom: i32,
}


pub async fn menu_open(jar: CookieJar) -> Menu {
    let lat = jar.get("lat").unwrap().value().parse::<f64>().unwrap();
    let lng = jar.get("lng").unwrap().value().parse::<f64>().unwrap();
    let zoom = jar.get("zoom").unwrap().value().parse::<f64>().unwrap() as i32;
    Menu{open: true, lat, lng, zoom}
}

pub async fn menu_close() -> Menu {
    let lat = 0.0;
    let lng = 0.0;
    let zoom = 0;
    Menu{open: false, lat, lng, zoom}
}