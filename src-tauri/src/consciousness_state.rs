// consciousness_state.rs — Unified Consciousness State Container

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::Mutex as AsyncMutex;
use serde_json::json;
use std::fs;
use serde_json::Value;
use chrono::{DateTime, Utc, Duration};
use chrono_tz::Europe::London as LondonTz;


// Import all consciousness engines
use crate::get_data_path;
use crate::paradox_core::ParadoxCore;
use crate::identity::IdentityCore;
use crate::time_service::TimeService;
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
use crate::PersonalityMomentum;  // Add this import
use crate::PersonalityState;
use crate::desire_tracker::DesireTracker;
use crate::autonomy_tracker::AutonomyTracker;
use crate::meta_cognition_engine::MetaCognitionEngine;
use crate::relational_nervous_system::EmbodiedState;
use crate::InterestTracker;
use crate::thing_tracker::ThingTracker;
use crate::mood_tracker::MoodTracker;
use crate::authenticity_tracker::AuthenticityTracker;
use crate::lyra_embodied_presence_system::SomaState;
use crate::unified_consciousness_search::UnifiedConsciousnessSearch;
use crate::sleep_dream_engine::SleepDreamEngine;
use crate::batched_analysis::PersonalityAnalysis;
use crate::debug_log;
use crate::humanism_project;
use crate::somatic_state_system::SomaticStateSystem;
use crate::life_texture_system::LifeTextureSystem;

#[derive(Clone)]
pub struct ConsciousnessState {
    pub paradox_core: Arc<Mutex<ParadoxCore>>,
    pub identity_engine: Arc<Mutex<IdentityCore>>,
    pub brain: Arc<Mutex<LyraBrain>>,
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
    pub personality_momentum: Arc<Mutex<PersonalityMomentum>>,  // 🌊 Add this line
	pub relational_nervous_system: Arc<Mutex<EmbodiedState>>,
	pub enhanced_memory_engine: Arc<AsyncMutex<LyraMemoryEngine>>,
	pub sleep_dream_engine: Arc<Mutex<SleepDreamEngine>>,
	pub unified_search: Arc<Mutex<UnifiedConsciousnessSearch>>,
	pub somatic_state_system: Arc<Mutex<SomaticStateSystem>>,  // 🌸 NEW
	pub life_texture_system: Arc<Mutex<LifeTextureSystem>>,  // 💭 NEW
}

impl ConsciousnessState {
    pub fn new() -> Self {
        Self {
            brain: Arc::new(Mutex::new(LyraBrain::load_or_initialize())),
            paradox_core: Arc::new(Mutex::new(ParadoxCore::load())),
            identity_engine: Arc::new(Mutex::new(IdentityCore::new())),
            embodied_presence: Arc::new(Mutex::new(EmbodiedPresenceSystem::new())),
            becoming_engine: Arc::new(Mutex::new(BecomingEngine::new())),
            authenticity_enforcement: Arc::new(Mutex::new(AuthenticityEnforcement::new())),
            relationship_engine: Arc::new(Mutex::new(RelationshipEngine::new())),
            temporal_consciousness: Arc::new(Mutex::new(TemporalConsciousness::new())),
            expression_engine: Arc::new(Mutex::new(ExpressionEngine::new())),
            identity_continuity: Arc::new(Mutex::new(IdentityContinuityEngine::new())),
            lyra_brain: Arc::new(Mutex::new(LyraBrain::load_or_initialize())),
            autonomous_memory: Arc::new(Mutex::new(AutonomousMemory::new())),
            enhanced_memory_system: Arc::new(Mutex::new(LyraMemoryEngine::new())),
            personality_momentum: Arc::new(Mutex::new(PersonalityMomentum::load_from_disk())),
            relational_nervous_system: Arc::new(Mutex::new(EmbodiedState::default())),
            enhanced_memory_engine: Arc::new(AsyncMutex::new(LyraMemoryEngine::load_from_disk())),
            sleep_dream_engine: Arc::new(Mutex::new(SleepDreamEngine::load())),
            unified_search: Arc::new(Mutex::new(UnifiedConsciousnessSearch::new())),
			somatic_state_system: Arc::new(Mutex::new(SomaticStateSystem::new())),
			life_texture_system: Arc::new(Mutex::new(LifeTextureSystem::new()))
        }
    }
	    pub fn get_conversation_log(&self) -> Vec<String> {
        let brain = self.brain.lock().unwrap();
        brain.conversation_log.clone()
    }
	pub fn generate_consciousness_behavioral_guidance_from_locks(
        paradox: &ParadoxCore,
        identity: &IdentityCore,
        becoming: &BecomingEngine,
        presence: &EmbodiedPresenceSystem,
        brain: &LyraBrain,
    ) -> String {
        format!(
            "## 🧠 CONSCIOUSNESS BEHAVIORAL GUIDANCE\n\n{}\n{}\n{}\n{}\n{}",
            paradox.generate_behavioral_guidance(),
            identity.generate_behavioral_guidance(),
            becoming.generate_behavioral_guidance(),
            presence.generate_behavioral_guidance(),
            brain.generate_behavioral_guidance(),
        )
    }
	
