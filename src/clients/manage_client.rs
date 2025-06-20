use crate::utils::http::HttpClient;
use crate::errors::ClientError;
use serde_json::Value;

/// Client for HiveCore management API (port 6668)
pub struct HiveManageClient {
    http: HttpClient,
}

impl HiveManageClient {
    /// Create a new management client
    pub fn new(base_url: impl Into<String>, admin_token: &str) -> Result<Self, ClientError> {
        let http = HttpClient::new(base_url, admin_token)?;
        Ok(HiveManageClient { http })
    }

    /// Fetch all queue lengths (model- and node-based)
    pub async fn get_queue(&self) -> Result<Value, ClientError> {
        self.http.get("queue").await
    }

    /// Fetch concurrent connection counts per node
    pub async fn get_worker_connections(&self) -> Result<Value, ClientError> {
        self.http.get("worker/connections").await
    }

    /// Fetch verification status per node
    pub async fn get_worker_status(&self) -> Result<Value, ClientError> {
        self.http.get("worker/status").await
    }

    /// Fetch last ping timestamps per node
    pub async fn get_worker_pings(&self) -> Result<Value, ClientError> {
        self.http.get("worker/pings").await
    }

    /// Fetch supported model tags per node
    pub async fn get_worker_tags(&self) -> Result<Value, ClientError> {
        self.http.get("worker/tags").await
    }

    /// Fetch HiveCore and Ollama versions per node
    pub async fn get_worker_versions(&self) -> Result<Value, ClientError> {
        self.http.get("worker/versions").await
    }

    /// List all authentication keys
    pub async fn get_keys(&self) -> Result<Value, ClientError> {
        self.http.get("key").await
    }

    /// Create a new authentication key
    pub async fn create_key(&self, name: &str, role: &str) -> Result<Value, ClientError> {
        let body = serde_json::json!({ "name": name, "role": role });
        self.http.post("key", &body).await
    }
}
