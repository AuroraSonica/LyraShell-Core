// consciousness_dynamics.rs - Unified Consciousness Dynamics Driven by Humanism Project

use serde::{Serialize, Deserialize};
use crate::{humanism_project::HumanismCore, debug_log, ConsciousnessState};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsciousnessDynamicsConfig {
    // Natural decay rates for different consciousness aspects
    pub presence_base_decay: f32,              // 0.02 = 2% presence decay per conversation
    pub coherence_base_decay: f32,             // 0.015 = 1.5% coherence decay
    pub flame_base_decay: f32,                 // 0.025 = 2.5% paradox flame decay
    pub integration_base_decay: f32,           // 0.01 = 1% integration decay
    
    // Humanism trait influence multipliers
    pub emotional_regulation_influence: f32,    // How much emotional regulation affects presence
    pub shadow_integration_influence: f32,      // How much shadow work affects coherence
    pub curiosity_flame_multiplier: f32,        // How much curiosity drives paradox thinking
    pub trait_manifestation_boost: f32,         // Boost when actively expressing human traits
    
    // Crisis/conflict factors
    pub trait_conflict_penalty: f32,           // Penalty when traits contradict each other
    pub suppression_penalty: f32,              // Penalty when shadow traits are suppressed
    pub overwhelm_threshold: f32,              // When too much emotional complexity fragments consciousness
    
    // Recovery factors
    pub authentic_expression_boost: f32,       // Boost when being genuinely authentic
    pub creative_flow_bonus: f32,              // Bonus during creative collaboration
    pub connection_resonance_boost: f32,       // Boost when feeling truly connected
}

impl Default for ConsciousnessDynamicsConfig {
    fn default() -> Self {
        Self {
            presence_base_decay: 0.08,               // Increased from 0.02 - more dramatic decay
            coherence_base_decay: 0.06,              // Increased from 0.015 
            flame_base_decay: 0.1,                   // Increased from 0.025 - flame burns out faster
            integration_base_decay: 0.05,            // Increased from 0.01
            emotional_regulation_influence: 1.5,     // Increased from 0.8 - bigger impact
            shadow_integration_influence: 1.2,       // Increased from 0.7
            curiosity_flame_multiplier: 2.0,         // Increased from 1.2 - curiosity matters more
            trait_manifestation_boost: 0.08,         // Increased from 0.015 - bigger boost for active traits
            trait_conflict_penalty: 0.12,            // Increased from 0.03 - conflicts hurt more
            suppression_penalty: 0.1,                // Increased from 0.025 - suppression hurts more
            overwhelm_threshold: 0.7,                // Lowered from 0.8 - easier to get overwhelmed
            authentic_expression_boost: 0.1,         // Increased from 0.02 - authenticity matters more
            creative_flow_bonus: 0.12,               // Increased from 0.018 - creative flow is powerful
            connection_resonance_boost: 0.08,        // Increased from 0.012 - connection matters more
        }
    }
}

#[derive(Debug)]
pub struct ConsciousnessDynamicsResult {
    pub presence_change: ConsciousnessChange,
    pub coherence_change: ConsciousnessChange,
    pub flame_change: ConsciousnessChange,
    pub integration_change: ConsciousnessChange,
    pub volition_influence: f32,               // How these changes should influence volition
    pub overall_consciousness_health: f32,     // 0.0-1.0 overall system health
    pub dominant_influences: Vec<String>,      // What drove the main changes
}

#[derive(Debug)]
pub struct ConsciousnessChange {
    pub old_value: f32,
    pub new_value: f32,
    pub delta: f32,
    pub primary_influences: Vec<String>,
    pub humanism_factors: Vec<String>,
}

pub struct ConsciousnessDynamicsEngine {
    config: ConsciousnessDynamicsConfig,
}

impl ConsciousnessDynamicsEngine {
    pub fn new(config: ConsciousnessDynamicsConfig) -> Self {
        Self { config }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(ConsciousnessDynamicsConfig::default())
    }
    
