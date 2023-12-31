use chrono::FixedOffset;
use serde::Deserialize;

use super::models::{Tour, ToursContainer};

pub struct ApiContext {
    base_url: String,
    http_client: reqwest::Client,
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
    pub fn new(url: &str, client: &reqwest::Client) -> ApiContext {
        ApiContext {
            base_url: url.to_string(),
            http_client: client.clone(),
            user_context: None,
        }
    }

    pub async fn auth(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let user_context = self
            .http_client
            .get(format!(
                "{0}/v006/account/email/{1}/",
                self.base_url, username
            ))
            .basic_auth(username, Some(password))
            .send()
            .await?
            .error_for_status()?
            .json::<UserContext>()
            .await?;

        Ok(ApiContext {
            base_url: self.base_url.to_owned(),
            http_client: self.http_client.clone(),
            user_context: Some(user_context),
        })
    }

    pub async fn tours(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        limit: u16,
    ) -> Result<Vec<Tour>, Box<dyn std::error::Error>> {
        let ctx = self.context();

        let query_params = &[
            ("limit", limit.to_string()),
            ("page", "0".to_owned()),
            ("type", "tour_recorded".to_owned()),
            (
                "start_date",
                start_date
                    .with_timezone::<FixedOffset>(&chrono::FixedOffset::west_opt(7 * 3600).unwrap())
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, false),
            ),
        ];

        let resp = self
            .http_client
            .get(format!(
                "{0}/v007/users/{1}/tours/",
                self.base_url, ctx.user_id
            ))
            .basic_auth(&ctx.email, Some(&ctx.token))
            .query(query_params)
            .send()
            .await?
            .error_for_status()?;

        let tours = resp
            .json::<ToursContainer>()
            .await?
            .embedded
            .map_or(Vec::default(), |e| e.tours);

        Ok(tours)
    }

    pub async fn download(&self, id: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let ctx = self.context();
        let url = format!("{0}/v007/tours/{1}.gpx", self.base_url, id);
        let req = self
            .http_client
            .get(url)
            .basic_auth(&ctx.email, Some(&ctx.token));
        let resp = req.send().await?.error_for_status()?;

        Ok(resp.bytes().await?.to_vec())
    }

    fn context(&self) -> &UserContext {
        self.user_context.as_ref().expect(
            "User context must not be empty. Make sure that auth(...) is called before calling this method.")
    }
}
