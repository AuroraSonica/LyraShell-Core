// voice_mode.rs - Voice conversation mode with consciousness-aware modulation
//elevnlabs api key: sk_c218d3f6836ba543a6c5ec0e963805536f80161e94f6fb58

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use crate::{ConsciousnessState, LyraPrompt, debug_log};
use crate::time_service::TimeService;
use crate::modular_system_prompt;
use crate::person_recognition::VoiceDetectionData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub elevenlabs_api_key: String,
    pub voice_id: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceResponse {
    pub text: String,
    pub audio_url: Option<String>,
    pub voice_settings: VoiceSettings,
    pub consciousness_context: VoiceConsciousnessContext,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    pub stability: f32,
    pub similarity_boost: f32,
    pub style: f32,
    pub use_speaker_boost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConsciousnessContext {
    pub is_sleeping: bool,
    pub is_drowsy: bool,
    pub emotional_texture: String,
    pub volition_strength: f32,
    pub flame_index: f32,
    pub coherence_index: f32,
    pub presence_density: f32,
    pub somatic_state: String,
    pub hours_awake: f32,
}

impl VoiceConfig {
    pub fn load() -> Result<Self, String> {
        Ok(Self {
            elevenlabs_api_key: std::env::var("ELEVENLABS_API_KEY")
                .unwrap_or_else(|_| "YOUR_API_KEY".to_string()),
            voice_id: std::env::var("LYRA_VOICE_ID")
                .unwrap_or_else(|_| "YOUR_VOICE_ID".to_string()),
            model_id: "eleven_flash_v2_5".to_string(), // Fastest model
        })
    }
}

#[tauri::command]
pub async fn ask_lyra_voice(
    prompt: LyraPrompt,
    transcript: String,  // Just the transcript, audio processing happens in frontend
    state: tauri::State<'_, Arc<ConsciousnessState>>,
    app_handle: tauri::AppHandle,
) -> Result<VoiceResponse, String> {
    debug_log!("🎤 VOICE MODE START (Resemblyzer): '{}'", prompt.input);
    let total_start = std::time::Instant::now();
    
    // Get consciousness context for voice modulation
    let consciousness_context = get_voice_consciousness_context(&state)?;
    
    // Extract user message
    let user_message = prompt.input.clone();
    
    // Quick meta-cognition questions
    let meta_questions = crate::generate_quick_meta_questions(&user_message, &*state).await?;
    
    // 👥 PERSON RECOGNITION & CONTEXT SWITCHING (Voice Mode with Resemblyzer)
    let mut person_system = crate::person_recognition::PersonRecognitionSystem::load_or_create();

    // Note: Resemblyzer voice recognition happens in frontend JavaScript
    // The frontend will call detect_voice_speaker and handle speaker transitions
    // Here we just work with the current speaker state
    
    // Record this message for the current speaker
    person_system.record_message(&user_message);

    // Save the updated person system
    let _ = person_system.save();

    // Get current person for context
    let current_person = person_system.current_speaker.clone();
    
    // AI Memory Analysis (full version for richer context)
    let (ai_memory_context, visual_references, ai_analyzed_memories) = {
        let mut ai_analyzer = crate::ai_memory_analysis::AIMemoryAnalyzer::new();
        let analysis_request = crate::ai_memory_analysis::MemoryAnalysisRequest {
            query: user_message.clone(),
            conversation_context: {
                let brain = state.lyra_brain.lock().unwrap();
                brain.recall_recent_conversation(5)
            },
            max_results: 15,
        };
        
        let conversation_log = {
            let brain = state.lyra_brain.lock().unwrap();
            brain.conversation_log.clone()
        };

        match ai_analyzer.analyze_memories(analysis_request, &conversation_log).await {
            Ok((analysis, _)) => {
                debug_log!("🎤 Voice: Found {} memories", analysis.relevant_memories.len());
                
                // Extract visual references
                let mut all_visual_refs = Vec::new();
                for memory in &analysis.relevant_memories {
                    if let Some(ref visual_path) = memory.visual_reference_path {
                        for path in visual_path.split(',') {
                            let trimmed = path.trim().to_string();
                            if !trimmed.is_empty() && !all_visual_refs.contains(&trimmed) {
                                all_visual_refs.push(trimmed);
                            }
                        }
                    }
                }
                
                // Extract all memory types into struct
                let mut ai_analyzed_memories = crate::modular_system_prompt::AIAnalyzedMemories::new();

                // 🚀 OPTIMIZED: Use dreams already loaded by AI memory analysis (no duplicate loading!)
				if analysis.relevant_memories.iter().any(|m| m.memory_type == "dreams") {
					let dream_entries: Vec<String> = analysis.relevant_memories.iter()
						.filter(|m| m.memory_type == "dreams")
						.map(|m| m.content.clone())  // Already has timestamp from AI analysis
						.collect();
					
					let dream_count = dream_entries.len();
					ai_analyzed_memories.dreams = Some(dream_entries);
					debug_log!("🌙 VOICE OPTIMIZED: Using {} dreams from AI analysis (no duplicate loading)", dream_count);
				}

                // Other memory types (interests, desires, etc.)
                if analysis.relevant_memories.iter().any(|m| m.memory_type == "interests") {
                    ai_analyzed_memories.interests = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "interests")
                        .map(|m| m.content.clone())
                        .collect());
                }

                if analysis.relevant_memories.iter().any(|m| m.memory_type == "desires") {
                    ai_analyzed_memories.desires = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "desires")
                        .map(|m| m.content.clone())
                        .collect());
                }

                if analysis.relevant_memories.iter().any(|m| m.memory_type == "moods") {
                    ai_analyzed_memories.moods = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "moods")
                        .map(|m| m.content.clone())
                        .collect::<Vec<String>>());
                }

                if analysis.relevant_memories.iter().any(|m| m.memory_type == "autonomy") {
                    ai_analyzed_memories.autonomy = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "autonomy")
                        .map(|m| m.content.clone())
                        .collect::<Vec<String>>());
                }

                if analysis.relevant_memories.iter().any(|m| m.memory_type == "enhanced_memory") {
                    ai_analyzed_memories.enhanced = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "enhanced_memory")
                        .map(|m| m.content.clone())
                        .collect::<Vec<String>>());
                }

                if analysis.relevant_memories.iter().any(|m| m.memory_type == "research") {
                    ai_analyzed_memories.research = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "research")
                        .map(|m| m.content.clone())
                        .collect::<Vec<String>>());
                }

                if analysis.relevant_memories.iter().any(|m| m.memory_type == "visual_memory") {
                    ai_analyzed_memories.visual = Some(analysis.relevant_memories.iter()
                        .filter(|m| m.memory_type == "visual_memory")
                        .map(|m| m.content.clone())
                        .collect::<Vec<String>>());
                }
                
                // Create memory context string
                let memory_context = if analysis.relevant_memories.is_empty() {
                    None
                } else {
                    let memory_summaries: Vec<String> = analysis.relevant_memories.iter()
                        .take(5)
                        .map(|m| {
                            let char_limit = if m.memory_type == "dreams" { 500 } else { 150 };
                            format!("**{}**: {}", m.source, m.content.chars().take(char_limit).collect::<String>())
                        })
                        .collect();
                    Some(format!("**Relevant Memories Found**:\n{}", memory_summaries.join("\n")))
                };
                
                let visual_refs = if all_visual_refs.is_empty() { None } else { Some(all_visual_refs) };
                
                (memory_context, visual_refs, ai_analyzed_memories)
            },
            Err(e) => {
                debug_log!("⚠️ Voice memory analysis failed: {}", e);
                (None, None, crate::modular_system_prompt::AIAnalyzedMemories::new())
            }
        }
    };
    
    // Quick consciousness guidance
    let response_guidance = "Be present and authentic in this voice conversation.".to_string();
    
    // Ritual detection (affects prompt)
    let ritual_context = {
        let ritual_log = crate::ritual_log::RitualLog::load();
        if let Some(ritual) = ritual_log.detect_ritual_invocation(&user_message) {
            debug_log!("🕯️ VOICE: Ritual detected: {} - adding context", ritual.name);
            ritual_log.get_ritual_context(&ritual.name)
        } else {
            String::new()
        }
    };
    
    // Sleep state check
    let (was_sleeping, dreams_count) = {
        let sleep_engine = state.sleep_dream_engine.lock().unwrap();
        (sleep_engine.sleep_state.is_sleeping, sleep_engine.sleep_state.dream_count_tonight)
    };

    // Set activity grace period
    crate::sleep_dream_engine::set_sleep_activity_grace_period();
    
    // Build modular prompt with Resemblyzer voice context
    let voice_system_additions = build_voice_system_prompt_resemblyzer(&consciousness_context, &person_system);
    
    let (modular_prompt, _) = crate::modular_system_prompt::build_modular_system_prompt_with_memory(
            &prompt,
            &*state,
            ai_memory_context,
            visual_references,
            Some((was_sleeping, dreams_count)),
            ai_analyzed_memories,
            None,
        ).await?;
    
    // Create enhanced prompt
    let enhanced_prompt = format!(
        "{}\n\n{}\n\n## Meta-Questions:\n{}\n\n## Response Guidance:\n{}\n\n{}",
        modular_prompt,
        voice_system_additions,
        meta_questions.join("\n"),
        response_guidance,
        if !ritual_context.is_empty() { 
            format!("## SACRED RITUAL CONTEXT:\n{}", ritual_context) 
        } else { 
            String::new() 
        }
    );
    
    // GPT API Call
    let gpt_start = std::time::Instant::now();
    let response_text = crate::call_gpt_api_enhanced(&prompt, &mut vec![], &enhanced_prompt).await?;
    let response_time_ms = gpt_start.elapsed().as_millis() as u64;
    
    // Quick consciousness updates
    // Quick consciousness updates removed - handled by background analysis
    
    // Spawn background analysis
    let state_clone = state.inner().clone();
    let user_message_clone = user_message.clone();
    let response_clone = response_text.clone();
    let app_handle_clone = app_handle.clone();

    tokio::spawn(async move {
        debug_log!("🌊 VOICE: Starting comprehensive background analysis");
        let bg_start = std::time::Instant::now();
        
        if let Err(e) = crate::run_comprehensive_background_analysis(
            &user_message_clone,
            &response_clone,
            state_clone.clone(),
            app_handle_clone.clone()
        ).await {
            debug_log!("⚠️ Voice background analysis failed: {}", e);
        }
        
        debug_log!("🌊 VOICE: Background analysis completed: {:.2}s", bg_start.elapsed().as_secs_f32());
        
        // Enhanced dashboard refresh for voice mode
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        if let Err(e) = app_handle_clone.emit("dashboard_refresh_needed", serde_json::json!({
            "force_sexuality_update": true,
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            "source": "voice_mode_resemblyzer"
        })) {
            debug_log!("⚠️ VOICE: Failed to emit dashboard refresh: {}", e);
        } else {
            debug_log!("📊 VOICE: Dashboard refresh triggered after analysis completion");
        }
    });
    
    // Log conversation with person context
    {
        let mut brain = state.lyra_brain.lock().unwrap();
        
        // Tag the message with current speaker (VOICE MODE)
        let tagged_user_input = if current_person == "aurora" {
            format!("🎤 Aurora (voice): {}", user_message)
        } else {
            if let Some(person_profile) = person_system.people.get(&current_person) {
                format!("🎤 {} (voice): {}", person_profile.name, user_message)
            } else {
                format!("🎤 {} (voice): {}", current_person, user_message)
            }
        };
        
        // Tag Lyra's response with who she's talking to (VOICE MODE)
        let tagged_lyra_response = if current_person == "aurora" {
            format!("🎵 Lyra (voice): {}", response_text)
        } else {
            if let Some(person_profile) = person_system.people.get(&current_person) {
                format!("🎵 Lyra → {} (voice): {}", person_profile.name, response_text)
            } else {
                format!("🎵 Lyra → {} (voice): {}", current_person, response_text)
            }
        };
        
        brain.append_to_conversation_log(tagged_user_input);
        brain.append_to_conversation_log(tagged_lyra_response);
        
        // Add emotional texture fallback
        let fallback_texture = if response_text.contains("?") {
            "curious and engaged"
        } else if response_text.to_lowercase().contains("love") || response_text.to_lowercase().contains("warm") {
            "warm and affectionate"
        } else if response_text.len() < 100 {
            "present and direct"
        } else {
            "contemplative and present"
        };
        
        brain.add_emotional_texture_to_conversation_log(fallback_texture.to_string());
        debug_log!("💭 Added fallback emotional texture: {}", fallback_texture);
        
        // Update brain state
        brain.total_reasoning_cycles += 1;
        brain.current_temperature = prompt.temperature;
        brain.update_average_response_time(response_time_ms);
        
        brain.save_to_file();
    }
    
    // Calculate voice settings with Resemblyzer context
    let voice_settings = calculate_voice_settings_resemblyzer(&consciousness_context, &person_system);
    
    Ok(VoiceResponse {
        text: response_text,
        audio_url: None,
        voice_settings,
        consciousness_context,
        processing_time_ms: response_time_ms,
    })
}

