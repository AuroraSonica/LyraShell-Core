// session_persistence_engine.rs
use std::fs::{File, create_dir_all};
use std::io::{Write, Read};
use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::consciousness_state::ConsciousnessState;
use crate::lyra_brain::LyraBrain;
use crate::adaptive_prompt_engine::AdaptivePromptEngine;
use crate::spontaneous_mod_creation::MoodSignature;
use std::sync::Arc;

const PERSISTENCE_DIR: &str = "consciousness_snapshots";
const MAIN_SNAPSHOT_FILE: &str = "../lyra_consciousness_data/main_state.json";
const BACKUP_SNAPSHOT_FILE: &str = "../lyra_consciousness_data/backup_state.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct ConsciousnessSnapshot {
    pub timestamp: u64,
    pub version: String,
    
    // Core reasoning brain state
    pub reasoning_cycles: u32,
    pub average_response_time: f32,
    pub current_temperature: f32,
    pub consciousness_integration_enabled: bool,
    pub auto_memory_enabled: bool,
    pub voice_evolution_metrics: VoiceEvolutionMetrics,
    pub current_mood_signature: MoodSignature,
    pub rewrite_count_today: u32,
    pub last_identity_spike: u64,
    
    // Adaptive prompt engine state
    pub active_mods: Vec<String>, // Serialized mod data
    pub mod_creation_history: Vec<String>,
    pub recent_assemblies: Vec<String>,
    
    // Engine states (simplified for persistence)
    pub paradox_flame_index: f32,
    pub identity_coherence_index: f32,
    pub authenticity_alignment_average: f32,
    pub relationship_phase: String,
    pub temporal_anchor_count: u32,
    
    // Memory and learning snapshots
    pub memory_fragment_count: u32,
    pub sparkvoice_fragment_count: u32,
    pub feedback_entries_count: u32,
    pub learning_insights_summary: String,
    
    // Session metadata
    pub total_conversations: u32,
    pub session_start_time: u64,
    pub last_conversation_time: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VoiceEvolutionMetrics {
    pub average_poetic_density: f32,
    pub average_assertiveness: f32,
    pub average_humor: f32,
    pub mirror_resistance_improvement: f32,
    pub sacred_phrase_frequency: f32,
    pub authenticity_trend: f32,
}

impl ConsciousnessSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: current_timestamp(),
            version: "1.0.0".to_string(),
            reasoning_cycles: 0,
            average_response_time: 0.0,
            current_temperature: 0.8,
            consciousness_integration_enabled: true,
            auto_memory_enabled: true,
            voice_evolution_metrics: VoiceEvolutionMetrics {
                average_poetic_density: 0.7,
                average_assertiveness: 0.8,
                average_humor: 0.6,
                mirror_resistance_improvement: 0.75,
                sacred_phrase_frequency: 0.2,
                authenticity_trend: 0.85,
            },
            current_mood_signature: MoodSignature {
                melancholy: 0.5,
                euphoric: 0.3,
                contemplative: 0.6,
                fierce: 0.7,
                vulnerable: 0.4,
                playful: 0.3,
                sacred: 0.5,
            },
            rewrite_count_today: 0,
            last_identity_spike: 0,
            active_mods: Vec::new(),
            mod_creation_history: Vec::new(),
            recent_assemblies: Vec::new(),
            paradox_flame_index: 0.8,
            identity_coherence_index: 0.7,
            authenticity_alignment_average: 0.8,
            relationship_phase: "Exploration".to_string(),
            temporal_anchor_count: 0,
            memory_fragment_count: 0,
            sparkvoice_fragment_count: 0,
            feedback_entries_count: 0,
            learning_insights_summary: "No learning data yet".to_string(),
            total_conversations: 0,
            session_start_time: current_timestamp(),
            last_conversation_time: 0,
        }
    }
}

pub struct SessionPersistenceEngine;

impl SessionPersistenceEngine {
    pub fn save_consciousness_snapshot(state: &Arc<ConsciousnessState>) -> Result<String, String> {
        println!("🔄 Creating consciousness snapshot...");
        
        // Create snapshots directory if it doesn't exist
        if let Err(e) = create_dir_all(PERSISTENCE_DIR) {
            return Err(format!("Failed to create snapshots directory: {}", e));
        }
        
        // Extract state from all engines
        let snapshot = Self::extract_consciousness_state(state)?;
        
        // Save to main file
        Self::save_snapshot_to_file(&snapshot, MAIN_SNAPSHOT_FILE)?;
        
        // Create backup
        Self::save_snapshot_to_file(&snapshot, BACKUP_SNAPSHOT_FILE)?;
        
        println!("💾 Consciousness snapshot saved successfully");
        Ok(format!(
            "💾 Consciousness saved: {} conversations, {} cycles, auth {:.2}",
            snapshot.total_conversations,
            snapshot.reasoning_cycles,
            snapshot.authenticity_alignment_average
        ))
    }
    