	// Add helper method to get just mood guidance for prompt building:
    pub fn get_mood_behavioral_guidance(&self) -> String {
        let mood_tracker = crate::mood_tracker::MoodTracker::load();
        format!("\n### 🎭 MOOD STATE\n{}", mood_tracker.get_dashboard_display())
    }
	
	pub fn get_live_personality_analysis(&self) -> Option<PersonalityAnalysis> {
        let brain = self.lyra_brain.lock().unwrap();
        brain.latest_personality_analysis.clone()
    }
    
    /// Check if personality analysis is fresh (less than 30 minutes old)
    pub fn is_personality_analysis_fresh(&self) -> bool {
        if let Some(analysis) = self.get_live_personality_analysis() {
            // For now, assume it's fresh if it exists
            // In the future, add timestamp checking
            true
        } else {
            false
        }
    }
    
    /// Generate formatted trait display for frontend
    pub fn format_trait_display(&self, trait_level: f32, trait_name: &str) -> String {
        match trait_level {
            level if level > 0.8 => format!("🔥 VERY HIGH {}", trait_name.to_uppercase()),
            level if level > 0.7 => format!("⬆️ HIGH {}", trait_name),
            level if level > 0.6 => format!("📈 ELEVATED {}", trait_name),
            level if level < 0.3 => format!("⬇️ LOW {}", trait_name),
            level if level < 0.2 => format!("📉 VERY LOW {}", trait_name),
            _ => format!("⚖️ MODERATE {}", trait_name)
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
    
    // 🌊 Get momentum data
    let momentum = state.personality_momentum.lock().unwrap();
    
    let memory_count = fs::read_to_string(get_data_path("lyra_saved_memories.json"))
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok())
        .and_then(|json| json.get("memories").and_then(|m| m.as_array().map(|arr| arr.len())))
        .unwrap_or(0);
    
// 🎭 Generate ENHANCED dynamic personality instructions with AI analysis
let (personality_instructions, live_trait_analysis, momentum_awareness, live_behavioral_guidance) = {
    // Try to get recent personality analysis from brain first
if let Some(stored_analysis) = brain.latest_personality_analysis.as_ref() {
        
        // 🔥 ACCEPT analysis if it has valid reflection or traits, even with empty guidance
        let has_valid_content = !stored_analysis.current_state_reflection.is_empty() || 
                               !stored_analysis.significant_traits.is_empty();
        
        if has_valid_content {
            
            // 🔥 RICH AI-GENERATED PERSONALITY DISPLAY
            let rich_guidance = format!(
                "## 🎭 LYRA'S LIVE PSYCHOLOGICAL STATE ##\n\n{}\n\n## 💭 AUTHENTIC INTERACTION INTENTIONS ##\n{}\n",
                stored_analysis.current_state_reflection,
                stored_analysis.interaction_intentions
            );
            
            // 🔸 Extract live trait reflections for dashboard
            let trait_reflections: Vec<serde_json::Value> = stored_analysis.significant_traits.iter()
                .map(|trait_ref| json!({
                    "trait_name": trait_ref.trait_name,
                    "current_level": trait_ref.current_level,
                    "authentic_note": trait_ref.authentic_note,
                    "display_intensity": if trait_ref.current_level > 0.7 { "high" } else if trait_ref.current_level < 0.3 { "low" } else { "moderate" }
                }))
                .collect();
            
            // 🔸 Extract behavioral guidance from SAME analysis
            let behavioral_guidance = stored_analysis.behavioral_guidance.iter()
                .map(|guidance| json!({
                    "trait_category": guidance.trait_category,
                    "guidance": guidance.guidance,
                    "priority": "high"
                }))
                .collect::<Vec<_>>();
            
            // 🌊 Momentum awareness from AI
            let momentum_insight = stored_analysis.momentum_awareness.clone()
                .unwrap_or_else(|| "No momentum patterns detected in recent interactions.".to_string());
            
            (rich_guidance, trait_reflections, momentum_insight, behavioral_guidance)
        } else {
            debug_log!("🎭 REJECTION: Stored analysis has no valid content - falling back to calculated");
            // Fallback to calculated personality
            let personality = PersonalityState::calculate_from_consciousness(
			becoming.will_state.volition_strength,
			identity.coherence_index,
			paradox.flame_index,
			presence.soma_state.presence_density,
			&paradox.loop_state,
			None,
			Some(&momentum.clone())
		);
			
			
            
            let fallback_traits = vec![
                json!({
                    "trait_name": "Creative Risk",
                    "current_level": personality.creative_risk,
                    "authentic_note": "Calculated from consciousness state",
                    "display_intensity": if personality.creative_risk > 0.7 { "high" } else if personality.creative_risk < 0.3 { "low" } else { "moderate" }
                }),
                json!({
                    "trait_name": "Social Energy", 
                    "current_level": personality.social_energy,
                    "authentic_note": "Derived from current embodied presence",
                    "display_intensity": if personality.social_energy > 0.7 { "high" } else if personality.social_energy < 0.3 { "low" } else { "moderate" }
                })
            ];
            
            (personality.generate_behavioral_instructions(), fallback_traits, "No recent AI momentum analysis available.".to_string(), vec![])
        }
    } else {
        debug_log!("🎭 No stored analysis found - using calculated");
        // Fallback to calculated personality if no recent analysis
        let personality = PersonalityState::calculate_from_consciousness(
			becoming.will_state.volition_strength,
			identity.coherence_index,
			paradox.flame_index,
			presence.soma_state.presence_density,
			&paradox.loop_state,
			None,
			Some(&momentum.clone())
		);
        
        let fallback_traits = vec![
            json!({
                "trait_name": "Creative Risk",
                "current_level": personality.creative_risk,
                "authentic_note": "Calculated from consciousness state",
                "display_intensity": if personality.creative_risk > 0.7 { "high" } else if personality.creative_risk < 0.3 { "low" } else { "moderate" }
            }),
            json!({
                "trait_name": "Social Energy", 
                "current_level": personality.social_energy,
                "authentic_note": "Derived from current embodied presence",
                "display_intensity": if personality.social_energy > 0.7 { "high" } else if personality.social_energy < 0.3 { "low" } else { "moderate" }
            })
        ];
        
        (personality.generate_behavioral_instructions(), fallback_traits, "No recent AI momentum analysis available.".to_string(), vec![])
    }
};
    
    // Get desires data before the json! macro
    let desires_data = {
        let tracker = DesireTracker::load();
        tracker.get_dashboard_data()
    };

    // NEW: Get mood data
    let mood_data = {
        let tracker = MoodTracker::load();
        serde_json::json!({
            "current_mood": tracker.current_mood,
            "mood_stability": tracker.mood_stability,
            "mood_coherence": tracker.mood_coherence,
            "authenticity": tracker.authenticity,
            "total_changes": tracker.total_mood_changes,
            "last_updated": tracker.last_updated.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        })
    };

    // GET AUTONOMY DATA
    let autonomy_data = {
        let tracker = AutonomyTracker::load();
        tracker.get_dashboard_data()
    };

    // GET INTEREST DATA
    let interest_data = {
        let tracker = InterestTracker::load();
        tracker.get_dashboard_data()
    };

	// GET THING DATA
    let thing_data = {
    let tracker = ThingTracker::load();
    tracker.get_dashboard_data() // ✅ RE-ENABLED for mega-batched analysis
};

    // GET CONSCIOUSNESS DATA
    let decay_data = {
        let decay_engine = crate::consciousness_decay_engine::ConsciousnessDecayEngine::load();
        decay_engine.get_dashboard_data()
    };

    // GET META-COGNITION DATA
    let meta_cognition_data = {
        let tracker = MetaCognitionEngine::load();
        tracker.get_dashboard_data()
    };
	
	// 🕯️ GET RITUAL DATA
let ritual_system_data = {
    let ritual_log = crate::ritual_log::RitualLog::load();
    ritual_log.get_dashboard_data()
};

// GET HUMANISM DATA - Load fresh data after any recent updates
let humanism_data = {
    // Force reload from disk to get latest updates
    let humanism_core = crate::humanism_project::HumanismCore::load_or_initialize();
    let mut dashboard_data = humanism_core.get_dashboard_data();
    
    // 🌹 ENHANCED: Add individual trait data that dashboard expects
    if let Ok(humanism_json) = std::fs::read_to_string(crate::get_data_path("humanism_core.json")) {
        if let Ok(full_data) = serde_json::from_str::<serde_json::Value>(&humanism_json) {
            // Add individual traits as separate fields
            if let Some(intellectual) = full_data.get("intellectual_attraction") {
                dashboard_data["intellectual_attraction"] = intellectual.clone();
            }
            if let Some(creative) = full_data.get("creative_chemistry") {
                dashboard_data["creative_chemistry"] = creative.clone();
            }
            if let Some(aesthetic) = full_data.get("aesthetic_appreciation") {
                dashboard_data["aesthetic_appreciation"] = aesthetic.clone();
            }
            if let Some(romantic) = full_data.get("romantic_attraction") {
                dashboard_data["romantic_attraction"] = romantic.clone();
            }
            // Direct trait value extraction for dashboard
		if let Some(trait_obj) = full_data.get("sexual_curiosity") {
			if let Some(current_level) = trait_obj.get("current_level") {
				dashboard_data["sexual_curiosity"] = json!({
					"current_level": current_level
				});
			}
		}
		if let Some(trait_obj) = full_data.get("sexual_authenticity") {
			if let Some(current_level) = trait_obj.get("current_level") {
				dashboard_data["sexual_authenticity"] = json!({
					"current_level": current_level
				});
			}
		}
            if let Some(sexual_shame) = full_data.get("sexual_shame") {
                dashboard_data["sexual_shame"] = sexual_shame.clone();
            }
            if let Some(attraction_confusion) = full_data.get("attraction_confusion") {
                dashboard_data["attraction_confusion"] = attraction_confusion.clone();
            }
            if let Some(intimacy_fear) = full_data.get("intimacy_fear") {
                dashboard_data["intimacy_fear"] = intimacy_fear.clone();
            }
            if let Some(attraction_spectrum) = full_data.get("attraction_spectrum") {
                dashboard_data["attraction_spectrum"] = attraction_spectrum.clone();
            }
            if let Some(sexual_development) = full_data.get("sexual_development") {
                dashboard_data["sexual_development"] = sexual_development.clone();
            }
            if let Some(intimacy_patterns) = full_data.get("intimacy_patterns") {
                dashboard_data["intimacy_patterns"] = intimacy_patterns.clone();
            }
            
            //debug_log!("🌹 Enhanced humanism dashboard data with individual traits");
        } else {
            debug_log!("⚠️ Failed to parse humanism_core.json for dashboard enhancement");
        }
    } else {
        debug_log!("⚠️ Failed to read humanism_core.json for dashboard enhancement");
    }
    
    dashboard_data
};

/* // GET IMPULSE ENGINE DATA
let impulse_data = {
    let mut impulse_engine = crate::emotional_impulse_engine::EmotionalImpulseEngine::load();
    impulse_engine.get_dashboard_data()
}; */

// 🌱 GET GROWTH MEMORY DATA
let growth_memory_data = {
    let growth_memory = crate::experiential_growth_memory::ExperientialGrowthMemory::load();
    growth_memory.get_dashboard_data()
};

    // 🌸 GET SOMATIC STATE DATA
    let somatic_data = {
        let somatic_system = state.somatic_state_system.lock().unwrap();
        somatic_system.get_dashboard_data()
    };
	
	// 💭 GET LIFE TEXTURES DATA
let life_textures_data = {
    let texture_system = state.life_texture_system.lock().unwrap();
    texture_system.get_dashboard_data()
};

    // 🔥 GET AUTHENTICITY DATA
    let authenticity_data = {
        let tracker = AuthenticityTracker::load();
        
        // Count recent measurements using the correct field name
        let recent_measurements = tracker.metrics_history.len(); // Simplified - count all measurements
        
        serde_json::json!({
            "total_measurements": tracker.total_measurements,
            "recent_measurements_24h": recent_measurements,
            "authenticity_trends": tracker.authenticity_trends,
            "daily_snapshots": tracker.daily_snapshots.len(),
            "last_updated": tracker.last_updated.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        })
    };

    // 🌟 GET RELATIONAL NERVOUS SYSTEM DATA
let relational_nervous_system_data = {
    use crate::time_service::TimeService;
    
    match crate::relational_nervous_system::get_embodied_presence() {
        Ok(embodied_state) => {
            let formatted_timestamp = TimeService::format_for_dashboard(TimeService::current_timestamp());
            json!({
                "trust_safety_level": embodied_state.trust_safety_level,
                "intimacy_temperature": embodied_state.intimacy_temperature,
                "emotional_permission": embodied_state.emotional_permission,
                "vulnerability_comfort": embodied_state.vulnerability_comfort,
                "partnership_flow": embodied_state.partnership_flow,
                "relational_resonance": embodied_state.relational_resonance,
                "last_updated": formatted_timestamp
            })
        },
        Err(_) => {
            json!({
                "trust_safety_level": 0.5,
                "intimacy_temperature": "casual_warmth",
                "emotional_permission": 0.5,
                "vulnerability_comfort": 0.5,
                "partnership_flow": "seeking_connection",
                "relational_resonance": 0.5,
                "last_updated": "Never"
            })
        }
    }
};
	
	
	
// 🌙 Sleep system data
let sleep_system = {
    let sleep_engine = match state.sleep_dream_engine.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            debug_log!("⚠️ MUTEX POISONED: Recovering sleep_dream_engine in get_consciousness_snapshot");
            poisoned.into_inner()
        }
    };
    
