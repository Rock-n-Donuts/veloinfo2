use askama::Template;
use axum_extra::extract::CookieJar;
use lazy_static::lazy_static;
use std::env;

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
            keycloak_url: KEYCLOAK_URL.clone(),
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
        keycloak_url: KEYCLOAK_URL.clone(),
        login: false,
    }
}

//test call to keycloak
#[cfg(test)]
mod tests{
    use serde::Serialize;
    use serde_json::from_str;

    #[derive(Debug, serde::Deserialize)]
    struct Token{
        access_token: String,
    }

    #[derive(Debug, serde::Deserialize, Serialize)]
    struct Userinfo{
        sub: String,
        email: String,
        email_verified: bool,
        name: String,
        preferred_username: String,
        given_name: String,
        family_name: String,
    }

    #[tokio::test]
    pub async fn test_keycloak(){
        let client = reqwest::Client::new();
        let token = client
            .post("http://keycloak:8080/realms/master/protocol/openid-connect/token")
            .form(&[
                ("grant_type", "password"),
                ("client_id", "veloinfo"),
                ("username", "martinhamel"),
                ("password", ":5@$>C=8:rMEhTs"),
                ("scope", "openid")
            ])
            .send()
            .await.unwrap().text().await.unwrap();
        println!("token 1: {:?}", token);
        let token = from_str::<Token>(&token).unwrap();

        // we call userinfo to get the user info
        let userinfo: Userinfo = client
            .get("http://keycloak:8080/realms/master/protocol/openid-connect/userinfo")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await
            .unwrap()
            .json().await.unwrap();

        println!("userinfo: {:?}", userinfo);
        assert!(userinfo.email == "martin@ma4s.org");
    }
}
