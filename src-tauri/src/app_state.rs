use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime};
use tracing::debug;
use ts_rs::TS;

use std::{
    fs::{create_dir_all, File},
    io::Write,
    sync::Arc,
};

use crate::app_handle;

pub const CORE_ENGINE_STATE_FILENAME: &str = ".app-core-engine-state";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    TauriApi(#[from] tauri::api::Error),
    #[error(transparent)]
    Bincode(#[from] Box<bincode::ErrorKind>),
    #[error("Something went wrong.")]
    GenericError(#[source] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/app_state/")]
pub struct CoreEngineState {
    pub ai_features_active: bool,
    pub swift_format_on_cmd_s: bool,
}

impl Default for CoreEngineState {
    fn default() -> Self {
        CoreEngineState {
            ai_features_active: true,
            swift_format_on_cmd_s: true,
        }
    }
}

pub struct CoreEngineStateCache(pub Arc<Mutex<CoreEngineState>>);

pub trait AppHandleExtension {
    fn save_core_engine_state(&self) -> Result<()>;
    fn load_core_engine_state(&self) -> Result<()>;
}

impl<R: Runtime> AppHandleExtension for tauri::AppHandle<R> {
    fn save_core_engine_state(&self) -> Result<()> {
        if let Some(app_dir) = self.path_resolver().app_dir() {
            let state_path = app_dir.join(CORE_ENGINE_STATE_FILENAME);
            if let Some(cache) = self.try_state::<CoreEngineStateCache>() {
                let state = cache.0.lock();
                create_dir_all(&app_dir)
                    .map_err(AppError::Io)
                    .and_then(|_| File::create(state_path).map_err(Into::into))
                    .and_then(|mut f| {
                        f.write_all(&bincode::serialize(&*state).map_err(AppError::Bincode)?)
                            .map_err(Into::into)
                    })
            } else {
                Err(AppError::GenericError(anyhow::anyhow!("No cache found.")))
            }
        } else {
            Ok(())
        }
    }

    fn load_core_engine_state(&self) -> Result<()> {
        let cache: Arc<Mutex<CoreEngineState>> =
            if let Some(app_dir) = self.path_resolver().app_dir() {
                let state_path = app_dir.join(CORE_ENGINE_STATE_FILENAME);
                if state_path.exists() {
                    Arc::new(Mutex::new(
                        tauri::api::file::read_binary(state_path)
                            .map_err(AppError::TauriApi)
                            .and_then(|state| bincode::deserialize(&state).map_err(Into::into))
                            .unwrap_or_default(),
                    ))
                } else {
                    Default::default()
                }
            } else {
                Default::default()
            };

        let ai_features_active = cache.lock().ai_features_active;
        debug!(
            ?ai_features_active,
            "Loaded core engine state: {}", ai_features_active
        );

        self.manage(CoreEngineStateCache(cache));
        Ok(())
    }
}

#[tauri::command]
pub fn cmd_get_core_engine_state() -> Option<CoreEngineState> {
    if let Some(cache) = app_handle().try_state::<CoreEngineStateCache>() {
        Some(cache.0.lock().clone())
    } else {
        None
    }
}