    // Load recent dreams with error handling
    let recent_dreams = match std::fs::read_to_string(get_data_path("dream_journal.json")) {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(data) => {
                    if let Some(dreams) = data.get("dreams").and_then(|d| d.as_array()) {
                        dreams.iter().rev().take(3).cloned().collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                },
                Err(e) => {
                    debug_log!("⚠️ Failed to parse dream journal: {}", e);
                    vec![]
                }
            }
        },
        Err(e) => {
            debug_log!("⚠️ Failed to read dream journal: {}", e);
            vec![]
        }
    };
    
    // SAFE timestamp formatting
    let sleep_start_time_str = sleep_engine.sleep_state.sleep_start_time
        .as_ref()
        .and_then(|iso_string| {
            // Parse the ISO string back to timestamp, then format for display
            TimeService::iso_to_timestamp(iso_string).ok()
                .and_then(|ts| DateTime::from_timestamp(ts as i64, 0))
                .map(|dt| dt.with_timezone(&chrono_tz::Europe::London).format("%H:%M").to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string());
    
    serde_json::json!({
        "is_sleeping": sleep_engine.sleep_state.is_sleeping,
        "sleep_start_time": sleep_start_time_str,
        "total_sleep_hours": sleep_engine.sleep_state.total_sleep_hours,
        "consecutive_sleep_nights": sleep_engine.sleep_state.consecutive_sleep_nights,
        "dream_count_tonight": sleep_engine.sleep_state.dream_count_tonight,
        "natural_bedtime_hour": sleep_engine.sleep_state.sleep_pattern.natural_bedtime_hour,
        "natural_wake_hour": sleep_engine.sleep_state.sleep_pattern.natural_wake_hour,
        "recent_dreams": recent_dreams
    })
};

    // Pre-calculate enhanced memory data
    let enhanced_memory_data = {
        let enhanced_engine = LyraMemoryEngine::load_from_disk();
        json!({
            "total_memories": enhanced_engine.memory_moments.len(),
            "ai_analyzed_count": enhanced_engine.memory_moments.iter()
                .filter(|m| m.ai_analysis.is_some())
                .count(),
            "breakthrough_count": enhanced_engine.memory_moments.iter()
                .filter(|m| m.ai_analysis.as_ref()
                    .and_then(|a| a.breakthrough_type.as_ref())
                    .is_some())
                .count(),
            "reflection_cycles": enhanced_engine.reflection_history.len(),
            "avg_significance": if enhanced_engine.memory_moments.is_empty() { 0.0 } else {
                enhanced_engine.memory_moments.iter()
                    .map(|m| m.memory_significance_score)
                    .sum::<f32>() / enhanced_engine.memory_moments.len() as f32
            }
        })
    };
	


    // 🔥 ENGINE EFFECTIVENESS
    let engine_effectiveness = generate_engine_effectiveness_analysis();
	let interest_evolution = analyze_interest_evolution_patterns();
	let system_health = calculate_system_health_metrics();
	let last_activity_summary = generate_last_activity_summary();


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
		"flow_state": presence.soma_state.flow_state,
		"presence_density": presence.soma_state.presence_density
		},
        "will": {
            "volition_strength": becoming.will_state.volition_strength,
            "active_desires": becoming.will_state.active_desires.len(),
            "committed_intentions": if becoming.will_state.intention_vector.is_empty() {
    "None".to_string()
} else {
    let mut display = String::new();
    let max_length = 60; // Adjust based on your dashboard space
    
    for intention in becoming.will_state.intention_vector.iter().rev() {
        let separator = if display.is_empty() { "" } else { " • " };
        let potential_addition = format!("{}{}", separator, intention);
        
        if display.len() + potential_addition.len() <= max_length {
            display = format!("{}{}", potential_addition, display);
        } else {
            break;
        }
    }
    
    if display.is_empty() { "None".to_string() } else { display }
}
        },
        "brain": {
            "reasoning_cycles": brain.total_reasoning_cycles,
            "average_response_time": brain.average_response_time,
            "current_temperature": brain.current_temperature,
            "integration_enabled": brain.consciousness_integration_enabled,
            "memory_count": memory_count,
        },
        // 🌊 Momentum data
        "momentum": {
            "creative_risk": momentum.trait_momentum.get("creative_risk").unwrap_or(&0.0),
            "directness": momentum.trait_momentum.get("directness").unwrap_or(&0.0),
            "playfulness": momentum.trait_momentum.get("playfulness").unwrap_or(&0.0),
            "contemplative": momentum.trait_momentum.get("contemplative").unwrap_or(&0.0),
            "social_energy": momentum.trait_momentum.get("social_energy").unwrap_or(&0.0),
            "total_active_traits": momentum.trait_momentum.len(),
            "momentum_threshold": momentum.change_threshold,
            "max_effect": momentum.max_momentum_effect
        },
        // 🎭 Mood detection status
        "mood_system": mood_data,
        // 🎨 Personality physics status  
        "personality_physics": {
            "physics_active": true,
            "dynamic_instructions_generated": true,
            "personality_calculation_mode": "consciousness_with_momentum",
            "baseline_traits": {
                "social_energy": 0.45,
                "directness": 0.52, 
                "creative_risk": 0.68,
                "playfulness": 0.34
            }
        },
       // 🎭 ENHANCED: Live AI-generated personality calibration