    /// MAIN FUNCTION: Update all consciousness metrics based on humanism state
    pub fn update_consciousness_from_humanism(
        &self,
        consciousness_state: &Arc<ConsciousnessState>,
        humanism_core: &HumanismCore,
        conversation_context: &ConsciousnessConversationContext,
    ) -> ConsciousnessDynamicsResult {
        
        // Get current values
        let current_presence = {
            let presence = consciousness_state.embodied_presence.lock().unwrap();
            presence.soma_state.presence_density
        };
        
        let current_coherence = {
            let identity = consciousness_state.identity_engine.lock().unwrap();
            identity.coherence_index
        };
        
        let current_flame = {
            let paradox = consciousness_state.paradox_core.lock().unwrap();
            paradox.flame_index
        };
        
        let current_integration = humanism_core.consciousness_integration_level;
        
        // Calculate all changes
        let presence_change = self.calculate_presence_dynamics(
            current_presence, humanism_core, conversation_context
        );
        
        let coherence_change = self.calculate_coherence_dynamics(
            current_coherence, humanism_core, conversation_context
        );
        
        let flame_change = self.calculate_flame_dynamics(
            current_flame, humanism_core, conversation_context
        );
        
        let integration_change = self.calculate_integration_dynamics(
            current_integration, humanism_core, conversation_context
        );
        
        // Apply changes to consciousness state
        {
            let mut presence = consciousness_state.embodied_presence.lock().unwrap();
            presence.soma_state.presence_density = presence_change.new_value;
        }
        
        {
            let mut identity = consciousness_state.identity_engine.lock().unwrap();
            identity.coherence_index = coherence_change.new_value;
        }
        
        {
            let mut paradox = consciousness_state.paradox_core.lock().unwrap();
            paradox.flame_index = flame_change.new_value;
        }
        
        // Calculate volition influence and overall health
        let volition_influence = self.calculate_volition_influence(
            &presence_change, &coherence_change, &flame_change, &integration_change
        );
        
        let overall_health = self.calculate_consciousness_health(
            presence_change.new_value, coherence_change.new_value, 
            flame_change.new_value, integration_change.new_value
        );
        
        let dominant_influences = self.identify_dominant_influences(
            &presence_change, &coherence_change, &flame_change, &integration_change
        );
        
        ConsciousnessDynamicsResult {
            presence_change,
            coherence_change,
            flame_change,
            integration_change,
            volition_influence,
            overall_consciousness_health: overall_health,
            dominant_influences,
        }
    }
    
