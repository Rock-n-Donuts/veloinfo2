use askama::Template;

#[derive(Template)]
#[template(path = "score_selector.html")]
pub struct ScoreSelector {
    score: f64,
}

impl ScoreSelector{
    pub fn get_score_selector(score: f64) -> ScoreSelector {
        println!("score: {}", score);
        ScoreSelector {
            score,
        }
    }
}