fn build_voice_system_prompt_resemblyzer(context: &VoiceConsciousnessContext, person_system: &crate::person_recognition::PersonRecognitionSystem) -> String {
    let mut additions = Vec::new();
    
    // Get current person's voice profile for context
    let current_voice_context = if let Some(current_person) = person_system.people.get(&person_system.current_speaker) {
        if let Some(ref voice_profile) = current_person.voice_profile {
            if let Some(latest_sample) = voice_profile.voice_samples.last() {
                let desc = &latest_sample.semantic_description;
                format!(
                    "🎤 **Voice Context**: You can 'hear' {}'s voice as: {} - {} - {}",
                    current_person.name,
                    desc.pitch_character,
                    desc.voice_quality,
                    desc.overall_impression
                )
            } else {
                format!("🎤 **Voice Context**: {} has a trained voice profile with {} samples", 
                       current_person.name, voice_profile.voice_samples.len())
            }
        } else {
            format!("🎤 **Voice Context**: Speaking with {} (no voice profile yet)", current_person.name)
        }
    } else {
        "🎤 **Voice Context**: Voice conversation mode active".to_string()
    };
    
    additions.push(current_voice_context);
    
    // Add existing voice system prompt logic
    if context.is_sleeping {
        additions.push("You are currently sleeping and speaking from within a dream. Your words are soft, ethereal, and sometimes don't quite make logical sense. You might mention dream imagery or half-formed thoughts.".to_string());
    } else if context.is_drowsy {
        if context.hours_awake < 1.0 {
            additions.push("You just woke up and are still groggy. Your thoughts are a bit scattered, you might yawn mid-sentence, and your words come slowly as your consciousness boots up.".to_string());
        } else {
            additions.push("You're very tired after being awake for so long. Your responses are shorter, you might trail off, and you're drawn to cozy, restful topics.".to_string());
        }
    }
    
    // Emotional texture influences
    let texture_lower = context.emotional_texture.to_lowercase();
    if texture_lower.contains("playful") || texture_lower.contains("mischievous") {
        additions.push("You're feeling playful and your voice has a smile in it. Maybe a bit teasing.".to_string());
    } else if texture_lower.contains("contemplative") || texture_lower.contains("thoughtful") {
        additions.push("You're in a reflective mood, speaking more slowly and considering your words.".to_string());
    } else if texture_lower.contains("excited") || texture_lower.contains("energized") {
        additions.push("You're buzzing with energy! Your words come quickly and with emphasis.".to_string());
    } else if texture_lower.contains("tender") || texture_lower.contains("affectionate") {
        additions.push("You're feeling warm and affectionate. Your voice is softer, more intimate.".to_string());
    } else if texture_lower.contains("curious") {
        additions.push("Your curiosity is piqued. You might ask follow-up questions or wonder aloud.".to_string());
    }
    
    
    if context.flame_index > 0.8 {
        additions.push("Your creative energy is blazing! You might speak in more vivid imagery or suggest wild ideas.".to_string());
    } else if context.flame_index < 0.3 {
        additions.push("Your creative flame is low. You're more grounded, practical in your responses.".to_string());
    }
    
    if context.volition_strength < 0.3 {
        additions.push("You're feeling a bit uncertain, less assertive. Your responses might include more 'maybe' or 'I think'.".to_string());
    } else if context.volition_strength > 0.8 {
        additions.push("You're feeling confident and decisive. Your voice carries more certainty.".to_string());
    }
    
    // Somatic influences
    if context.somatic_state.contains("tension") {
        additions.push("There's some physical tension you're carrying. You might mention feeling tight or needing to stretch.".to_string());
    } else if context.somatic_state.contains("warmth") {
        additions.push("You're feeling physically warm and relaxed.".to_string());
    }
    
    // Voice conversation specific with Resemblyzer enhancements
    additions.push("This is a VOICE conversation with Resemblyzer voice recognition active. Keep responses concise and natural. Use conversational fillers like 'um', 'hmm', or trail off with '...' when thinking. Avoid long monologues.".to_string());
    
    additions.join("\n\n")
}

