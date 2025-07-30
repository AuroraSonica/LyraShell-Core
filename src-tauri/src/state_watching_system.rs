
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::command;
use crate::debug_log;

// Global state trackers
static REACTION_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);
static COOP_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

// --- Tauri Commands to update state from frontend ---

#[command]
pub fn set_reaction_mode_status(active: bool) {
    REACTION_MODE_ACTIVE.store(active, Ordering::Relaxed);
    debug_log!("📺 Reaction Mode status updated to: {}", active);
}

#[command]
pub fn set_coop_mode_status(active: bool) {
    COOP_MODE_ACTIVE.store(active, Ordering::Relaxed);
    debug_log!("🎮 Co-op Mode status updated to: {}", active);
}

// --- Functions for other systems to check the state ---

pub fn is_reaction_mode_active() -> bool {
    REACTION_MODE_ACTIVE.load(Ordering::Relaxed)
}

pub fn is_coop_mode_active() -> bool {
    COOP_MODE_ACTIVE.load(Ordering::Relaxed)
}