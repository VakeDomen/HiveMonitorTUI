use crate::models::{AuthKeys, QueueMap, WorkerConnections, WorkerStatuses, WorkerTags, WorkerVersions};
use crate::utils::http::HttpClient;
use crate::errors::ClientError;


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
    pub async fn get_queue(&self) -> Result<QueueMap, ClientError> {
        let raw = self.http.get("queue").await?;
        Ok(serde_json::from_value(raw)?)
    }

    /// Fetch concurrent connection counts per node
    pub async fn get_worker_connections(&self) -> Result<WorkerConnections, ClientError> {
        let raw = self.http.get("worker/connections").await?;
        Ok(serde_json::from_value(raw)?)
    }

    /// Fetch verification status per node
    pub async fn get_worker_status(&self) -> Result<WorkerStatuses, ClientError> {
        let raw = self.http.get("worker/status").await?;
        Ok(serde_json::from_value(raw)?)
    }

    /// Fetch last ping timestamps per node
    pub async fn get_worker_pings(&self) -> Result<crate::models::WorkerPings, ClientError> {
        let raw = self.http.get("worker/pings").await?;
        Ok(crate::utils::parsing::parse_worker_pings(raw))
    }

    /// Fetch supported model tags per node
    pub async fn get_worker_tags(&self) -> Result<WorkerTags, ClientError> {
        let raw = self.http.get("worker/tags").await?;
        Ok(serde_json::from_value(raw)?)
    }

    /// Fetch HiveCore and Ollama versions per node
    pub async fn get_worker_versions(&self) -> Result<WorkerVersions, ClientError> {
        let raw = self.http.get("worker/versions").await?;
        Ok(serde_json::from_value(raw)?)
    }

    /// List all authentication keys
    pub async fn get_keys(&self) -> Result<AuthKeys, ClientError> {
        let raw = self.http.get("key").await?;
        Ok(crate::utils::parsing::parse_auth_keys(raw))
    }

    /// Create a new authentication key
    pub async fn create_key(&self, name: &str, role: &str) -> Result<AuthKeys, ClientError> {
        let body = serde_json::json!({ "name": name, "role": role });
        let raw = self.http.post("key", &body).await?;
        Ok(crate::utils::parsing::parse_auth_keys(raw))
    }
}
