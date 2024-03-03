use std::error;

use chrono::FixedOffset;
use futures::{
    stream::{self},
    Stream,
};
use serde::Deserialize;

use super::models::{Tour, ToursContainer};

#[derive(Debug)]
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
    ) -> Result<Self, Box<dyn error::Error>> {
        let url = format!("{0}/v006/account/email/{1}/", Self::BASE_URL, username);
        let req = client.get(url).basic_auth(username, Some(password));
        let resp = req.send().await?.error_for_status()?;
        let ctx = resp.json::<UserContext>().await?;

        Ok(Self {
            http: client.clone(),
            user_context: ctx,
        })
    }

    pub fn tours_stream<'a>(
        &'a self,
        start_date: chrono::DateTime<chrono::Utc>,
        page_size: u8,
    ) -> impl Stream<Item = Result<Vec<Tour>, Box<dyn error::Error>>> + 'a {
        stream::try_unfold((0, None), move |state| async move {
            match state {
                (curr_page, Some(total_pages)) if curr_page >= total_pages => Ok(None),
                (curr_page, _) => self
                    .tours(start_date, curr_page, page_size)
                    .await
                    .map(|(tours, total_pages)| Some((tours, (curr_page + 1, Some(total_pages))))),
            }
        })
    }

    async fn tours(
        &self,
        start_date: chrono::DateTime<chrono::Utc>,
        page: u16,
        limit: u8,
    ) -> Result<(Vec<Tour>, u16), Box<dyn error::Error>> {
        let start_date = start_date
            .with_timezone::<FixedOffset>(&chrono::FixedOffset::west_opt(7 * 3600).unwrap())
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, false);

        let query_params = &[
            ("limit", limit.to_string()),
            ("page", page.to_string()),
            ("type", "tour_recorded".to_owned()),
            ("start_date", start_date),
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
        let payload = resp.json::<ToursContainer>().await?;

        Ok((
            payload.embedded.map_or(Vec::default(), |e| e.tours),
            payload.page.total_pages,
        ))
    }

    pub async fn download(&self, id: u32) -> Result<Vec<u8>, Box<dyn error::Error>> {
        let url = format!("{0}/v007/tours/{1}.gpx", Self::BASE_URL, id);
        let req = self
            .http
            .get(url)
            .basic_auth(&self.user_context.email, Some(&self.user_context.token));
        let resp = req.send().await?.error_for_status()?;

        Ok(resp.bytes().await?.to_vec())
    }
}
