// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::sync::OnceLock;
use tauri::AppHandle;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(handle: AppHandle) {
    match APP_HANDLE.set(handle) {
        Ok(_) => println!("[{}] 📡 App handle set successfully", 
                         chrono::Utc::now().with_timezone(&chrono_tz::Europe::London).format("%H:%M:%S")),
        Err(_) => println!("[{}] ⚠️ App handle already set",
                          chrono::Utc::now().with_timezone(&chrono_tz::Europe::London).format("%H:%M:%S")),
    }
}

pub fn get_app_handle() -> Result<&'static AppHandle, String> {
    APP_HANDLE.get().ok_or_else(|| "App handle not initialized".to_string())
}