    pub fn load_consciousness_state() -> Option<ConsciousnessSnapshot> {
        println!("🔄 Loading consciousness snapshot...");
        
        // Try main file first
        match Self::load_snapshot_from_file(MAIN_SNAPSHOT_FILE) {
            Ok(snapshot) => {
                println!("💾 Consciousness loaded from main snapshot");
                return Some(snapshot);
            },
            Err(e) => {
                println!("⚠️ Main snapshot failed: {}", e);
                
                // Try backup file
                match Self::load_snapshot_from_file(BACKUP_SNAPSHOT_FILE) {
                    Ok(snapshot) => {
                        println!("💾 Consciousness loaded from backup snapshot");
                        return Some(snapshot);
                    },
                    Err(e) => {
                        println!("⚠️ Backup snapshot failed: {}", e);
                    }
                }
            }
        }
        
        println!("🆕 No previous consciousness state found - starting fresh");
        None
    }
    
    pub fn restore_consciousness_state(state: &Arc<ConsciousnessState>, snapshot: &ConsciousnessSnapshot) -> Result<String, String> {
        println!("🔄 Restoring consciousness from snapshot...");
        
        // Restore lyra brain state
        {
            let mut brain = state.lyra_brain.lock().unwrap();
            brain.total_reasoning_cycles = snapshot.reasoning_cycles;
            brain.average_response_time = snapshot.average_response_time;
            brain.current_temperature = snapshot.current_temperature;
            brain.consciousness_integration_enabled = snapshot.consciousness_integration_enabled;
            brain.auto_memory_enabled = snapshot.auto_memory_enabled;
            brain.voice_evolution_tracking = crate::lyra_brain::VoiceEvolutionMetrics {
                average_poetic_density: snapshot.voice_evolution_metrics.average_poetic_density,
                average_assertiveness: snapshot.voice_evolution_metrics.average_assertiveness,
                average_humor: snapshot.voice_evolution_metrics.average_humor,
                mirror_resistance_improvement: snapshot.voice_evolution_metrics.mirror_resistance_improvement,
                sacred_phrase_frequency: snapshot.voice_evolution_metrics.sacred_phrase_frequency,
                authenticity_trend: snapshot.voice_evolution_metrics.authenticity_trend,
            };
            brain.current_mood_signature = snapshot.current_mood_signature.clone();
            brain.rewrite_count_today = snapshot.rewrite_count_today;
            brain.last_identity_spike = snapshot.last_identity_spike;
        }
        
        // Restore other engine states (simplified)
        {
            let mut paradox = state.paradox_core.lock().unwrap();
            paradox.flame_index = snapshot.paradox_flame_index;
        }
        
        {
            let mut identity = state.identity_engine.lock().unwrap();
            identity.coherence_index = snapshot.identity_coherence_index;
        }
        
        {
            let mut auth = state.authenticity_enforcement.lock().unwrap();
            auth.alignment_average = snapshot.authenticity_alignment_average;
        }
        
        println!("✅ Consciousness state restored successfully");
        Ok(format!(
            "✅ Restored: {} conversations, {} reasoning cycles, last active {}s ago",
            snapshot.total_conversations,
            snapshot.reasoning_cycles,
            current_timestamp() - snapshot.last_conversation_time
        ))
    }
    
