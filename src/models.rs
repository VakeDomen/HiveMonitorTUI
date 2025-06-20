use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Versions returned by /worker/versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerVersion {
    pub hive: String,
    pub ollama: String,
}
pub type WorkerVersions = HashMap<String, WorkerVersion>;

// Status returned by /worker/status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Verified,
    Waiting,
    #[serde(other)]
    Unknown,
}
pub type WorkerStatuses = HashMap<String, NodeStatus>;

// Connections returned by /worker/connections
pub type WorkerConnections = HashMap<String, usize>;

// Pings returned by /worker/pings
pub type WorkerPings = HashMap<String, DateTime<Utc>>;

// Supported tags per worker
pub type WorkerTags = HashMap<String, Vec<String>>;

// Queue map: model name or node name to count
pub type QueueMap = HashMap<String, usize>;

// Authentication key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthKey {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(with = "chrono::serde::ts_seconds")]  
    pub created_at: DateTime<Utc>,
}
pub type AuthKeys = Vec<AuthKey>;

// Inference request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
}

// Inference response (non-streamed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub result: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// Chat API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

// Embedding API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
