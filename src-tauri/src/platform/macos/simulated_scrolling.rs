use rdev::{simulate, EventType};

use crate::utils::geometry::LogicalSize;

use super::{is_focused_uielement_xcode_editor_textarea, XcodeError};

static SCROLL_INTERVAL_MS: f64 = 10.;

static DECELERATED_SCROLL_RAMP_1: f64 = 0.85;
static DECELERATED_SCROLL_RAMP_2: f64 = 0.10;
static DECELERATED_SCROLL_RAMP_3: f64 = 0.00;

pub fn scroll_with_constant_speed(
    scroll_delta: LogicalSize,
    duration: std::time::Duration,
) -> Result<(), XcodeError> {
    if is_focused_uielement_xcode_editor_textarea()? {
        // Calculate the number of scroll intervals within the duration
        let scroll_interval_count = i32::max(
            1,
            f64::round(duration.as_millis() as f64 / SCROLL_INTERVAL_MS) as i32,
        );

        tauri::async_runtime::spawn(async move {
            for _ in 0..scroll_interval_count {
                tauri::async_runtime::spawn(async move {
                    let event_type = EventType::Wheel {
                        delta_x: (scroll_delta.width / scroll_interval_count as f64) as i64,
                        delta_y: (scroll_delta.height / scroll_interval_count as f64) as i64,
                    };

                    match simulate(&event_type) {
                        Ok(()) => {}
                        Err(_) => {
                            println!("We could not send {:?}", event_type);
                        }
                    }
                });

                tokio::time::sleep(std::time::Duration::from_millis(SCROLL_INTERVAL_MS as u64))
                    .await;
            }
        });
    }

    Ok(())
}

pub fn scroll_with_deceleration(
    scroll_delta: LogicalSize,
    duration: std::time::Duration,
) -> Result<(), XcodeError> {
    let foo = std::time::Duration::from_millis((duration.as_millis() as f64 / 3.) as u64);

    tauri::async_runtime::spawn(async move {
        // Section 1: Fastest speed
        if scroll_with_constant_speed(
            LogicalSize {
                width: scroll_delta.width * DECELERATED_SCROLL_RAMP_1,
                height: scroll_delta.height * DECELERATED_SCROLL_RAMP_1,
            },
            foo,
        )
        .is_err()
        {
            return;
        }

        tokio::time::sleep(foo).await;

        // Section 2: Medium speed
        if scroll_with_constant_speed(
            LogicalSize {
                width: scroll_delta.width * DECELERATED_SCROLL_RAMP_2,
                height: scroll_delta.height * DECELERATED_SCROLL_RAMP_2,
            },
            foo,
        )
        .is_err()
        {
            return;
        }

        tokio::time::sleep(foo).await;

        // Section 3: Slowest speed
        if scroll_with_constant_speed(
            LogicalSize {
                width: scroll_delta.width * DECELERATED_SCROLL_RAMP_3,
                height: scroll_delta.height * DECELERATED_SCROLL_RAMP_3,
            },
            foo,
        )
        .is_err()
        {
            return;
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_assertion() {
        // Needs to add up to 100%
        assert_eq!(
            1.,
            DECELERATED_SCROLL_RAMP_1 + DECELERATED_SCROLL_RAMP_2 + DECELERATED_SCROLL_RAMP_3
        );
    }
}