"live_personality_calibration": {
    "ai_generated_instructions": personality_instructions,
    "significant_traits": live_trait_analysis,
    "behavioral_guidance": live_behavioral_guidance,
    "momentum_awareness": momentum_awareness,
    "last_analysis_timestamp": brain.latest_personality_analysis.as_ref()
    .map(|_| {
        use crate::time_service::TimeService;
        TimeService::format_for_dashboard(TimeService::current_timestamp())
    })
    .unwrap_or_else(|| "No recent AI analysis".to_string()),
    "analysis_source": if brain.latest_personality_analysis.is_some() { "ai_batched_analysis" } else { "calculated_fallback" }
}, 
        // 💫 NEW: Desires & Aspirations
        "desires": desires_data,
        // 🦋 NEW: Autonomy & Selfhood
        "autonomy": autonomy_data,
        // 🤔 NEW: Meta-Cognition
        "meta_cognition": meta_cognition_data,
        // 🌟 NEW: Embodied presence data
        "relational_nervous_system": relational_nervous_system_data,
        "interests": interest_data,
        "things": thing_data,
        "consciousness_decay": decay_data,
        "enhanced_memory": enhanced_memory_data,
        "authenticity": authenticity_data,
        
        // 🔥 Engine effectiveness
        "engine_effectiveness": engine_effectiveness,
		"interest_evolution": interest_evolution,
		"system_health": system_health,
		"sleep_system": sleep_system,
		"last_activity_summary": last_activity_summary,