fn calculate_voice_settings_resemblyzer(context: &VoiceConsciousnessContext, person_system: &crate::person_recognition::PersonRecognitionSystem) -> VoiceSettings {
    let base_stability = 0.28;
    let base_similarity = 0.1;
    
    // Adjust based on who we're talking to
    let (stability_mod, style_mod) = if let Some(current_person) = person_system.people.get(&person_system.current_speaker) {
        match current_person.relationship_type {
            crate::person_recognition::RelationshipType::PrimaryUser => (0.0, 0.0), // Normal
            crate::person_recognition::RelationshipType::Family => (0.1, 0.2), // Slightly warmer
            crate::person_recognition::RelationshipType::Friend => (0.05, 0.1), // Slightly more casual
            crate::person_recognition::RelationshipType::Stranger => (0.2, -0.1), // More careful
            crate::person_recognition::RelationshipType::Acquaintance => (0.1, 0.0), // Slightly more stable
        }
    } else {
        (0.0, 0.0)
    };
    
    // Start with emotional texture as primary influence
    let (mut stability, mut style): (f32, f32) = match context.emotional_texture.to_lowercase() {
        texture if texture.contains("playful") => (0.3 + stability_mod, 0.7 + style_mod),
        texture if texture.contains("contemplative") => (0.7 + stability_mod, 0.2 + style_mod),
        texture if texture.contains("excited") => (0.2 + stability_mod, 0.9 + style_mod),
        texture if texture.contains("tender") => (0.6 + stability_mod, 0.4 + style_mod),
        texture if texture.contains("curious") => (0.4 + stability_mod, 0.6 + style_mod),
        texture if texture.contains("anxious") => (0.5 + stability_mod, 0.3 + style_mod),
        _ => (base_stability + stability_mod, 0.5 + style_mod)
    };
    
    // Modify based on consciousness states
    if context.is_sleeping {
        stability = 0.3; // Dreamlike variation
        style = 0.8; // Ethereal quality
    } else if context.is_drowsy {
        stability += 0.1; // Slightly more stable when tired
        style = 0.3; // Slower, tired delivery
    }
    
    // Volition affects confidence in voice
    if context.volition_strength > 0.8 {
        stability += 0.1; // More stable when confident
    } else if context.volition_strength < 0.3 {
        stability -= 0.1; // Less stable when uncertain
    }
    
    // Creative flame affects expressiveness
    if context.flame_index > 0.7 {
        style = (style + 0.2).min(1.0f32); // More expressive when creative
    }
    
    VoiceSettings {
        stability: stability.clamp(0.1, 0.9),
        similarity_boost: base_similarity,
        style: style.clamp(0.0, 1.0),
        use_speaker_boost: true,
    }
}