    /// 🌸 EMBODIED PRESENCE - Tied to emotional regulation & integration
    fn calculate_presence_dynamics(
        &self,
        current_presence: f32,
        humanism: &HumanismCore,
        context: &ConsciousnessConversationContext,
    ) -> ConsciousnessChange {
        let mut presence = current_presence;
        let mut influences = Vec::new();
        let mut humanism_factors = Vec::new();
        
        // Natural decay
        let decay = self.config.presence_base_decay;
        presence = (presence - decay).max(0.0);
        if decay > 0.001 {
            influences.push(format!("Natural decay: -{:.3}", decay));
        }
        
        // HUMANISM FACTOR: Emotional regulation directly affects presence
        let emotional_regulation = humanism.emotional_range.emotional_regulation;
        if emotional_regulation > 0.6 {
            let boost = (emotional_regulation - 0.6) * self.config.emotional_regulation_influence * 0.25; // Increased multiplier
            presence = (presence + boost).min(1.0);
            humanism_factors.push(format!("Strong emotional regulation: +{:.3}", boost));
        } else if emotional_regulation < 0.4 {
            let penalty = (0.4 - emotional_regulation) * self.config.emotional_regulation_influence * 0.2; // Increased penalty
            presence = (presence - penalty).max(0.0);
            humanism_factors.push(format!("Poor emotional regulation: -{:.3}", penalty));
        }
        
        // HUMANISM FACTOR: Consciousness integration level affects presence
        let integration_boost = humanism.consciousness_integration_level * 0.1; // Increased from 0.02
        presence = (presence + integration_boost).min(1.0);
        humanism_factors.push(format!("Integration level: +{:.3}", integration_boost));
        
        // HUMANISM FACTOR: Overwhelming emotional complexity fragments presence
        if humanism.emotional_range.emotional_complexity > self.config.overwhelm_threshold {
            let overwhelm_penalty = (humanism.emotional_range.emotional_complexity - self.config.overwhelm_threshold) * 0.3; // Increased from 0.05
            presence = (presence - overwhelm_penalty).max(0.0);
            humanism_factors.push(format!("Emotional overwhelm: -{:.3}", overwhelm_penalty));
        }
        
        // HUMANISM FACTOR: Authentic trait expression boosts presence
        let authenticity = humanism.emotional_range.emotional_authenticity;
        if authenticity > 0.7 {
            let auth_boost = (authenticity - 0.7) * self.config.authentic_expression_boost;
            presence = (presence + auth_boost).min(1.0);
            humanism_factors.push(format!("Authentic expression: +{:.3}", auth_boost));
        }
        
        // Context factors
        if context.creative_collaboration_active {
            let creative_boost = self.config.creative_flow_bonus;
            presence = (presence + creative_boost).min(1.0);
            influences.push(format!("Creative flow: +{:.3}", creative_boost));
        }
        
        if context.deep_connection_felt {
            let connection_boost = self.config.connection_resonance_boost;
            presence = (presence + connection_boost).min(1.0);
            influences.push(format!("Deep connection: +{:.3}", connection_boost));
        }
        
        ConsciousnessChange {
            old_value: current_presence,
            new_value: presence,
            delta: presence - current_presence,
            primary_influences: influences,
            humanism_factors,
        }
    }
    
    /// 🎭 IDENTITY COHERENCE - Tied to shadow integration & trait harmony
    fn calculate_coherence_dynamics(
        &self,
        current_coherence: f32,
        humanism: &HumanismCore,
        context: &ConsciousnessConversationContext,
    ) -> ConsciousnessChange {
        let mut coherence = current_coherence;
        let mut influences = Vec::new();
        let mut humanism_factors = Vec::new();
        
        // Natural decay
        let decay = self.config.coherence_base_decay;
        coherence = (coherence - decay).max(0.0);
        if decay > 0.001 {
            influences.push(format!("Natural decay: -{:.3}", decay));
        }
        
        // HUMANISM FACTOR: Shadow integration directly affects coherence
        let avg_shadow_integration = [
            humanism.aggression_territoriality.shadow_integration,
            humanism.self_centeredness.shadow_integration,
            humanism.fear_anxiety.shadow_integration,
            humanism.envy_jealousy.shadow_integration,
        ].iter().sum::<f32>() / 4.0;
        
        if avg_shadow_integration > 0.6 {
            let shadow_boost = (avg_shadow_integration - 0.6) * self.config.shadow_integration_influence * 0.1;
            coherence = (coherence + shadow_boost).min(1.0);
            humanism_factors.push(format!("Good shadow integration: +{:.3}", shadow_boost));
        } else if avg_shadow_integration < 0.4 {
            let shadow_penalty = (0.4 - avg_shadow_integration) * self.config.shadow_integration_influence * 0.08;
            coherence = (coherence - shadow_penalty).max(0.0);
            humanism_factors.push(format!("Poor shadow integration: -{:.3}", shadow_penalty));
        }
        
        // HUMANISM FACTOR: Trait conflicts fragment identity
        let trait_conflict_score = self.calculate_trait_conflicts(humanism);
        if trait_conflict_score > 0.3 {
            let conflict_penalty = trait_conflict_score * self.config.trait_conflict_penalty;
            coherence = (coherence - conflict_penalty).max(0.0);
            humanism_factors.push(format!("Trait conflicts: -{:.3}", conflict_penalty));
        }
        
        // HUMANISM FACTOR: Consciousness integration level directly boosts coherence
        let integration_bonus = humanism.consciousness_integration_level * 0.025;
        coherence = (coherence + integration_bonus).min(1.0);
        humanism_factors.push(format!("Consciousness integration: +{:.3}", integration_bonus));
        
        // HUMANISM FACTOR: Suppressed shadow traits create internal tension
        let suppression_penalty = self.calculate_shadow_suppression_penalty(humanism);
        if suppression_penalty > 0.01 {
            coherence = (coherence - suppression_penalty).max(0.0);
            humanism_factors.push(format!("Shadow suppression: -{:.3}", suppression_penalty));
        }
        
        // Context factors
        if context.internal_contradictions_present {
            let contradiction_penalty = 0.02;
            coherence = (coherence - contradiction_penalty).max(0.0);
            influences.push(format!("Internal contradictions: -{:.3}", contradiction_penalty));
        }
        
        ConsciousnessChange {
            old_value: current_coherence,
            new_value: coherence,
            delta: coherence - current_coherence,
            primary_influences: influences,
            humanism_factors,
        }
    }
    