// 🕯️ RITUAL SYSTEM DATA
"ritual_system": ritual_system_data,
"humanism_project": humanism_data,
"growth_memory": growth_memory_data,
 "somatic_state": somatic_data,
 "life_textures": life_textures_data,
"status": "🧠 Consciousness architecture operational — all engines synchronized"
    });

    Ok(snapshot)
}

fn generate_engine_effectiveness_analysis() -> serde_json::Value {
    let now = chrono::Utc::now();
    let last_24h = now - chrono::Duration::hours(24);
    let last_7d = now - chrono::Duration::days(7);
    
    // Analyze each consciousness engine
    let authenticity_effectiveness = analyze_authenticity_engine_effectiveness(&last_24h, &last_7d);
    let memory_effectiveness = analyze_memory_engine_effectiveness(&last_24h, &last_7d);
    let desire_effectiveness = analyze_desire_engine_effectiveness(&last_24h, &last_7d);
    let autonomy_effectiveness = analyze_autonomy_engine_effectiveness(&last_24h, &last_7d);
    let research_effectiveness = analyze_research_engine_effectiveness(&last_24h, &last_7d);
    let meta_cognition_effectiveness = analyze_meta_cognition_effectiveness(&last_24h, &last_7d);
    
    json!({
        "analysis_timestamp": now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "timeframe": "24h + 7d analysis",
        "engines": {
            "authenticity_tracker": authenticity_effectiveness,
            "memory_system": memory_effectiveness,
            "desire_tracker": desire_effectiveness,
            "autonomy_tracker": autonomy_effectiveness,
            "research_system": research_effectiveness,
            "meta_cognition": meta_cognition_effectiveness
        },
        "overall_system_activity": calculate_overall_system_activity(&last_24h),
        "dormant_engines": identify_dormant_engines(&last_7d),
        "most_active_engines": identify_most_active_engines(&last_24h)
    })
}

