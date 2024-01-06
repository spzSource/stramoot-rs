use chrono::FixedOffset;
use futures::TryStreamExt;
use serde::Deserialize;

use super::models::{DownloadError, Tour, ToursContainer};

pub struct ApiContext {
    http: reqwest::Client,
    user_context: Option<UserContext>,
}

#[derive(Debug, Deserialize)]
pub struct UserContext {
    email: String,
    #[serde(rename = "username")]
    user_id: String,
    #[serde(rename = "password")]
    token: String,
}

impl ApiContext {
    const BASE_URL: &'static str = "https://api.komoot.de";

    pub fn new(client: &reqwest::Client) -> Self {
        Self {
            http: client.clone(),
            user_context: None,
        }
    }

    pub async fn auth(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!("{0}/v006/account/email/{1}/", Self::BASE_URL, username);
        let req = self.http.get(url).basic_auth(username, Some(password));
        let resp = req.send().await?.error_for_status()?;
        let ctx = resp.json::<UserContext>().await?;

        Ok(ApiContext {
            http: self.http.clone(),
            user_context: Some(ctx),
        })
    }

    pub async fn tours(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Tour>, Box<dyn std::error::Error>> {
        let ctx = self.context();

        let query_params = &[
            ("page", "0".to_owned()),
            ("type", "tour_recorded".to_owned()),
            (
                "start_date",
                start_date
                    .with_timezone::<FixedOffset>(&chrono::FixedOffset::west_opt(7 * 3600).unwrap())
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, false),
            ),
        ];

        let url = format!("{0}/v007/users/{1}/tours/", Self::BASE_URL, ctx.user_id);
        let req = self
            .http
            .get(url)
            .basic_auth(&ctx.email, Some(&ctx.token))
            .query(query_params);
        let resp = req.send().await?.error_for_status()?;
        let tours = resp
            .json::<ToursContainer>()
            .await?
            .embedded
            .map_or(Vec::default(), |e| e.tours);

        Ok(tours)
    }

    pub async fn stream(
        &self,
        id: u32,
    ) -> Result<
        impl futures::Stream<Item = Result<bytes::Bytes, DownloadError>> + 'static,
        Box<dyn std::error::Error>,
    > {
        let ctx = self.context();
        let url = format!("{0}/v007/tours/{1}.gpx", Self::BASE_URL, id);
        let req = self.http.get(url).basic_auth(&ctx.email, Some(&ctx.token));
        let resp = req.send().await?.error_for_status()?;

        Ok(resp
            .bytes_stream()
            .map_err(move |e| DownloadError::StreamRead { id, inner: e }))
    }

    fn context(&self) -> &UserContext {
        self.user_context.as_ref().expect(
            "User context must not be empty. Make sure that auth(...) is called before calling this method.")
    }
}
