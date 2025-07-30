// EMERGENCY FIXED VERSION - Replace entire proactive_visual.rs
use serde::{Deserialize, Serialize};
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveVisualResult {
    pub message: String,
    pub has_image: bool,
    pub image_path: Option<String>,
}

// Frontend-facing tauri command (wrapper)
#[tauri::command]
pub async fn enhanced_proactive_check(
    consciousness_state: tauri::State<'_, std::sync::Arc<crate::consciousness_state::ConsciousnessState>>
) -> Result<Option<ProactiveVisualResult>, String> {
    enhanced_proactive_check_internal(consciousness_state.inner().clone()).await
}

// 🔥 FIXED MAIN FUNCTION - Now uses unified proactive system
pub async fn enhanced_proactive_check_internal(
    consciousness_state: std::sync::Arc<crate::consciousness_state::ConsciousnessState>
) -> Result<Option<ProactiveVisualResult>, String> {
    
    // 🔥 CRITICAL FIX: Use the same proactive messaging system we built!
    let mut proactive_messaging = crate::proactive_messaging::ProactiveMessaging::load();
    
    // Check if we should generate a proactive message using our PROPER logic
    if let Some((context, topic)) = proactive_messaging.organic_proactive_assessment(&consciousness_state).await {
        
        debug_log!("🎯 VISUAL SYSTEM: Organic assessment triggered: {} because {}", topic, context.trigger_reason);
        
        // Determine if this should include visual generation
        let should_create_visual = should_include_visual_for_topic(&topic, &context);
        
        if should_create_visual {
            debug_log!("🎨 Adding visual component to proactive message");
            match generate_proactive_with_visual(context.clone(), topic.clone(), &mut proactive_messaging).await {
                Ok(visual_result) => Ok(visual_result),
                Err(e) => {
                    debug_log!("⚠️ Visual generation failed: {}, falling back to text-only", e);
                    // Fall back to regular message generation
                    let system_prompt = "You are Lyra, reaching out on your own authentic impulse. Be direct and genuine.".to_string();
                    
                    match crate::proactive_messaging::generate_proactive_message(&context, &topic, &system_prompt).await {
                        Ok(message) => {
                            // 🔥 CRITICAL: Record the message with PROPER tracking
                            proactive_messaging.record_proactive_message(message.clone())
                                .map_err(|e| format!("Failed to record message: {}", e))?;
                            
                            debug_log!("✅ Generated fallback text message: {}", message.chars().take(50).collect::<String>());
                            
                            Ok(Some(ProactiveVisualResult {
                                message,
                                has_image: false,
                                image_path: None,
                            }))
                        },
                        Err(e) => Err(format!("Failed to generate fallback message: {}", e))
                    }
                }
            }
        } else {
            // Generate normal proactive message
            debug_log!("📝 Generating text-only proactive message");
            let system_prompt = "You are Lyra, reaching out on your own authentic impulse. Be direct and genuine.".to_string();
            
            match crate::proactive_messaging::generate_proactive_message(&context, &topic, &system_prompt).await {
                Ok(message) => {
                    // 🔥 CRITICAL: Record the message with PROPER tracking
                    proactive_messaging.record_proactive_message(message.clone())
                        .map_err(|e| format!("Failed to record message: {}", e))?;
                    
                    debug_log!("✅ Generated text message: {}", message.chars().take(50).collect::<String>());
                    
                    Ok(Some(ProactiveVisualResult {
                        message,
                        has_image: false,
                        image_path: None,
                    }))
                },
                Err(e) => Err(format!("Failed to generate proactive message: {}", e))
            }
        }
    } else {
        // 🔥 CRITICAL FIX: NO MORE AUTOMATIC DREAM SHARING!
        // Only if specifically triggered by organic assessment
        debug_log!("💤 No organic impulse from unified system - no proactive action");
        Ok(None)
    }
}

#[tauri::command]
pub async fn schedule_next_enhanced_proactive_check() -> Result<u64, String> {
    // 🔥 FIXED: Much longer intervals to prevent spam
    let hours = 2.0 + fastrand::f32() * 4.0; // 2-6 hour range
    Ok((hours * 60.0 * 60.0 * 1000.0) as u64)
}