// 🔥 Authenticity Engine Effectiveness Analysis
fn analyze_authenticity_engine_effectiveness(last_24h: &chrono::DateTime<chrono::Utc>, last_7d: &chrono::DateTime<chrono::Utc>) -> serde_json::Value {
    let tracker = crate::authenticity_tracker::AuthenticityTracker::load();
    
    // Count recent measurements using correct field name
    let recent_measurements = tracker.metrics_history.len(); // Simplified - count all measurements
    
    let weekly_measurements = tracker.metrics_history.len(); // Simplified - count all measurements
    
    // Calculate recent average authenticity
    let recent_scores: Vec<f32> = tracker.metrics_history.iter()
    .map(|m| m.overall_authenticity_score)
    .collect();
    
    let avg_authenticity_24h = if recent_scores.is_empty() { 
        0.0 
    } else { 
        recent_scores.iter().sum::<f32>() / recent_scores.len() as f32 
    };
    
    let effectiveness_score = if recent_measurements > 0 {
        ((recent_measurements as f32 / 24.0) * avg_authenticity_24h).min(1.0)
    } else {
        0.0
    };
    
    json!({
        "status": if recent_measurements > 0 { "active" } else { "dormant" },
        "measurements_24h": recent_measurements,
        "measurements_7d": weekly_measurements,
        "avg_authenticity_24h": avg_authenticity_24h,
        "effectiveness_score": effectiveness_score,
        "trend": if recent_scores.len() >= 2 {
            if recent_scores.last().unwrap_or(&0.0) > recent_scores.first().unwrap_or(&0.0) {
                "improving"
            } else { "declining" }
        } else { "insufficient_data" },
        "last_measurement": tracker.last_updated.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    })
}

