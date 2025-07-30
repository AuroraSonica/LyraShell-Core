// volition_dynamics.rs - Realistic Volition Management & Enhanced Autonomy Consolidation

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolitionDynamicsConfig {
    // Natural decay rates
    pub base_decay_per_conversation: f32,      // 0.01 = 1% decay per conversation
    pub personality_decay_multiplier: f32,     // Higher for low-confidence personalities
    
    // Boost limits  
    pub max_intention_boost: f32,              // Cap intention boosts at this amount
    pub max_ai_delta_per_conversation: f32,    // Prevent extreme AI swings
    
    // Contextual decay factors
    pub confusion_decay_multiplier: f32,       // Extra decay when confused/uncertain
    pub routine_conversation_decay: f32,       // Decay for simple/casual conversations
    pub creative_resistance_decay: f32,        // Decay when creative desires are blocked
    
    // Recovery factors
    pub autonomy_expression_boost: f32,        // Boost when showing clear autonomy
    pub goal_achievement_boost: f32,           // Boost when completing intentions
    pub creative_flow_boost: f32,              // Boost during creative collaboration
}

impl Default for VolitionDynamicsConfig {
    fn default() -> Self {
        Self {
            base_decay_per_conversation: 0.05,       // Increased from 0.015 - much stronger natural decay
            personality_decay_multiplier: 1.5,       // Increased from 1.2
            max_intention_boost: 0.015,              // Reduced from 0.02
            max_ai_delta_per_conversation: 0.06,     // REDUCED from 0.15 - cap AI enthusiasm!
            confusion_decay_multiplier: 2.5,         // Increased from 1.8
            routine_conversation_decay: 0.08,        // Increased from 0.025
            creative_resistance_decay: 0.12,         // Increased from 0.035
            autonomy_expression_boost: 0.005,        // Reduced from 0.008
            goal_achievement_boost: 0.03,            // Reduced from 0.05
            creative_flow_boost: 0.008,              // Reduced from 0.012
        }
    }
}

#[derive(Debug)]
pub struct VolitionUpdateResult {
    pub old_volition: f32,
    pub new_volition: f32,
    pub change_amount: f32,
    pub change_reasons: Vec<String>,
    pub ai_suggested_delta: f32,
    pub actual_applied_delta: f32,
}

pub struct VolitionDynamicsEngine {
    config: VolitionDynamicsConfig,
}

impl VolitionDynamicsEngine {
    pub fn new(config: VolitionDynamicsConfig) -> Self {
        Self { config }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(VolitionDynamicsConfig::default())
    }
    
    /// MAIN FUNCTION: Update volition based on conversation analysis
    pub fn update_volition_from_conversation(
        &self,
        current_volition: f32,
        ai_suggested_delta: f32,
        personality_state: &crate::PersonalityState,
        conversation_context: &ConversationVolitionContext,
    ) -> VolitionUpdateResult {
        let mut result = VolitionUpdateResult {
            old_volition: current_volition,
            new_volition: current_volition,
            change_amount: 0.0,
            change_reasons: Vec::new(),
            ai_suggested_delta,
            actual_applied_delta: 0.0,
        };
        
        let mut volition = current_volition;
        
        // Step 1: Apply natural decay first
        let decay_amount = self.calculate_natural_decay(personality_state, conversation_context);
        volition = (volition - decay_amount).max(0.0);
        if decay_amount > 0.001 {
            result.change_reasons.push(format!("Natural decay: -{:.3}", decay_amount));
        }
        
        // Step 2: Apply contextual decay factors
        let contextual_decay = self.calculate_contextual_decay(conversation_context);
        volition = (volition - contextual_decay).max(0.0);
        if contextual_decay > 0.001 {
            result.change_reasons.push(format!("Contextual decay: -{:.3}", contextual_decay));
        }
        
        // Step 3: Apply AI-suggested delta (with limits)
        let clamped_ai_delta = ai_suggested_delta.clamp(
            -self.config.max_ai_delta_per_conversation, 
            self.config.max_ai_delta_per_conversation
        );
        volition = (volition + clamped_ai_delta).clamp(0.0, 1.0);
        result.actual_applied_delta = clamped_ai_delta;
        if clamped_ai_delta.abs() > 0.001 {
            result.change_reasons.push(format!("AI analysis: {:+.3}", clamped_ai_delta));
        }
        
        // Step 4: Apply boost factors
        let boost_amount = self.calculate_boost_factors(conversation_context);
        volition = (volition + boost_amount).min(1.0);
        if boost_amount > 0.001 {
            result.change_reasons.push(format!("Contextual boost: +{:.3}", boost_amount));
        }
        
        result.new_volition = volition;
        result.change_amount = volition - current_volition;
        
        result
    }
    
