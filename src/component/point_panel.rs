use askama::Template;

#[derive(Template)]
#[template(path = "point_panel.html", escape = "none")]
pub struct PointPanel {
    pub lng: f64,
    pub lat: f64,
}

impl PointPanel {
    pub async fn select() -> PointPanel {
        PointPanel { lng: 0., lat: 0. }
    }
}
