use std::{
    error::Error,
    fmt::{self, Display},
};

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
pub(super) struct ToursContainer {
    #[serde(rename = "_embedded")]
    pub(super) embedded: Option<Embedded>,
}

#[derive(Debug)]
pub enum DownloadError {
    StreamRead { id: u32, inner: reqwest::Error },
}

impl Into<reqwest::Error> for DownloadError {
    fn into(self) -> reqwest::Error {
        match self {
            DownloadError::StreamRead { id: _, inner } => inner,
        }
    }
}

impl Error for DownloadError {}

impl Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DownloadError::StreamRead { id, inner } => {
                write!(
                    f,
                    "Download for {}. Failed when reading stream. {}",
                    id, inner
                )
            }
        }
    }
}
