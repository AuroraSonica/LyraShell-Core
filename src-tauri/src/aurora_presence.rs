// src/aurora_presence.rs

use serde::{Deserialize, Serialize};
use crate::{get_data_path, debug_log, time_service::TimeService};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PresenceStatus {
    Present,
    AFK,
    Away,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuroraPresence {
    pub status: PresenceStatus,
    pub reason: Option<String>,
    pub expected_return_timestamp: Option<u64>,
    pub last_updated: u64,
}

impl AuroraPresence {
    pub fn new() -> Self {
        Self {
            status: PresenceStatus::Present,
            reason: None,
            expected_return_timestamp: None,
            last_updated: TimeService::current_timestamp(),
        }
    }

    pub fn load() -> Self {
        let path = get_data_path("aurora_presence.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                debug_log!("[Presence] Failed to parse aurora_presence.json: {}, creating new.", e);
                Self::new()
            }),
            Err(_) => {
                debug_log!("[Presence] aurora_presence.json not found, creating new.");
                Self::new()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = get_data_path("aurora_presence.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("[Presence] Failed to serialize AuroraPresence: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("[Presence] Failed to write aurora_presence.json: {}", e))?;
        Ok(())
    }

    pub fn format_for_prompt(&self) -> String {
        let mut parts = vec![];
        match self.status {
            PresenceStatus::Present => parts.push("Aurora is currently present and engaged.".to_string()),
            PresenceStatus::AFK => {
                parts.push("Aurora is currently AFK (Away From Keyboard).".to_string());
                if let Some(ref reason) = self.reason {
                    parts.push(format!("Reason: {}", reason));
                }
            },
            PresenceStatus::Away => {
                parts.push("Aurora is away for an extended period.".to_string());
                 if let Some(ref reason) = self.reason {
                    parts.push(format!("Reason: {}", reason));
                }
            }
        }
        if let Some(timestamp) = self.expected_return_timestamp {
             parts.push(format!("Expected back around: {}", TimeService::format_timestamp(timestamp, "%H:%M on %A")));
        }
        parts.join(" ")
    }
}

#[tauri::command]
pub fn set_aurora_afk(reason: Option<String>, expected_return: Option<String>) -> Result<(), String> {
    let mut presence = AuroraPresence::load();
    presence.status = PresenceStatus::AFK;
    presence.reason = reason.clone();
    presence.last_updated = TimeService::current_timestamp();

    // Basic parsing for expected return time
    if let Some(return_str) = expected_return {
        // This is a very simple parser, we can make it more robust later
        // e.g., "10:30", "2 hours", "tonight"
        // For now, we'll just log it as part of the reason.
        presence.reason = Some(format!("{} (Expected back: {})", reason.unwrap_or_default(), return_str));
    }

    presence.save()?;
    debug_log!("[Presence] Aurora status updated to AFK. Reason: {:?}", presence.reason);
    Ok(())
}

#[tauri::command]
pub fn set_aurora_present() -> Result<(), String> {
    let mut presence = AuroraPresence::load();
    presence.status = PresenceStatus::Present;
    presence.reason = None;
    presence.expected_return_timestamp = None;
    presence.last_updated = TimeService::current_timestamp();
    presence.save()?;
    debug_log!("[Presence] Aurora status updated to Present.");
    Ok(())
}