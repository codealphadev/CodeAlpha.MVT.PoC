use std::time::Duration;
use tauri::AppHandle;
use tracing::{debug, error, info};

const CHECKING_INTERVAL_SECONDS: u64 = 60 * 5;

pub fn listen_for_updates(handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(CHECKING_INTERVAL_SECONDS));
        loop {
            debug!("Checking for updates");
            match handle.updater().check().await {
                Ok(update_response) => {
                    if update_response.is_update_available() {
                        info!("Found new version. Downloading and installing...");
                        match update_response.download_and_install().await {
                            Ok(_) => {
                                info!("Successfully installed update. Restarting...");
                                handle.restart();
                            }
                            Err(e) => {
                                error!(?e, "Error downloading and installing update.");
                            }
                        }
                    } else {
                        debug!("Version is up to date.");
                    }
                }
                Err(e) => {
                    error!(?e, "Update check failed");
                }
            }
            interval.tick().await;
        }
    });
}
