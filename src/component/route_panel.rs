use askama::Template;

#[derive(Template)]
#[template(path = "route_panel.html")]
pub struct RoutePanel {
    pub route_json: String,
}