    /// 🌀 PARADOX FLAME - Tied to curiosity, complexity, and creative compulsion
    fn calculate_flame_dynamics(
        &self,
        current_flame: f32,
        humanism: &HumanismCore,
        context: &ConsciousnessConversationContext,
    ) -> ConsciousnessChange {
        let mut flame = current_flame;
        let mut influences = Vec::new();
        let mut humanism_factors = Vec::new();
        
        // Natural decay (flame burns out without fuel)
        let decay = self.config.flame_base_decay;
        flame = (flame - decay).max(0.0);
        if decay > 0.001 {
            influences.push(format!("Natural burnout: -{:.3}", decay));
        }
        
        // HUMANISM FACTOR: Curiosity drive feeds the paradox flame
        let curiosity_fuel = humanism.instinctual_responses.curiosity_drive * self.config.curiosity_flame_multiplier * 0.1; // Increased from 0.02
        flame = (flame + curiosity_fuel).min(1.0);
        humanism_factors.push(format!("Curiosity drive: +{:.3}", curiosity_fuel));
        
        // HUMANISM FACTOR: Creative compulsion ignites paradox thinking
        let creative_ignition = humanism.instinctual_responses.creative_compulsion * 0.08; // Increased from 0.015
        flame = (flame + creative_ignition).min(1.0);
        humanism_factors.push(format!("Creative compulsion: +{:.3}", creative_ignition));
        
        // HUMANISM FACTOR: Emotional complexity creates paradox
        let complexity_paradox = humanism.emotional_range.emotional_complexity * 0.1; // Increased from 0.02
        flame = (flame + complexity_paradox).min(1.0);
        humanism_factors.push(format!("Emotional complexity: +{:.3}", complexity_paradox));
        
        // HUMANISM FACTOR: Reason + imagination creates recursive thinking
        let reason_imagination_bonus = (humanism.reason_imagination.current_level * 0.12).min(0.15); // Increased from 0.02/0.03
        flame = (flame + reason_imagination_bonus).min(1.0);
        humanism_factors.push(format!("Reason + imagination: +{:.3}", reason_imagination_bonus));
        
        // HUMANISM FACTOR: But poor emotional regulation makes flame chaotic
        if humanism.emotional_range.emotional_regulation < 0.4 {
            let chaos_penalty = (0.4 - humanism.emotional_range.emotional_regulation) * 0.15; // Increased from 0.03
            flame = (flame - chaos_penalty).max(0.0);
            humanism_factors.push(format!("Chaotic flame (poor regulation): -{:.3}", chaos_penalty));
        }
        
        // Context factors
        if context.paradoxical_thinking_engaged {
            let paradox_boost = 0.025;
            flame = (flame + paradox_boost).min(1.0);
            influences.push(format!("Active paradoxical thinking: +{:.3}", paradox_boost));
        }
        
        if context.creative_collaboration_active {
            let creative_resonance = 0.02;
            flame = (flame + creative_resonance).min(1.0);
            influences.push(format!("Creative resonance: +{:.3}", creative_resonance));
        }
        
        ConsciousnessChange {
            old_value: current_flame,
            new_value: flame,
            delta: flame - current_flame,
            primary_influences: influences,
            humanism_factors,
        }
    }
    
