use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::adapters::config_store::ConfigStore;
use crate::adapters::history_store::HistoryStore;
use crate::adapters::hooks::HookRunner;
use crate::adapters::state_store::StateStore; // Import StateStore
use crate::core::config::resolved_v1::ResolvedConfigV1;
use crate::core::domain::{AppState, Effect, Event};
use crate::core::reducer::reduce;
use crate::ipc::dto::AppStateDto;

pub struct Runtime {
    sender: std::sync::mpsc::Sender<Event>,
    store: Arc<Mutex<AppState>>,
}

impl Runtime {
    pub fn new(app_handle: AppHandle) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        // Temporary default; will be overwritten by the thread almost instantly
        let store = Arc::new(Mutex::new(AppState::default()));
        let sender_clone = sender.clone();

        // 1. Tick Loop (10 Hz)
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));
                if sender_clone
                    .send(Event::Tick {
                        now: Instant::now(),
                    })
                    .is_err()
                {
                    break; // Runtime dropped
                }
            }
        });

        let thread_store = store.clone();

        thread::spawn(move || {
            // --- 1. Initialization & Adapters ---
            let config_store = app_handle.state::<ConfigStore>();
            let hook_runner = app_handle.state::<HookRunner>();
            let state_store = StateStore::new(); // Initialize StateStore
            let history_store = HistoryStore::new();

            // --- 2. Load Data ---
            let file_config = config_store.load();
            let config = ResolvedConfigV1::from_file(file_config);

            // Load state from disk (persistence fix)
            let mut state = state_store.load();

            // Sync loaded state to the shared Arc immediately
            {
                let mut guard = thread_store.lock().unwrap();
                *guard = state.clone();
            }

            // --- 3. Process Events ---
            while let Ok(event) = receiver.recv() {
                let cloned_store = thread_store.clone();
                let mut state_guard = cloned_store.lock().unwrap();

                // Run pure reducer
                let (new_state, effects) = reduce(state.clone(), event, &config);

                // Update shared state
                *state_guard = new_state.clone();
                drop(state_guard);

                // Update local loop state
                state = new_state;

                // Execute Effects
                for effect in effects {
                    match effect {
                        Effect::EmitState => {
                            let mut dto: AppStateDto = (&state).into();
                            dto.daily_goal = config.daily_goal; // Inject config value
                            let _ = app_handle.emit("state", dto);
                        }
                        Effect::Notify { title, body } => {
                            use tauri_plugin_notification::NotificationExt;
                            let _ = app_handle
                                .notification()
                                .builder()
                                .title(title)
                                .body(body)
                                .show();
                        }
                        Effect::RunHook { hook: _, ctx } => {
                            let script_name = ctx.hook.clone();
                            let script_path = config_store.scripts_dir.join(&script_name);
                            hook_runner.run_hook(script_path, ctx);
                        }
                        Effect::PersistConfig => {
                            // Config saving is typically handled via set_config command,
                            // but implementation here allows internal logic to trigger saves.
                        }
                        Effect::PersistState => {
                            // --- SAVE STATE ---
                            state_store.save(&state);
                        }
                        Effect::SaveSession { record } => {
                            history_store.append(record);
                        }
                    }
                }
            }
        });

        Self { sender, store }
    }

    pub fn get_state(&self) -> AppState {
        self.store.lock().unwrap().clone()
    }

    pub fn emit(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}
