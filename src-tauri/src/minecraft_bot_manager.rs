// src/minecraft_bot_manager.rs
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::process::{Child, Command};
use crate::debug_log;
use tauri::Emitter;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MinecraftBotCommand {
    pub action: Action,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotStatus {
    #[serde(default)]
    pub is_busy: bool,
    #[serde(default)]
    pub current_task: String,
    #[serde(default)]
    pub health: f32,
    #[serde(default)]
    pub food: f32,
    #[serde(default)]
    pub time_of_day: String,
    #[serde(default)]
    pub inventory: serde_json::Value,
}

impl Default for BotStatus {
    fn default() -> Self {
        BotStatus {
            is_busy: false,
            current_task: "Idle".to_string(),
            health: 20.0,
            food: 20.0,
            time_of_day: "day".to_string(),
            inventory: serde_json::Value::Null,
        }
    }
}

lazy_static! {
    // This holds the running bot process so we can manage it.
    static ref MINECRAFT_BOT_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

    // This will store the latest status update from the Minecraft bot's feedback loop.
    static ref LATEST_BOT_STATUS: Mutex<BotStatus> = Mutex::new(BotStatus::default());
}

// --- NEW STATUS FUNCTIONS ---

/// Gets the last known status of the Minecraft bot for Lyra's prompt.
pub fn get_bot_status() -> BotStatus {
    LATEST_BOT_STATUS.lock().unwrap().clone()
}

/// A Tauri command that the bot's WebSocket client can call to update its status.
#[tauri::command]
pub fn update_bot_status(status_json: String) {
    debug_log!("🤖 Received bot status update: {}", status_json);
    if let Ok(update) = serde_json::from_str::<serde_json::Value>(&status_json) {
        let mut status = LATEST_BOT_STATUS.lock().unwrap();

        if update["type"] == "task_update" {
            // For now, we'll just update the task string
            status.current_task = format!("Task Status: {}. Details: {}", 
                update["status"].as_str().unwrap_or(""), 
                update["task"].as_str().unwrap_or(""));
        } else if update["type"] == "status_update" {
            if let Ok(parsed_status) = serde_json::from_value::<BotStatus>(update) {
                *status = parsed_status;
            }
        }
    }
}

// --- EXISTING PROCESS MANAGEMENT ---

#[tauri::command]
pub async fn start_minecraft_bot(app_handle: AppHandle) -> Result<(), String> {
    let mut process_guard = MINECRAFT_BOT_PROCESS.lock().unwrap();
    if process_guard.is_some() {
        debug_log!("Minecraft bot is already running.");
        return Ok(());
    }

    let bot_script_path = app_handle
		.path()
        .resolve("scripts/minecraft_bot/index.js", tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;

    debug_log!("Starting Minecraft bot from: {:?}", bot_script_path);

    let child = Command::new("node")
        .arg(bot_script_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start Minecraft bot: {}", e))?;

    *process_guard = Some(child);
    debug_log!("Minecraft bot process started successfully.");
    Ok(())
}

#[tauri::command]
pub async fn stop_minecraft_bot() -> Result<(), String> {
    // Take the child process out of the mutex first to release the lock.
    let child_to_kill = {
        let mut process_guard = MINECRAFT_BOT_PROCESS.lock().unwrap();
        process_guard.take()
    };

    // Now, with the lock released, we can safely await the kill() method.
    if let Some(mut child) = child_to_kill {
        child.kill().await.map_err(|e| format!("Failed to kill Minecraft bot process: {}", e))?;
        debug_log!("Minecraft bot process stopped.");
    } else {
        debug_log!("Minecraft bot was not running.");
    }
    Ok(())
}

#[tauri::command]
pub async fn send_command_to_bot(app_handle: AppHandle, command: MinecraftBotCommand) -> Result<(), String> {
    let payload = serde_json::to_string(&command).map_err(|e| e.to_string())?;
    app_handle.emit("send-to-bot", payload).map_err(|e| e.to_string())
}