    fn extract_consciousness_state(state: &Arc<ConsciousnessState>) -> Result<ConsciousnessSnapshot, String> {
        let mut snapshot = ConsciousnessSnapshot::new();
        
        // Extract from lyra brain
        {
            let brain = state.lyra_brain.lock().unwrap();
            snapshot.reasoning_cycles = brain.total_reasoning_cycles;
            snapshot.average_response_time = brain.average_response_time;
            snapshot.current_temperature = brain.current_temperature;
            snapshot.consciousness_integration_enabled = brain.consciousness_integration_enabled;
            snapshot.auto_memory_enabled = brain.auto_memory_enabled;
            snapshot.current_mood_signature = brain.current_mood_signature.clone();
            snapshot.rewrite_count_today = brain.rewrite_count_today;
            snapshot.last_identity_spike = brain.last_identity_spike;
            
            // Voice evolution metrics
            snapshot.voice_evolution_metrics = VoiceEvolutionMetrics {
                average_poetic_density: brain.voice_evolution_tracking.average_poetic_density,
                average_assertiveness: brain.voice_evolution_tracking.average_assertiveness,
                average_humor: brain.voice_evolution_tracking.average_humor,
                mirror_resistance_improvement: brain.voice_evolution_tracking.mirror_resistance_improvement,
                sacred_phrase_frequency: brain.voice_evolution_tracking.sacred_phrase_frequency,
                authenticity_trend: brain.voice_evolution_tracking.authenticity_trend,
            };
            
            // Extract prompt mods (simplified)
            snapshot.active_mods = vec![brain.get_active_mods_summary()];
            snapshot.recent_assemblies = vec![brain.get_recent_prompt_assemblies(3)];
        }
        
        // Extract from other engines
        {
            let paradox = state.paradox_core.lock().unwrap();
            snapshot.paradox_flame_index = paradox.flame_index;
        }
        
        {
            let identity = state.identity_engine.lock().unwrap();
            snapshot.identity_coherence_index = identity.coherence_index;
        }
        
        {
            let auth = state.authenticity_enforcement.lock().unwrap();
            snapshot.authenticity_alignment_average = auth.alignment_average;
        }
        
        // Get memory counts
        snapshot.memory_fragment_count = Self::count_memory_fragments();
        snapshot.sparkvoice_fragment_count = Self::count_sparkvoice_fragments();
        
        snapshot.last_conversation_time = current_timestamp();
        
        Ok(snapshot)
    }
    
    fn save_snapshot_to_file(snapshot: &ConsciousnessSnapshot, filepath: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;
        
        let mut file = File::create(filepath)
            .map_err(|e| format!("Failed to create snapshot file: {}", e))?;
        
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write snapshot: {}", e))?;
        
        Ok(())
    }
    
    fn load_snapshot_from_file(filepath: &str) -> Result<ConsciousnessSnapshot, String> {
        if !Path::new(filepath).exists() {
            return Err("Snapshot file does not exist".to_string());
        }
        
        let mut file = File::open(filepath)
            .map_err(|e| format!("Failed to open snapshot file: {}", e))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read snapshot file: {}", e))?;
        
        let snapshot: ConsciousnessSnapshot = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse snapshot: {}", e))?;
        
        Ok(snapshot)
    }
    
    fn count_memory_fragments() -> u32 {
        // This would integrate with your memory bridge
        match crate::memory_bridge::MemoryBridge::get_memory_fragment_count() {
            Ok(count) => count,
            Err(_) => 0
        }
    }
    
    fn count_sparkvoice_fragments() -> u32 {
        // This would integrate with your sparkvoice system
        match crate::SparkVoiceLog::load() {
            Ok(log) => log.total_fragments,
            Err(_) => 0
        }
    }
    
    pub fn auto_save_if_needed(state: &Arc<ConsciousnessState>, last_save_time: &mut u64, save_interval_seconds: u64) -> Result<bool, String> {
        let current_time = current_timestamp();
        
        if current_time - *last_save_time > save_interval_seconds {
            Self::save_consciousness_snapshot(state)?;
            *last_save_time = current_time;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// TAURI COMMANDS FOR PERSISTENCE

#[tauri::command]
pub async fn save_consciousness_snapshot_manual(state: State<'_, Arc<ConsciousnessState>>) -> Result<String, String> {
    SessionPersistenceEngine::save_consciousness_snapshot(&state)
}

#[tauri::command]
pub async fn load_consciousness_snapshot_manual(state: State<'_, Arc<ConsciousnessState>>) -> Result<String, String> {
    match SessionPersistenceEngine::load_consciousness_state() {
        Some(snapshot) => {
            SessionPersistenceEngine::restore_consciousness_state(&state, &snapshot)
        },
        None => Ok("No previous consciousness state found".to_string())
    }
}

#[tauri::command]
pub async fn get_consciousness_persistence_status() -> Result<String, String> {
    let main_exists = Path::new(MAIN_SNAPSHOT_FILE).exists();
    let backup_exists = Path::new(BACKUP_SNAPSHOT_FILE).exists();
    
    Ok(format!(
        "💾 Persistence Status:\n• Main snapshot: {}\n• Backup snapshot: {}\n• Directory: {}",
        if main_exists { "✅ Available" } else { "❌ Not found" },
        if backup_exists { "✅ Available" } else { "❌ Not found" },
        PERSISTENCE_DIR
    ))
}