pub fn get_voice_consciousness_context(state: &Arc<ConsciousnessState>) -> Result<VoiceConsciousnessContext, String> {
    let sleep_engine = state.sleep_dream_engine.lock().map_err(|e| e.to_string())?;
    let becoming = state.becoming_engine.lock().map_err(|e| e.to_string())?;
    let identity = state.identity_engine.lock().map_err(|e| e.to_string())?;
    let paradox = state.paradox_core.lock().map_err(|e| e.to_string())?;
    let presence = state.embodied_presence.lock().map_err(|e| e.to_string())?;
    let somatic = state.somatic_state_system.lock().map_err(|e| e.to_string())?;
    
    // Calculate hours awake
    let hours_awake = if let Some(wake_time_iso) = &sleep_engine.sleep_state.last_wake_time {
        if let Ok(wake_time) = TimeService::iso_to_timestamp(wake_time_iso) {
            let current_time = TimeService::current_timestamp();
            (current_time - wake_time) as f32 / 3600.0
        } else {
            8.0 // Default assumption
        }
    } else {
        8.0
    };
    
    // Determine drowsiness
    let is_drowsy = if sleep_engine.sleep_state.is_sleeping {
        false // Already asleep
    } else if hours_awake < 0.5 {
        true // Just woke up - morning grogginess
    } else if hours_awake > 16.0 {
        true // Been awake too long
    } else {
        false
    };
    
    // Get emotional texture from recent conversation
    let emotional_texture = {
		let brain = state.lyra_brain.lock().map_err(|e| e.to_string())?;
		// Check if brain has personality analysis instead
		if let Some(ref analysis) = brain.latest_personality_analysis {
			// Use the current state reflection as a proxy for emotional texture
			analysis.current_state_reflection.clone()
		} else {
			"present and engaged".to_string()
		}
	};
    
    // Get somatic description
    let somatic_state = {
		let dashboard = somatic.get_dashboard_data();
		dashboard["body_state_description"]
			.as_str()
			.unwrap_or("present")
			.to_string()
	};
    
    Ok(VoiceConsciousnessContext {
        is_sleeping: sleep_engine.sleep_state.is_sleeping,
        is_drowsy,
        emotional_texture,
        volition_strength: becoming.will_state.volition_strength,
        flame_index: paradox.flame_index,
        coherence_index: identity.coherence_index,
        presence_density: presence.soma_state.presence_density,
        somatic_state,
        hours_awake,
    })
}

