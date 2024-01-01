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
