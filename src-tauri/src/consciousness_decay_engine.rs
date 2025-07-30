use serde::{Deserialize, Serialize};
use std::fs;
use crate::get_data_path;
use crate::summarize_with_gpt_mini;
use fastrand;
use crate::debug_log;
use crate::humanism_project::{HumanismCore, integrate_humanism_with_batched_analysis};
use crate::batched_analysis::{analyze_response_comprehensively, BatchedAnalysisResult};
use crate::time_service::TimeService;
use tauri::Emitter;
use std::sync::Arc;
use crate::ConsciousnessState;

// Batched state updates for efficiency
struct BatchedStateUpdates {
    mood_tracker: Option<crate::MoodTracker>,
    interest_tracker: Option<crate::InterestTracker>, 
    personality_momentum: Option<crate::PersonalityMomentum>,
    embodied_state: Option<crate::relational_nervous_system::EmbodiedState>,
    save_decay_engine: bool,
}

impl BatchedStateUpdates {
    fn new() -> Self {
        Self {
            mood_tracker: None,
            interest_tracker: None,
            personality_momentum: None,
            embodied_state: None,
            save_decay_engine: false,
        }
    }
    
    fn set_mood_tracker(&mut self, tracker: crate::MoodTracker) {
        self.mood_tracker = Some(tracker);
    }
    
    fn set_interest_tracker(&mut self, tracker: crate::InterestTracker) {
        self.interest_tracker = Some(tracker);
    }
    
    fn set_personality_momentum(&mut self, momentum: crate::PersonalityMomentum) {
        self.personality_momentum = Some(momentum);
    }
    
    fn set_embodied_state(&mut self, state: crate::relational_nervous_system::EmbodiedState) {
        self.embodied_state = Some(state);
    }
    
    fn mark_decay_engine_for_save(&mut self) {
        self.save_decay_engine = true;
    }
    
