use crate::VeloinfoState;
use askama::Template;
use axum::extract::{Path, State};

#[derive(Template)]
#[template(path = "point_panel.html", escape = "none")]
pub struct PointPanel {}

pub async fn point_panel_lng_lat(
    Path((lng, lat)): Path<(f64, f64)>,
    state: State<VeloinfoState>,
) -> PointPanel {
    PointPanel {}
}
