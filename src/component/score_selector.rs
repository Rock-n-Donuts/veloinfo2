use askama::Template;

#[derive(PartialEq)]
pub enum Category {
    Good,
    Problems,
    Dangerous,
    Closed,
}

#[derive(Template)]
#[template(path = "score_selector.html")]
pub struct ScoreSelector {
    category: Category,
    score: f64,
}

impl ScoreSelector {
    pub fn get_score_selector(score: f64) -> ScoreSelector {
        let category = if score == 0.0 {
            Category::Closed
        } else if score <= 0.34 {
            Category::Dangerous
        } else if score <= 0.67 {
            Category::Problems
        } else {
            Category::Good
        };
        ScoreSelector { score, category }
    }
}
