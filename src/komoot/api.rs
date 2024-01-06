use chrono::FixedOffset;
use serde::Deserialize;

use super::models::{Tour, ToursContainer};

pub struct ApiContext {
    http: reqwest::Client,
    user_context: UserContext,
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

    pub async fn auth(
        username: &str,
        password: &str,
        client: &reqwest::Client,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!("{0}/v006/account/email/{1}/", Self::BASE_URL, username);
        let req = client.get(url).basic_auth(username, Some(password));
        let resp = req.send().await?.error_for_status()?;
        let ctx = resp.json::<UserContext>().await?;

        Ok(ApiContext {
            http: client.clone(),
            user_context: ctx,
        })
    }

    pub async fn tours(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Tour>, Box<dyn std::error::Error>> {
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

        let url = format!(
            "{0}/v007/users/{1}/tours/",
            Self::BASE_URL,
            self.user_context.user_id
        );
        let req = self
            .http
            .get(url)
            .basic_auth(&self.user_context.email, Some(&self.user_context.token))
            .query(query_params);
        let resp = req.send().await?.error_for_status()?;
        let tours = resp
            .json::<ToursContainer>()
            .await?
            .embedded
            .map_or(Vec::default(), |e| e.tours);

        Ok(tours)
    }

    pub async fn download(&self, id: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let url = format!("{0}/v007/tours/{1}.gpx", Self::BASE_URL, id);
        let req = self
            .http
            .get(url)
            .basic_auth(&self.user_context.email, Some(&self.user_context.token));
        let resp = req.send().await?.error_for_status()?;

        Ok(resp.bytes().await?.to_vec())
    }
}
