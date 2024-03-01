use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::from_str;
use sqlx::error;
use std::env;

use askama::Template;
use axum_extra::extract::CookieJar;

lazy_static! {
    static ref KEYCLOAK_URL: String = env::var("KEYCLOAK_URL").expect("KEYCLOAK_URL must be set");
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
    pub fn logout(lat: f64, lng: f64, zoom: i32) -> Menu {
        Menu {
            open: true,
            lat,
            lng,
            zoom,
            keycloak_url: KEYCLOAK_URL.clone(),
            login: false,
        }
    }
    pub fn login(lat: f64, lng: f64, zoom: i32) -> Menu {
        Menu {
            open: true,
            lat,
            lng,
            zoom,
            keycloak_url: KEYCLOAK_URL.clone(),
            login: true,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Jwt{
    access_token: String,
}

pub async fn menu_open(jar: CookieJar) -> (CookieJar, Menu) {
    let lat = jar.get("lat").unwrap().value().parse::<f64>().unwrap();
    let lng = jar.get("lng").unwrap().value().parse::<f64>().unwrap();
    let zoom = jar.get("zoom").unwrap().value().parse::<f64>().unwrap() as i32;
    let jwt: Jwt = match jar.get("jwt") {
        Some(c) => {
            println!("JWT 21: {:?}", c.value());
            from_str(c.value()).unwrap()},
        None => {
            println!("No JWT");
            return (jar, Menu::login(lat, lng, zoom));
        }
    };
    let client = reqwest::Client::new();
    let login = match client
        .get("http://keycloak:8080/realms/master/protocol/openid-connect/userinfo")
        .header("Authorization", format!("Bearer {}", jwt.access_token))
        .send()
        .await
    {
        Ok(jwt) => match jwt.text().await {
            Ok(jwt) => {
                eprintln!("JWT 23: {:?}", jwt);
                false
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return (jar, Menu::login(lat, lng, zoom));
            }
        },
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return (jar, Menu::login(lat, lng, zoom));
        }
    };

    println!("Login: {:?}", login);

    (
        jar,
        Menu {
            open: true,
            lat,
            lng,
            zoom,
            keycloak_url: KEYCLOAK_URL.clone(),
            login,
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
        keycloak_url: KEYCLOAK_URL.clone(),
        login: false,
    }
}
