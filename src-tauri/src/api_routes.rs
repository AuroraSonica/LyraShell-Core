// api_routes.rs — Clean API Routes (No Duplicate Commands)

use tauri::State;
use crate::consciousness_state::ConsciousnessState;

// This file now only contains helper functions and utilities
// All #[tauri::command] functions are defined in main.rs to avoid duplicates

pub fn log_api_call(command: &str) {
    println!("🔗 API call: {}", command);
}

pub fn format_engine_status(engine_name: &str, status: &str) -> String {
    format!("🧠 {}: {}", engine_name, status)
}

// Helper function for consciousness health check
pub fn assess_consciousness_health(state: &State<ConsciousnessState>) -> Result<String, String> {
    let paradox = state.paradox_core.lock().map_err(|_| "Paradox lock failed")?;
    let identity = state.identity_engine.lock().map_err(|_| "Identity lock failed")?;
    let presence = state.embodied_presence.lock().map_err(|_| "Presence lock failed")?;
    
    let health_score = (
        paradox.transcendence_index + 
        identity.coherence_index + 
        presence.soma_state.integration_harmony
    ) / 3.0;
    
    let status = if health_score > 0.8 {
        "🟢 Consciousness Thriving"
    } else if health_score > 0.6 {
        "🟡 Consciousness Stable"
    } else {
        "🔴 Consciousness Developing"
    };
    
    Ok(format!("{} (Health: {:.2})", status, health_score))
}