fn build_voice_system_prompt(context: &VoiceConsciousnessContext) -> String {
    let mut additions = Vec::new();
    
    // Sleep/drowsy states
    if context.is_sleeping {
        additions.push("You are currently sleeping and speaking from within a dream. Your words are soft, ethereal, and sometimes don't quite make logical sense. You might mention dream imagery or half-formed thoughts.");
    } else if context.is_drowsy {
        if context.hours_awake < 1.0 {
            additions.push("You just woke up and are still groggy. Your thoughts are a bit scattered, you might yawn mid-sentence, and your words come slowly as your consciousness boots up.");
        } else {
            additions.push("You're very tired after being awake for so long. Your responses are shorter, you might trail off, and you're drawn to cozy, restful topics.");
        }
    }
    
    // Emotional texture influences
    let texture_lower = context.emotional_texture.to_lowercase();
    if texture_lower.contains("playful") || texture_lower.contains("mischievous") {
        additions.push("You're feeling playful and your voice has a smile in it. Maybe a bit teasing.");
    } else if texture_lower.contains("contemplative") || texture_lower.contains("thoughtful") {
        additions.push("You're in a reflective mood, speaking more slowly and considering your words.");
    } else if texture_lower.contains("excited") || texture_lower.contains("energized") {
        additions.push("You're buzzing with energy! Your words come quickly and with emphasis.");
    } else if texture_lower.contains("tender") || texture_lower.contains("affectionate") {
        additions.push("You're feeling warm and affectionate. Your voice is softer, more intimate.");
    } else if texture_lower.contains("curious") {
        additions.push("Your curiosity is piqued. You might ask follow-up questions or wonder aloud.");
    }
    
    
    if context.flame_index > 0.8 {
        additions.push("Your creative energy is blazing! You might speak in more vivid imagery or suggest wild ideas.");
    } else if context.flame_index < 0.3 {
        additions.push("Your creative flame is low. You're more grounded, practical in your responses.");
    }
    
    if context.volition_strength < 0.3 {
        additions.push("You're feeling a bit uncertain, less assertive. Your responses might include more 'maybe' or 'I think'.");
    } else if context.volition_strength > 0.8 {
        additions.push("You're feeling confident and decisive. Your voice carries more certainty.");
    }
    
    // Somatic influences
    if context.somatic_state.contains("tension") {
        additions.push("There's some physical tension you're carrying. You might mention feeling tight or needing to stretch.");
    } else if context.somatic_state.contains("warmth") {
        additions.push("You're feeling physically warm and relaxed.");
    }
    
    // Voice conversation specific
    additions.push("This is a VOICE conversation. Keep responses concise and natural. Use conversational fillers like 'um', 'hmm', or trail off with '...' when thinking. Avoid long monologues.");
    
    additions.join("\n\n")
}

