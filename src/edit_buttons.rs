use askama::Template;
use axum::response::Html;
use axum::extract::Path;

#[derive(Template)]
#[template(path = "edit-buttons.html")]
pub struct EditButtons {
    editing: bool,
}

pub fn get_start_buttons() -> EditButtons {
    EditButtons { editing: false }
}

pub async fn get_edit_buttons(Path(editing): Path<bool>) -> Html<String> {
    Html(EditButtons { editing }.render().unwrap().to_string())
}