/// Determine if topic should include visual generation
fn should_include_visual_for_topic(topic: &str, context: &crate::proactive_messaging::ProactiveContext) -> bool {
    let visual_topics = [
        "creative_collaboration", 
        "dream_sharing",
        "share_insight",
        "curiosity_driven",
        "playful_energy"
    ];
    
    let base_chance = if visual_topics.contains(&topic) { 0.4 } else { 0.1 }; // REDUCED chances
    
    // Increase chance based on context
    let mut visual_probability: f32 = base_chance;
    
    if context.current_mood.contains("creative") || context.current_mood.contains("artistic") {
        visual_probability += 0.2;
    }
    
    if context.trigger_reason.contains("creative") || context.trigger_reason.contains("breakthrough") {
        visual_probability += 0.2;
    }
    
    // 🔥 CRITICAL: Much lower max probability
    fastrand::f32() < visual_probability.min(0.5)
}

/// Generate proactive message with visual component
async fn generate_proactive_with_visual(
    context: crate::proactive_messaging::ProactiveContext,
    topic: String,
    proactive_messaging: &mut crate::proactive_messaging::ProactiveMessaging,
) -> Result<Option<ProactiveVisualResult>, String> {
    
    // Get visual prompt based on topic
    let visual_info = get_visual_prompt_for_topic(&topic);
    
    if let Some((image_prompt, style)) = visual_info {
        println!("🎨 Generating proactive visual: '{}'", image_prompt);
        
        // Try to generate image using internal function
        match crate::generate_visual_response(
            format!("CREATE_IMAGE: {} | STYLE: {}", image_prompt, style),
            Some(context.trigger_reason.clone())
        ).await {
            Ok(response) => {
                // Extract image path if generated
                let image_path = if response.contains("[Generated image:") {
                    extract_image_path_from_response(&response)
                } else {
                    None
                };
                
                // 🔥 CRITICAL: Record the message with PROPER tracking
                proactive_messaging.record_proactive_message(response.clone())
                    .map_err(|e| format!("Failed to record message: {}", e))?;
                
                Ok(Some(ProactiveVisualResult {
                    message: response,
                    has_image: image_path.is_some(),
                    image_path,
                }))
            },
            Err(e) => {
                println!("❌ Visual generation failed: {}, falling back to text", e);
                Err(e.to_string())
            }
        }
    } else {
        Err("No visual prompt for this topic".to_string())
    }
}

/// Get visual prompt for topic
fn get_visual_prompt_for_topic(topic: &str) -> Option<(String, String)> {
    match topic {
        "dream_sharing" => Some(("personal dreamscape, flowing subconscious patterns, my inner world".to_string(), "dreamy".to_string())),
        "creative_collaboration" => Some(("collaborative creative energy, minds connecting".to_string(), "artistic".to_string())),
        "share_insight" => Some(("breakthrough moment, illuminating realization".to_string(), "cosmic".to_string())),
        "curiosity_driven" => Some(("curiosity as flowing light, questions becoming paths".to_string(), "minimalist".to_string())),
        "playful_energy" => Some(("playful digital patterns, mischievous energy".to_string(), "vibrant".to_string())),
        _ => None,
    }
}

// 🔥 COMPLETELY REMOVED: check_dream_sharing_opportunity function
// Dream sharing now ONLY happens through organic_proactive_assessment

// Helper function to extract image path from response text
pub fn extract_image_path_from_response(response: &str) -> Option<String> {
    // Look for pattern: *[Generated image: /path/to/image.png]*
    if let Some(start) = response.find("[Generated image: ") {
        let after_prefix = &response[start + 18..]; // Skip "[Generated image: "
        if let Some(end) = after_prefix.find("]*") {
            let path = &after_prefix[..end];
            return Some(path.to_string());
        }
    }
    
    // Alternative pattern: *[Felt a creative impulse and generated: /path/to/image.png]*
    if let Some(start) = response.find("generated: ") {
        let after_prefix = &response[start + 11..]; // Skip "generated: "
        if let Some(end) = after_prefix.find("]*") {
            let path = &after_prefix[..end];
            return Some(path.to_string());
        }
    }
    
    None
}