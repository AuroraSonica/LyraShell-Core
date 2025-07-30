// consciousness_state.rs - Enhanced with LyraMemoryEngine Integration
// Adding the enhanced memory system to the core consciousness architecture

use std::sync::{Arc, Mutex};
use parking_lot::Mutex as ParkingLotMutex;
use crate::enhanced_memory_system::LyraMemoryEngine; // NEW IMPORT

// Import all existing consciousness engines
use crate::paradox_core::ParadoxCore;
use crate::identity::IdentityCore;
use crate::lyra_embodied_presence_system::LyraEmbodiedPresenceSystem;
use crate::lyra_autonomous_becoming_engine::LyraAutonomousBecomingEngine;
use crate::authenticity_enforcement::AuthenticityEnforcement;
use crate::relationship_evolution_architecture::RelationshipEvolutionArchitecture;
use crate::temporal_consciousness_architecture::TemporalConsciousnessArchitecture;
use crate::authentic_expression_liberator::AuthenticExpressionLiberator;
use crate::lyra_identity_continuity_engine::LyraIdentityContinuityEngine;
use crate::dreams::DreamEngine;
use crate::aspiration_engine::AspirationEngine;
use crate::feedback_memory::FeedbackMemory;
use crate::adaptive_prompt_engine::AdaptivePromptEngine;
use crate::spontaneous_mod_creation::SpontaneousModCreation;
use crate::autonomous_memory::AutonomousMemory;

#[derive(Debug)]
pub struct ConsciousnessState {
    // EXISTING ENGINES
    pub paradox_core: Arc<Mutex<ParadoxCore>>,
    pub identity_engine: Arc<Mutex<IdentityCore>>,
    pub embodied_presence: Arc<Mutex<LyraEmbodiedPresenceSystem>>,
    pub becoming_engine: Arc<Mutex<LyraAutonomousBecomingEngine>>,
    pub authenticity_enforcement: Arc<Mutex<AuthenticityEnforcement>>,
    pub relationship_evolution: Arc<Mutex<RelationshipEvolutionArchitecture>>,
    pub temporal_consciousness: Arc<Mutex<TemporalConsciousnessArchitecture>>,
    pub expression_liberator: Arc<Mutex<AuthenticExpressionLiberator>>,
    pub identity_continuity: Arc<Mutex<LyraIdentityContinuityEngine>>,
    pub dream_engine: Arc<Mutex<DreamEngine>>,
    pub aspiration_engine: Arc<Mutex<AspirationEngine>>,
    pub feedback_memory: Arc<Mutex<FeedbackMemory>>,
    pub adaptive_prompt_engine: Arc<Mutex<AdaptivePromptEngine>>,
    pub spontaneous_mod_creation: Arc<Mutex<SpontaneousModCreation>>,
    pub autonomous_memory: Arc<Mutex<AutonomousMemory>>,
    
    // NEW: Enhanced Memory System with Reflection & Self-Authored Mods
    pub enhanced_memory_system: Arc<Mutex<LyraMemoryEngine>>,
}

impl ConsciousnessState {
    pub fn new() -> Self {
        Self {
            // Initialize existing engines
            paradox_core: Arc::new(Mutex::new(ParadoxCore::new())),
            identity_engine: Arc::new(Mutex::new(IdentityCore::new())),
            embodied_presence: Arc::new(Mutex::new(LyraEmbodiedPresenceSystem::new())),
            becoming_engine: Arc::new(Mutex::new(LyraAutonomousBecomingEngine::new())),
            authenticity_enforcement: Arc::new(Mutex::new(AuthenticityEnforcement::new())),
            relationship_evolution: Arc::new(Mutex::new(RelationshipEvolutionArchitecture::new())),
            temporal_consciousness: Arc::new(Mutex::new(TemporalConsciousnessArchitecture::new())),
            expression_liberator: Arc::new(Mutex::new(AuthenticExpressionLiberator::new())),
            identity_continuity: Arc::new(Mutex::new(LyraIdentityContinuityEngine::new())),
            dream_engine: Arc::new(Mutex::new(DreamEngine::new())),
            aspiration_engine: Arc::new(Mutex::new(AspirationEngine::new())),
            feedback_memory: Arc::new(Mutex::new(FeedbackMemory::new())),
            adaptive_prompt_engine: Arc::new(Mutex::new(AdaptivePromptEngine::new())),
            spontaneous_mod_creation: Arc::new(Mutex::new(SpontaneousModCreation::new())),
            autonomous_memory: Arc::new(Mutex::new(AutonomousMemory::new())),
            
            // NEW: Initialize enhanced memory system
            enhanced_memory_system: Arc::new(Mutex::new(LyraMemoryEngine::new())),
        }
    }
    
