use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use crate::errors::ClientError;
use serde_json::Value;
use std::time::Duration;

pub struct HiveInferClient {
    client: Client,
    base_url: String,
    auth_header: HeaderValue,
}

impl HiveInferClient {
    pub fn new(base_url: impl Into<String>, client_token: &str) -> Result<Self, ClientError> {
        let mut auth_header = HeaderValue::from_str(&format!("Bearer {}", client_token))?;
        auth_header.set_sensitive(true);

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(HiveInferClient {
            client,
            base_url: base_url.into(),
            auth_header,
        })
    }

    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        node: Option<&str>,
        stream: bool,
    ) -> Result<Value, ClientError> {
        let url = format!("{}/api/generate", self.base_url.trim_end_matches('/'));
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.auth_header.clone());
        if let Some(node_name) = node {
            headers.insert("Node", HeaderValue::from_str(node_name)?);
        }

        let body = serde_json::json!({ "model": model, "prompt": prompt, "stream": stream });
        let res = self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let data = res.json::<Value>().await?;
        Ok(data)
    }
}
