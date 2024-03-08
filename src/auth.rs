use axum::{extract::Query, response::Redirect};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

lazy_static! {
    static ref KEYCLOAK_LOCAL_URL: String =
        std::env::var("KEYCLOAK_LOCAL_URL").expect("KEYCLOAK_URL must be set");
}

#[derive(Debug, Deserialize)]
pub struct Auth {
    code: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Token {
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
        ("redirect_uri", "http://localhost:3000/auth"),
    ];
    let (token, token_string) = match client
        .post(format!(
            "{}{}",
            KEYCLOAK_LOCAL_URL.to_string(),
            "/protocol/openid-connect/token"
        ))
        .form(&params)
        .send()
        .await
    {
        Ok(token) => match token.text().await {
            Ok(token) => (match from_str::<Token>(&token) {
                Ok(token) => token,
                Err(e) => {
                    eprintln!("Token: {:?}", token);
                    eprintln!("Error json: {:?}", e);
                    return (jar.clone(), Redirect::to("/"));
                }
            }, token),
            Err(e) => {
                eprintln!("Error get text from the response: {:?}", e);
                return (jar.clone(), Redirect::to("/"));
            }
        },
        Err(e) => {
            eprintln!("Error making the token request: {:?}", e);
            return (jar.clone(), Redirect::to("/"));
        }
    };

    println!("token: {:?}", token_string);

    let userinfo = match client
        .get(KEYCLOAK_LOCAL_URL.to_string() + "/protocol/openid-connect/userinfo")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Authorization", format!("Bearer {}", token.access_token))
        .send()
        .await
    {
        Ok(userinfo) => {
            match userinfo.text().await {
            Ok(userinfo) => userinfo,
            Err(e) => {
                eprintln!("Error json: {:?}", e);
                return (jar.clone(), Redirect::to("/"));
            }
        }},
        Err(e) => {
            eprintln!("Error calling keycloak: {:?}", e);
            return (jar.clone(), Redirect::to("/"));
        }
    };

    println!("userinfo: {}", userinfo);

    (
        jar.add(Cookie::new("userinfo", userinfo)),
        Redirect::to("/"),
    )
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Userinfo {
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

    (jar.remove(Cookie::build("userinfo")), Redirect::to("/"))
}

//test call to keycloak
#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::from_str;

    #[derive(Debug, serde::Deserialize)]
    struct Token {
        access_token: String,
    }

    #[derive(Debug, serde::Deserialize, Serialize)]
    struct Userinfo {
        sub: String,
        email: String,
        email_verified: bool,
        name: String,
        preferred_username: String,
        given_name: String,
        family_name: String,
    }

    #[tokio::test]
    pub async fn test_keycloak() {
        let client = reqwest::Client::new();
        println!(
            "KEYCLOAK_LOCAL_URL: {:?}",
            crate::auth::KEYCLOAK_LOCAL_URL.to_string() + "/protocol/openid-connect/token"
        );
        let token = client
            .post(format!(
                "{}{}",
                crate::auth::KEYCLOAK_LOCAL_URL.to_string(),
                "/protocol/openid-connect/token"
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "veloinfo"),
                ("username", "martinhamel"),
                ("password", ":5@$>C=8:rMEhTs"),
                ("scope", "openid"),
            ])
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        println!("token 1: {:?}", token);
        let token = from_str::<Token>(&token).unwrap();

        // we call userinfo to get the user info
        let userinfo: Userinfo = client
            .get(format!(
                "{}{}",
                crate::auth::KEYCLOAK_LOCAL_URL.to_string(),
                "/protocol/openid-connect/userinfo"
            ))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Bearer {}", token.access_token))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        println!("userinfo: {:?}", userinfo);
        assert!(userinfo.email == "martin@ma4s.org");
    }
}
