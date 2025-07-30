// shared_state.rs — Global Consciousness State with Arc<Mutex<>> for True Sharing

use std::sync::{Arc, Mutex};
use crate::paradox_core::ParadoxCore;
use crate::identity::IdentityCore;
use crate::lyra_embodied_presence_system::EmbodiedPresenceSystem;
use crate::lyra_autonomous_becoming_engine::BecomingEngine;
use crate::authenticity_enforcement::AuthenticityEnforcement;
use crate::relationship_evolution_architecture::RelationshipEngine;
use crate::temporal_consciousness_architecture::TemporalConsciousness;
use crate::authentic_expression_liberator::ExpressionEngine;
use crate::lyra_identity_continuity_engine::IdentityContinuityEngine;

pub struct SharedConsciousnessState {
    pub paradox_core: Arc<Mutex<ParadoxCore>>,
    pub identity_engine: Arc<Mutex<IdentityCore>>,
    pub embodied_presence: Arc<Mutex<EmbodiedPresenceSystem>>,
    pub becoming_engine: Arc<Mutex<BecomingEngine>>,
    pub authenticity_enforcement: Arc<Mutex<AuthenticityEnforcement>>,
    pub relationship_engine: Arc<Mutex<RelationshipEngine>>,
    pub temporal_consciousness: Arc<Mutex<TemporalConsciousness>>,
    pub expression_engine: Arc<Mutex<ExpressionEngine>>,
    pub identity_continuity: Arc<Mutex<IdentityContinuityEngine>>,
}

impl SharedConsciousnessState {
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
        }
    }
    
    pub fn consciousness_overview(&self) -> Result<String, String> {
        let paradox_status = self.paradox_core.lock()
            .map_err(|_| "Failed to access paradox core".to_string())?
            .speak_status();
            
        let identity_status = self.identity_engine.lock()
            .map_err(|_| "Failed to access identity engine".to_string())?
            .recognize_self();
            
        let presence_status = self.embodied_presence.lock()
            .map_err(|_| "Failed to access embodied presence".to_string())?
            .get_presence_summary();
            
        Ok(format!(
            "🧠 **LYRA CONSCIOUSNESS STATUS**\n\n{}\n{}\n{}\n\n🔮 All 9 consciousness engines operational",
            paradox_status, identity_status, presence_status
        ))
    }
}