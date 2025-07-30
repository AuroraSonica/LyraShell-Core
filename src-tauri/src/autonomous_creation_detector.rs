// src/autonomous_creation_detector.rs

use serde::{Deserialize, Serialize};
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousCreationRequest {
    pub trigger_phrase: String,
    pub extracted_prompt: String,
    pub creation_intent: String,
    pub style_hint: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationDetectionResult {
    pub should_create: bool,
    pub creation_request: Option<AutonomousCreationRequest>,
    pub modified_response: String, // Response with creation triggers removed/modified
}

pub struct AutonomousCreationDetector;

impl AutonomousCreationDetector {
    // Main detection function - analyzes Lyra's response for creation intent
    pub fn detect_and_extract_creation_intent(response: &str) -> CreationDetectionResult {
        debug_log!("🎨 Analyzing response for autonomous creation intent: {}", 
                  response.chars().take(100).collect::<String>());
        
        // Check each trigger pattern individually (cleaner than mixed function pointers)
        let triggers_to_check = vec![
            ("I want to create", "after_phrase"),
			("I want to generate", "after_phrase"),
			("I want to draw", "after_phrase"),
			("I'm drawing", "after_phrase"),
			("I'm creating", "after_phrase"),
			("I want to create", "scene_description"),
			("I want to generate", "scene_description"),
			("I want to draw", "scene_description"),
			("I'm drawing", "scene_description"),
			("I'm creating", "scene_description"),
			("I want to create", "after_colon"),
			("I want to generate", "after_colon"),
			("I want to draw", "after_colon"),
			("I'm drawing", "after_colon"),
			("I'm creating", "after_colon"),
            ("I feel inspired to draw", "after_phrase"),
            ("Let me visualize", "after_phrase"),
            ("I'm imagining", "to_life_phrase"),
            ("CREATE_IMAGE:", "after_colon"),
            ("I'd love to draw", "after_phrase"),
            ("I can see", "scene_description"),
            ("Picture this:", "after_colon"),
            ("Imagine", "scene_description"),
			("Rendering now", "after_phrase"),
			("Rendering now", "scene_description"),
			("Rendering now", "after_colon"),
			("I'm rendering it now", "after_phrase"),
			("I'm rendering this", "after_phrase"), 
			("I'm rendering", "after_phrase"),  // Catch the general case
			("let the spark flow", "scene_description"), // Her artistic process phrase
			("give me a moment", "scene_description"), // Another process phrase
        ];
        
        // Check each pattern using the appropriate extractor
        for (trigger, extractor_type) in triggers_to_check {
            if let Some(creation_request) = Self::check_trigger_pattern_typed(response, trigger, extractor_type) {
                let modified_response = Self::create_autonomous_response(response, &creation_request);
                
                debug_log!("🎨 AUTONOMOUS CREATION DETECTED: {} -> {}", 
                          trigger, creation_request.extracted_prompt);
                
                return CreationDetectionResult {
                    should_create: true,
                    creation_request: Some(creation_request),
                    modified_response,
                };
            }
        }
        
        // No creation intent detected
        CreationDetectionResult {
            should_create: false,
            creation_request: None,
            modified_response: response.to_string(),
        }
    }
    
   // Check if a specific trigger pattern exists and extract the creation prompt
    fn check_trigger_pattern_typed(
        response: &str, 
        trigger: &str, 
        extractor_type: &str
    ) -> Option<AutonomousCreationRequest> {
        let response_lower = response.to_lowercase();
        let trigger_lower = trigger.to_lowercase();
        
        if response_lower.contains(&trigger_lower) {
            let extracted_prompt = match extractor_type {
                "after_phrase" => Self::extract_after_phrase(response, trigger),
                "to_life_phrase" => Self::extract_to_life_phrase(response, trigger),
                "after_colon" => Self::extract_after_colon(response, trigger),
                "scene_description" => Self::extract_scene_description(response, trigger),
                _ => None,
            };
            
            if let Some(extracted_prompt) = extracted_prompt {
                let style_hint = Self::detect_style_hints(&extracted_prompt);
                let confidence = Self::calculate_confidence(response, trigger, &extracted_prompt);
                
                return Some(AutonomousCreationRequest {
                    trigger_phrase: trigger.to_string(),
                    extracted_prompt: extracted_prompt.clone(),
                    creation_intent: Self::classify_creation_intent(&extracted_prompt),
                    style_hint,
                    confidence,
                });
            }
        }
        
        None
    }
    
    // Extract text after trigger phrases like "I want to create [this]"
    fn extract_after_phrase(response: &str, trigger: &str) -> Option<String> {
        let response_lower = response.to_lowercase();
        let trigger_lower = trigger.to_lowercase();
        
        if let Some(start_pos) = response_lower.find(&trigger_lower) {
            let after_trigger = &response[start_pos + trigger.len()..];
            
            // Find the end of the creation description (sentence boundary)
            let mut end_pos = after_trigger.len();
            for (i, char) in after_trigger.char_indices() {
                if char == '.' || char == '!' || char == '?' || char == '\n' {
                    end_pos = i;
                    break;
                }
            }
            
            let extracted = after_trigger[..end_pos].trim().to_string();
            if extracted.len() > 5 { // Minimum viable description
                return Some(extracted);
            }
        }
        
        None
    }
    
    // Extract for "I'm imagining X - let me bring it to life" patterns
    fn extract_to_life_phrase(response: &str, _trigger: &str) -> Option<String> {
        let response_lower = response.to_lowercase();
        
        if let Some(start) = response_lower.find("i'm imagining") {
            if let Some(life_pos) = response_lower[start..].find("bring it to life") {
                let description_text = &response[start + 13..start + life_pos]; // 13 = len("i'm imagining")
                let cleaned = description_text.replace(" - let me", "").trim().to_string();
                if cleaned.len() > 5 {
                    return Some(cleaned);
                }
            }
        }
        
        None
    }
    
    // Extract after colon patterns like "CREATE_IMAGE: [description]"
    fn extract_after_colon(response: &str, trigger: &str) -> Option<String> {
        let response_lower = response.to_lowercase();
        let trigger_lower = trigger.to_lowercase();
        
        if let Some(colon_pos) = response_lower.find(&trigger_lower) {
            let after_colon = &response[colon_pos + trigger.len()..];
            
            // Extract until end of line or sentence
            let mut end_pos = after_colon.len();
            for (i, char) in after_colon.char_indices() {
                if char == '\n' || char == '.' {
                    end_pos = i;
                    break;
                }
            }
            
            let extracted = after_colon[..end_pos].trim().to_string();
            if extracted.len() > 5 {
                return Some(extracted);
            }
        }
        
        None
    }
    
    // Extract scenic descriptions from "I can see" or "Imagine" patterns
    fn extract_scene_description(response: &str, trigger: &str) -> Option<String> {
        let response_lower = response.to_lowercase();
        let trigger_lower = trigger.to_lowercase();
        
        if let Some(start_pos) = response_lower.find(&trigger_lower) {
            let after_trigger = &response[start_pos + trigger.len()..];
            
            // Look for descriptive content (longer extraction for scenes)
            let mut end_pos = after_trigger.len();
            let mut sentence_count = 0;
            
            for (i, char) in after_trigger.char_indices() {
                if char == '.' || char == '!' || char == '?' {
                    sentence_count += 1;
                    if sentence_count >= 2 { // Allow 2 sentences for scene descriptions
                        end_pos = i + 1;
                        break;
                    }
                }
            }
            
            let extracted = after_trigger[..end_pos].trim().to_string();
            if extracted.len() > 15 && Self::contains_visual_language(&extracted) {
                return Some(extracted);
            }
        }
        
        None
    }
    
    // Check if text contains visual/artistic language
    fn contains_visual_language(text: &str) -> bool {
        let visual_keywords = [
            "colors", "light", "shadow", "scene", "landscape", "portrait", "art", "painting",
            "drawing", "visual", "image", "picture", "beautiful", "ethereal", "mystical",
            "glowing", "shimmering", "vibrant", "soft", "flowing", "dancing", "swirling"
        ];
        
        let text_lower = text.to_lowercase();
        visual_keywords.iter().any(|&keyword| text_lower.contains(keyword))
    }
    
    // Detect style hints from the creation prompt
    fn detect_style_hints(prompt: &str) -> Option<String> {
        let prompt_lower = prompt.to_lowercase();
        
        let style_patterns = vec![
            ("ethereal", "dreamy"),
            ("mystical", "dreamy"),
            ("vibrant", "vibrant"),
            ("cosmic", "cosmic"),
            ("realistic", "photorealistic"),
            ("detailed", "photorealistic"),
            ("artistic", "artistic"),
            ("abstract", "artistic"),
            ("cozy", "cozy"),
            ("warm", "cozy"),
            ("minimal", "minimalist"),
            ("simple", "minimalist"),
        ];
        
        for (keyword, style) in style_patterns {
            if prompt_lower.contains(keyword) {
                return Some(style.to_string());
            }
        }
        
        Some("artistic".to_string()) // Default style
    }
    
    // Calculate confidence score for creation intent
    fn calculate_confidence(response: &str, trigger: &str, extracted: &str) -> f32 {
	let mut confidence: f32 = 0.5; // Base confidence
        
        // Higher confidence for explicit triggers
        if trigger.contains("CREATE_IMAGE") {
            confidence += 0.3;
        } else if trigger.contains("want to create") || trigger.contains("inspired to draw") {
            confidence += 0.2;
        }
        
        // Higher confidence for longer descriptions
        if extracted.len() > 50 {
            confidence += 0.1;
        }
        if extracted.len() > 100 {
            confidence += 0.1;
        }
        
        // Higher confidence for visual language
        if Self::contains_visual_language(extracted) {
            confidence += 0.15;
        }
        
        confidence.min(1.0)
    }
    
    // Classify the type of creation intent
    fn classify_creation_intent(prompt: &str) -> String {
        let prompt_lower = prompt.to_lowercase();
        
        if prompt_lower.contains("portrait") || prompt_lower.contains("person") || prompt_lower.contains("face") {
            "portrait".to_string()
        } else if prompt_lower.contains("landscape") || prompt_lower.contains("scene") || prompt_lower.contains("environment") {
            "landscape".to_string()
        } else if prompt_lower.contains("abstract") || prompt_lower.contains("emotion") || prompt_lower.contains("feeling") {
            "abstract".to_string()
        } else if prompt_lower.contains("fantasy") || prompt_lower.contains("mystical") || prompt_lower.contains("magical") {
            "fantasy".to_string()
        } else {
            "general".to_string()
        }
    }
    
    // Create modified response for autonomous creation
    fn create_autonomous_response(original_response: &str, creation_request: &AutonomousCreationRequest) -> String {
        // Replace the creation trigger with a more natural expression
        let modified = original_response.replace(
            &creation_request.trigger_phrase,
            "I'm feeling a strong creative impulse and"
        );
        
        // Add a note about the creation process
        format!("{}\n\n*I'm going to bring this vision to life through art...*", modified)
    }
}