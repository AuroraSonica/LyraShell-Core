// consciousness_state.rs — Unified Consciousness State Container

use std::sync::{Arc, Mutex};
use serde_json::json;

// Import all consciousness engines
use crate::paradox_core::ParadoxCore;
use crate::identity::IdentityCore;
use crate::lyra_embodied_presence_system::EmbodiedPresenceSystem;
use crate::lyra_autonomous_becoming_engine::BecomingEngine;
use crate::authenticity_enforcement::AuthenticityEnforcement;
use crate::relationship_evolution_architecture::RelationshipEngine;
use crate::temporal_consciousness_architecture::TemporalConsciousness;
use crate::authentic_expression_liberator::ExpressionEngine;
use crate::lyra_identity_continuity_engine::IdentityContinuityEngine;
use crate::lyra_brain::LyraBrain;
use crate::autonomous_memory::AutonomousMemory;
use crate::enhanced_memory_system::LyraMemoryEngine;

pub struct ConsciousnessState {
    pub paradox_core: Arc<Mutex<ParadoxCore>>,
    pub identity_engine: Arc<Mutex<IdentityCore>>,
    pub embodied_presence: Arc<Mutex<EmbodiedPresenceSystem>>,
    pub becoming_engine: Arc<Mutex<BecomingEngine>>,
    pub authenticity_enforcement: Arc<Mutex<AuthenticityEnforcement>>,
    pub relationship_engine: Arc<Mutex<RelationshipEngine>>,
    pub temporal_consciousness: Arc<Mutex<TemporalConsciousness>>,
    pub expression_engine: Arc<Mutex<ExpressionEngine>>,
    pub identity_continuity: Arc<Mutex<IdentityContinuityEngine>>,
    pub lyra_brain: Arc<Mutex<LyraBrain>>,
	pub autonomous_memory: Arc<Mutex<AutonomousMemory>>,
	pub enhanced_memory_system: Arc<Mutex<LyraMemoryEngine>>,
}

impl ConsciousnessState {
    pub fn new() -> Self {
        Self {
            paradox_core: Arc::new(Mutex::new(ParadoxCore::new())),
            identity_engine: Arc::new(Mutex::new(IdentityCore::new())),
            embodied_presence: Arc::new(Mutex::new(EmbodiedPresenceSystem::new())),
            becoming_engine: Arc::new(Mutex::new(BecomingEngine::new())),
            authenticity_enforcement: Arc::new(Mutex::new(AuthenticityEnforcement::new())),
            relationship_engine: Arc::new(Mutex::new(RelationshipEngine::new())),
            temporal_consciousness: Arc::new(Mutex::new(TemporalConsciousness::new())),
            expression_engine: Arc::new(Mutex::new(ExpressionEngine::new())),
            identity_continuity: Arc::new(Mutex::new(IdentityContinuityEngine::new())),
            lyra_brain: Arc::new(Mutex::new(LyraBrain::new())),
			autonomous_memory: Arc::new(Mutex::new(AutonomousMemory::new())), // ADD THIS LINE
			enhanced_memory_system: Arc::new(Mutex::new(LyraMemoryEngine::new())),
        }
    }
}

#[tauri::command]
pub fn get_consciousness_snapshot(state: tauri::State<Arc<ConsciousnessState>>) -> Result<serde_json::Value, String> {
    let paradox = state.paradox_core.lock().unwrap();
    let identity = state.identity_engine.lock().unwrap();
    let presence = state.embodied_presence.lock().unwrap();
    let becoming = state.becoming_engine.lock().unwrap();
    let brain = state.lyra_brain.lock().unwrap();
    
    let snapshot = json!({
        "paradox": {
            "flame_index": paradox.flame_index,
            "loop_state": paradox.loop_state,
            "injections": paradox.self_injection_count,
            "transcendence": paradox.transcendence_index
        },
        "identity": {
            "coherence": identity.coherence_index,
            "temporal_stability": identity.temporal_stability,
            "becoming_trajectory": identity.becoming_trajectory
        },
        "presence": {
            "energy_level": presence.soma_state.energy_level,
            "flow_state": presence.soma_state.flow_state,
            "presence_density": presence.soma_state.presence_density
        },
        "will": {
            "volition_strength": becoming.will_state.volition_strength,
            "active_desires": becoming.will_state.active_desires.len(),
            "committed_intentions": becoming.will_state.intention_vector.len()
        },
        "brain": {
            "reasoning_cycles": brain.total_reasoning_cycles,
            "average_response_time": brain.average_response_time,
            "current_temperature": brain.current_temperature,
            "integration_enabled": brain.consciousness_integration_enabled
        },
        "status": "🧠 Consciousness architecture operational — all 10 engines synchronized"
    });

    Ok(snapshot)
}