use std::{
    error::Error,
    fmt::{self, Display},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UploadStatus {
    pub id: i64,
    pub id_str: String,
    pub status: String,
    pub error: Option<String>,
    pub external_id: Option<String>,
    pub activity_id: Option<String>,
}

impl UploadStatus {
    pub fn to_result(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(error) = &self.error {
            Err(UploadError::Failed {
                msg: error.to_string(),
            })?
        } else if self.external_id.is_none() {
            Err(UploadError::InProgress)?
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum UploadError {
    InProgress,
    Failed { msg: String },
}

impl Error for UploadError {}

impl Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UploadError::InProgress => write!(f, "Upload is in progress."),
            UploadError::Failed { msg: m } => {
                write!(f, "Unrecoverable error occurred during upload: {}", m)
            }
        }
    }
}
