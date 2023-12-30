use serde::Deserialize;

pub struct ApiContext {
    base_url: String,
    http_client: reqwest::Client,
    pub user_context: Option<UserContext>,
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
}

pub struct Tour {
    id: u32,
    name: String,
    status: String,
    r#type: String,
    date: String,
}
