use askama::Template;


#[derive(Template)]
#[template(path = "menu.html", escape = "none")]
pub struct Menu{
    open: bool,
}


pub async fn menu_open() -> Menu {
    Menu{open: true}
}

pub async fn menu_close() -> Menu {
    Menu{open: false}
}