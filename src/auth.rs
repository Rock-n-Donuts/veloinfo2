use axum::{extract::Query, response::Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Debug, Deserialize)]
pub struct Auth {
    code: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Token{
    access_token: String,
    expires_in: i32,
    refresh_expires_in: i32,
    refresh_token: String,
    token_type: String,
}

pub async fn auth(auth: Query<Auth>, jar: CookieJar) -> (CookieJar, Redirect) {
    let code = auth.code.clone();

    println!("Auth: {:?}", code);
    let client = reqwest::Client::new();
    let params = [
        ("code", code.as_str()),
        ("grant_type", "authorization_code"),
        ("client_id", "veloinfo"),
        ("scope", "openid"),
    ];
    let (token, token_string) = match client
        .post("http://keycloak:8080/realms/master/protocol/openid-connect/token")
        .form(&params)
        .send()
        .await
    {
        Ok(token) => match token.text().await {
            Ok(token) => 
                (from_str::<Token>(&token).unwrap(), token),
            Err(e) => {
                println!("Error: {:?}", e);
                return (jar.clone(), Redirect::to("/"));
            }
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return (jar.clone(), Redirect::to("/"));
        }
    };

    println!("Token: {:?}", token);

    let userinfo = match client
        .get("http://keycloak:8080/realms/master/protocol/openid-connect/userinfo")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .send()
        .await
    {
        Ok(userinfo) => {
            // println!("Userinfo: {:?}", userinfo);
            match userinfo.json::<Userinfo>().await {
            Ok(userinfo) => {
                userinfo
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return (jar.clone(), Redirect::to("/"));
            }}
        },
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return (jar.clone(), Redirect::to("/"));
        }
    };

    println!("userinfo: {:?}", userinfo);

    (
        jar.add(Cookie::new("token",  token_string)),
        Redirect::to("/"),
    )
}

#[derive(Debug, Deserialize, Serialize)]
struct Userinfo{
    sub: String,
    email: String,
    email_verified: bool,
    name: String,
    preferred_username: String,
    given_name: String,
    family_name: String,
}


pub async fn logout(jar: CookieJar) -> (CookieJar, Redirect) {
    println!("Logout");

    (jar.remove(Cookie::build("jwt")), Redirect::to("/"))
}