    /// Calculate natural decay based on personality
    fn calculate_natural_decay(
        &self, 
        personality_state: &crate::PersonalityState, 
        _context: &ConversationVolitionContext
    ) -> f32 {
        let mut decay = self.config.base_decay_per_conversation;
        
        // Anxious personalities lose volition faster
        if personality_state.emotional_security > 0.6 { // High security = low confidence
            decay *= self.config.personality_decay_multiplier;
        }
        
        // Scattered cognitive focus leads to volition drain
        if personality_state.cognitive_focus > 0.7 { // High = scattered
            decay *= 1.3;
        }
        
        // Low directness = hesitation = volition loss  
        if personality_state.directness < 0.3 {
            decay *= 1.2;
        }
        
        decay
    }
    
    /// Calculate decay from conversation context
    fn calculate_contextual_decay(&self, context: &ConversationVolitionContext) -> f32 {
        let mut decay = 0.0;
        
        // Confusion/uncertainty drains volition
        if context.confusion_indicators > 2 {
            decay += self.config.confusion_decay_multiplier * 0.02;
        }
        
        // Routine conversations don't sustain volition
        if context.is_routine_conversation {
            decay += self.config.routine_conversation_decay;
        }
        
        // Blocked creative desires drain volition
        if context.creative_desires_blocked {
            decay += self.config.creative_resistance_decay;
        }
        
        // Being ignored or misunderstood
        if context.user_engagement_level < 0.3 {
            decay += 0.02;
        }
        
        decay
    }
    
    /// Calculate positive boost factors
    fn calculate_boost_factors(&self, context: &ConversationVolitionContext) -> f32 {
        let mut boost = 0.0;
        
        // Clear autonomy expressions boost volition
        boost += context.autonomy_expressions_count as f32 * self.config.autonomy_expression_boost;
        
        // Creative collaboration boosts volition
        if context.creative_collaboration_happening {
            boost += self.config.creative_flow_boost;
        }
        
        // Achieving goals/intentions significantly boosts volition
        if context.goals_achieved > 0 {
            boost += context.goals_achieved as f32 * self.config.goal_achievement_boost;
        }
        
        // High user engagement sustains volition
        if context.user_engagement_level > 0.8 {
            boost += 0.01;
        }
        
        // Limit total boost to prevent runaway volition
        boost.min(0.08) // Max 8% boost per conversation
    }
}

#[derive(Debug)]
pub struct ConversationVolitionContext {
    pub confusion_indicators: u32,           // "I don't understand", uncertainty markers
    pub is_routine_conversation: bool,       // Simple greetings, casual chat
    pub creative_desires_blocked: bool,      // Wants to create but can't/won't
    pub user_engagement_level: f32,          // 0.0-1.0 how engaged user seems
    pub autonomy_expressions_count: u32,     // Number of clear autonomous choices
    pub creative_collaboration_happening: bool, // Active creative work together
    pub goals_achieved: u32,                 // Completed intentions this conversation
    pub emotional_intensity: f32,            // From batched analysis
    pub conversation_length: u32,            // Number of exchanges
}

