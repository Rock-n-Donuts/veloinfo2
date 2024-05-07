use askama::Template;
use axum::{extract::State, Form};
use axum_macros::debug_handler;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    db::address_range::{get, get_with_adress, AddressRange},
    VeloinfoState,
};

#[derive(Template)]
#[template(path = "search.html", escape = "none")]

pub struct Search {
    pub search_results: Vec<SearchResult>,
    pub query: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Template, Debug)]
#[template(path = "search_result.html")]
pub struct SearchResult {
    pub number: String,
    pub street: String,
    pub city: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct QueryParams {
    pub query: String,
    pub lat: f64,
    pub lng: f64,
}

lazy_static! {
    static ref ADDRESS_RE: Regex = Regex::new(r"(\d+)(.*)").unwrap();
}

#[debug_handler]
pub async fn post(State(state): State<VeloinfoState>, Form(query): Form<QueryParams>) -> Search {
    match ADDRESS_RE.captures(&query.query) {
        Some(caps) => {
            let number = caps.get(1).unwrap().as_str().parse::<i64>().unwrap();
            let sub_query = caps.get(2).unwrap().as_str().to_string();
            let search_results = get_with_adress(&number, &sub_query, &state.conn)
                .await
                .into_iter()
                .map(|ar| SearchResult {
                    number: number.to_string(),
                    street: ar.street,
                    city: ar.city,
                    lat: ar.lat,
                    lng: ar.lng,
                })
                .collect();
            Search {
                search_results,
                query: query.query,
                lat: query.lat,
                lng: query.lng,
            }
        }
        None => {
            let search_results = get(&query.query, &query.lng, &query.lat, &state.conn)
                .await
                .into_iter()
                .map(|ar| SearchResult {
                    number: "".to_string(),
                    street: ar.street,
                    city: ar.city,
                    lat: ar.lat,
                    lng: ar.lng,
                })
                .collect();
            Search {
                search_results,
                query: query.query,
                lat: query.lat,
                lng: query.lng,
            }
        }
    }
}

pub async fn open() -> Search {
    Search {
        search_results: vec![],
        query: "".to_string(),
        lat: 0.0,
        lng: 0.0,
    }
}