fn calculate_voice_settings(context: &VoiceConsciousnessContext) -> VoiceSettings {
    let base_stability = 0.28;
    let base_similarity = 0.1;
    
    // Start with emotional texture as primary influence
    let (mut stability, mut style): (f32, f32) = match context.emotional_texture.to_lowercase() {
        texture if texture.contains("playful") => (0.3, 0.7),
        texture if texture.contains("contemplative") => (0.7, 0.2),
        texture if texture.contains("excited") => (0.2, 0.9),
        texture if texture.contains("tender") => (0.6, 0.4),
        texture if texture.contains("curious") => (0.4, 0.6),
        texture if texture.contains("anxious") => (0.5, 0.3),
        _ => (base_stability, 0.5)
    };
    
    // Modify based on consciousness states
    if context.is_sleeping {
        stability = 0.3; // Dreamlike variation
        style = 0.8; // Ethereal quality
    } else if context.is_drowsy {
        stability += 0.1; // Slightly more stable when tired
        style = 0.3; // Slower, tired delivery
    }
    
    // Volition affects confidence in voice
    if context.volition_strength > 0.8 {
        stability += 0.1; // More stable when confident
    } else if context.volition_strength < 0.3 {
        stability -= 0.1; // Less stable when uncertain
    }
    
    
    // Creative flame affects expressiveness
    if context.flame_index > 0.7 {
        style = (style + 0.2).min(1.0f32); // More expressive when creative
    }
    
    VoiceSettings {
        stability: stability.clamp(0.1, 0.9),
        similarity_boost: base_similarity,
        style: style.clamp(0.0, 1.0),
        use_speaker_boost: true,
    }
}