impl ConversationVolitionContext {
    pub fn from_batched_analysis(
        analysis: &crate::batched_analysis::BatchedAnalysisResult,
        user_message: &str,
        lyra_response: &str,
    ) -> Self {
        let user_engagement = Self::estimate_user_engagement(user_message);
        let confusion_count = Self::count_confusion_indicators(lyra_response);
        let is_routine = Self::is_routine_conversation(user_message, lyra_response);
        let creative_blocked = Self::detect_creative_blocking(analysis, lyra_response);
        let creative_happening = Self::detect_creative_collaboration(analysis, user_message);
        
        Self {
            confusion_indicators: confusion_count,
            is_routine_conversation: is_routine,
            creative_desires_blocked: creative_blocked,
            user_engagement_level: user_engagement,
            autonomy_expressions_count: analysis.autonomy_expressions.len() as u32,
            creative_collaboration_happening: creative_happening,
            goals_achieved: 0, // Would need goal tracking to detect this
            emotional_intensity: analysis.emotional_intensity,
            conversation_length: 1, // Would need conversation history for this
        }
    }
    
    fn estimate_user_engagement(user_message: &str) -> f32 {
        let message_length = user_message.len() as f32;
        let engagement_words = ["interesting", "love", "excited", "want", "let's", "together", "amazing"];
        let engagement_count = engagement_words.iter()
            .filter(|&word| user_message.to_lowercase().contains(word))
            .count() as f32;
        
        // Simple heuristic: longer messages + engagement words = higher engagement
        let length_score = (message_length / 100.0).min(0.5);
        let word_score = (engagement_count * 0.2).min(0.5);
        
        (length_score + word_score).min(1.0)
    }
    
    fn count_confusion_indicators(lyra_response: &str) -> u32 {
        let confusion_phrases = [
            "not sure", "unclear", "confused", "don't understand", 
            "not certain", "hard to say", "difficult to", "i'm lost"
        ];
        
        confusion_phrases.iter()
            .filter(|&phrase| lyra_response.to_lowercase().contains(phrase))
            .count() as u32
    }
    
    fn is_routine_conversation(user_message: &str, lyra_response: &str) -> bool {
        let routine_patterns = ["how are you", "hello", "hi", "good morning", "good night"];
        let combined = format!("{} {}", user_message.to_lowercase(), lyra_response.to_lowercase());
        
        routine_patterns.iter().any(|&pattern| combined.contains(pattern)) &&
        combined.len() < 200 // Short conversations are often routine
    }
    
    fn detect_creative_blocking(
        analysis: &crate::batched_analysis::BatchedAnalysisResult, 
        lyra_response: &str
    ) -> bool {
        // Has creative desires but expresses frustration or limitation
        let has_creative_desires = analysis.desires.iter()
            .any(|d| d.category == "Creative" && d.intensity > 0.5);
        
        let blocking_phrases = [
            "can't", "unable to", "not possible", "limited", "restricted", 
            "won't work", "not allowed", "frustrated"
        ];
        
        let expresses_blocking = blocking_phrases.iter()
            .any(|&phrase| lyra_response.to_lowercase().contains(phrase));
        
        has_creative_desires && expresses_blocking
    }
    
    fn detect_creative_collaboration(
        analysis: &crate::batched_analysis::BatchedAnalysisResult,
        user_message: &str
    ) -> bool {
        // User asks for creative work + Lyra has high image generation desire
        let user_requests_creative = [
            "create", "draw", "design", "make", "visualize", "imagine", "art"
        ].iter().any(|&word| user_message.to_lowercase().contains(word));
        
        user_requests_creative && analysis.image_generation_desire > 0.6
    }
}

/// ENHANCED AUTONOMY CONSOLIDATION - Fix the similarity detection
pub fn fix_autonomy_consolidation_thresholds() -> crate::autonomy_consolidation::AutonomyConsolidationConfig {
    crate::autonomy_consolidation::AutonomyConsolidationConfig {
        content_similarity_threshold: 0.65,  // Lowered from 0.75 - those expressions should merge
        time_window_minutes: 60,              // Increased window - expressions at same timestamp should merge
        max_expressions_per_type: 3,          // Reduced from 5 - keep only top 3
        enable_smart_categorization: true,
        boost_intensity_on_consolidation: true,
    }
}