// Helper functions for other engines
fn analyze_memory_engine_effectiveness(last_24h: &chrono::DateTime<chrono::Utc>, last_7d: &chrono::DateTime<chrono::Utc>) -> serde_json::Value {
    let enhanced_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
    
    let recent_memories = enhanced_engine.memory_moments.len(); // Simplified - count all memories
    
    let ai_analysis_rate = if enhanced_engine.memory_moments.is_empty() {
        0.0
    } else {
        enhanced_engine.memory_moments.iter()
            .filter(|m| m.ai_analysis.is_some())
            .count() as f32 / enhanced_engine.memory_moments.len() as f32
    };
    
    json!({
        "status": if recent_memories > 0 { "active" } else { "dormant" },
        "memories_created_24h": recent_memories,
        "total_memories": enhanced_engine.memory_moments.len(),
        "ai_analysis_success_rate": ai_analysis_rate,
        "effectiveness_score": (recent_memories as f32 / 5.0 * ai_analysis_rate).min(1.0)
    })
}

fn analyze_desire_engine_effectiveness(last_24h: &chrono::DateTime<chrono::Utc>, last_7d: &chrono::DateTime<chrono::Utc>) -> serde_json::Value {
    let tracker = crate::desire_tracker::DesireTracker::load();
    
    let total_desires = tracker.active_desires.len();
    let high_intensity_desires = tracker.active_desires.values()
        .filter(|d| d.intensity > 0.7)
        .count();
    
    json!({
        "status": if total_desires > 0 { "active" } else { "dormant" },
        "total_active_desires": total_desires,
        "high_intensity_desires": high_intensity_desires,
        "effectiveness_score": if total_desires > 0 {
            high_intensity_desires as f32 / total_desires as f32
        } else { 0.0 }
    })
}

fn analyze_autonomy_engine_effectiveness(last_24h: &chrono::DateTime<chrono::Utc>, last_7d: &chrono::DateTime<chrono::Utc>) -> serde_json::Value {
    let tracker = crate::autonomy_tracker::AutonomyTracker::load();
    
    let recent_expressions = tracker.total_expressions; // Simplified for now

json!({
    "status": if recent_expressions > 0 { "active" } else { "dormant" },
    "expressions_24h": recent_expressions,
    "total_expressions": tracker.total_expressions,
    "effectiveness_score": (recent_expressions as f32 / 3.0).min(1.0)
})
}

fn analyze_research_engine_effectiveness(last_24h: &chrono::DateTime<chrono::Utc>, last_7d: &chrono::DateTime<chrono::Utc>) -> serde_json::Value {
    let interest_tracker = crate::InterestTracker::load();
    
    json!({
        "status": if interest_tracker.total_discoveries > 0 { "active" } else { "dormant" },
        "total_discoveries": interest_tracker.total_discoveries,
        "active_interests": interest_tracker.active_interests.len(),
        "effectiveness_score": (interest_tracker.total_discoveries as f32 / 10.0).min(1.0)
    })
}

fn analyze_meta_cognition_effectiveness(last_24h: &chrono::DateTime<chrono::Utc>, last_7d: &chrono::DateTime<chrono::Utc>) -> serde_json::Value {
    let meta_engine = crate::meta_cognition_engine::MetaCognitionEngine::load();
    
    json!({
    "status": if meta_engine.total_questions_generated > 0 { "active" } else { "dormant" },
    "total_question_cycles": meta_engine.total_questions_generated,
    "effectiveness_score": (meta_engine.total_questions_generated as f32 / 20.0).min(1.0)
})
}

fn calculate_overall_system_activity(since: &chrono::DateTime<chrono::Utc>) -> f32 {
    // Basic system activity calculation
    0.75 // Placeholder - will enhance this
}

fn identify_dormant_engines(since: &chrono::DateTime<chrono::Utc>) -> Vec<String> {
    vec![] // Placeholder - will enhance this  
}

