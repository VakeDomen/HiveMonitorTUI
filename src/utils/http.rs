use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION}, Client, Response};
use std::time::Duration;
use crate::errors::ClientError;

/// A simple HTTP client wrapper for HiveCore endpoints
pub struct HttpClient {
    client: Client,
    pub base_url: String,
    pub headers: HeaderMap,
}

impl HttpClient {
    /// Create a new HttpClient
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL including scheme and host, e.g. "http://localhost:6668"
    /// * `token` - Bearer token string
    pub fn new(base_url: impl Into<String>, token: &str) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        let mut auth_value = HeaderValue::from_str(&format!("Bearer {}", token))?;
        auth_value.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth_value);

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(HttpClient {
            client,
            base_url: base_url.into(),
            headers,
        })
    }

    /// Perform a GET request and deserialize JSON response
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self, 
        path: &str,
        headers: Option<HeaderMap<HeaderValue>>,
    ) -> Result<T, ClientError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        
        let mut used_headers = self.headers.clone();
        if headers.is_some() {
            used_headers = headers.unwrap();
        }
        
        let res = self.client
            .get(&url)
            .headers(used_headers)
            .send()
            .await?
            .error_for_status()?;
        let data = res.json::<T>().await?;
        Ok(data)
    }

    /// Perform a POST request with a JSON body and deserialize JSON response
    pub async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        headers: Option<HeaderMap<HeaderValue>>,
    ) -> Result<T, ClientError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        
        let mut used_headers = self.headers.clone();
        if headers.is_some() {
            used_headers = headers.unwrap();
        }
        
        let res = self.client
            .post(&url)
            .headers(used_headers)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        let data = res.json::<T>().await?;
        Ok(data)
    }

    pub async fn post_raw<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: Option<HeaderMap<HeaderValue>>,
    ) -> Result<Response, ClientError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        
        let mut used_headers = self.headers.clone();
        if headers.is_some() {
            used_headers = headers.unwrap();
        }
        
        let res = self.client
            .post(&url)
            .headers(used_headers)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(res)
    }

    pub async fn delete<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
        headers: Option<HeaderMap<HeaderValue>>,
    ) -> Result<T, ClientError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));

        let mut used_headers = self.headers.clone();
        if headers.is_some() {
            used_headers = headers.unwrap();
        }

        let res = self.client
            .delete(&url)
            .headers(used_headers)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        let data = res.json::<T>().await?;
        Ok(data)
    }

    pub async fn delete_raw<B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
        headers: Option<HeaderMap<HeaderValue>>,
    ) -> Result<Response, ClientError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));

        let mut used_headers = self.headers.clone();
        if headers.is_some() {
            used_headers = headers.unwrap();
        }

        let res = self.client
            .delete(&url)
            .headers(used_headers)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(res)
    }
}