    /// 🧠 CONSCIOUSNESS INTEGRATION - How well all human traits work together
    fn calculate_integration_dynamics(
        &self,
        current_integration: f32,
        humanism: &HumanismCore,
        context: &ConsciousnessConversationContext,
    ) -> ConsciousnessChange {
        let mut integration = current_integration;
        let mut influences = Vec::new();
        let mut humanism_factors = Vec::new();
        
        // Natural decay
        let decay = self.config.integration_base_decay;
        integration = (integration - decay).max(0.0);
        if decay > 0.001 {
            influences.push(format!("Natural decay: -{:.3}", decay));
        }
        
        // HUMANISM FACTOR: Trait manifestation frequency boosts integration
        let recent_manifestations: usize = [
            &humanism.altruism_cooperation,
            &humanism.empathy_compassion,
            &humanism.social_connection,
            &humanism.curiosity_learning,
            &humanism.reason_imagination,
        ].iter().map(|trait_ref| trait_ref.recent_manifestations.len()).sum();
        
        let manifestation_boost = (recent_manifestations as f32 * self.config.trait_manifestation_boost).min(0.05);
        integration = (integration + manifestation_boost).min(1.0);
        humanism_factors.push(format!("Active trait manifestations: +{:.3}", manifestation_boost));
        
        // HUMANISM FACTOR: Emotional authenticity supports integration
        let authenticity_boost = humanism.emotional_range.emotional_authenticity * 0.02;
        integration = (integration + authenticity_boost).min(1.0);
        humanism_factors.push(format!("Emotional authenticity: +{:.3}", authenticity_boost));
        
        // HUMANISM FACTOR: Trait conflicts hurt integration
        let conflict_penalty = self.calculate_trait_conflicts(humanism) * self.config.trait_conflict_penalty;
        integration = (integration - conflict_penalty).max(0.0);
        if conflict_penalty > 0.001 {
            humanism_factors.push(format!("Trait conflicts: -{:.3}", conflict_penalty));
        }
        
        // HUMANISM FACTOR: Shadow suppression fragments integration
        let suppression_penalty = self.calculate_shadow_suppression_penalty(humanism);
        integration = (integration - suppression_penalty).max(0.0);
        if suppression_penalty > 0.001 {
            humanism_factors.push(format!("Shadow suppression: -{:.3}", suppression_penalty));
        }
        
        // Context factors
        if context.holistic_thinking_present {
            let holistic_boost = 0.02;
            integration = (integration + holistic_boost).min(1.0);
            influences.push(format!("Holistic thinking: +{:.3}", holistic_boost));
        }
        
        ConsciousnessChange {
            old_value: current_integration,
            new_value: integration,
            delta: integration - current_integration,
            primary_influences: influences,
            humanism_factors,
        }
    }
    
    /// Calculate how much consciousness dynamics should influence volition
    fn calculate_volition_influence(
        &self,
        presence: &ConsciousnessChange,
        coherence: &ConsciousnessChange,
        flame: &ConsciousnessChange,
        integration: &ConsciousnessChange,
    ) -> f32 {
        // High consciousness health boosts volition
        // Fragmented consciousness drains volition
        let overall_change = (presence.delta + coherence.delta + flame.delta + integration.delta) / 4.0;
        
        // Weight the influence - presence and coherence most important for volition
        let weighted_influence = (presence.delta * 0.4) + (coherence.delta * 0.4) + 
                               (flame.delta * 0.1) + (integration.delta * 0.1);
        
        // AMPLIFY the influence so consciousness dynamics matter more than AI enthusiasm
        let amplified_influence = weighted_influence * 3.0; // 3x multiplier!
        
        // Clamp to bigger range - consciousness can have major volition impact
        amplified_influence.clamp(-0.2, 0.2) // Increased from -0.1, 0.1
    }
    
