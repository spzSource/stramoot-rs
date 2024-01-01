use oauth2::{
    basic::BasicClient, AccessToken, AuthUrl, ClientId, ClientSecret, RefreshToken, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::multipart;

use super::models::UploadStatus;

#[derive(Debug)]
pub struct ApiContext {
    pub http_client: reqwest::Client,
    pub access_token: Option<AccessToken>,
    pub refresh_token: Option<RefreshToken>,
}

impl ApiContext {
    const BASE_URL: &'static str = "https://www.strava.com";

    pub fn new(client: &reqwest::Client) -> Self {
        Self {
            http_client: client.clone(),
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
            http_client: self.http_client.clone(),
            access_token: Some(token_response.access_token().to_owned()),
            refresh_token: token_response.refresh_token().map(|e| e.clone()),
        })
    }

    pub async fn upload(
        &self,
        external_id: &str,
        name: &str,
        content: &[u8],
    ) -> Result<UploadStatus, Box<dyn std::error::Error>> {
        let token = self
            .access_token
            .as_ref()
            .expect("Access token is empty, make sure that auth(...) was called.");

        let resp = self
            .http_client
            .post(format!("{}/api/v3/uploads", Self::BASE_URL))
            .bearer_auth(token.secret())
            .multipart(Self::multipart_form(external_id, name, content))
            .send()
            .await?;

        let status = resp.json::<UploadStatus>().await?;

        Ok(status)
    }

    fn multipart_form(external_id: &str, name: &str, content: &[u8]) -> multipart::Form {
        multipart::Form::new()
            .text("activity_type", "ride")
            .text("trainer", "0")
            .text("commute", "0")
            .text("data_type", "gpx")
            .text("name", name.to_string())
            .text("external_id", external_id.to_string())
            .part("data", multipart::Part::bytes(content.to_owned()))
    }
}
