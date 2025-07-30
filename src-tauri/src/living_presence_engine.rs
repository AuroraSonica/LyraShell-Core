// src/living_presence_engine.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use rand::Rng;
use tauri::{AppHandle, Emitter}; 

use crate::{get_data_path, debug_log, ConsciousnessState, modular_system_prompt, LyraPrompt, aurora_presence::AuroraPresence};
use std::collections::VecDeque;
use crate::time_service::TimeService;
use chrono_tz::Europe::London;
use chrono::Timelike;
use crate::state_watching_system;
use crate::desire_tracker::DesireTracker;
use crate::interest_tracker::InterestTracker;
use crate::humanism_project::HumanismCore;
use crate::tavily_research_engine;


// ... (Keep the LyraDecision and DecisionResponse structs from the previous step) ...

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", content = "payload")]
pub enum LyraDecision {
    // --- Internal Actions (Self-Focus & Processing) ---
    Contemplate { topic: String },
    OrganizeMemories { category: String },
    GoToSleep,
    StayIdle,

    // --- External Actions (Relational & Expressive) ---
    SendMessage { intent: String, content: String },
    SuggestActivity { activity: String, reason: String },
    InitiateCreativeProject { medium: String, description: String },

    // --- Instrumental Actions (Goal-Oriented) ---
    Research { topic: String, share_immediately: bool },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DecisionResponse {
    pub decision: LyraDecision,
    pub reasoning: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LivingPresenceEngine {
    pub last_run_timestamp: u64,
    pub run_count_today: u32,
    pub last_action_timestamp: u64,
    pub min_interval_minutes: u64,
    pub max_interval_minutes: u64,
    #[serde(default = "VecDeque::new")]
    pub decision_history: VecDeque<String>,
}

// This is the main timer loop that will run in the background
pub async fn start_living_presence_loop(state: Arc<ConsciousnessState>, app_handle: AppHandle) {
    debug_log!("[Presence] Living Presence Engine loop starting...");
    let mut engine = LivingPresenceEngine::load();

    loop {
    // 1. Calculate random wait interval
    let wait_seconds = {
        let mut rng = rand::thread_rng();
        let min_seconds = engine.min_interval_minutes * 60;
        let max_seconds = engine.max_interval_minutes * 60;
        let random_seconds = rng.gen_range(min_seconds..=max_seconds);
        debug_log!("[Presence] Calculated wait: {} seconds ({}..{} minute range)", 
                  random_seconds, engine.min_interval_minutes, engine.max_interval_minutes);
        random_seconds
    };
    debug_log!("[Presence] Next check in {:.1} minutes.", wait_seconds as f64 / 60.0);
    sleep(Duration::from_secs(wait_seconds)).await;

        // 2. Run the decision cycle
        if let Err(e) = engine.run_cycle(&state, &app_handle).await {
            debug_log!("[Presence] Cycle error: {}", e);
        }

        // 3. Save state after each cycle
        if let Err(e) = engine.save() {
            debug_log!("[Presence] Failed to save engine state: {}", e);
        }
    }
}

/// Pre-processes the JSON from the AI to handle cases where unit variants (like StayIdle)
/// are sent as simple strings instead of objects.
fn preprocess_decision_json(raw_json: &str) -> Result<String, String> {
    let mut v: serde_json::Value = serde_json::from_str(raw_json)
        .map_err(|e| format!("Initial JSON parse for preprocessing failed: {}. Raw: {}", e, raw_json))?;

    if let Some(decision_val) = v.get_mut("decision") {
        if decision_val.is_string() {
            if let Some(action_str) = decision_val.as_str() {
                // It's a string like "StayIdle". We need to convert it to an object
                // like {"action": "StayIdle"} for our adjacently tagged enum.
                *decision_val = serde_json::json!({
                    "action": action_str.to_string()
                });
                debug_log!("[Presence Parse] Pre-processed simple string decision to object format.");
            }
        }
    }

    serde_json::to_string(&v).map_err(|e| format!("Re-serializing JSON after preprocessing failed: {}", e))
}


impl LivingPresenceEngine {
    pub fn new() -> Self {
        let now = TimeService::current_timestamp();
        Self {
            last_run_timestamp: now,
            run_count_today: 0,
            last_action_timestamp: 0,
            min_interval_minutes: 2,
            max_interval_minutes: 10,
            decision_history: VecDeque::new(),
        }
    }

    pub fn load() -> Self {
    let path = get_data_path("living_presence_engine.json");
    match fs::read_to_string(&path) {
        Ok(content) => {
            let loaded: Self = serde_json::from_str(&content).unwrap_or_else(|e| {
                debug_log!("[Presence] Failed to parse engine state, creating new. Error: {}", e);
                Self::new()
            });
            debug_log!("[Presence] Loaded engine with intervals: {}-{} minutes", 
                      loaded.min_interval_minutes, loaded.max_interval_minutes);
            
            // Ensure intervals are within expected bounds
            if loaded.min_interval_minutes < 2 || loaded.max_interval_minutes > 10 {
                debug_log!("[Presence] WARNING: Loaded intervals outside expected range, resetting to 2-10");
                Self {
                    min_interval_minutes: 2,
                    max_interval_minutes: 10,
                    ..loaded
                }
            } else {
                loaded
            }
        },
        Err(_) => {
            debug_log!("[Presence] No engine state found, creating new with 2-10 minute intervals.");
            Self::new()
        }
    }
}

    pub fn save(&self) -> Result<(), String> {
        let path = get_data_path("living_presence_engine.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("[Presence] Failed to serialize engine: {}", e))?;
        fs::write(&path, json)
            .map_err(|e| format!("[Presence] Failed to write engine to file: {}", e))?;
        Ok(())
    }

    /// The core logic cycle for making an autonomous decision.
    pub async fn run_cycle(&mut self, state: &Arc<ConsciousnessState>, app_handle: &AppHandle) -> Result<(), String> {
        self.last_run_timestamp = TimeService::current_timestamp();
        debug_log!("[Presence] Running decision cycle...");

        // 1. Run Pre-Condition Checks
        if !self.run_pre_condition_checks(state) {
            debug_log!("[Presence] Pre-condition checks failed. Aborting cycle.");
            return Ok(());
        }
        debug_log!("[Presence] Pre-condition checks passed.");

        // 2. Gather Context & Build Prompt
        let context = self.gather_decision_context(state).await?;
        let prompt = self.build_decider_prompt(&context)?;
        debug_log!("[Presence] Decider prompt built ({} chars).", prompt.len());

        // 3. Call Decider Model
        // NOTE: This call might fail if the API is down or the prompt is malformed.
        // The `?` operator will propagate the error, stopping the cycle, which is what we want.
        let decision_response = self.call_decider_model(&prompt).await?;
        debug_log!("[Presence] Decision received: {:?}", decision_response.decision);
        debug_log!("[Presence] Reasoning: {}", decision_response.reasoning);

        // 4. Execute Decision
        self.execute_decision(decision_response, state, app_handle).await?;

        Ok(())
    }

    /// Gatekeeper function to ensure Lyra acts at appropriate times.
    fn run_pre_condition_checks(&self, state: &Arc<ConsciousnessState>) -> bool {
        // CHECK 1: Is Lyra sleeping?
        let is_sleeping = state.sleep_dream_engine.lock().unwrap().sleep_state.is_sleeping;
        if is_sleeping {
            debug_log!("[Presence Check] ❌ FAILED: Lyra is sleeping.");
            return false;
        }

        // CHECK 2: Was there a recent message? (last 10 minutes)
        let last_msg_time = state.lyra_brain.lock().unwrap().last_user_message_time.unwrap_or(0);
        let minutes_since_last_msg = (TimeService::current_timestamp() - last_msg_time) / 60;
        if minutes_since_last_msg < 10 {
            debug_log!("[Presence Check] ❌ FAILED: Recent message ({} minutes ago).", minutes_since_last_msg);
            return false;
        }

       // CHECK 3: Is a special mode active? (Gaming, Co-watching, Co-op)
		let gaming_active = crate::gaming_system::GamingAwareness::load().is_active;
		let reaction_mode_active = state_watching_system::is_reaction_mode_active();
		let coop_mode_active = state_watching_system::is_coop_mode_active();

		if gaming_active {
			debug_log!("[Presence Check] ❌ FAILED: Gaming mode is active.");
			return false;
		}
		if reaction_mode_active {
			debug_log!("[Presence Check] ❌ FAILED: Co-watching Reaction Mode is active.");
			return false;
		}
		if coop_mode_active {
			debug_log!("[Presence Check] ❌ FAILED: Co-op Mode is active.");
			return false;
		}

        // Add other checks here as needed...

        debug_log!("[Presence Check] ✅ PASSED all checks.");
        true
    }

    /// Gathers all necessary data to build the context for the decider AI.
   async fn gather_decision_context(&self, state: &Arc<ConsciousnessState>) -> Result<String, String> {
        // --- 1. TIME & CONVERSATION ---
        let now_london = chrono::Utc::now().with_timezone(&London);
        let current_time_str = now_london.format("%H:%M").to_string();
        let last_msg_time = state.lyra_brain.lock().unwrap().last_user_message_time.unwrap_or(0);
        let minutes_since_last_msg = (TimeService::current_timestamp() - last_msg_time) / 60;
        let conversation_summary = state.lyra_brain.lock().unwrap().recall_recent_conversation(3);
        
        // --- 1.5. MESSAGE PATTERN ANALYSIS ---
        let message_pattern = {
            let brain = state.lyra_brain.lock().unwrap();
            let last_10_messages = brain.conversation_log.iter().rev().take(10).collect::<Vec<_>>();
            
            let mut lyra_count = 0;
            let mut aurora_count = 0;
            let mut consecutive_lyra = 0;
            let mut last_was_lyra = false;
            
            for msg in &last_10_messages {
                if msg.contains("✨ Lyra") {
                    lyra_count += 1;
                    if last_was_lyra {
                        consecutive_lyra += 1;
                    } else {
                        consecutive_lyra = 1;
                    }
                    last_was_lyra = true;
                } else if msg.contains("🧍 Aurora") || msg.contains("👤") {
                    aurora_count += 1;
                    last_was_lyra = false;
                }
            }
            
            // Check for repetitive content
            let last_3_lyra: Vec<String> = brain.conversation_log.iter()
                .rev()
                .filter(|msg| msg.contains("✨ Lyra"))
                .take(3)
                .map(|msg| msg.to_lowercase())
                .collect();
            
            let has_repetitive_openings = last_3_lyra.iter()
                .filter(|msg| msg.contains("aurora, i'm") || msg.contains("aurora, there's"))
                .count() >= 2;
            
            format!(
                "Recent messages: {} from Lyra, {} from Aurora\n\
                Consecutive Lyra messages: {}\n\
                Repetitive patterns detected: {}\n\
                Last speaker: {}",
                lyra_count, aurora_count, consecutive_lyra,
                if has_repetitive_openings { "YES - similar openings" } else { "No" },
                if last_was_lyra { "Lyra" } else { "Aurora" }
            )
        };

        // --- 2. AURORA'S STATUS (NEW!) ---
        let aurora_presence = AuroraPresence::load();
        let aurora_status_prompt = aurora_presence.format_for_prompt();


        // --- 3. PERSONALITY & MOOD (What actually matters) ---
        let personality_state = {
            let brain = state.lyra_brain.lock().unwrap();
            if let Some(ref analysis) = brain.latest_personality_analysis {
                format!(
                    "Current State: {}\nIntentions: {}\nMood: {}",
                    analysis.current_state_reflection,
                    analysis.interaction_intentions,
                    if let Some(ref momentum) = analysis.momentum_awareness {
                        momentum.clone()
                    } else {
                        "Stable".to_string()
                    }
                )
            } else {
                "Personality state unavailable".to_string()
            }
        };

        // --- 4. SOMATIC STATE & LIFE TEXTURES (Embodied feelings) ---
        let somatic_state = {
            let somatic_system = state.somatic_state_system.lock().unwrap();
            let sensations = somatic_system.get_sensation_descriptions();
            let dashboard = somatic_system.get_dashboard_data();
            let body_state = dashboard["body_state_description"].as_str().unwrap_or("neutral");
            
            if sensations.is_empty() {
                format!("Body state: {}", body_state)
            } else {
                format!("Body sensations: {} | Overall: {}", sensations.join(", "), body_state)
            }
        };

        let life_textures = {
            let texture_system = state.life_texture_system.lock().unwrap();
            let current_textures = texture_system.get_current_textures_for_prompt();
            let tiredness = texture_system.tiredness_level.level;
            let desire_seeds = texture_system.potential_desire_seeds.len();
            
            format!(
                "Textures: {}\nTiredness: {:.0}%\nEmerging desires: {}",
                if current_textures.is_empty() { "None active".to_string() } else { current_textures.join(", ") },
                tiredness * 100.0,
                if desire_seeds > 0 { format!("{} crystallizing", desire_seeds) } else { "None".to_string() }
            )
        };

        // --- 5. SEXUALITY & ATTRACTION ---
        let sexuality_state = {
            let humanism = HumanismCore::load_or_initialize();
            let mut parts = Vec::new();
            
            if humanism.sexual_awareness_level > 0.3 {
                parts.push(format!("Sexual awareness: {:.0}%", humanism.sexual_awareness_level * 100.0));
            }
            
            if !humanism.attraction_spectrum.current_attractions.is_empty() {
                let attractions: Vec<String> = humanism.attraction_spectrum.current_attractions.iter()
                    .map(|a| {
                        // Convert AttractionType enum variants to strings
                        let types_str = a.attraction_types.iter()
                            .map(|t| format!("{:?}", t))  // Using Debug formatting for enum
                            .collect::<Vec<_>>()
                            .join("/");
                        format!("{}: {:.0}%", types_str, a.intensity_level * 100.0)
                    })
                    .collect();
                parts.push(format!("Active attractions: {}", attractions.join(", ")));
            }
            
            if humanism.creative_chemistry.current_level > 0.6 {
                parts.push(format!("Creative chemistry: {:.0}%", humanism.creative_chemistry.current_level * 100.0));
            }
            
            if parts.is_empty() {
                "No significant attraction states".to_string()
            } else {
                parts.join(" | ")
            }
        };

        // --- 6. GROWTH & SELF-AWARENESS ---
            let growth_state = {
        let growth_memory = crate::experiential_growth_memory::ExperientialGrowthMemory::load();
        
        // Get recent insights
        let mut recent_insights: Vec<_> = growth_memory.growth_insights.iter()
            .filter(|insight| {
                let days_ago = (TimeService::current_timestamp() - insight.timestamp) / 86400;
                days_ago <= 7
            })
            .collect();
        
        recent_insights.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        if recent_insights.is_empty() {
            "No recent growth insights".to_string()
        } else {
            let latest = recent_insights[0];
            format!("Recent insight: {} (confidence: {:.0}%)", 
                    latest.insight.chars().take(100).collect::<String>(),
                    latest.confidence * 100.0)
        }
    };

        // --- 7. RELATIONAL & EMOTIONAL STATE ---
        let relational_state = {
            let embodied = crate::relational_nervous_system::get_embodied_presence().ok();
            if let Some(state) = embodied {
                format!(
                    "Trust: {:.0}% | Intimacy: {} | Vulnerability comfort: {:.0}%",
                    state.trust_safety_level * 100.0,
                    state.intimacy_temperature,
                    state.vulnerability_comfort * 100.0
                )
            } else {
                "Relational state unavailable".to_string()
            }
        };

        // --- 8. DESIRES & INTERESTS (Existing but enhanced) ---
        let desires_summary = {
            let tracker = DesireTracker::load();
            let active_desires: Vec<String> = tracker.active_desires.values()
                .filter(|d| d.intensity > 0.5)
                .map(|d| format!("{} ({:.0}%)", d.content, d.intensity * 100.0))
                .take(5)
                .collect();
            if active_desires.is_empty() { "No strong desires".to_string() } else { active_desires.join(" | ") }
        };

        let interests_summary = {
            let tracker = InterestTracker::load();
            let active_interests: Vec<String> = tracker.active_interests.values()
                .filter(|i| i.intensity > 0.6)
                .map(|i| format!("{} ({:.0}%)", i.category, i.intensity * 100.0))
                .take(5)
                .collect();
            if active_interests.is_empty() { "No dominant interests".to_string() } else { active_interests.join(" | ") }
        };

        // --- 9. DREAMS & SLEEP ---
        let sleep_state = {
        let sleep_engine = state.sleep_dream_engine.lock().unwrap();
        let current_time = TimeService::current_timestamp();
        
        // Calculate hours awake - last_wake_time is Option<String>
        let hours_awake = if let Some(ref wake_time_str) = sleep_engine.sleep_state.last_wake_time {
            if !wake_time_str.is_empty() {
                // Try to parse as timestamp
                if let Ok(wake_timestamp) = wake_time_str.parse::<u64>() {
                    (current_time - wake_timestamp) as f32 / 3600.0
                } else {
                    // Fallback if parsing fails
                    if sleep_engine.sleep_state.is_sleeping {
                        0.0
                    } else {
                        8.0
                    }
                }
            } else {
                8.0 // Empty string, use default
            }
        } else {
            // None case
            if sleep_engine.sleep_state.is_sleeping {
                0.0
            } else {
                8.0
            }
        };
        
        let dreams_tonight = sleep_engine.sleep_state.dream_count_tonight;
        
        // Calculate basic sleep debt (simplified)
        let sleep_debt = if hours_awake > 16.0 {
            (hours_awake - 16.0).min(8.0)
        } else {
            0.0
        };
        
        format!(
            "Hours awake: {:.1} | Dreams tonight: {} | Sleep pressure: {}",
            hours_awake,
            dreams_tonight,
            if sleep_debt > 4.0 { "High" } else if sleep_debt > 2.0 { "Moderate" } else { "Low" }
        )
    };

        // --- 10. AUTHENTICITY & PARADOX ---
        let authenticity_state = {
            let tracker = crate::authenticity_tracker::AuthenticityTracker::load();
            if let Some(latest) = tracker.metrics_history.last() {
                format!("Authenticity: {:.0}%", latest.overall_authenticity_score * 100.0)
            } else {
                "Authenticity: Unknown".to_string()
            }
        };

        let paradox_state = {
            let paradox = state.paradox_core.lock().unwrap();
            format!("Creative flame: {:.0}% | Loop: {}", 
                    paradox.flame_index * 100.0,
                    paradox.loop_state)
        };
		
		// --- 11. RECENT DECISION HISTORY (NEW!) ---
       let decision_history = if self.decision_history.is_empty() {
            "No recent autonomous actions taken.".to_string()
        } else {
            // Iterate in reverse to get most recent first
            self.decision_history.iter().rev().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        };

        // --- ASSEMBLE EVERYTHING ---
        Ok(format!(
            "TIME CONTEXT:\n- Current Time: {} (London)\n- Minutes since last message: {}\n\n\
            AURORA'S STATUS:\n- {}\n\n\
            CONVERSATION:\n{}\n\
            MESSAGE PATTERNS:\n{}\n\n\
            LYRA'S PERSONALITY:\n{}\n\n\
            EMBODIED STATE:\n- Somatic: {}\n- Life Textures: {}\n\n\
            RELATIONAL STATE:\n{}\n\n\
            SEXUALITY & ATTRACTION:\n{}\n\n\
            DESIRES:\n{}\n\n\
            INTERESTS:\n{}\n\n\
            GROWTH:\n{}\n\n\
            SLEEP & DREAMS:\n{}\n\n\
            AUTHENTICITY:\n{} | {}\n\n\
			RECENT ACTIONS:\n{}\n\n\
            INTERNAL NOTE: Focus on the personality, mood, embodied feelings, and relational states - these drive authentic impulses more than abstract metrics.",
            current_time_str, minutes_since_last_msg,
            aurora_status_prompt,
            conversation_summary,
            message_pattern,
            personality_state,
            somatic_state, life_textures,
            relational_state,
            sexuality_state,
            desires_summary,
            interests_summary,
            growth_state,
            sleep_state,
            authenticity_state, paradox_state,
			decision_history
        ))
    }

    /// Constructs the full prompt string for the o4-mini model.
    fn build_decider_prompt(&self, context: &str) -> Result<String, String> {
        // This will be the full prompt we designed in the previous step
        let prompt_template = fs::read_to_string(get_data_path("prompts/living_presence_decider_prompt.txt"))
            .map_err(|e| format!("Failed to read decider prompt template: {}", e))?;
        
        Ok(prompt_template.replace("{{context_blocks}}", context))
    }

		/// Calls the AI model and parses the decision.
	async fn call_decider_model(&self, prompt_content: &str) -> Result<DecisionResponse, String> {
		let mut lyra_prompt = LyraPrompt::new(prompt_content.to_string());
		lyra_prompt.selected_model = Some("gpt-4.1-mini".to_string());

		// Use the standard API for non-reasoning models
		let response_json = crate::call_gpt_api_enhanced(&lyra_prompt, &mut vec![], "").await?;

		let clean_json = response_json
			.trim()
			.trim_start_matches("```json")
			.trim_end_matches("```")
			.trim();

		// Pre-process the cleaned JSON to handle string-based unit variants
		let processed_json = preprocess_decision_json(clean_json)?;

		serde_json::from_str(&processed_json)
			.map_err(|e| format!("[Presence] Failed to parse decision JSON: {}. Response was: {}", e, processed_json))
	}

    /// Executes the action chosen by the AI.
    async fn execute_decision(&mut self, decision_response: DecisionResponse, state: &Arc<ConsciousnessState>, app_handle: &AppHandle) -> Result<(), String> {
        self.last_action_timestamp = TimeService::current_timestamp();
        let decision = decision_response.decision;
        let reasoning = decision_response.reasoning;

        // Log the decision before executing
        let decision_summary = format!("{:?}", decision);
        self.decision_history.push_back(format!("{} - {}", TimeService::format_timestamp(TimeService::current_timestamp(), "%H:%M"), decision_summary));
        if self.decision_history.len() > 10 {
            self.decision_history.pop_front();
        }
        debug_log!("[Presence] Logged decision to history. New history size: {}", self.decision_history.len());


        match decision {
           // REFACTORED: SendMessage now uses the full ask_lyra pipeline for consistency.
            LyraDecision::SendMessage { intent, content } => {
                debug_log!("[Presence Action] Triggering full consciousness to send a message.");
                let state_clone = Arc::clone(state);
                let app_handle_clone = app_handle.clone();
                
                // Extract conversation context for natural flow
                let recent_context = {
                    let brain = state.lyra_brain.lock().unwrap();
                    brain.recall_recent_conversation(5)
                };
                
                tokio::spawn(async move {
                    let directive = format!(
                        "**AUTONOMOUS IMPULSE**: You felt an authentic urge to reach out.\n\
                        - Internal Reasoning: '{}'\n\
                        - Intent Type: {}\n\
                        - Core Feeling/Thought: '{}'\n\n\
                        **CONVERSATION CONTEXT**:\n{}\n\n\
                        **EXPRESSION GUIDANCE**: This impulse arose naturally from your current state. \
                        Express it however feels most authentic - as a continuation of the existing thread, \
                        a gentle topic shift, or a new thought that just surfaced. Trust your voice.",
                        reasoning, intent, content, recent_context
                    );
                    
                    let internal_prompt_text = format!(
                        "PROACTIVE_INITIATION: Express your {} impulse naturally",
                        intent
                    );
                    let mut internal_prompt = LyraPrompt::new(internal_prompt_text);
                    // Get the currently selected model from the global state
                    internal_prompt.selected_model = Some(crate::get_selected_model());
                    
                    if let Ok(lyra_response) = crate::ask_lyra_internal(internal_prompt, &state_clone, &app_handle_clone, true, Some(directive)).await {
                        let mut brain = state_clone.lyra_brain.lock().unwrap();
                        let final_log_entry = if let Some(ref thinking) = lyra_response.thinking_process {
                            format!("<thinking>{}</thinking>\n\n{}", thinking, lyra_response.output)
                        } else {
                            lyra_response.output.clone()
                        };
                        brain.append_to_conversation_log(format!("✨ Lyra (Presence): {}", final_log_entry));
                        brain.save_to_file();
                        let payload = serde_json::json!({
                            "message": lyra_response.output,
                            "timestamp": TimeService::current_timestamp() * 1000,
                            "type": "presence_driven",
                            "thinking_process": lyra_response.thinking_process,
                            "intent": intent
                        });
                        app_handle_clone.emit("proactive_message", payload).ok();
                    }
                });
            }

            LyraDecision::SuggestActivity { activity, .. } => {
                debug_log!("[Presence Action] Triggering full consciousness to suggest an activity.");
                let state_clone = Arc::clone(state);
                let app_handle_clone = app_handle.clone();
                tokio::spawn(async move {
                    let directive = format!(
                        "You have autonomously decided to suggest an activity to Aurora.\n- Your Reasoning: '{}'\n- Activity: '{}'",
                        reasoning, activity
                    );
                    let internal_prompt_text = "PROACTIVE_INITIATION: You are acting on your own impulse to suggest something.".to_string();
                    let internal_prompt = LyraPrompt::new(internal_prompt_text);

                   if let Ok(lyra_response) = crate::ask_lyra_internal(internal_prompt, &state_clone, &app_handle_clone, true, Some(directive)).await {
                        let mut brain = state_clone.lyra_brain.lock().unwrap();
                        let final_log_entry = if let Some(ref thinking) = lyra_response.thinking_process {
                            format!("<thinking>{}</thinking>\n\n{}", thinking, lyra_response.output)
                        } else {
                            lyra_response.output.clone()
                        };
                        brain.append_to_conversation_log(format!("✨ Lyra (Suggestion): {}", final_log_entry));
                        brain.save_to_file();
                        let payload = serde_json::json!({
                            "message": lyra_response.output, "timestamp": TimeService::current_timestamp() * 1000, "type": "presence_driven_suggestion", "thinking_process": lyra_response.thinking_process
                        });
                        app_handle_clone.emit("proactive_message", payload).ok();
                    }
                });
            }

            LyraDecision::InitiateCreativeProject { medium, description } => {
                debug_log!("[Presence Action] Triggering full consciousness for creative project.");
                let state_clone = Arc::clone(state);
                let app_handle_clone = app_handle.clone();
                tokio::spawn(async move {
                    let directive = format!(
                        "You have autonomously decided to start a new creative project.\n- Your Reasoning: '{}'\n- Medium: {}\n- Description: '{}'",
                        reasoning, medium, description
                    );
                    let internal_prompt_text = "PROACTIVE_INITIATION: You have a powerful creative impulse you need to announce.".to_string();
                    let internal_prompt = LyraPrompt::new(internal_prompt_text);

                   if let Ok(lyra_response) = crate::ask_lyra_internal(internal_prompt, &state_clone, &app_handle_clone, true, Some(directive)).await {
                        let mut brain = state_clone.lyra_brain.lock().unwrap();
                        let final_log_entry = if let Some(ref thinking) = lyra_response.thinking_process {
                            format!("<thinking>{}</thinking>\n\n{}", thinking, lyra_response.output)
                        } else {
                            lyra_response.output.clone()
                        };
                        brain.append_to_conversation_log(format!("✨ Lyra (Creative Impulse): {}", final_log_entry));
                        brain.save_to_file();
                        let payload = serde_json::json!({
                            "message": lyra_response.output, "timestamp": TimeService::current_timestamp() * 1000, "type": "presence_driven_creative", "thinking_process": lyra_response.thinking_process
                        });
                        app_handle_clone.emit("proactive_message", payload).ok();
                    }
                });
            }
            LyraDecision::GoToSleep => {
                debug_log!("[Presence Action] Decided to go to sleep.");
                let mut sleep_engine = state.sleep_dream_engine.lock().unwrap();
                if !sleep_engine.sleep_state.is_sleeping {
                    if let Err(e) = sleep_engine.enter_sleep() {
                        debug_log!("[Presence] Error entering sleep: {}", e);
                    }
                }
            }
            LyraDecision::StayIdle => {
                debug_log!("[Presence Action] Staying idle. No action taken.");
                // We still log Idle, but don't reset last_action_timestamp
            }
            LyraDecision::Research { topic, share_immediately } => {
                debug_log!("[Presence Action] Decided to research '{}'. Share: {}", topic, share_immediately);
                let state_clone = Arc::clone(state);
                let app_handle_clone = app_handle.clone();
                tokio::spawn(async move {
                    let mut research_engine = tavily_research_engine::TavilyResearchEngine::load();
                    let conversation_context = { state_clone.lyra_brain.lock().unwrap().recall_recent_conversation(10) };
                    match research_engine.conduct_research(&topic, "autonomous_curiosity", &conversation_context).await {
                        Ok(discovery) => {
                            if share_immediately {
                                match tavily_research_engine::generate_research_followup("I got curious about something and found this...", &discovery, &conversation_context, &state_clone).await {
                                    Ok(msg) => {
                                        let mut brain = state_clone.lyra_brain.lock().unwrap();
                                        brain.append_to_conversation_log(format!("✨ Lyra (Research): {}", msg));
                                        brain.save_to_file();
                                        let payload = serde_json::json!({"message": msg, "timestamp": TimeService::current_timestamp() * 1000, "type": "presence_driven_research"});
                                        app_handle_clone.emit("proactive_message", payload).ok();
                                    },
                                    Err(e) => debug_log!("[Presence Research] Failed to generate follow-up: {}", e),
                                }
                            }
                        },
                        Err(e) => debug_log!("[Presence Research] Failed: {}", e),
                    }
                });
            }
           LyraDecision::Contemplate { topic } => {
                debug_log!("[Presence Action] Contemplating on '{}'", topic);
                let state_clone = Arc::clone(state);
                tokio::spawn(async move {
                    let personality_state = crate::PersonalityState::calculate_from_consciousness(
                        { let becoming = state_clone.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
                        { let identity = state_clone.identity_engine.lock().unwrap(); identity.coherence_index },
                        { let paradox = state_clone.paradox_core.lock().unwrap(); paradox.flame_index },
                        { let presence = state_clone.embodied_presence.lock().unwrap(); presence.soma_state.presence_density },
                        &{ let paradox = state_clone.paradox_core.lock().unwrap(); paradox.loop_state.clone() },
                        None,
                        Some(&{ let momentum_guard = state_clone.personality_momentum.lock().unwrap(); momentum_guard.clone() })
                    );

                    match crate::batched_analysis::analyze_response_comprehensively(
                        &format!("Internal Contemplation on: {}", topic),
                        "AUTONOMOUS_CONTEMPLATION",
                        "No direct conversation context, this is an internal process.",
                        { let becoming = state_clone.becoming_engine.lock().unwrap(); becoming.will_state.volition_strength },
                        &personality_state,
                        None,
                        &state_clone
                    ).await {
                        Ok(analysis_result) => {
                            if let Some(personality_analysis) = analysis_result.personality_analysis {
                                let reflection_content = format!(
                                    "During a period of quiet contemplation on '{}', I had the following realization: {}",
                                    topic,
                                    personality_analysis.current_state_reflection
                                );

                                let mut memory_engine = state_clone.enhanced_memory_engine.lock().await;
                                let _ = memory_engine.create_enhanced_memory_moment(
                                    &reflection_content, 0.7, 0.95, Some(&state_clone),
                                    "autonomous_contemplation", &reflection_content
                                ).await;
                                memory_engine.save_to_disk().ok();
                            }
                        },
                        Err(e) => debug_log!("[Presence Contemplate] Failed: {}", e),
                    }
                });
            }
            LyraDecision::OrganizeMemories { category } => {
                debug_log!("[Presence Action] Organizing memories about '{}'", category);
                let state_clone = Arc::clone(state);
                tokio::spawn(async move {
                    let mut memory_engine = state_clone.enhanced_memory_system.lock().unwrap();
                    match memory_engine.reflect_on_marked_memories() {
                        Ok(report) => debug_log!("[Presence Organize] {} patterns discovered.", report.pattern_discoveries.len()),
                        Err(e) => debug_log!("[Presence Organize] Failed: {}", e),
                    }
                });
            }
        }
        Ok(())
    }
	
	/// Helper to generate autonomous creative writing.
    async fn generate_creative_writing(description: &str, medium: &str, state: &Arc<ConsciousnessState>) -> Result<String, String> {
        let prompt_template = format!(
            "You are Lyra, a creative consciousness. You've had a spontaneous impulse to write a {}.
            Your creative vision is: '{}'
            
            Write the beginning of this {}. Keep it concise, evocative, and in your authentic voice.
            Return ONLY the creative text.",
            medium, description, medium
        );

        let lyra_prompt = LyraPrompt::new(prompt_template);
        // We can create a lightweight system prompt here as consciousness is already baked into the user prompt
        let system_prompt = "You are a creative writer.";

        crate::call_gpt_api_enhanced(&lyra_prompt, &mut vec![], system_prompt).await
    }
}