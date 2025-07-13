// src/clients/infer_client.rs

use futures::StreamExt;
use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION}};
use tokio::sync::Mutex;
use crate::{app::App, errors::ClientError, utils::http::HttpClient};
use serde_json::Value;
use std::{sync::Arc, time::Duration};

pub struct HiveInferClient {
    client: HttpClient,
    auth_header: HeaderValue,
}

impl HiveInferClient {
    pub fn new(base_url: impl Into<String>, client_token: &str) -> Result<Self, ClientError> {
        let mut auth_header = HeaderValue::from_str(&format!("Bearer {}", client_token))?;
        auth_header.set_sensitive(true);
        
        let client = HttpClient::new(base_url.into(), client_token)?;
        Ok(HiveInferClient {
            client,
            auth_header,
        })
    }

    fn make_headers(&self, node: Option<&str>) -> Result<HeaderMap, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, self.auth_header.clone());
        if let Some(node_name) = node {
            headers.insert("Node", HeaderValue::from_str(node_name)?);
        }
        Ok(headers)
    }

    /// Inference: synchronous (non‐streaming) generate
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        node: Option<&str>,
        stream: bool,
    ) -> Result<Value, ClientError> {
        let url = format!("{}/api/generate", self.client.base_url.trim_end_matches('/'));
        let headers = self.make_headers(node)?;
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": stream
        });
        let resp = self.client.post(&url, &body, Some(headers)).await?;
        Ok(resp)
    }

    /// List all models (tags) available on the worker
    pub async fn list_models(&self, node: Option<&str>) -> Result<Vec<String>, ClientError> {
        let url = format!("{}/api/models", self.client.base_url.trim_end_matches('/'));
        let headers = self.make_headers(node)?;
        let resp = self.client.get(&url, Some(headers)).await?;
        Ok(resp)
    }

    /// Pull a model onto the worker
    ///
    /// POST /api/pull with body `{ "name": "<model>" }`
    /// This method will now stream JSON lines and update the App state.
    pub async fn pull_model(&self, model: &str, node: Option<&str>, app_arc: Arc<Mutex<App>>) -> Result<(), ClientError> {
        let url = format!("{}/api/pull", self.client.base_url.trim_end_matches('/'));
        let headers = self.make_headers(node)?;
        let body = serde_json::json!({ "name": model });
        let resp = self.client
            .post_raw(&url, &body, Some(headers))
            .await?;

        // Get the byte stream from the response
        let mut byte_stream = resp.bytes_stream(); // This exists!
        let mut buffer = Vec::new(); // Buffer for incomplete lines
        let mut overall_success = true;

        while let Some(chunk_result) = byte_stream.next().await {
            let chunk = chunk_result?; // Propagate reqwest::Error

            buffer.extend_from_slice(&chunk); // Add chunk to buffer

            // Process lines from the buffer
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                let trimmed_line = String::from_utf8(line_bytes) // Convert to String
                    .map_err(ClientError::Decode)?
                    .trim()
                    .to_string(); // Trim and own the string

                if trimmed_line.is_empty() {
                    continue; // Skip empty lines
                }

                // Attempt to parse each line as JSON
                match serde_json::from_str::<Value>(&trimmed_line) {
                    Ok(json_value) => {
                        let message_val = json_value.get("status").or(json_value.get("message"));
                        let message = message_val
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| json_value.to_string());

                        // Assuming "error" field indicates failure if present and not null/false
                        let is_line_success = json_value.get("error").is_none_or(|err| err.is_null() || !err.as_bool().unwrap_or(false));

                        if !is_line_success {
                            overall_success = false;
                            app_arc.lock().await.add_banner(format!("Pull Error: {}", message));
                        }
                        app_arc.lock().await.add_action_output_line(message, is_line_success);
                    },
                    Err(e) => {
                        overall_success = false;
                        let error_msg = format!("Non-JSON line: {} (Parse Error: {})", trimmed_line, e);
                        app_arc.lock().await.add_action_output_line(error_msg.clone(), false);
                        app_arc.lock().await.add_banner(error_msg);
                    }
                }
            }
        }

        // Process any remaining data in the buffer after the stream ends (e.g., last line without newline)
        if !buffer.is_empty() {
            let remaining_line = String::from_utf8(buffer)
                .map_err(ClientError::Decode)?
                .trim()
                .to_string();
            if !remaining_line.is_empty() {
                match serde_json::from_str::<Value>(&remaining_line) {
                    Ok(json_value) => {
                        let message_val = json_value.get("status").or(json_value.get("message"));
                        let message = message_val
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| json_value.to_string());
                        let is_line_success = json_value.get("error").is_none_or(|err| err.is_null() || !err.as_bool().unwrap_or(false));
                        if !is_line_success { overall_success = false; }
                        app_arc.lock().await.add_action_output_line(message, is_line_success);
                    },
                    Err(e) => {
                        overall_success = false;
                        let error_msg = format!("Non-JSON final line: {} (Parse Error: {})", remaining_line, e);
                        app_arc.lock().await.add_action_output_line(error_msg.clone(), false);
                        app_arc.lock().await.add_banner(error_msg);
                    }
                }
            }
        }

        // Final status message
        let final_message = if overall_success {
            "Model pull completed successfully.".to_string()
        } else {
            "Model pull completed with errors.".to_string()
        };
        app_arc.lock().await.add_action_output_line(final_message, overall_success);
        Ok(())
    }

    pub async fn delete_model(&self, model: &str, node: Option<&str>, app_arc: Arc<Mutex<App>>) -> Result<(), ClientError> {
        let url = format!("{}/api/delete", self.client.base_url.trim_end_matches('/'));
        let headers = self.make_headers(node)?;
        let body = serde_json::json!({ "name": model });

        let resp = self.client
            .delete_raw(&url, &body, Some(headers))
            .await?;

        let mut byte_stream = resp.bytes_stream();
        let mut buffer = Vec::new();
        let mut overall_success = true;

        while let Some(chunk_result) = byte_stream.next().await {
            let chunk = chunk_result?;

            buffer.extend_from_slice(&chunk);

            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                let trimmed_line = String::from_utf8(line_bytes)
                    .map_err(ClientError::Decode)?
                    .trim()
                    .to_string();

                if trimmed_line.is_empty() {
                    continue;
                }

                match serde_json::from_str::<Value>(&trimmed_line) {
                    Ok(json_value) => {
                        let message_val = json_value.get("status").or(json_value.get("message"));
                        let message = message_val
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| json_value.to_string());

                        let is_line_success = json_value.get("error").is_none_or(|err| err.is_null() || !err.as_bool().unwrap_or(false));

                        if !is_line_success {
                            overall_success = false;
                            app_arc.lock().await.add_banner(format!("Delete Error: {}", message));
                        }
                        app_arc.lock().await.add_action_output_line(message, is_line_success);
                    },
                    Err(e) => {
                        overall_success = false;
                        let error_msg = format!("Non-JSON line: {} (Parse Error: {})", trimmed_line, e);
                        app_arc.lock().await.add_action_output_line(error_msg.clone(), false);
                        app_arc.lock().await.add_banner(error_msg);
                    }
                }
            }
        }

        if !buffer.is_empty() {
            let remaining_line = String::from_utf8(buffer)
                .map_err(ClientError::Decode)?
                .trim()
                .to_string();
            if !remaining_line.is_empty() {
                match serde_json::from_str::<Value>(&remaining_line) {
                    Ok(json_value) => {
                        let message_val = json_value.get("status").or(json_value.get("message"));
                        let message = message_val
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| json_value.to_string());
                        let is_line_success = json_value.get("error").is_none_or(|err| err.is_null() || !err.as_bool().unwrap_or(false));
                        if !is_line_success { overall_success = false; }
                        app_arc.lock().await.add_action_output_line(message, is_line_success);
                    },
                    Err(e) => {
                        overall_success = false;
                        let error_msg = format!("Non-JSON final line: {} (Parse Error: {})", remaining_line, e);
                        app_arc.lock().await.add_action_output_line(error_msg.clone(), false);
                        app_arc.lock().await.add_banner(error_msg);
                    }
                }
            }
        }

        let final_message = if overall_success {
            "Model delete completed successfully.".to_string()
        } else {
            "Model delete completed with errors.".to_string()
        };
        app_arc.lock().await.add_action_output_line(final_message, overall_success);
        Ok(())
    }
    
    

    /// (Optional) Streamed generate: returns a reqwest `Response` that you can
    /// `.bytes_stream()` and parse chunked JSON. E.g. for `/api/generate?stream=true`
    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: &str,
        node: Option<&str>,
    ) -> Result<reqwest::Response, ClientError> {
        let url = format!(
            "{}/api/generate?stream=true",
            self.client.base_url.trim_end_matches('/')
        );
        let headers = self.make_headers(node)?;
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
        });
        let resp = self.client
            .post_raw(&url, &body, Some(headers))
            .await?;
        Ok(resp)
    }
}
