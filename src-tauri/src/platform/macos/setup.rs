use std::{collections::HashMap, sync::Arc};

use accessibility::AXObserver;
use lazy_static::lazy_static;
use parking_lot::Mutex;

use super::{
    app::register_observer_app, observer_device_events::subscribe_mouse_events,
    xcode::register_observer_xcode,
};

static LOOP_TIME_IN_MS: u64 = 500;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ObserverType {
    App,
    XCode,
}

lazy_static! {
    static ref REGISTERED_AX_OBSERVERS: Arc<Mutex<HashMap<ObserverType, (i32, AXObserver)>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// This is the entry point for the Observer registrations. We register observers
// for the following notifications:
// - notifications from accessibility api for Xcode
// - notifications from accessibility api for our app
// - subscribe to mouse(/keyboard) events
// It is called from the main thread at program startup
pub fn setup_observers() {
    startup_observer_registration();

    start_monitoring_editor_pids();
}

fn startup_observer_registration() {
    std::thread::spawn(|| {
        subscribe_mouse_events();
    });

    std::thread::spawn(|| {
        // As our application is still starting up, we need to wait before registering its AX observer.
        // We found that 2 seconds is enough.
        std::thread::sleep(std::time::Duration::from_millis(2000));
        _ = register_observer_app();
    });
}

fn start_monitoring_editor_pids() {
    // This task periodically checks if editor apps are running and registers the observers
    tauri::async_runtime::spawn(async move {
        loop {
            _ = register_observer_xcode();

            tokio::time::sleep(std::time::Duration::from_millis(LOOP_TIME_IN_MS)).await;
        }
    });
}

pub fn store_registered_ax_observer(
    pid: i32,
    observer_type: ObserverType,
    observer: &AXObserver,
) -> Option<(i32, AXObserver)> {
    REGISTERED_AX_OBSERVERS
        .lock()
        .insert(observer_type, (pid, observer.to_owned()))
}

pub fn get_registered_ax_observer(observer_type: ObserverType) -> Option<(i32, AXObserver)> {
    REGISTERED_AX_OBSERVERS
        .lock()
        .get(&observer_type)
        .map(|(pid, observer)| (*pid, observer.to_owned()))
}

pub fn remove_registered_ax_observer(observer_type: ObserverType) -> Option<(i32, AXObserver)> {
    REGISTERED_AX_OBSERVERS.lock().remove(&observer_type)
}
