use futures::TryStreamExt;
use oauth2::{
    basic::BasicClient, AccessToken, AuthUrl, ClientId, ClientSecret, RefreshToken, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::multipart;

use crate::komoot::models::DownloadError;

use super::models::{UploadError, UploadStatus};

#[derive(Debug)]
pub struct ApiContext {
    pub http: reqwest::Client,
    pub access_token: Option<AccessToken>,
    pub refresh_token: Option<RefreshToken>,
}

impl ApiContext {
    const BASE_URL: &'static str = "https://www.strava.com";

    pub fn new(client: &reqwest::Client) -> Self {
        Self {
            http: client.clone(),
            access_token: None,
            refresh_token: None,
        }
    }

    pub async fn auth(
        &self,
        client_id: &str,
        client_secret: &str,
        refresh_token: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = BasicClient::new(
            ClientId::new(client_id.to_string()),
            Some(ClientSecret::new(client_secret.to_string())),
            AuthUrl::new(format!("{}/oauth/authorize", Self::BASE_URL))?,
            Some(TokenUrl::new(format!("{}/oauth/token", Self::BASE_URL))?),
        )
        .set_auth_type(oauth2::AuthType::RequestBody);

        let refresh_token_old = RefreshToken::new(refresh_token.to_string());
        let token_response = client
            .exchange_refresh_token(&refresh_token_old)
            .add_scope(Scope::new("activity:write_permission".to_string()))
            .add_extra_param("token_type", "Bearer")
            .request_async(oauth2::reqwest::async_http_client)
            .await?;

        Ok(Self {
            http: self.http.clone(),
            access_token: Some(token_response.access_token().to_owned()),
            refresh_token: token_response.refresh_token().map(|e| e.clone()),
        })
    }

    pub async fn upload(
        &self,
        external_id: &str,
        name: &str,
        stream: impl futures::Stream<Item = Result<bytes::Bytes, DownloadError>> + Send + Sync + 'static,
    ) -> Result<UploadStatus, Box<dyn std::error::Error>> {
        let resp = self
            .http
            .post(format!("{}/api/v3/uploads", Self::BASE_URL))
            .bearer_auth(self.access_token().secret())
            .multipart(Self::multipart_form(
                external_id,
                name,
                stream.map_err(DownloadError::into),
            ))
            .send()
            .await?
            .error_for_status()?;

        let status = resp.json::<UploadStatus>().await?;

        Ok(status)
    }

    pub async fn upload_status(
        &self,
        upload_id: i64,
    ) -> Result<UploadStatus, Box<dyn std::error::Error>> {
        let resp = self
            .http
            .get(format!("{}/api/v3/uploads/{}", Self::BASE_URL, upload_id))
            .bearer_auth(self.access_token().secret())
            .send()
            .await?
            .error_for_status()?;

        let status = resp.json::<UploadStatus>().await?;

        Ok(status)
    }

    pub async fn wait_for_upload(
        &self,
        upload_id: i64,
        attempts: u8,
        delay: chrono::Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut retries = 0;

        loop {
            match self.upload_status(upload_id).await?.to_result() {
                Err(err @ UploadError::InProgress { id: _ }) if retries < attempts => {
                    retries += 1;
                    println!("{}, retrying after {}", err, delay);
                    tokio::time::sleep(delay.to_std()?).await;
                }
                res => return res.map_err(|e| e.into()),
            }
        }
    }

    fn multipart_form(
        external_id: &str,
        name: &str,
        content: impl futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>>
            + Sync
            + Send
            + 'static,
    ) -> multipart::Form {
        let sw = reqwest::Body::wrap_stream(content);
        multipart::Form::new()
            .text("trainer", "0")
            .text("commute", "0")
            .text("data_type", "gpx")
            .text("activity_type", "ride")
            .text("name", name.to_string())
            .text("external_id", external_id.to_string())
            .part("data", multipart::Part::stream(sw))
    }

    fn access_token(&self) -> &AccessToken {
        self.access_token
            .as_ref()
            .expect("Access token is empty, make sure that auth(...) was called.")
    }
}
