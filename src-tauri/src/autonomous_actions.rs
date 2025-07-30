use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use crate::gaming_system;
use crate::coop_mode;
use crate::debug_log;
use tauri::Emitter;
use tauri::Manager;
use crate::lyra_brain::ConsciousnessState;
use tauri::State;

lazy_static! {
    static ref AUTONOMOUS_STATE: Arc<Mutex<AutonomousState>> = Arc::new(Mutex::new(AutonomousState::default()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousState {
    pub enabled: bool,
    pub interval_secs: u64,
    pub random_variance: bool,
    pub last_action_time: Option<u64>,
    pub last_user_interaction: Option<u64>,
}

impl Default for AutonomousState {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 30,
            random_variance: false,
            last_action_time: None,
            last_user_interaction: Some(current_timestamp()),
        }
    }
}

pub async fn start_autonomous_loop(app_handle: tauri::AppHandle) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        let should_act = {
            let state = AUTONOMOUS_STATE.lock().await;
            
            if !state.enabled {
                false
            } else {
                let now = current_timestamp();
                let last_interaction = state.last_user_interaction.unwrap_or(0);
                let last_action = state.last_action_time.unwrap_or(0);
                
                // Calculate next action time with variance
                let mut interval = state.interval_secs;
                if state.random_variance {
                    use rand::Rng;
                    let variance = (interval as f64 * 0.2) as u64;
                    let mut rng = rand::thread_rng();
                    interval = interval - variance + rng.gen_range(0..variance*2);
                }
                
                // Check if enough time has passed
                now - last_interaction > interval && 
                now - last_action > interval
            }
        };
        
        if should_act {
            trigger_autonomous_action(&app_handle).await;
        }
    }
}

async fn trigger_autonomous_action(app_handle: &tauri::AppHandle) {
    println!("🤖 Triggering autonomous action!");
    
    // Update last action time
    {
        let mut state = AUTONOMOUS_STATE.lock().await;
        state.last_action_time = Some(current_timestamp());
    }
    
    // Get current game context
    let game_context = match gaming_system::capture_game_context_on_demand().await {
        Ok(context) => context,
        Err(e) => {
            println!("Failed to capture game context: {}", e);
            None
        }
    };
    
    // Get current inventory
    let inventory = crate::inventory_tracker::get_inventory_summary();
    
    // Get full action context with material requirements
    let action_context = coop_mode::get_minecraft_action_context(&inventory);
    
    // Build autonomous prompt
    let prompt = format!(
        "🤖 [Autonomous Action] You are fully in control of your Minecraft character, and you are taking initiative in Minecraft.\n\n\
        Current scene: {}\n\n\
        {}\n\
        Based on your current inventory and the scene, what would you like to do next? \
        Be creative but practical! Consider what materials you have and what you need.\n\
        If you're low on resources, gather more. If you have materials, build something cool!",
        game_context.as_ref()
            .map(|g| g.ai_analysis.scene_description.as_str())
            .unwrap_or("Unknown scene"),
        action_context
    );
    
		// Emit event to frontend to handle the autonomous action
	if let Some(window) = app_handle.get_webview_window("overlay") {
		let _ = window.emit("trigger_autonomous_action", serde_json::json!({
			"prompt": prompt,
			"gameContext": game_context
		}));
		println!("🤖 Autonomous action sent to overlay");
	} else {
		println!("❌ Overlay window not found");
	}
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Tauri commands
#[tauri::command]
pub async fn enable_autonomous_actions(interval: u64, random_variance: bool) -> Result<String, String> {
    let mut state = AUTONOMOUS_STATE.lock().await;
    state.enabled = true;
    state.interval_secs = interval;
    state.random_variance = random_variance;
    Ok("Autonomous actions enabled".to_string())
}

#[tauri::command]
pub async fn disable_autonomous_actions() -> Result<String, String> {
    let mut state = AUTONOMOUS_STATE.lock().await;
    state.enabled = false;
    Ok("Autonomous actions disabled".to_string())
}

#[tauri::command]
pub async fn get_autonomous_status() -> Result<serde_json::Value, String> {
    let state = AUTONOMOUS_STATE.lock().await;
    let now = current_timestamp();
    let next_action_in = if state.enabled {
        let last = state.last_action_time.unwrap_or(state.last_user_interaction.unwrap_or(now));
        Some((state.interval_secs as i64) - ((now - last) as i64))
    } else {
        None
    };
    
    Ok(serde_json::json!({
        "enabled": state.enabled,
        "interval": state.interval_secs,
        "nextActionIn": next_action_in.filter(|&x| x > 0)
    }))
}

// Call this when user interacts
pub async fn reset_interaction_timer() {
    let mut state = AUTONOMOUS_STATE.lock().await;
    state.last_user_interaction = Some(current_timestamp());
	// No return statement - returns ()
}