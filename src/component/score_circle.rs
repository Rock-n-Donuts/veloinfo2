use askama::Template;

#[derive(Template, Clone)]
#[template(path = "score_circle.html")]
pub struct ScoreCircle {
    pub score: f64,
}

