use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tour {
    pub id: u32,
    pub name: String,
    pub status: String,
    pub r#type: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct Embedded {
    pub(super) tours: Vec<Tour>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Page {
    #[serde(rename = "totalPages")]
    pub(super) total_pages: u16,
}

#[derive(Debug, Deserialize)]
pub(super) struct ToursContainer {
    #[serde(rename = "_embedded")]
    pub(super) embedded: Option<Embedded>,

    pub(super) page: Page,
}