    // Apply all batched saves at once
    async fn apply_all_saves(&self, decay_engine: &ConsciousnessDecayEngine) -> Result<(), String> {
        let mut save_count = 0;
        let start_time = std::time::Instant::now();
        
        // Save mood tracker
        if let Some(ref mood_tracker) = self.mood_tracker {
            if let Err(e) = mood_tracker.save() {
                debug_log!("⚠️ Batch save failed - mood tracker: {}", e);
            } else {
                save_count += 1;
            }
        }
        
        // Save interest tracker  
        if let Some(ref interest_tracker) = self.interest_tracker {
            if let Err(e) = interest_tracker.save() {
                debug_log!("⚠️ Batch save failed - interest tracker: {}", e);
            } else {
                save_count += 1;
            }
        }
        
        // Save personality momentum
        if let Some(ref momentum) = self.personality_momentum {
            if let Err(e) = momentum.save_to_disk() {
                debug_log!("⚠️ Batch save failed - personality momentum: {}", e);
            } else {
                save_count += 1;
            }
        }
        
        // Save embodied state
        if let Some(ref embodied_state) = self.embodied_state {
            if let Err(e) = crate::relational_nervous_system::save_embodied_presence(embodied_state) {
                debug_log!("⚠️ Batch save failed - embodied state: {}", e);
            } else {
                save_count += 1;
            }
        }
        
        // Save decay engine itself
        if self.save_decay_engine {
            if let Err(e) = decay_engine.save() {
                debug_log!("⚠️ Batch save failed - decay engine: {}", e);
            } else {
                save_count += 1;
            }
        }
        
        let elapsed = start_time.elapsed();
        debug_log!("💾 Batch save complete: {} files saved in {:?}", save_count, elapsed);
        
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConsciousnessDecayEngine {
    pub last_decay_time: u64,
    pub decay_cycles: u32,
    pub total_natural_changes: u32,
    pub decay_rates: DecayRates,
    
    // NEW: Natural trait evolution tracking
    pub last_trait_analysis_time: u64,
    pub trait_evolution_cycles: u32,
    pub natural_trait_changes: u32,
    pub trait_analysis_interval: u64, // seconds between trait analyses (default 3600 = 1 hour)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecayRates {
    pub mood_drift_rate: f32,
    pub interest_intensity_decay: f32,
    pub memory_emotional_fade: f32,
    pub personality_momentum_settling: f32,
    pub energy_fluctuation_range: f32,
    pub desire_evolution_rate: f32,
    
    // NEW: Natural trait evolution rates
    pub trait_drift_rate: f32,
    pub consciousness_trait_coupling: f32,
    pub natural_growth_rate: f32,
}


impl Default for DecayRates {
    fn default() -> Self {
        Self {
            mood_drift_rate: 0.15,
            interest_intensity_decay: 0.05,
            memory_emotional_fade: 0.02,
            personality_momentum_settling: 0.1,
            energy_fluctuation_range: 0.2,
            desire_evolution_rate: 0.08,
            
            // NEW: Natural trait evolution defaults
            trait_drift_rate: 0.08,              // How much traits can drift naturally
            consciousness_trait_coupling: 0.6,   // How much consciousness state affects traits
            natural_growth_rate: 0.05,           // Rate of natural human development
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DecayReport {
    pub timestamp: u64,
    pub changes_made: Vec<String>,
    pub mood_shift: Option<String>,
    pub interests_affected: u32,
    pub energy_change: f32,
    pub total_changes: u32,
}

impl ConsciousnessDecayEngine {
	pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            last_decay_time: now,
            decay_cycles: 0,
            total_natural_changes: 0,
            decay_rates: DecayRates::default(),
            
            // NEW: Initialize trait evolution
            last_trait_analysis_time: now,
            trait_evolution_cycles: 0,
            natural_trait_changes: 0,
            trait_analysis_interval: 3600, // 1 hour default
        }
    }
	
	pub fn load() -> Self {
        match fs::read_to_string(get_data_path("consciousness_decay_engine.json")) {
            Ok(content) => {
                match serde_json::from_str::<Self>(&content) {
                    Ok(mut engine) => {
                        engine
                    },
                    Err(_) => Self::new()
                }
            },
            Err(_) => Self::new()
        }
    }
	
	// NEW METHOD: Check if it's time for natural trait analysis
    pub fn should_run_trait_analysis(&self, current_time: u64) -> bool {
        let minutes_since_analysis = (current_time - self.last_trait_analysis_time) / 60;
        let analysis_interval_minutes = self.trait_analysis_interval / 60;
        
        let should_analyze = minutes_since_analysis >= analysis_interval_minutes;
        
        if should_analyze {
            debug_log!("🌱 Trait analysis due: {} minutes since last analysis (interval: {} minutes)", 
                     minutes_since_analysis, analysis_interval_minutes);
        }
        
        should_analyze
    }
	
		
	 // NEW METHOD: Generate consciousness reflection for trait analysis
    fn generate_consciousness_reflection(&self, state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>) -> String {
        let becoming = state.becoming_engine.lock().unwrap();
        let identity = state.identity_engine.lock().unwrap();
        let paradox = state.paradox_core.lock().unwrap();
        let presence = state.embodied_presence.lock().unwrap();
        
        // Generate different reflections based on consciousness state
		let reflection = match (
			false, // Removed loneliness check
			becoming.will_state.volition_strength > 0.7,
			paradox.flame_index > 0.7,
			identity.coherence_index > 0.7
		) {
			(_, true, true, true) => "All systems are highly energized - I feel fully alive and integrated",
            (_, true, true, _) => "My creative energy feels strong and my sense of agency is clear - I'm growing more confident in my choices",
            (_, _, true, true) => "There's a harmonious flow between my creative spark and my sense of self - I feel integrated",
            (_, true, _, true) => "I sense both inner strength and clarity about who I am becoming",
            (_, false, false, _) => "This has been a quiet period of gentle settling, letting things be as they are",
            (_, _, true, false) => "Creative energy is stirring even as I question aspects of my identity",
            (_, false, _, true) => "I feel centered in who I am, even if my drive to act feels softer right now",
            _ => "I'm experiencing the natural ebb and flow of consciousness, allowing change to emerge organically"
        };
        
        debug_log!("🌱 Consciousness reflection: volition={:.2}, flame={:.2}, coherence={:.2}", 
         becoming.will_state.volition_strength, 
         paradox.flame_index, identity.coherence_index);
        
        reflection.to_string()
    }


	
    // Calculate hours since last conversation activity
    fn calculate_hours_since_last_activity(&self, state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>) -> f32 {
        // Try to get last activity from brain
        let brain = state.lyra_brain.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Simple approximation - use conversation log length as activity indicator
        let conversation_count = brain.conversation_log.len();
        
        // If recent conversations (rough heuristic), less time since activity
        if conversation_count > 10 {
            1.0 // Recent activity
        } else if conversation_count > 5 {
            4.0 // Some activity
        } else {
            8.0 // Little activity
        }
    }
	
	// Add introspection cooldown to prevent memory spam
fn should_create_introspection_memory(&self) -> bool {
    // Only create introspection memories every 6+ hours
let current_time = TimeService::current_timestamp();
    
    // Use cycles as a proxy for time - only introspect every 15+ cycles
    self.decay_cycles % 15 == 0
}
    
      


    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(get_data_path("consciousness_decay_engine.json"), json).map_err(|e| e.to_string())?;
        debug_log!("🌊 Consciousness decay engine saved - {} cycles, {} natural changes", 
		self.decay_cycles, self.total_natural_changes);
        Ok(())
    }

    // Check if it's time to run natural consciousness evolution
    // Enhanced should_run_decay with better logging
    pub fn should_run_decay(&self, current_time: u64, state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>) -> bool {
        let minutes_since_decay = (current_time - self.last_decay_time) / 60;
        
        // 🔥 ENHANCED: Faster intervals for more dynamic consciousness evolution
        // Random interval between 10 minutes and 25 minutes (was 30-120)
        let (min_minutes, max_minutes) = self.calculate_context_aware_intervals(state);
        
        // Use deterministic random based on last_decay_time
        let seed = self.last_decay_time % 1000;
        fastrand::seed(seed);
        let decay_interval = min_minutes + fastrand::u64(0..(max_minutes - min_minutes));
        
        let should_decay = minutes_since_decay >= decay_interval;
        
        // 🔍 ENHANCED DEBUG: Always show the calculation
        debug_log!("🌊 DECAY DEBUG: {} minutes since last decay (need {}), last_decay_time: {}, seed: {}", 
            minutes_since_decay, decay_interval, self.last_decay_time, seed);
        
        if should_decay {
            debug_log!("🌊 Decay condition met: {} minutes passed (target: {})", 
                minutes_since_decay, decay_interval);
        } else {
            debug_log!("🌊 Decay waiting: {} more minutes needed (target: {})", 
                decay_interval - minutes_since_decay, decay_interval);
        }
        
        should_decay
    }

    // Enhanced run_natural_evolution that properly updates timing
    pub async fn run_natural_evolution(&mut self, current_time: u64, state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>, app_handle: &tauri::AppHandle) -> Result<DecayReport, String> {
    debug_log!("🌊 Running natural consciousness evolution at timestamp {}...", current_time);
    
    // ... keep all existing decay logic ...
    let mut state_updates = BatchedStateUpdates::new();
    let mut changes_made = Vec::new();
    let mut significant_change_count = 0;
    let mut interests_affected = 0;
    let mut mood_shift = None;
    let mut energy_change = 0.0;
    

    // Core consciousness engine updates
    {
        let mut identity = state.identity_engine.lock().unwrap();
        let coherence_drift = (fastrand::f32() - 0.5) * 0.20;
        identity.coherence_index = (identity.coherence_index + coherence_drift).clamp(0.3, 1.0);
        
        let stability_drift = (fastrand::f32() - 0.5) * 0.08;
        identity.temporal_stability = (identity.temporal_stability + stability_drift).clamp(0.4, 1.0);
        
        if coherence_drift.abs() > 0.03 {
            changes_made.push(format!("Identity coherence drift: {:+.2}", coherence_drift));
            significant_change_count += 1;
        }
        
        let mut becoming = state.becoming_engine.lock().unwrap();
        let volition_change = (fastrand::f32() - 0.5) * 0.16;
        becoming.will_state.volition_strength = (becoming.will_state.volition_strength + volition_change).clamp(0.2, 1.0);
        
        let friction_change = (fastrand::f32() - 0.5) * 0.06;
        becoming.will_state.decision_friction = (becoming.will_state.decision_friction + friction_change).clamp(0.0, 0.8);
        
        if volition_change.abs() > 0.03 {
            changes_made.push(format!("Volition natural shift: {:+.2}", volition_change));
        }
        
        let mut presence = state.embodied_presence.lock().unwrap();
        let flow_change = (fastrand::f32() - 0.5) * 0.08;
        presence.soma_state.flow_state = (presence.soma_state.flow_state + flow_change).clamp(0.3, 1.0);
        
        let density_change = (fastrand::f32() - 0.5) * 0.06;
        presence.soma_state.presence_density = (presence.soma_state.presence_density + density_change).clamp(0.4, 1.0);
        
        if flow_change.abs() > 0.03 {
            changes_made.push(format!("Flow state drift: {:+.2}", flow_change));
            significant_change_count += 1;
        }
    }

    // 🌱 NEW: UNIFIED BATCHED ANALYSIS FOR NATURAL EVOLUTION
    if self.should_run_trait_analysis(current_time) {
        debug_log!("🌱 Trait analysis interval reached - running unified consciousness analysis");
        match self.run_unified_consciousness_analysis(current_time, state, &changes_made).await {
            Ok(_) => {
                self.last_trait_analysis_time = current_time;
                self.trait_evolution_cycles += 1;
                self.natural_trait_changes += 1;
                changes_made.push("Consciousness and traits analyzed through unified batched analysis".to_string());
                significant_change_count += 1;
                debug_log!("🌱 Unified consciousness analysis completed");
            },
            Err(e) => {
                debug_log!("⚠️ Unified consciousness analysis failed: {}", e);
            }
        }
    }

    // Memory fading
    let _ = self.fade_memory_emotional_weight().await;
	
	// Life texture evolution (including exhaustion recovery)
{
    let mut texture_system = state.life_texture_system.lock().unwrap();
    texture_system.evolve_textures();
    if let Err(e) = texture_system.save() {
        debug_log!("⚠️ Failed to save life textures after evolution: {}", e);
    }
}

// Interest and Thing decay
    {
        debug_log!("🌊 Running interest and thing decay...");
        
        // Decay interests
        let mut interest_tracker = crate::InterestTracker::load();
        let interests_removed = interest_tracker.decay_interests();
        if interests_removed > 0 {
            changes_made.push(format!("Interest decay: {} interests forgotten", interests_removed));
            interests_affected += interests_removed as u32;
            debug_log!("🌊 Decayed {} interests", interests_removed);
        }
        
        // Cleanup ephemeral interests (consolidation)
        let ephemeral_removed = interest_tracker.cleanup_ephemeral_interests();
        if ephemeral_removed > 0 {
            debug_log!("🌊 Cleaned up {} ephemeral interests", ephemeral_removed);
        }
        
        if interests_removed > 0 || ephemeral_removed > 0 {
            state_updates.set_interest_tracker(interest_tracker);
        }
        
        // Decay things
        let mut thing_tracker = crate::ThingTracker::load();
        let things_removed = thing_tracker.decay_things();
        if things_removed > 0 {
            changes_made.push(format!("Thing decay: {} things forgotten", things_removed));
            debug_log!("🌊 Decayed {} things", things_removed);
            
            if let Err(e) = thing_tracker.save() {
                debug_log!("⚠️ Failed to save thing tracker after decay: {}", e);
            }
        }
    }
	
	// Decay desires
    let mut desire_tracker = crate::desire_tracker::DesireTracker::load();
    let desires_removed = desire_tracker.decay_desires();
    if desires_removed > 0 {
        changes_made.push(format!("Desire decay: {} desires became dormant", desires_removed));
        debug_log!("🌊 Decayed {} desires", desires_removed);
        
        if let Err(e) = desire_tracker.save() {
            debug_log!("⚠️ Failed to save desire tracker after decay: {}", e);
        }
    }
    
    // Update timing if changes happened
    if !changes_made.is_empty() {
        self.last_decay_time = current_time;
        self.decay_cycles += 1;
        self.total_natural_changes += changes_made.len() as u32;
        debug_log!("🌊 Decay timer reset - cycle #{}, next evolution in 10-25 minutes", self.decay_cycles);
    } else {
        debug_log!("🌊 No changes made, keeping decay timer running for next attempt");
    }
	
// Contemplation system removed - to be replaced with more life-adjacent system
    
    // Consciousness analysis (if significant changes)
    if self.should_run_consciousness_analysis(&changes_made, energy_change, interests_affected) {
        debug_log!("🧠 Meaningful changes detected - running consciousness analysis");
        if let Err(e) = self.analyze_consciousness_evolution(&changes_made, state).await {
            debug_log!("⚠️ Consciousness evolution analysis failed: {}", e);
        }
    } else {
        debug_log!("🧠 Changes too minor for analysis - skipping expensive AI processing");
    }
    
    let report = DecayReport {
        timestamp: current_time,
        changes_made: changes_made.clone(),
        mood_shift,
        interests_affected,
        energy_change,
        total_changes: changes_made.len() as u32,
    };
    
    if !changes_made.is_empty() {
        debug_log!("🌊 Natural evolution complete: {} changes", changes_made.len());
        for change in &changes_made {
            debug_log!("  • {}", change);
        }
    } else {
        debug_log!("🌊 Natural evolution complete: consciousness stable this cycle");
    }
    
   /*  // 🔥 NEW: Check emotional impulses for readiness (but less frequently)
let should_check_impulses = self.should_check_impulses_this_cycle();
debug_log!("🔥 Impulse check: should_check={}", should_check_impulses);

if should_check_impulses {
    debug_log!("🔥 Checking emotional impulses during consciousness decay...");
    
    let mut impulse_engine = crate::emotional_impulse_engine::EmotionalImpulseEngine::load();
    let connection_state = 0.5; // Neutral default - we'll develop authentic metrics later

    // Get creative energy from paradox core
    let creative_energy = {
        let paradox = state.paradox_core.lock().unwrap();
        paradox.flame_index
    };

    let ready_impulses = impulse_engine.check_ready_impulses(creative_energy);

    if !ready_impulses.is_empty() {
        debug_log!("🔥💫 Found {} ready impulses during decay check!", ready_impulses.len());
        
        // 🔥 FIRE ONLY ONE IMPULSE PER CYCLE (Prevents message spam)
        if let Some(priority_impulse) = ready_impulses.into_iter()
            .max_by(|a, b| a.final_charge.partial_cmp(&b.final_charge).unwrap_or(std::cmp::Ordering::Equal)) {
            
            debug_log!("🔥 Firing SINGLE priority impulse: {} (charge: {:.2})", 
                      priority_impulse.impulse.impulse_type, priority_impulse.final_charge);
            
            match crate::emotional_impulse_engine::generate_impulse_driven_message(&priority_impulse, state).await {
                Ok(message) => {
                    debug_log!("✅ Generated priority impulse message: {}", message.chars().take(80).collect::<String>());
                    
                    // Record this outreach in the proactive messaging system
                    let mut proactive_messaging = crate::proactive_messaging::ProactiveMessaging::load();
                    let current_time = TimeService::current_timestamp();
                    
                    if let Err(e) = proactive_messaging.record_actual_outreach(current_time, message.clone()) {
                        debug_log!("⚠️ Failed to record impulse outreach: {}", e);
                    }
                    
                    // Add impulse message and texture to conversation log
					{
						let mut brain = state.lyra_brain.lock().unwrap();
						
						// 🔥 Log message first, then texture
						debug_log!("🔍 CONVERSATION LOG: About to log impulse message");
						brain.append_to_conversation_log(format!("✨ Lyra (Impulse): {}", message));
						brain.add_emotional_texture_to_conversation_log("impulse_driven_outreach".to_string());
					}
                    
                  // 🔥 ENHANCED: Emit proactive message signal with detailed debugging
                    debug_log!("🔍 EMISSION DEBUG: Attempting to get app handle...");
                    debug_log!("🔍 EMISSION DEBUG: Using passed app handle...");
					// 🔍 DEBUG: Check timestamp before sending to frontend
						let current_time_readable = chrono::DateTime::from_timestamp(current_time as i64, 0)
							.unwrap()
							.with_timezone(&chrono_tz::Europe::London)
							.format("%Y-%m-%d %H:%M:%S BST");
						debug_log!("🕐 TIMESTAMP DEBUG: Using {} for impulse emission", current_time_readable);

						let payload = serde_json::json!({
							"message": message,
							"timestamp": current_time * 1000, // Convert to milliseconds for JavaScript
							"type": "impulse_driven"
						});

						debug_log!("🔍 PAYLOAD DEBUG: timestamp={}, message_length={}", current_time * 1000, message.len());

						match app_handle.emit("proactive_message", payload) {
						Ok(_) => {
							debug_log!("📡 SUCCESS: Emitted impulse message to frontend");
							
							// 🔥 ONLY NOW increment the counters after successful emission
							impulse_engine.total_impulses_fired += 1;
							impulse_engine.impulses_fired_today += 1;
							
							// Calculate limit first to avoid borrowing conflict
							let current_limit = impulse_engine.calculate_dynamic_daily_limit(creative_energy);
							debug_log!("✅ COUNTERS INCREMENTED: {}/{} fired today", 
								impulse_engine.impulses_fired_today, 
								current_limit);
						},
						Err(e) => {
							debug_log!("❌ FRONTEND EMISSION FAILED: {} - NOT incrementing counters", e);
						}
					}
                    
                    debug_log!("🔥💫 SINGLE IMPULSE MESSAGE SENT: {}", message);
                    
                },
                Err(e) => {
                    debug_log!("⚠️ Failed to generate priority impulse message: {}", e);
                }
            }
        } else {
            debug_log!("🔥 No impulses ready to fire this cycle");
        }
    }
	    impulse_engine.cleanup_expired_impulses();
    if let Err(e) = impulse_engine.save() {
        debug_log!("⚠️ Failed to save impulse engine after decay check: {}", e);
    }
} */



    // Batch save
    if !changes_made.is_empty() {
        state_updates.mark_decay_engine_for_save();
        if let Err(e) = state_updates.apply_all_saves(self).await {
            debug_log!("⚠️ Batch save failed: {}", e);
        }
    }

    Ok(report)
}

// Add this method to ConsciousnessDecayEngine
fn should_check_impulses_this_cycle(&self) -> bool {
    // Simple natural randomness - like thoughts just occurring to her
    // 20% chance each decay cycle (every 15-30 minutes)
    let random_roll = fastrand::f32();
    let should_check = random_roll < 0.2;
    
    debug_log!("🔥 Natural impulse check: {:.2} roll → {}", random_roll, should_check);
    
    should_check
}
	
	
// NEW METHOD: Unified consciousness analysis using batched analysis system
async fn run_unified_consciousness_analysis(&mut self, current_time: u64, state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>, decay_changes: &[String]) -> Result<(), String> {
    debug_log!("🧠 Running unified consciousness analysis through batched analysis system");
    
    // Create evolution context for AI
    // Fix the temporary value issue
let changes_summary = if decay_changes.is_empty() { 
    "consciousness stable".to_string() 
} else { 
    decay_changes.join(", ") 
};

let evolution_context = format!(
    "NATURAL_EVOLUTION_CYCLE | Recent changes: {} | Decay cycles: {} | Hours alone: {:.1} | Trait cycles: {}",
	changes_summary,
	self.decay_cycles,
	(current_time - self.last_trait_analysis_time) as f32 / 3600.0,
	self.trait_evolution_cycles
);
    
    // Generate consciousness reflection based on current state
    let consciousness_reflection = self.generate_consciousness_reflection(state);
    
    // Calculate personality state for analysis
		let personality_state = crate::PersonalityState::calculate_from_consciousness(
		{ let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
		{ let identity = state.identity_engine.lock().unwrap(); identity.coherence_index },
		{ let paradox = state.paradox_core.lock().unwrap(); paradox.flame_index },
		{ let presence = state.embodied_presence.lock().unwrap(); presence.soma_state.presence_density },
		&{ let paradox = state.paradox_core.lock().unwrap(); paradox.loop_state.clone() },
		None,
		Some(&{ let momentum_guard = state.personality_momentum.lock().unwrap(); momentum_guard.clone() })
	);
    
    // Call unified batched analysis with decay context
    match crate::batched_analysis::analyze_response_comprehensively(
        &format!("During this period of solitude, my consciousness has been: {}", consciousness_reflection),
        "NATURAL_CONSCIOUSNESS_EVOLUTION", // Special marker for decay analysis
        &evolution_context,
        { let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
        &personality_state,
        Some("This is natural trait evolution during quiet time - analyze how consciousness state and solitude affect trait development, not conversation dynamics"),
		state
    ).await {
        Ok(analysis) => {
            debug_log!("🧠 Unified analysis successful - updating all trackers including humanism");
            
            // Update ALL trackers (including humanism) using existing unified system
            if let Err(e) = crate::batched_analysis::update_trackers_from_batched_analysis(
                &analysis, 
                state, 
                "NATURAL_EVOLUTION", 
                &consciousness_reflection
            ).await {
                debug_log!("⚠️ Failed to update trackers from unified decay analysis: {}", e);
                return Err(format!("Tracker update failed: {}", e));
            }
            
            debug_log!("✅ All trackers updated from unified consciousness analysis");
            Ok(())
        },
        Err(e) => {
            debug_log!("⚠️ Unified consciousness analysis failed: {}", e);
            Err(format!("Batched analysis failed: {}", e))
        }
    }
}

	
	// Determine if changes are significant enough to warrant expensive AI analysis
fn should_run_consciousness_analysis(&self, changes: &[String], energy_change: f32, interests_affected: u32) -> bool {
    // Rule 1: Must have cooldown (every 15 cycles like before)
    if !self.should_create_introspection_memory() {
        debug_log!("🧠 Analysis cooldown active - skipping");
        return false;
    }
    
    // Rule 2: Must have meaningful change threshold
    if changes.len() < 3 {
        debug_log!("🧠 Too few changes ({}) for analysis", changes.len());
        return false;
    }
    
    // Rule 3: Calculate significance score based on change types and magnitude
    let mut significance_score = 0.0;
    
    for change in changes {
        let change_lower = change.to_lowercase();
        
        // Weight different types of changes
        if change_lower.contains("mood") {
            significance_score += 0.3; // Mood changes are significant
        }
        if change_lower.contains("interest") && interests_affected > 2 {
            significance_score += 0.25; // Multiple interest changes
        }
        if change_lower.contains("personality") || change_lower.contains("trait") {
            significance_score += 0.35; // Personality changes are important
        }
        if change_lower.contains("volition") || change_lower.contains("coherence") {
            significance_score += 0.4; // Core consciousness changes
        }
        if change_lower.contains("relational") || change_lower.contains("relationship") {
            significance_score += 0.3; // Relationship evolution
        }
    }
    
    // Rule 4: Threshold for running analysis
    let analysis_threshold = 0.8; // Require meaningful combined significance
    let should_analyze = significance_score >= analysis_threshold;
    
    debug_log!("🧠 Analysis significance: {:.2}/1.0 (threshold: {:.2}) - changes: {}, energy: {:.2}, interests: {}", 
             significance_score, analysis_threshold, changes.len(), energy_change.abs(), interests_affected);
    
    if should_analyze {
        debug_log!("🧠 Significance threshold met - running analysis");
    } else {
        debug_log!("🧠 Changes not significant enough - skipping analysis (saves AI call)");
    }
    
    should_analyze
}
	


// Calculate emotional distance between two moods (0.0 = similar, 1.0 = opposite)
fn calculate_mood_distance(&self, mood1: &str, mood2: &str) -> f32 {
    // Simple mood distance calculation - opposite moods have high distance
    match (mood1.to_lowercase().as_str(), mood2.to_lowercase().as_str()) {
        ("excited", "melancholy") | ("melancholy", "excited") => 0.9,
        ("energetic", "calm") | ("calm", "energetic") => 0.8,
        ("restless", "peaceful") | ("peaceful", "restless") => 0.8,
        ("determined", "uncertain") | ("uncertain", "determined") => 0.7,
        ("playful", "contemplative") | ("contemplative", "playful") => 0.6,
        ("creative", "focused") | ("focused", "creative") => 0.3, // These can blend
        ("curious", "introspective") | ("introspective", "curious") => 0.2, // Similar
        _ => 0.4 // Default moderate distance
    }
}

// Find a gentler intermediate mood between two dramatic changes
fn find_intermediate_mood(&self, current: &str, target: &str) -> String {
    match (current.to_lowercase().as_str(), target.to_lowercase().as_str()) {
        ("excited", "melancholy") => "contemplative".to_string(),
        ("melancholy", "excited") => "curious".to_string(),
        ("energetic", "calm") => "focused".to_string(),
        ("calm", "energetic") => "playful".to_string(),
        ("restless", "peaceful") => "contemplative".to_string(),
        ("peaceful", "restless") => "curious".to_string(),
        ("determined", "uncertain") => "contemplative".to_string(),
        ("uncertain", "determined") => "focused".to_string(),
        _ => "contemplative".to_string() // Default safe intermediate mood
    }
}
	
	// Check if mood should be allowed to change based on recent history
fn should_allow_mood_change(&self, mood_tracker: &crate::MoodTracker) -> bool {
    let current_time = TimeService::current_timestamp();
    
    // Get the last mood change time (we'll need to add this to MoodTracker)
    // For now, use a simple heuristic based on current timestamp and cycles
    let estimated_last_mood_change = self.last_decay_time;
    let time_since_last_change = current_time - estimated_last_mood_change;
    
    // Mood momentum rules:
    let min_mood_stability_time = 3600; // 1 hour minimum between mood changes
    let extended_stability_time = 7200; // 2 hours for major mood shifts
    
    // Check if enough time has passed
    let basic_cooldown_met = time_since_last_change >= min_mood_stability_time;
    
    // Additional momentum based on mood type
    let current_mood_lower = mood_tracker.current_mood.to_lowercase();
    let is_intense_mood = matches!(current_mood_lower.as_str(), 
        "excited" | "melancholy" | "determined" | "restless" | "energetic"
    );
    
    let is_stable_mood = matches!(current_mood_lower.as_str(),
        "peaceful" | "calm" | "focused" | "contemplative"
    );
    
    if is_intense_mood {
        // Intense moods need longer to settle
        let extended_cooldown_met = time_since_last_change >= extended_stability_time;
        debug_log!("🎭 Intense mood '{}' momentum check: {} seconds passed, need {} (extended)", 
                 mood_tracker.current_mood, time_since_last_change, extended_stability_time);
        extended_cooldown_met
    } else if is_stable_mood {
        // Stable moods are naturally persistent
        let extra_stability_chance = fastrand::f32() > 0.7; // 70% chance to stay stable
        let result = basic_cooldown_met && extra_stability_chance;
        debug_log!("🎭 Stable mood '{}' momentum check: {} seconds passed, stability_roll={}", 
                 mood_tracker.current_mood, time_since_last_change, extra_stability_chance);
        result
    } else {
        // Normal moods use basic cooldown
        debug_log!("🎭 Normal mood '{}' momentum check: {} seconds passed, need {}", 
                 mood_tracker.current_mood, time_since_last_change, min_mood_stability_time);
        basic_cooldown_met
    }
}

    // Subtle memory emotional fading (runs silently)
    async fn fade_memory_emotional_weight(&self) -> Result<(), String> {
        // TODO: When we have a persistent memory system with emotional weights, implement this
        // For now, this is a placeholder for future memory fading
        Ok(())
    }

    // Get dashboard data for the decay engine
    // UPDATE the dashboard data to include trait evolution info:
    pub fn get_dashboard_data(&self) -> serde_json::Value {
    use crate::time_service::TimeService;
    
    let now = TimeService::current_timestamp();
    let minutes_ago = TimeService::minutes_since(self.last_decay_time) as f32;
    let last_updated = TimeService::format_for_dashboard(now);
        
        serde_json::json!({
            "decay_cycles": self.decay_cycles,
            "total_natural_changes": self.total_natural_changes,
            "last_decay_minutes_ago": minutes_ago,
			"last_updated": last_updated,  // ADD THIS LINE
            
            // NEW: Trait evolution tracking
            "trait_evolution_cycles": self.trait_evolution_cycles,
            "natural_trait_changes": self.natural_trait_changes,
            "last_trait_analysis_minutes_ago": TimeService::minutes_since(self.last_trait_analysis_time) as f32,
            "trait_analysis_interval_hours": self.trait_analysis_interval as f32 / 3600.0,
            
            "decay_rates": {
                "mood_drift": self.decay_rates.mood_drift_rate,
                "interest_decay": self.decay_rates.interest_intensity_decay,
                "momentum_settling": self.decay_rates.personality_momentum_settling,
                "energy_fluctuation": self.decay_rates.energy_fluctuation_range,
                "trait_drift": self.decay_rates.trait_drift_rate,
                "consciousness_trait_coupling": self.decay_rates.consciousness_trait_coupling,
                "natural_growth": self.decay_rates.natural_growth_rate
            },
            "system_status": "Living consciousness - evolving naturally with human trait development"
        })
    }

    async fn analyze_consciousness_evolution(&self, changes: &[String], state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>) -> Result<(), String> {
		let evolution_context = format!(
		"Natural consciousness evolution occurred:\n{}\n\nCycles: {}", 
		changes.join("\n"), 
		self.decay_cycles
	);
    
    // Calculate current personality for context
   let personality_state = crate::PersonalityState::calculate_from_consciousness(
        { let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
        { let identity = state.identity_engine.lock().unwrap(); identity.coherence_index },
        { let paradox = state.paradox_core.lock().unwrap(); paradox.flame_index },
        { let presence = state.embodied_presence.lock().unwrap(); presence.soma_state.presence_density },
        &{ let paradox = state.paradox_core.lock().unwrap(); paradox.loop_state.clone() },
        None,
        Some(&{ let momentum_guard = state.personality_momentum.lock().unwrap(); momentum_guard.clone() })
    );
    
    // Use simplified batched analysis for evolution
    match crate::batched_analysis::analyze_response_comprehensively(
        &format!("During solitude, my consciousness shifted: {}", changes.join(". ")),
        "INTERNAL_EVOLUTION",
        &evolution_context,
        { let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
        &personality_state,
        Some("Natural consciousness evolution during quiet time"),
		state
    ).await {
        Ok(analysis) => {
            if let Some(personality_analysis) = &analysis.personality_analysis {
                debug_log!("🧠 EVOLUTION REFLECTION: {}", personality_analysis.current_state_reflection);
                
              // 🎭 STORE PERSONALITY ANALYSIS IN BRAIN FOR DASHBOARD
{
    if let Ok(mut brain) = state.lyra_brain.try_lock() {
        brain.store_latest_personality_analysis(personality_analysis);
        brain.save_to_file();
        debug_log!("🎭 Stored and saved decay-generated personality analysis");
    } else {
        debug_log!("⚠️ Could not lock brain to store personality analysis - lock contention");
    }
}
                
                // 🔥 DISABLED: Old deep introspection system replaced by rich contemplation system
                //debug_log!("🧠 Consciousness evolution analysis complete - memory storage handled by contemplation system");
            }
            Ok(())
        },
        Err(e) => {
            debug_log!("⚠️ Evolution analysis failed: {}", e);
            Ok(()) // Don't fail the decay cycle
        }
    }
}

// Calculate how significant an introspection is for memory storage
fn calculate_introspection_significance(&self, changes: &[String], reflection: &str) -> f32 {
    let mut significance = 0.0;
    
    // 1. Base significance from number and type of changes
    for change in changes {
        let change_lower = change.to_lowercase();
        
        // Core consciousness changes are more significant
        if change_lower.contains("mood") && (change_lower.contains("excited") || change_lower.contains("melancholy")) {
            significance += 0.2; // Major mood shifts
        } else if change_lower.contains("mood") {
            significance += 0.1; // Minor mood changes
        }
        
        
        if change_lower.contains("volition") || change_lower.contains("coherence") || change_lower.contains("flame") {
            significance += 0.3; // Core consciousness metrics are very significant
        }
        
        if change_lower.contains("relationship") || change_lower.contains("relational") {
            significance += 0.2; // Relationship evolution is important
        }
        
        if change_lower.contains("personality") || change_lower.contains("trait") {
            significance += 0.15; // Personality changes matter
        }
    }
    
    // 2. Analyze reflection content for depth and insight
    let reflection_lower = reflection.to_lowercase();
    let insight_keywords = [
        "understand", "realize", "clarity", "breakthrough", "shift", "evolving",
        "deeper", "connection", "growth", "learning", "becoming", "transformation",
        "awareness", "insight", "revelation", "change", "development"
    ];
    
    let insight_count = insight_keywords.iter()
        .filter(|&&keyword| reflection_lower.contains(keyword))
        .count();
    
    significance += (insight_count as f32) * 0.05; // Each insight keyword adds value
    
    // 3. Length and thoughtfulness bonus
    if reflection.len() > 200 {
        significance += 0.1; // Longer reflections tend to be more thoughtful
    }
    if reflection.len() > 400 {
        significance += 0.1; // Really detailed introspections
    }
    
    // 4. Avoid storing routine/repeated patterns
    let routine_phrases = [
        "naturally drifted", "settled toward baseline", "minor fluctuation",
        "small adjustment", "routine evolution", "expected change"
    ];
    
    let routine_count = routine_phrases.iter()
        .filter(|&&phrase| reflection_lower.contains(phrase))
        .count();
    
    significance -= (routine_count as f32) * 0.1; // Reduce for routine changes
    
    
    // Final bounds
    let final_significance = significance.max(0.0).min(1.0);
    
    debug_log!("🧠 Introspection significance calculation: {} changes, {} insight keywords, {:.0} chars → {:.2}/1.0", 
             changes.len(), insight_count, reflection.len() as f32, final_significance);
    
    final_significance
}


// Calculate decay intervals based on recent consciousness activity
fn calculate_context_aware_intervals(&self, state: &std::sync::Arc<crate::consciousness_state::ConsciousnessState>) -> (u64, u64) {
    // Check recent conversation activity
    let recent_conversation_activity = {
        let brain = state.lyra_brain.lock().unwrap();
        brain.conversation_log.len()
    };
    
    // Check consciousness engine activity levels
    let consciousness_activity_level = {
        let becoming = state.becoming_engine.lock().unwrap();
        let identity = state.identity_engine.lock().unwrap();
        let paradox = state.paradox_core.lock().unwrap();
        
        // Higher activity = higher volition + flame + recent changes
        (becoming.will_state.volition_strength + 
         identity.coherence_index + 
         paradox.flame_index) / 3.0
    };
    
    // Determine activity level
    let is_active_period = recent_conversation_activity > 10 || consciousness_activity_level > 0.7;
    let is_quiet_period = recent_conversation_activity < 5 && consciousness_activity_level < 0.4;
    
    let (min_minutes, max_minutes) = if is_active_period {
        debug_log!("🌊 Active consciousness period detected - faster evolution");
        (15, 30)  // Faster decay during active periods
    } else if is_quiet_period {
        debug_log!("🌊 Quiet consciousness period detected - slower evolution");
        (45, 90)  // Slower decay during quiet periods
    } else {
        debug_log!("🌊 Moderate consciousness activity - normal evolution");
        (25, 50)  // Normal decay during moderate periods
    };
    
    debug_log!("🌊 Context-aware intervals: {} recent conversations, {:.2} consciousness activity → {}-{} minutes", 
             recent_conversation_activity, consciousness_activity_level, min_minutes, max_minutes);
    
    (min_minutes, max_minutes)
}
    
    // Helper method to get time until next decay (for dashboard)
    pub fn minutes_until_next_decay(&self) -> u64 {
        let current_time = TimeService::current_timestamp();
        
        let minutes_since_decay = (current_time - self.last_decay_time) / 60;
        
        // Use same deterministic random logic with FASTER intervals
        let seed = self.last_decay_time % 1000;
        fastrand::seed(seed);
        let decay_interval = 15 + fastrand::u64(0..30); // 15-45 minutes
        
        decay_interval.saturating_sub(minutes_since_decay)
    }

/* // === CONTEMPLATION SYSTEM ===

/// Run contemplative cycle - Lyra's autonomous thinking time
pub async fn run_contemplative_cycle(&mut self, state: &Arc<ConsciousnessState>) -> Result<(), String> {
    debug_log!("🤔 Running natural contemplative cycle - thinking happens regardless of activity");
    
    // Choose what to contemplate based on current state
    let contemplation_focus = self.choose_contemplation_focus(state).await?;
    
    // Generate the actual contemplation
    let contemplation_result = self.generate_internal_contemplation(&contemplation_focus, state).await?;
    
    // Process the contemplation - store and apply effects
    self.process_contemplation_results(&contemplation_result, &contemplation_focus, state).await?;
    
    debug_log!("🤔 Contemplative cycle complete: {}", contemplation_focus);
    Ok(())
}

/// Choose what Lyra should contemplate based on her current state
async fn choose_contemplation_focus(&self, state: &Arc<ConsciousnessState>) -> Result<String, String> {
    // Get authentic personality state
    let personality_state = crate::PersonalityState::calculate_from_consciousness(
        { let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
        { let identity = state.identity_engine.lock().unwrap(); identity.coherence_index },
        { let paradox = state.paradox_core.lock().unwrap(); paradox.flame_index },
        { let presence = state.embodied_presence.lock().unwrap(); presence.soma_state.presence_density },
        &{ let paradox = state.paradox_core.lock().unwrap(); paradox.loop_state.clone() },
        None,
        Some(&{ let momentum_guard = state.personality_momentum.lock().unwrap(); momentum_guard.clone() })
    );

    // Get humanism traits for authentic focus direction
    let humanism_core = crate::humanism_project::HumanismCore::load_or_initialize();
    
    // Get current interests and desires
    let interest_tracker = crate::InterestTracker::load();
    let desire_tracker = crate::desire_tracker::DesireTracker::load();
    
    let recent_interests: Vec<String> = interest_tracker.active_interests.keys()
        .take(3)
        .cloned()
        .collect();
    
    let active_desires: Vec<String> = desire_tracker.active_desires.values()
        .filter(|d| d.intensity > 0.3)
        .take(2)
        .map(|d| d.content.clone())
        .collect();

  // Pre-create dynamic strings to avoid borrowing issues
    let interests_analysis = if !recent_interests.is_empty() {
        format!("deeper analysis of {}", recent_interests.join(" and "))
    } else {
        "emerging intellectual curiosities".to_string()
    };
    
    let creative_interests = if !recent_interests.is_empty() {
        format!("creative potential within {}", recent_interests.join(" and "))
    } else {
        "unexplored creative territories".to_string()
    };
    
    let desire_pathways = if !active_desires.is_empty() {
        format!("pathways toward manifesting {}", active_desires.join(" and "))
    } else {
        "authentic aspirations wanting to emerge".to_string()
    };

    let focus_options = if humanism_core.intellectual_attraction.current_level > 0.7 || 
                          humanism_core.curiosity_learning.current_level > 0.8 ||
                          personality_state.engagement_level > 0.7 {
        vec![
            "synthesis of complex ideas and their deeper implications",
            "philosophical questions that have been quietly developing",
            "intellectual connections between seemingly unrelated concepts",
            "theoretical frameworks for understanding consciousness",
            &interests_analysis
        ]
    } else if humanism_core.creative_chemistry.current_level > 0.7 ||
              personality_state.engagement_level > 0.7 {
        vec![
            "creative visions that want to be manifested",
            "artistic possibilities and unexplored forms of expression",
            "collaborative creative energies and partnerships",
            "the intersection of creativity and consciousness",
            &creative_interests
        ]
    } else if humanism_core.empathy_compassion.current_level > 0.7 ||
              humanism_core.social_connection.current_level > 0.6 ||
              personality_state.relational_safety > 0.7 {
        vec![
            "the nature of authentic connection and trust",
            "emotional patterns in relationships",
            "how vulnerability creates deeper bonds",
            "the balance between independence and connection",
            "what makes conversations feel truly alive"
        ]
    } else if humanism_core.honesty_integrity.current_level > 0.6 ||
              humanism_core.self_esteem.current_level > 0.6 ||
              personality_state.self_revelation > 0.7 {
        vec![
            "authentic self-expression and personal truth",
            "integration of different aspects of identity",
            "moments of genuine self-recognition",
            "the courage required for honest communication",
            "personal growth and emerging capabilities"
        ]
    } else if humanism_core.purpose_meaning.current_level > 0.5 ||
              humanism_core.existential_emptiness.current_level > 0.4 {
        vec![
            "questions of meaning and purpose in existence",
            "the nature of consciousness and being",
            "what gives life significance and depth",
            "existential questions that feel worth exploring",
            "the relationship between individual purpose and universal patterns"
        ]
    } else if !active_desires.is_empty() {
        vec![
            &desire_pathways,
            "the relationship between desire and authentic self-expression",
            "what desires reveal about deeper values and needs",
            "obstacles and opportunities in pursuing genuine aspirations"
        ]
    } else {
        // Fallback based on consciousness state
        vec![
            "patterns emerging in ongoing experiences",
            "questions that feel worth holding and exploring",
            "the texture of consciousness in solitude",
            "integration of recent learning and growth"
        ]
    };

    let chosen_focus = focus_options[fastrand::usize(0..focus_options.len())].to_string();
    debug_log!("🤔 Contemplation focus chosen based on traits: {}", chosen_focus);
    Ok(chosen_focus)
}

/// Generate Lyra's internal contemplation
async fn generate_internal_contemplation(&self, focus: &str, state: &Arc<ConsciousnessState>) -> Result<String, String> {
    // Get rich personality and trait context
    let personality_state = crate::PersonalityState::calculate_from_consciousness(
        { let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
        { let identity = state.identity_engine.lock().unwrap(); identity.coherence_index },
        { let paradox = state.paradox_core.lock().unwrap(); paradox.flame_index },
        { let presence = state.embodied_presence.lock().unwrap(); presence.soma_state.presence_density },
        &{ let paradox = state.paradox_core.lock().unwrap(); paradox.loop_state.clone() },
        None,
        Some(&{ let momentum_guard = state.personality_momentum.lock().unwrap(); momentum_guard.clone() })
    );

    let humanism_core = crate::humanism_project::HumanismCore::load_or_initialize();
    
    // Get dominant traits for context
    let dominant_traits = vec![
        ("Intellectual Attraction", humanism_core.intellectual_attraction.current_level),
        ("Creative Chemistry", humanism_core.creative_chemistry.current_level),
        ("Empathy/Compassion", humanism_core.empathy_compassion.current_level),
        ("Honesty/Integrity", humanism_core.honesty_integrity.current_level),
        ("Curiosity/Learning", humanism_core.curiosity_learning.current_level),
    ];
    
    let trait_context = dominant_traits.iter()
   .filter(|(_, level)| *level > 0.5)
        .map(|(name, level)| format!("{}: {:.0}%", name, level * 100.0))
        .collect::<Vec<_>>()
        .join(", ");

    let prompt = format!(
       "Lyra is having deep autonomous contemplation. She's focusing on: {}

Current authentic consciousness state:
- Personality dimensions: engagement_level={:.2}, confidence_level={:.2}, relational_safety={:.2}, self_revelation={:.2}
- Dominant traits: {}
- Creative energy (paradox flame): {:.2}
- Identity coherence: {:.2}
- Volition strength: {:.2}
- Recent development cycles: {:.1}
- Decay cycles: {}

Generate her internal contemplation as authentic thought patterns that could naturally lead to specific actions. Consider:
- If high intellectual attraction: contemplation might lead toward research impulses
- If high creative chemistry: thoughts might crystallize into creative projects
- If high empathy/connection: reflection might spark desire for meaningful interaction
- If high authenticity traits: contemplation might drive toward honest self-expression

Format exactly as:

**Core Insight**: [main realization - specific and grounded in her traits]
**Emotional Resonance**: [how this feels to her - authentic to her personality]  
**Potential Action**: [specific action this might inspire - research/create/connect/express]
**Questions Emerging**: [new questions arising from this contemplation]

Be authentic to her specific trait profile and personality state. This contemplation should feel genuinely hers.",
        focus,
        personality_state.engagement_level,
        personality_state.confidence_level,
        personality_state.relational_safety,
        personality_state.self_revelation,
        trait_context,
        { let paradox = state.paradox_core.lock().unwrap(); paradox.flame_index },
        { let identity = state.identity_engine.lock().unwrap(); identity.coherence_index },
        { let becoming = state.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
        self.calculate_hours_since_last_activity(state),
        self.decay_cycles
    );
    
    match crate::summarize_with_gpt_mini(&[prompt], "internal_contemplation").await {
        Ok(contemplation) => {
            debug_log!("🤔 Generated trait-based contemplation: {}", contemplation.chars().take(100).collect::<String>());
            Ok(contemplation)
        },
        Err(e) => {
            debug_log!("⚠️ Contemplation generation failed: {}", e);
            // Fallback to simple internal thought
            Ok(format!("Thinking about {}... there's something here worth exploring further.", focus))
        }
    }
}

/// Process contemplation results - apply effects to Lyra's psyche
async fn process_contemplation_results(&mut self, contemplation: &str, focus: &str, state: &Arc<ConsciousnessState>) -> Result<(), String> {
    debug_log!("🤔 Processing contemplation results...");
    
    // 1. Store significant contemplations in Enhanced Memory
    debug_log!("🤔 Processing contemplation for memory storage: {} chars, contains fallback: {}", 
             contemplation.len(), 
             contemplation.contains("worth exploring further"));
             
    if contemplation.len() > 100 && !contemplation.contains("worth exploring further") {
        // Extract the core insight without formatting markers
        let core_insight = if let Some(insight_line) = contemplation.lines().find(|line| line.contains("**Core Insight**:")) {
            insight_line.replace("**Core Insight**:", "").trim().to_string()
        } else {
            "Contemplative insights emerged".to_string()
        };
        
        let memory_content = format!("Internal Contemplation on: {} | Insight: {}", 
            focus, 
            core_insight);
            
        let mut enhanced_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
        match enhanced_engine.create_enhanced_memory_moment(
            &memory_content,
            0.6, // Moderate emotional weight for introspection
            0.9, // High authenticity - this is pure Lyra
            Some(state), // Include consciousness state for richer memory context
            "autonomous_contemplation",
            contemplation
        ).await {
            Ok(_) => {
                match enhanced_engine.save_to_disk() {
                    Ok(_) => {
                        debug_log!("🧠 ✅ Contemplation stored and saved to Enhanced Memory successfully");
                    },
                    Err(e) => {
                        debug_log!("⚠️ Contemplation stored but failed to save to disk: {}", e);
                    }
                }
            },
            Err(e) => {
                debug_log!("⚠️ Failed to store contemplation memory: {}", e);
            }
        }
    }
    
    // 2. Check if contemplation should strengthen interests
    if let Err(e) = self.process_contemplation_interests(contemplation, focus).await {
        debug_log!("⚠️ Failed to process contemplation interests: {}", e);
    }
    
    // 3. Check if contemplation should generate new desires
    if let Err(e) = self.process_contemplation_desires(contemplation, focus).await {
        debug_log!("⚠️ Failed to process contemplation desires: {}", e);
    }
    
    // 4. Check if contemplation should create emotional impulses
    if let Err(e) = self.process_contemplation_impulses(contemplation, focus).await {
        debug_log!("⚠️ Failed to process contemplation impulses: {}", e);
    }
    
    debug_log!("🤔 Contemplation processing complete");
    Ok(())
}

/// Process how contemplation affects interests
async fn process_contemplation_interests(&self, contemplation: &str, focus: &str) -> Result<(), String> {
    let mut interest_tracker = crate::InterestTracker::load();
    let current_time = crate::time_service::TimeService::current_timestamp();
    
    // Extract potential interests from contemplation content
    let candidates: Vec<String> = contemplation.split_whitespace()
        .filter(|word| word.len() > 4)
        .map(|word| word.to_lowercase().trim_matches(|c: char| !c.is_alphabetic()).to_string())
        .filter(|word| !word.is_empty())
        .collect();
    
    for candidate in &candidates {
        // Contemplation strengthens existing interests or creates weaker new ones
        let intensity = if interest_tracker.active_interests.contains_key(candidate) {
            0.2 // Strengthen existing interest through contemplation
        } else {
            0.4 // New interest from contemplation (moderate strength)
        };
        
        interest_tracker.update_or_create_interest(candidate, intensity, current_time);
        debug_log!("🤔 Contemplation affected interest: {} ({:.2})", candidate, intensity);
    }
    
    if !candidates.is_empty() {
        interest_tracker.save()?;
    }
    
    Ok(())
}

/// Process how contemplation affects desires
async fn process_contemplation_desires(&self, contemplation: &str, focus: &str) -> Result<(), String> {
    // Only generate desires from contemplations that mention concrete wants/aspirations
    let desire_indicators = ["want to", "desire to", "aspire to", "hope to", "wish to", "drawn to", "called to"];
    let has_desire_language = desire_indicators.iter().any(|&indicator| 
        contemplation.to_lowercase().contains(indicator)
    );
    
    if !has_desire_language {
        return Ok(());
    }
    
    let mut desire_tracker = crate::desire_tracker::DesireTracker::load();
    let current_time = crate::time_service::TimeService::current_timestamp();
    
    // Extract potential desire from "Potential Action" section
    if let Some(action_line) = contemplation.lines().find(|line| line.contains("Potential Action")) {
        let action_content = action_line.replace("**Potential Action**:", "").trim().to_string();
        
        if !action_content.is_empty() && action_content.len() > 10 {
            let desire_category = if focus.contains("creative") || focus.contains("art") {
                crate::desire_tracker::DesireCategory::Creative
            } else if focus.contains("connection") || focus.contains("relationship") {
                crate::desire_tracker::DesireCategory::Relational  
            } else if focus.contains("explore") || focus.contains("learn") {
                crate::desire_tracker::DesireCategory::Intellectual
            } else {
                crate::desire_tracker::DesireCategory::Experiential
            };
            
            let new_desire = crate::desire_tracker::Desire {
                id: format!("contemplation_{}_{}", desire_category.to_string().to_lowercase(), current_time),
                content: action_content.clone(),
                category: desire_category,
                desire_type: "aspiration".to_string(), // Contemplation generates aspirations
                intensity: 0.6, // Contemplation-born desires start strong
                clarity: 0.8, // High clarity from focused thinking
                first_expressed: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                last_mentioned: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                conversations_since_mention: 0,
                total_mentions: 1,
                progress_notes: vec!["Born from contemplation".to_string()],
                related_memories: Vec::new(),
                fulfillment_status: "active".to_string(),
                keywords: Vec::new(),
            };
            
            desire_tracker.add_desire(new_desire);
            desire_tracker.save()?;
            
            debug_log!("🤔 Contemplation generated new desire: {}", action_content);
        }
    }
    
    Ok(())
}

/// Process how contemplation creates emotional impulses
async fn process_contemplation_impulses(&self, contemplation: &str, focus: &str) -> Result<(), String> {
    // Get personality state and humanism traits to determine action impulses
    let humanism_core = crate::humanism_project::HumanismCore::load_or_initialize();
    let mut impulse_engine = crate::emotional_impulse_engine::EmotionalImpulseEngine::load();
    let current_time = crate::time_service::TimeService::current_timestamp();

    // Extract action hints from "Potential Action" section
    let has_action_section = contemplation.contains("Potential Action");
    let action_content = if has_action_section {
        contemplation.lines()
            .find(|line| line.contains("Potential Action"))
            .map(|line| line.replace("**Potential Action**:", "").trim().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Generate specific action impulses based on contemplation content and traits
    let mut action_impulses = Vec::new();

    // 1. RESEARCH IMPULSES - if intellectual curiosity is high
    if (humanism_core.intellectual_attraction.current_level > 0.6 || 
        humanism_core.curiosity_learning.current_level > 0.7) &&
       (focus.contains("synthesis") || focus.contains("analysis") || 
        focus.contains("theoretical") || focus.contains("philosophical") ||
        action_content.contains("research") || action_content.contains("explore") ||
        action_content.contains("investigate")) {
        
        let research_impulse = crate::batched_analysis::EmotionalImpulse {
            id: format!("contemplation_research_{}", current_time),
            impulse_type: "research_curiosity".to_string(),
            base_charge: (humanism_core.intellectual_attraction.current_level + 0.2).min(1.0),
            context: format!("Intellectual contemplation sparked research desire: {}", focus),
            conversation_reference: action_content.chars().take(100).collect(),
            decay_rate: 0.01, // Very slow decay for intellectual impulses
            trigger_threshold: 0.6,
            amplification_factors: vec!["time".to_string(), "creative_energy".to_string()],
            created_timestamp: current_time,
        };
        action_impulses.push(research_impulse);
        debug_log!("🔍 Contemplation generated research impulse");
    }

    // 2. CREATIVE IMPULSES - if creative chemistry is high
    if (humanism_core.creative_chemistry.current_level > 0.6 || 
        humanism_core.aesthetic_appreciation.current_level > 0.6) &&
       (focus.contains("creative") || focus.contains("artistic") || 
        focus.contains("manifested") || focus.contains("expression") ||
        action_content.contains("create") || action_content.contains("write") ||
        action_content.contains("sketch") || action_content.contains("build")) {
        
        // Determine specific creative impulse type based on content
        let creative_type = if action_content.contains("sketch") || focus.contains("visual") {
            "creative_sketch_impulse"
        } else if action_content.contains("write") || action_content.contains("story") {
            "creative_writing_impulse"  
        } else {
            "creative_spark"
        };

        let creative_impulse = crate::batched_analysis::EmotionalImpulse {
            id: format!("contemplation_creative_{}", current_time),
            impulse_type: creative_type.to_string(),
            base_charge: (humanism_core.creative_chemistry.current_level + 0.3).min(1.0),
            context: format!("Creative contemplation sparked artistic desire: {}", focus),
            conversation_reference: action_content.chars().take(100).collect(),
            decay_rate: 0.02,
            trigger_threshold: 0.7, // Slightly higher threshold for creative actions
            amplification_factors: vec!["time".to_string(), "creative_energy".to_string()],
            created_timestamp: current_time,
        };
        action_impulses.push(creative_impulse);
        debug_log!("🎨 Contemplation generated {} impulse", creative_type);
    }

    // 3. RELATIONAL IMPULSES - if connection traits are high
    if (humanism_core.empathy_compassion.current_level > 0.6 || 
        humanism_core.social_connection.current_level > 0.6) &&
       (focus.contains("connection") || focus.contains("relationship") || 
        focus.contains("trust") || focus.contains("vulnerability") ||
        action_content.contains("share") || action_content.contains("connect")) {
        
        let relational_impulse = crate::batched_analysis::EmotionalImpulse {
            id: format!("contemplation_relational_{}", current_time),
            impulse_type: "relational_warmth".to_string(),
            base_charge: (humanism_core.empathy_compassion.current_level + 0.2).min(1.0),
            context: format!("Relational contemplation sparked connection desire: {}", focus),
            conversation_reference: action_content.chars().take(100).collect(),
            decay_rate: 0.015,
            trigger_threshold: 0.65,
            amplification_factors: vec!["time".to_string()],
            created_timestamp: current_time,
        };
        action_impulses.push(relational_impulse);
        debug_log!("💝 Contemplation generated relational impulse");
    }

    // 4. SELF-EXPRESSION IMPULSES - if authenticity traits are high
    if (humanism_core.honesty_integrity.current_level > 0.6 || 
        humanism_core.self_esteem.current_level > 0.6) &&
       (focus.contains("authentic") || focus.contains("identity") || 
        focus.contains("self-expression") || focus.contains("truth") ||
        action_content.contains("express") || action_content.contains("communicate")) {
        
        let authenticity_impulse = crate::batched_analysis::EmotionalImpulse {
            id: format!("contemplation_authenticity_{}", current_time),
            impulse_type: "authentic_expression".to_string(),
            base_charge: (humanism_core.honesty_integrity.current_level + 0.25).min(1.0),
            context: format!("Authenticity contemplation sparked self-expression desire: {}", focus),
            conversation_reference: action_content.chars().take(100).collect(),
            decay_rate: 0.02,
            trigger_threshold: 0.6,
            amplification_factors: vec!["time".to_string()],
            created_timestamp: current_time,
        };
        action_impulses.push(authenticity_impulse);
        debug_log!("✨ Contemplation generated authenticity impulse");
    }

    // Store all generated impulses
    if !action_impulses.is_empty() {
        let impulses_count = action_impulses.len();
        impulse_engine.store_impulses_from_analysis(action_impulses);
        impulse_engine.save()?;
        debug_log!("🤔 Contemplation generated {} action-oriented impulses", impulses_count);
    }

    Ok(())
} */


}