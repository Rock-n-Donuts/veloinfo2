use askama::Template;
use axum::{extract::State, Form};
use axum_macros::debug_handler;

use crate::{db::address_range::AddressRange, VeloinfoState};

#[derive(Template)]
#[template(path = "search.html")]

pub struct Search {
    pub search_results: Vec<SearchResult>,
    pub query: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Template, Debug)]
#[template(path = "search_result.html")]
pub struct SearchResult {
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

#[debug_handler]
pub async fn post(State(state): State<VeloinfoState>, Form(query): Form<QueryParams>) -> Search {
    let search_results = AddressRange::get(&query.query, &query.lng, &query.lat, &state.conn)
        .await
        .into_iter()
        .map(|ar| SearchResult {
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

pub async fn open() -> Search {
    Search {
        search_results: vec![],
        query: "".to_string(),
        lat: 0.0,
        lng: 0.0,
    }
}
