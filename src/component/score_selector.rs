use askama::Template;

#[derive(Template)]
#[template(path = "score_selector.html")]
pub struct ScoreSelector {
    category: usize,
    edit: bool,
}

impl ScoreSelector{
    pub fn get_score_selector(score: f64, edit: bool) -> ScoreSelector {
        println!("score: {}", score);
        let category = {
            if score <= 0.0 {
                0
            } else if score <= 0.25 {
                1
            } else if score <= 0.5 {
                2
            } else if score <= 0.75 {
                3
            } else if score <= 1.0 {
                4
            } else {
                10
            }
        };
        ScoreSelector {
            category,
            edit
        }
    }
}
