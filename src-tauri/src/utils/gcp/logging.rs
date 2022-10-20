use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::async_runtime::block_on;
use tracing::{debug, warn, Level};

use crate::utils::{gcp::auth, tracing::Metadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    DEFAULT,
    DEBUG,
    INFO,
    WARNING,
    ERROR,
}

impl Severity {
    fn from_tracing_level(level: &Level) -> Severity {
        match *level {
            Level::DEBUG => Severity::DEBUG,
            Level::INFO => Severity::INFO,
            Level::WARN => Severity::WARNING,
            Level::ERROR => Severity::ERROR,
            _ => Severity::DEFAULT,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MonitoredResource {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    jsonPayload: Value,
    labels: HashMap<String, String>,
    log_name: String,
    resource: MonitoredResource,
    severity: Severity,
    timestamp: String,
}

impl LogEntry {
    pub fn new(log_name: String, jsonPayload: Value, metadata: Metadata) -> Self {
        let mut labels = HashMap::new();
        if let Ok(machine_id) = machine_uid::get() {
            labels.insert("machine_id".to_string(), machine_id);
        }

        let monitored_resource = MonitoredResource {
            type_: "global".to_string(),
        };

        Self {
            jsonPayload: jsonPayload,
            labels,
            log_name: log_name,
            resource: monitored_resource,
            severity: Severity::from_tracing_level(&metadata.level),
            timestamp: metadata.timestamp.to_rfc3339(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoggingEntriesWriteRequest {
    entries: Vec<LogEntry>,
}

pub struct GcpLogging {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    log_name: String,
}

impl GcpLogging {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            log_name: "projects/client-backend-x/logs/client".to_string(),
        }
    }

    pub fn start_remote(&self) {
        tauri::async_runtime::spawn({
            let entries = self.entries.clone();
            async move {
                let mut auth = auth::GcpAuth::new();
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                    let token = auth.token_str().await;

                    let current_entries;
                    {
                        let mut entries = entries.lock();
                        current_entries = entries.clone();
                        entries.clear();
                    }

                    if let Some(access_token) = token {
                        publish_to_gcp(current_entries, access_token).await.ok();
                    }
                }
            }
        });
    }

    pub fn add_entry(&mut self, message: Value, metadata: Metadata) {
        (*self.entries.lock()).push(LogEntry::new(self.log_name.clone(), message, metadata));
    }

    pub fn publish_entry_synchronously(&mut self, message: Value, metadata: Metadata) {
        let entries = vec![LogEntry::new(self.log_name.clone(), message, metadata)];
        // get token
        let mut auth = auth::GcpAuth::new();
        let token = block_on(auth.token_str());

        // publish to gcp
        if let Some(access_token) = token {
            block_on(publish_to_gcp(entries, access_token)).ok();
        }
    }

    #[allow(dead_code)] // Used in tests
    pub fn entries(&self) -> Vec<LogEntry> {
        (*self.entries.lock()).clone()
    }
}

async fn publish_to_gcp(
    entries: Vec<LogEntry>,
    access_token: String,
) -> Result<(), reqwest::Error> {
    if entries.is_empty() {
        return Ok(());
    }

    let req_body = LoggingEntriesWriteRequest {
        entries: entries.clone(),
    };
    let response_result = reqwest::Client::new()
        .post("https://logging.googleapis.com/v2/entries:write")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .json(&req_body)
        .send()
        .await;

    match response_result {
        Ok(response) => {
            if response.status().is_success() {
                debug!(
                    no_remote = true,
                    "successfully published {} logs to GCP",
                    entries.len()
                );
            } else {
                warn!(
                    no_remote = true,
                    response = ?response,
                    "failed to publish {} logs to GCP",
                    entries.len()
                );
            }
            Ok(())
        }
        Err(e) => {
            warn!(no_remote = true, "failed to publish logs to GCP: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {

    use chrono::Utc;
    use tauri::async_runtime::block_on;
    use tracing::Level;

    use crate::utils::tracing::Metadata;

    #[test]
    fn write_log() {
        let mut gcp_logging = super::GcpLogging::new();
        let first_message =
            serde_json::from_str(r#"{"message": "TEST message 1", "log_test": "true"}"#).unwrap();
        gcp_logging.add_entry(
            first_message,
            Metadata {
                timestamp: Utc::now(),
                name: "event src/utils/gcp/logging.rs:153".to_string(),
                level: Level::DEBUG,
            },
        );

        gcp_logging.start_remote();
        let second_message =
            serde_json::from_str(r#"{"message": "TEST message 2", "log_test": "true"}"#).unwrap();

        gcp_logging.add_entry(
            second_message,
            Metadata {
                timestamp: Utc::now(),
                name: "event src/utils/gcp/logging.rs:153".to_string(),
                level: Level::DEBUG,
            },
        );
        block_on(async {
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        });
    }
}
