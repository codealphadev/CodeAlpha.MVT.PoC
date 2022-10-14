use crate::{app_handle, platform};

pub fn ax_permissions_check() {
    // Continuously check if the accessibility APIs are enabled, show popup if not
    let ax_apis_enabled_at_start = platform::macos::is_application_trusted();
    tauri::async_runtime::spawn(async move {
        let mut popup_was_shown = false;
        loop {
            let api_enabled;
            if popup_was_shown {
                api_enabled = platform::macos::is_application_trusted();
            } else {
                api_enabled = platform::macos::is_application_trusted_with_prompt();
                popup_was_shown = true;
            }

            if api_enabled {
                // In case AX apis were not enabled at program start, restart the app to
                // ensure the AX observers are properly registered.
                if !ax_apis_enabled_at_start {
                    app_handle().restart();
                }
            }

            if !api_enabled && ax_apis_enabled_at_start {
                // in this case the permissions were withdrawn at runtime, restart the app
                std::process::exit(0);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });
}
