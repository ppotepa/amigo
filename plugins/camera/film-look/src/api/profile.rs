use super::FilmLookResponse2d;

#[derive(Clone, Debug, PartialEq)]
pub struct FilmLookProfile2d {
    pub id: String,
    pub label: String,
    pub response: FilmLookResponse2d,
}

impl FilmLookProfile2d {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            response: FilmLookResponse2d::default(),
        }
    }
}