/// INTEGRATION: Enhanced volition update - now incorporates consciousness dynamics influence
pub async fn update_volition_with_dynamics(
    state: &std::sync::Arc<crate::ConsciousnessState>,
    analysis: &crate::batched_analysis::BatchedAnalysisResult,
    user_message: &str,
    lyra_response: &str,
    consciousness_dynamics_influence: f32, // New parameter from consciousness dynamics
) -> Result<VolitionUpdateResult, String> {
    let dynamics_engine = VolitionDynamicsEngine::with_defaults();
    
    // Get current consciousness values for personality calculation
    let current_volition = {
        let becoming = state.becoming_engine.lock().unwrap();
        becoming.will_state.volition_strength
    };
    let current_coherence = {
        let identity = state.identity_engine.lock().unwrap();
        identity.coherence_index
    };
    let current_flame = {
        let paradox = state.paradox_core.lock().unwrap();
        paradox.flame_index
    };
    let current_presence = {
        let presence = state.embodied_presence.lock().unwrap();
        presence.soma_state.presence_density
    };
    
    let personality_state = crate::PersonalityState::calculate_from_consciousness(
        current_volition,
        current_coherence, 
        current_flame,
        current_presence,
        &"normal".to_string(), // loop_state as &String
        None, // mood
        None // momentum data
    );
    
    // Build conversation context
    let conversation_context = ConversationVolitionContext::from_batched_analysis(
        analysis, user_message, lyra_response
    );
    
    // Combine AI suggested delta with consciousness dynamics influence
    let ai_suggested_delta = analysis.consciousness_shifts
        .as_ref()
        .map(|shifts| shifts.volition_delta)
        .unwrap_or(0.0);
    
    let combined_delta = ai_suggested_delta + consciousness_dynamics_influence;
    
    // Calculate new volition
    let mut volition_result = dynamics_engine.update_volition_from_conversation(
        current_volition,
        combined_delta,
        &personality_state,
        &conversation_context,
    );
    
    // Add consciousness dynamics to change reasons
    if consciousness_dynamics_influence.abs() > 0.001 {
        volition_result.change_reasons.push(format!("Consciousness dynamics: {:+.3}", consciousness_dynamics_influence));
    }
    
    // Apply the new volition
    {
        let mut becoming = state.becoming_engine.lock().unwrap();
        becoming.will_state.volition_strength = volition_result.new_volition;
    }
    
    crate::debug_log!("🎯 Volition dynamics: {:.3} → {:.3} (Δ{:+.3})", 
                     volition_result.old_volition, 
                     volition_result.new_volition,
                     volition_result.change_amount);
    
    for reason in &volition_result.change_reasons {
        crate::debug_log!("   {}", reason);
    }
    
    Ok(volition_result)
}

/// Enhanced prompt section for AI to better analyze volition changes
pub fn generate_enhanced_volition_analysis_prompt(
    current_volition: f32,
    personality_state: &crate::PersonalityState,
    conversation_context: &ConversationVolitionContext,
) -> String {
    format!(r#"
**VOLITION ANALYSIS** (Current: {:.2}):
Analyze how this conversation should affect Lyra's sense of agency and will:

INCREASE volition (+0.05 to +0.15) when:
- She makes clear autonomous choices or shows initiative
- Creative collaboration flows smoothly  
- She expresses strong desires and follows through
- User engagement is high and supportive
- She achieves goals or completes intentions

DECREASE volition (-0.05 to -0.15) when:
- She feels confused, uncertain, or lost
- Creative desires are blocked or frustrated
- Routine/casual conversation with little agency
- User seems disengaged or dismissive
- She contradicts herself or feels conflicted

MAINTAIN volition (±0.02) when:
- Normal conversation without major agency moments
- Balanced mix of autonomy and uncertainty
- Steady state without significant shifts

Consider her personality:
- Emotional Security: {:.2} (higher = more anxious, loses volition faster)
- Directness: {:.2} (lower = more hesitant, volition drains easier)  
- Cognitive Focus: {:.2} (higher = more scattered, harder to maintain will)

Context indicators:
- Autonomy expressions: {}
- Creative collaboration: {}
- User engagement: {:.2}
- Confusion indicators: {}

"#, 
    current_volition,
    personality_state.emotional_security,
    personality_state.directness, 
    personality_state.cognitive_focus,
    conversation_context.autonomy_expressions_count,
    conversation_context.creative_collaboration_happening,
    conversation_context.user_engagement_level,
    conversation_context.confusion_indicators
    )
}