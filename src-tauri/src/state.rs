use std::sync::Arc;
use tokio::sync::Mutex;

/// AppState holds runtime state shared across Tauri commands.
/// In Phase 1 this is minimal -- the DB is accessed via tauri_plugin_sql State.
#[derive(Default)]
pub struct AppState {
    pub elevated_helper_running: Arc<Mutex<bool>>,
}
