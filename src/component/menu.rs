use askama::Template;
use axum_extra::extract::CookieJar;
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    static ref KEYCLOAK_EXTERN_URL: String = env::var("KEYCLOAK_EXTERN_URL").expect("KEYCLOAK_URL must be set");
}

#[derive(Template)]
#[template(path = "menu.html", escape = "none")]
pub struct Menu {
    open: bool,
    lat: f64,
    lng: f64,
    zoom: i32,
    keycloak_url: String,
    login: bool,
}

impl Menu {
    pub fn login(lat: f64, lng: f64, zoom: i32) -> Menu {
        Menu {
            open: true,
            lat,
            lng,
            zoom,
            keycloak_url: KEYCLOAK_EXTERN_URL.to_string() + "/protocol/openid-connect",
            login: true,
        }
    }
}

pub async fn menu_open(jar: CookieJar) -> (CookieJar, Menu) {
    let lat = match jar.get("lat") {
        Some(c) => c.value().parse::<f64>().unwrap_or_default(),
        None => {
            return (jar, Menu::login(0.0, 0.0, 0));
        }
    };
    let lng = match jar.get("lng") {
        Some(c) => c.value().parse::<f64>().unwrap_or_default(),
        None => {
            return (jar, Menu::login(0.0, 0.0, 0));
        }
    };
    let zoom = match jar.get("zoom") {
        Some(c) => c.value().parse::<f64>().unwrap_or_default() as i32,
        None => {
            return (jar, Menu::login(0.0, 0.0, 0));
        }
    };
    let openid = match jar.get("openid") {
        Some(openid) => {
            openid.value()
        }
        None => {
            eprintln!("No openid in cookie jar");
            return (jar, Menu::login(lat, lng, zoom));
        }
    };

    println!("OpenID: {:?}", openid);

    (
        jar,
        Menu {
            open: true,
            lat,
            lng,
            zoom,
            keycloak_url: KEYCLOAK_EXTERN_URL.to_string() + "/protocol/openid-connect",
            login: true,
        },
    )
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
        keycloak_url: KEYCLOAK_EXTERN_URL.to_string() + "/protocol/openid-connect",
        login: false,
    }
}