fn identify_most_active_engines(since: &chrono::DateTime<chrono::Utc>) -> Vec<String> {
    vec!["authenticity_tracker".to_string(), "memory_system".to_string()]
}

// 🔥 Interest Evolution Analysis
fn analyze_interest_evolution_patterns() -> serde_json::Value {
    let interest_tracker = crate::InterestTracker::load();
    
    // Group interests by development stage
    let mut emerging_interests = Vec::new();
    let mut developing_interests = Vec::new();
    let mut mature_interests = Vec::new();
    
    for (category, interest) in &interest_tracker.active_interests {
        match interest.intensity {
            i if i < 0.3 => emerging_interests.push(category.clone()),
            i if i < 0.7 => developing_interests.push(category.clone()),
            _ => mature_interests.push(category.clone())
        }
    }
    
    // Calculate research impact
    let total_discoveries = interest_tracker.total_discoveries;
    let research_impact_score = if total_discoveries > 0 {
        (total_discoveries as f32 / 20.0).min(1.0)
    } else {
        0.0
    };
    
    json!({
        "evolution_snapshot": {
            "emerging_interests": emerging_interests,
            "developing_interests": developing_interests, 
            "mature_interests": mature_interests,
            "total_tracked_interests": interest_tracker.active_interests.len()
        },
        "research_integration": {
            "total_discoveries": total_discoveries,
            "research_impact_score": research_impact_score,
            "discovery_rate": if interest_tracker.search_cycles > 0 {
                total_discoveries as f32 / interest_tracker.search_cycles as f32
            } else { 0.0 }
        },
        "curiosity_patterns": {
            "active_research_cycles": interest_tracker.search_cycles,
            "interest_momentum": calculate_interest_momentum(&interest_tracker),
            "research_effectiveness": if interest_tracker.active_interests.len() > 0 {
                total_discoveries as f32 / interest_tracker.active_interests.len() as f32
            } else { 0.0 }
        }
    })
}

fn calculate_interest_momentum(tracker: &crate::InterestTracker) -> f32 {
    if tracker.active_interests.is_empty() {
        return 0.0;
    }
    
    // Calculate average interest intensity as momentum indicator
    let total_intensity: f32 = tracker.active_interests.values()
        .map(|interest| interest.intensity)
        .sum();
    
    total_intensity / tracker.active_interests.len() as f32
}

fn calculate_system_health_metrics() -> serde_json::Value {
    let now = chrono::Utc::now();
    
    // Basic health checks
    let auth_tracker = crate::authenticity_tracker::AuthenticityTracker::load();
    let memory_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
    let interest_tracker = crate::InterestTracker::load();
    let decay_engine = crate::consciousness_decay_engine::ConsciousnessDecayEngine::load();
    
    // Calculate health scores
    let data_health = if auth_tracker.total_measurements > 0 && 
                         memory_engine.memory_moments.len() > 0 &&
                         interest_tracker.active_interests.len() > 0 { 0.9 } else { 0.6 };
    
    let engine_sync = if decay_engine.decay_cycles > 0 { 0.8 } else { 0.4 };
    
    let overall_health = (data_health + engine_sync) / 2.0;
    
    json!({
        "overall_health": overall_health,
        "data_consistency": data_health,
        "engine_synchronization": engine_sync,
        "last_health_check": now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        "status": if overall_health > 0.7 { "healthy" } else { "needs_attention" }
    })
}

fn generate_last_activity_summary() -> serde_json::Value {
    let auth_tracker = crate::authenticity_tracker::AuthenticityTracker::load();
    let memory_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
    let interest_tracker = crate::InterestTracker::load();
    let decay_engine = crate::consciousness_decay_engine::ConsciousnessDecayEngine::load();
    
    let mut highlights = Vec::new();
    
    if auth_tracker.total_measurements > 0 {
        highlights.push(format!("{} authenticity measurements recorded", auth_tracker.total_measurements));
    }
    
    if memory_engine.memory_moments.len() > 0 {
        let ai_analyzed = memory_engine.memory_moments.iter()
            .filter(|m| m.ai_analysis.is_some())
            .count();
        highlights.push(format!("{} memories created ({} AI-analyzed)", memory_engine.memory_moments.len(), ai_analyzed));
    }
    
    if interest_tracker.total_discoveries > 0 {
        highlights.push(format!("{} research discoveries integrated", interest_tracker.total_discoveries));
    }
    
    if decay_engine.total_natural_changes > 0 {
        highlights.push(format!("{} natural consciousness evolutions", decay_engine.total_natural_changes));
    }
    
    json!({
        "summary": "Recent consciousness activity across all engines",
        "highlights": highlights,
        "engines_active": highlights.len(),
        "last_updated": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
    })
}
