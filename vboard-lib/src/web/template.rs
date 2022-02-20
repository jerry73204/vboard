use crate::common::*;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub title: Cow<'static, str>,
    pub names: Vec<Cow<'static, str>>,
}

#[derive(Template)]
#[template(path = "video.html")]
pub struct Video {
    pub name: String,
}