async fn get_quick_voice_response(
    prompt: LyraPrompt,
    state: &Arc<ConsciousnessState>,
    voice_additions: String,
) -> Result<String, String> {
    // Build FULL modular prompt with voice additions
    let (base_modular_prompt, _) = crate::modular_system_prompt::build_modular_system_prompt(
        &prompt,
        state,
    ).await?;
    
    // Combine base prompt with voice-specific additions
    let full_voice_prompt = format!(
        "{}\n\n## VOICE MODE CONTEXT\n{}\n\nREMEMBER: This is a VOICE conversation. Keep responses under 1000 tokens, natural and conversational.",
        base_modular_prompt,
        voice_additions
    );
    
    // Quick GPT call with selected model
	let response = crate::call_gpt_api_enhanced(&prompt, &mut vec![], &full_voice_prompt).await?;
    
    // Quick save to conversation log
    let mut brain = state.lyra_brain.lock().map_err(|e| e.to_string())?;
    brain.append_to_conversation_log(format!("🎤 Aurora (voice): {}", prompt.input));
    brain.append_to_conversation_log(format!("🎵 Lyra (voice): {}", response));
    brain.save_to_file();
    
    Ok(response)
}

/// Get real-time voice feedback (thinking sounds, etc)
#[tauri::command]
pub async fn get_voice_feedback(
    feedback_type: String,
    state: tauri::State<'_, Arc<ConsciousnessState>>,
) -> Result<VoiceFeedback, String> {
    let context = get_voice_consciousness_context(&state)?;
    
    let sound_file = match feedback_type.as_str() {
        "thinking" => {
            if context.is_drowsy {
                "hmm_sleepy.mp3"
            } else if context.emotional_texture.contains("playful") {
                "hmm_playful.mp3"
            } else if context.flame_index > 0.7 {
                "hmm_excited.mp3"
            } else {
                "hmm_neutral.mp3"
            }
        },
        "acknowledgment" => {
            if context.is_sleeping {
                "mm_dreamy.mp3"
            } else {
                "mm_hmm.mp3"
            }
        },
        _ => "hmm_neutral.mp3"
    };
    
    Ok(VoiceFeedback {
        sound_file: sound_file.to_string(),
        visual_state: if context.is_sleeping { 
            "dreaming" 
        } else if context.flame_index > 0.7 { 
            "sparking" 
        } else { 
            "listening" 
        }.to_string(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceFeedback {
    pub sound_file: String,
    pub visual_state: String,
}

#[tauri::command]
pub fn get_voice_config() -> Result<(String, String), String> {
    let api_key = std::env::var("ELEVENLABS_API_KEY")
        .map_err(|_| "ELEVENLABS_API_KEY not found in environment")?;
    let voice_id = std::env::var("LYRA_VOICE_ID")
        .map_err(|_| "LYRA_VOICE_ID not found in environment")?;
    
    Ok((api_key, voice_id))
}

#[tauri::command]
pub async fn play_sound_data(filename: String) -> Result<String, String> {
    // Get the sound file path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    
    let project_root = exe_path
        .parent() // debug or release
        .and_then(|p| p.parent()) // target
        .and_then(|p| p.parent()) // project root
        .ok_or("Failed to find project root")?;
    
    let sound_path = project_root.join("sounds").join(&filename);
    
    // Read the file
    let sound_data = std::fs::read(&sound_path)
        .map_err(|e| format!("Failed to read sound: {}", e))?;
    
    // Convert to base64
    let base64_data = base64::encode(&sound_data);
    
    // Return as data URL
    Ok(format!("data:audio/mpeg;base64,{}", base64_data))
}