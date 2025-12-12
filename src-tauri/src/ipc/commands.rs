use crate::adapters::config_store::ConfigStore;
use crate::core::config::file_v1::FileConfigV1;
use crate::core::domain::Event;
use crate::engine::runtime::Runtime;
use crate::ipc::dto::AppStateDto;
use crate::ipc::dto::IntentDto;
use tauri::State;

#[tauri::command]
pub fn get_config(store: State<ConfigStore>) -> FileConfigV1 {
    store.load()
}

#[tauri::command]
pub fn set_config(store: State<ConfigStore>, config: FileConfigV1) -> Result<(), String> {
    store.save(config)
}

// Legacy wrapper to keep frontend working during migration
#[tauri::command]
pub fn fetch_config(store: State<ConfigStore>) -> String {
    // Return JSON string of the NEW config structure
    // Frontend will receive different keys now (camelCase), but won't crash.
    // We will fix the frontend type definition in a later step.
    let config = store.load();
    serde_json::to_string(&config).unwrap_or_default()
}

#[tauri::command]
pub fn intent(runtime: State<Runtime>, payload: IntentDto) {
    let event: Event = payload.into();
    runtime.emit(event);
}

#[tauri::command]
pub fn get_state(runtime: State<Runtime>) -> AppStateDto {
    let state = runtime.get_state();
    (&state).into()
}

#[tauri::command]
fn get_time(runtime: tauri::State<Runtime>) -> String {
    let state = runtime.get_state();
    // Reconstruct remaining or elapsed?
    // Old UI expects "Time Display".
    // In new engine: Running -> calculated.
    // Ideally frontend subscribes to events.
    // If we MUST support polling:
    let dto: crate::ipc::dto::AppStateDto = (&state).into();
    let seconds = dto.remaining_ms / 1000;
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds % 3600) / 60,
        seconds % 60
    )
}