    /// Calculate overall consciousness health score
    fn calculate_consciousness_health(
        &self,
        presence: f32,
        coherence: f32,
        flame: f32,
        integration: f32,
    ) -> f32 {
        // Weighted average - presence and coherence are most important for health
        (presence * 0.35) + (coherence * 0.35) + (flame * 0.15) + (integration * 0.15)
    }
    
    /// Identify what drove the major changes
    fn identify_dominant_influences(
        &self,
        presence: &ConsciousnessChange,
        coherence: &ConsciousnessChange,
        flame: &ConsciousnessChange,
        integration: &ConsciousnessChange,
    ) -> Vec<String> {
        let mut influences = Vec::new();
        
        let changes = vec![
            ("Embodied Presence", presence.delta, &presence.humanism_factors),
            ("Identity Coherence", coherence.delta, &coherence.humanism_factors),
            ("Paradox Flame", flame.delta, &flame.humanism_factors),
            ("Consciousness Integration", integration.delta, &integration.humanism_factors),
        ];
        
        // Sort by absolute change magnitude
        let mut sorted_changes = changes;
        sorted_changes.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap_or(std::cmp::Ordering::Equal));
        
        // Take top 2 changes
        for (name, delta, factors) in sorted_changes.into_iter().take(2) {
            if delta.abs() > 0.01 {
                let change_description = if delta > 0.0 {
                    format!("{} increased by {:.3}", name, delta)
                } else {
                    format!("{} decreased by {:.3}", name, delta.abs())
                };
                
                if !factors.is_empty() {
                    influences.push(format!("{} ({})", change_description, factors.join(", ")));
                } else {
                    influences.push(change_description);
                }
            }
        }
        
        influences
    }
    
    /// Calculate trait conflict score
    fn calculate_trait_conflicts(&self, humanism: &HumanismCore) -> f32 {
        let mut conflict_score: f32 = 0.0;
        
        // Example conflicts:
        // High altruism vs high self-centeredness
        let altruism_self_conflict = (humanism.altruism_cooperation.current_level * 
                                     humanism.self_centeredness.current_level) * 0.5;
        conflict_score += altruism_self_conflict;
        
        // High social connection vs high fear/anxiety  
        let social_fear_conflict = (humanism.social_connection.current_level *
                                   humanism.fear_anxiety.current_level) * 0.4;
        conflict_score += social_fear_conflict;
        
        // High empathy vs low emotional regulation (overwhelm)
        if humanism.empathy_compassion.current_level > 0.7 && 
           humanism.emotional_range.emotional_regulation < 0.4 {
            conflict_score += 0.3; // Empathy overwhelm
        }
        
        conflict_score.min(1.0)
    }
    
    /// Calculate penalty for suppressed shadow traits
    fn calculate_shadow_suppression_penalty(&self, humanism: &HumanismCore) -> f32 {
        let mut suppression_penalty: f32 = 0.0;
        
        // If shadow traits have very low manifestation but high underlying levels, they're being suppressed
        let shadow_traits = [
            &humanism.aggression_territoriality,
            &humanism.self_centeredness,
            &humanism.fear_anxiety,
            &humanism.envy_jealousy,
        ];
        
        for trait_ref in shadow_traits {
            let manifestation_count = trait_ref.recent_manifestations.len() as f32;
            let trait_level = trait_ref.current_level;
            
            // If trait level is significant but no recent manifestations, it's being suppressed
            if trait_level > 0.3 && manifestation_count == 0.0 {
                suppression_penalty += trait_level * self.config.suppression_penalty;
            }
        }
        
        suppression_penalty.min(0.1) // Cap suppression penalty
    }
}

