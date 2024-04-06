use crate::VeloinfoState;
use askama::Template;
use axum::extract::State;

#[derive(Template)]
#[template(path = "point_panel.html", escape = "none")]
pub struct PointPanel {
    pub lng: f64,
    pub lat: f64,
}

impl PointPanel {
    pub async fn select(State(state): State<VeloinfoState>) -> PointPanel {
        PointPanel { lng: 0., lat: 0. }
    }
}
