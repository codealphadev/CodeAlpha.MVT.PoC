use std::sync::Arc;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde_json::json;
use tracing_serde::AsSerde;

use tracing::{metadata::LevelFilter, Level, Subscriber};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, Layer};

use super::gcp::logging::GcpLogging;

struct GcpLayer {
    gcp_logging: Arc<Mutex<GcpLogging>>,
}

impl GcpLayer {
    #[allow(unreachable_code, unused_variables)]
    fn should_send_to_remote(event: &tracing::Event<'_>) -> bool {
        #[cfg(debug_assertions)] // only include this code on debug builds
        {
            return false;
        }

        for field in event.fields() {
            if field.name() == "no_remote" {
                return false;
            }
        }

        true
    }
}

impl<S: Subscriber> Layer<S> for GcpLayer {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        if let Some(module) = metadata.module_path() {
            if module.contains("CodeAlpha") {
                // Filter all non-CodeAlpha events
                return true;
            }
        }
        false
    }

    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let timestamp = Utc::now();
        let name = event.metadata().name().to_string();
        let level = event.metadata().level().to_owned();

        if Self::should_send_to_remote(event) {
            let mut gcp_logging = self.gcp_logging.lock();
            gcp_logging.add_entry(
                json!(event.as_serde()),
                Metadata {
                    timestamp,
                    name,
                    level,
                },
            );
        }
    }
}

pub struct Metadata {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub level: Level,
}

pub struct TracingSubscriber {}

impl TracingSubscriber {
    pub fn new() {
        let gcp_logging = GcpLogging::new();
        gcp_logging.start_remote();

        let gcp_layer = GcpLayer {
            gcp_logging: Arc::new(Mutex::new(gcp_logging)),
        };
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .finish()
            .with(gcp_layer);

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use crate::utils::tracing::TracingSubscriber;

    #[test]
    fn log_info() {
        TracingSubscriber::new();
        let foo = 22;
        info!(foo, no_remote = true, "Here is the message");
    }
}
