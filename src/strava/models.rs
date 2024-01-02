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
    pub fn to_result(&self) -> Result<(), UploadError> {
        if let Some(error) = &self.error {
            Err(UploadError::Failed {
                id: self.id,
                msg: error.to_string(),
            })?
        } else if self.external_id.is_none() {
            Err(UploadError::InProgress { id: self.id })?
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum UploadError {
    InProgress { id: i64 },
    Failed { id: i64, msg: String },
}

impl Error for UploadError {}

impl Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UploadError::InProgress { id } => write!(f, "Upload {}. Upload is in progress", id),
            UploadError::Failed { id, msg } => {
                write!(f, "Upload {}. Unrecoverable error: {}", id, msg)
            }
        }
    }
}