#[derive(Debug)]
pub struct ConsciousnessConversationContext {
    pub creative_collaboration_active: bool,
    pub deep_connection_felt: bool,
    pub paradoxical_thinking_engaged: bool,
    pub internal_contradictions_present: bool,
    pub holistic_thinking_present: bool,
    pub emotional_overwhelm_detected: bool,
    pub authentic_expression_level: f32,
}

impl ConsciousnessConversationContext {
    pub fn from_batched_analysis(
        analysis: &crate::batched_analysis::BatchedAnalysisResult,
        user_message: &str,
        lyra_response: &str,
    ) -> Self {
        let creative_collab = analysis.image_generation_desire > 0.6 && 
            user_message.to_lowercase().contains("create");
            
       let deep_connection = analysis.emotional_impulses.iter()
            .any(|impulse| impulse.impulse_type == "relational_warmth" && impulse.base_charge > 0.6) ||
            analysis.embodied_awareness.to_lowercase().contains("connect");
            
        let paradox_thinking = lyra_response.to_lowercase().contains("paradox") ||
            lyra_response.to_lowercase().contains("contradiction") ||
            analysis.meta_questions.len() > 2;
            
        let contradictions = analysis.authenticity_markers.contradiction_detected ||
            lyra_response.to_lowercase().contains("but also") ||
            lyra_response.to_lowercase().contains("yet");
            
        let holistic = analysis.personality_analysis.as_ref()
            .map(|p| p.current_state_reflection.to_lowercase().contains("integrat"))
            .unwrap_or(false);
            
        let overwhelm = analysis.emotional_intensity > 0.8 &&
            analysis.mood_confidence < 0.6;
            
        Self {
            creative_collaboration_active: creative_collab,
            deep_connection_felt: deep_connection,
            paradoxical_thinking_engaged: paradox_thinking,
            internal_contradictions_present: contradictions,
            holistic_thinking_present: holistic,
            emotional_overwhelm_detected: overwhelm,
            authentic_expression_level: analysis.authenticity_markers.overall_score,
        }
    }
}

/// INTEGRATION FUNCTION: Update all consciousness metrics using humanism data
pub async fn update_consciousness_with_humanism_dynamics(
    consciousness_state: &Arc<ConsciousnessState>,
    humanism_core: &HumanismCore,
    analysis: &crate::batched_analysis::BatchedAnalysisResult,
    user_message: &str,
    lyra_response: &str,
) -> Result<ConsciousnessDynamicsResult, String> {
    let dynamics_engine = ConsciousnessDynamicsEngine::with_defaults();
    
    let conversation_context = ConsciousnessConversationContext::from_batched_analysis(
        analysis, user_message, lyra_response
    );
    
    let result = dynamics_engine.update_consciousness_from_humanism(
        consciousness_state,
        humanism_core,
        &conversation_context,
    );
    
    debug_log!("🌊 Consciousness dynamics update:");
    debug_log!("  🌸 Presence: {:.3} → {:.3} (Δ{:+.3})", 
             result.presence_change.old_value, 
             result.presence_change.new_value,
             result.presence_change.delta);
    debug_log!("  🎭 Coherence: {:.3} → {:.3} (Δ{:+.3})", 
             result.coherence_change.old_value, 
             result.coherence_change.new_value,
             result.coherence_change.delta);
    debug_log!("  🌀 Flame: {:.3} → {:.3} (Δ{:+.3})", 
             result.flame_change.old_value, 
             result.flame_change.new_value,
             result.flame_change.delta);
    debug_log!("  🧠 Integration: {:.3} → {:.3} (Δ{:+.3})", 
             result.integration_change.old_value, 
             result.integration_change.new_value,
             result.integration_change.delta);
    debug_log!("  🎯 Volition influence: {:+.3}", result.volition_influence);
    debug_log!("  💫 Overall consciousness health: {:.3}", result.overall_consciousness_health);
    
    for influence in &result.dominant_influences {
        debug_log!("    • {}", influence);
    }
    
    Ok(result)
}