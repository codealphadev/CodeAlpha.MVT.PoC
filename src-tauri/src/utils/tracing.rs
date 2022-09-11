use std::{panic, sync::Arc};

use backtrace::Backtrace;
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde_json::json;
use tracing_serde::AsSerde;

use tracing::{error, metadata::LevelFilter, Level, Subscriber, info};
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

    fn should_send_sync(event: &tracing::Event<'_>) -> bool {
        for field in event.fields() {
            if field.name() == "publish_sync" {
                return true;
            }
        }
        false
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
            let message = json!(event.as_serde());
            let metadata = Metadata {
                timestamp,
                name,
                level,
            };
            let mut gcp_logging = self.gcp_logging.lock();
            if Self::should_send_sync(event) {
                gcp_logging.publish_entry_synchronously(message, metadata);
            } else {
                gcp_logging.add_entry(message, metadata);
            }
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

        let default_panic = std::panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let backtrace = Backtrace::new();
            error!(
                publish_sync = true,
                ?panic_info,
                ?backtrace,
                "Application Panic"
            );
            default_panic(panic_info);
        }));

        info!("Starting Tracing");
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