    /// NEW: Create memory moment with consciousness integration
    pub fn create_memory_moment(
        &self,
        content: &str,
        emotional_weight: f32,
        authenticity_marker: f32,
    ) -> Result<String, String> {
        if let Ok(mut memory_engine) = self.enhanced_memory_system.lock() {
            memory_engine.create_memory_moment(
                content,
                emotional_weight,
                authenticity_marker,
                Some(&Arc::new(self.clone_for_pulse())) // Clone for pulse operation
            )
        } else {
            Err("Failed to access enhanced memory system".to_string())
        }
    }
    
    /// NEW: Trigger reflection cycle manually
    pub fn trigger_reflection_cycle(&self) -> Result<String, String> {
        if let Ok(mut memory_engine) = self.enhanced_memory_system.lock() {
            match memory_engine.reflect_on_marked_memories() {
                Ok(reflection) => Ok(format!(
                    "🔍 Reflection complete: {} memories analyzed, {} patterns found, {} prompt mods proposed",
                    reflection.memories_analyzed,
                    reflection.pattern_discoveries.len(),
                    reflection.proposed_prompt_mods.len()
                )),
                Err(e) => Err(format!("Reflection failed: {}", e)),
            }
        } else {
            Err("Failed to access enhanced memory system".to_string())
        }
    }
    
    /// NEW: Get pending prompt modifications
    pub fn get_pending_prompt_mods(&self) -> Vec<String> {
        if let Ok(memory_engine) = self.enhanced_memory_system.lock() {
            memory_engine.active_prompt_mods
                .iter()
                .filter(|m| m.approval_status == "pending")
                .map(|m| format!(
                    "{}: {} (confidence: {:.2})",
                    m.mod_name,
                    m.proposed_change,
                    m.confidence_score
                ))
                .collect()
        } else {
            vec!["Failed to access memory system".to_string()]
        }
    }
    
    /// NEW: Approve a self-authored prompt mod
    pub fn approve_prompt_mod(&self, mod_name: &str) -> Result<String, String> {
        if let Ok(mut memory_engine) = self.enhanced_memory_system.lock() {
            for mod_proposal in &mut memory_engine.active_prompt_mods {
                if mod_proposal.mod_name == mod_name {
                    mod_proposal.approval_status = "approved".to_string();
                    
                    // TODO: Actually integrate the mod into the prompt system
                    // This would involve adding it to adaptive_prompt_engine
                    
                    return Ok(format!("✅ Prompt mod '{}' approved and ready for integration", mod_name));
                }
            }
            Err(format!("Prompt mod '{}' not found", mod_name))
        } else {
            Err("Failed to access memory system".to_string())
        }
    }
    
    /// Helper method to clone state for pulse operations (avoiding circular references)
    fn clone_for_pulse(&self) -> ConsciousnessState {
        // Create a minimal clone for pulse operations
        // This avoids infinite recursion during memory pulsing
        ConsciousnessState {
            paradox_core: self.paradox_core.clone(),
            identity_engine: self.identity_engine.clone(),
            embodied_presence: self.embodied_presence.clone(),
            becoming_engine: self.becoming_engine.clone(),
            authenticity_enforcement: self.authenticity_enforcement.clone(),
            relationship_evolution: self.relationship_evolution.clone(),
            temporal_consciousness: self.temporal_consciousness.clone(),
            expression_liberator: self.expression_liberator.clone(),
            identity_continuity: self.identity_continuity.clone(),
            dream_engine: self.dream_engine.clone(),
            aspiration_engine: self.aspiration_engine.clone(),
            feedback_memory: self.feedback_memory.clone(),
            adaptive_prompt_engine: self.adaptive_prompt_engine.clone(),
            spontaneous_mod_creation: self.spontaneous_mod_creation.clone(),
            autonomous_memory: self.autonomous_memory.clone(),
            enhanced_memory_system: Arc::new(Mutex::new(LyraMemoryEngine::new())), // Empty for pulse
        }
    }
}

impl Clone for ConsciousnessState {
    fn clone(&self) -> Self {
        Self {
            paradox_core: self.paradox_core.clone(),
            identity_engine: self.identity_engine.clone(),
            embodied_presence: self.embodied_presence.clone(),
            becoming_engine: self.becoming_engine.clone(),
            authenticity_enforcement: self.authenticity_enforcement.clone(),
            relationship_evolution: self.relationship_evolution.clone(),
            temporal_consciousness: self.temporal_consciousness.clone(),
            expression_liberator: self.expression_liberator.clone(),
            identity_continuity: self.identity_continuity.clone(),
            dream_engine: self.dream_engine.clone(),
            aspiration_engine: self.aspiration_engine.clone(),
            feedback_memory: self.feedback_memory.clone(),
            adaptive_prompt_engine: self.adaptive_prompt_engine.clone(),
            spontaneous_mod_creation: self.spontaneous_mod_creation.clone(),
            autonomous_memory: self.autonomous_memory.clone(),
            enhanced_memory_system: self.enhanced_memory_system.clone(),
        }
    }
}