use askama::Template;
use axum::{extract::State, Form};
use axum_macros::debug_handler;

use crate::{db::address_range::AddressRange, VeloinfoState};

#[derive(Template)]
#[template(path = "search.html")]

pub struct Search {
    pub search_results: Vec<SearchResult>,
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
}

#[debug_handler]
pub async fn post(State(state): State<VeloinfoState>, Form(query): Form<QueryParams>) -> Search {
    println!("params {:?}", query);
    let search_results = AddressRange::get(&query.query, &0., &0., &state.conn)
        .await
        .into_iter()
        .map(|ar| SearchResult {
            street: ar.street,
            city: ar.city,
            lat: ar.lat,
            lng: ar.lng,
        })
        .collect();
    Search { search_results }
}

pub async fn open() -> Search {
    Search {
        search_results: vec![],
    }
}
