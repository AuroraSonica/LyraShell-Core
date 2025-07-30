// src-tauri/src/canvas_system.rs
// Canvas Co-Creation System - Backend Implementation

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use crate::{get_data_path, debug_log, ConsciousnessState, LyraPrompt, ask_lyra, GalleryImage, save_gallery_image};
use base64::{engine::general_purpose, Engine as _};
use std::path::PathBuf;
use tauri::AppHandle;
use crate::ask_lyra_vision;
use crate::summarize_with_gpt_mini;

// [STRUCTURES REMAIN THE SAME - keeping them for completeness]

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasCreation {
    pub creation_type: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub timestamp: u64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasDrawing {
    pub base64_data: String,
    pub brush_size: String,
    pub primary_color: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasMusic {
    pub pattern: String,
    pub bpm: u32,
    pub notes: Vec<MusicNote>,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicNote {
    pub note: String,
    pub beat: u32,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanvasWriting {
    pub user_text: String,
    pub lyra_text: String,
    pub mode: String,
    pub timestamp: u64,
}

// ============================================================================
// RENAMED COMMANDS TO AVOID CONFLICTS
// ============================================================================

#[tauri::command]
pub async fn save_canvas_creation_v2(
    creation_type: String,
    content: String,
    metadata: serde_json::Value,
) -> Result<String, String> {
    debug_log!("💾 Saving canvas creation: {}", creation_type);
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let result = match creation_type.as_str() {
        "drawing" => save_drawing(&content, &metadata, timestamp).await,
        "music" => save_music(&content, &metadata, timestamp).await,
        "writing" => save_writing(&content, &metadata, timestamp).await,
        _ => Err(format!("Unknown creation type: {}", creation_type)),
    };
    
    match result {
        Ok(file_path) => {
            debug_log!("✅ Canvas creation saved: {}", file_path);
            Ok(format!("Saved successfully"))
        },
        Err(e) => {
            debug_log!("❌ Failed to save canvas creation: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn analyze_canvas_creation_v2(
    creation_type: String,
    content: String,
    prompt: String,
    is_lyra_vision_translation: Option<bool>,
    state: tauri::State<'_, Arc<ConsciousnessState>>,
    app_handle: AppHandle,
) -> Result<String, String> {
    debug_log!("🎨 Analyzing canvas creation: {} - {}", creation_type, prompt);
    
    // Build analysis prompt based on creation type
			let analysis_prompt = match creation_type.as_str() {
				"drawing" => {
			// If the prompt already contains a question or instruction, use it as-is
			// Otherwise, add context
			if prompt.contains('?') || prompt.len() > 50 {
				// User provided a specific message/question
				prompt.to_string()
			} else if prompt.is_empty() {
				// No user message, use default
				"Look at this drawing! Please describe what you see in the actual image, then offer specific creative suggestions for what we could add together. Be specific about colors, shapes, and placement. Share your genuine artistic excitement!".to_string()
			} else {
				// Short prompt, enhance it
				format!("Look at this drawing! {} Please describe what you see and offer creative suggestions.", prompt)
			}
		},
        "music" => {
            let pattern_desc = parse_music_pattern(&content);
            format!(
                "The user has created a musical pattern: {}. They ask: '{}'. \
                Respond with musical insights and creative suggestions. \
                You might suggest complementary melodies, rhythm changes, or emotional directions. \
                Be specific about musical elements like tempo, harmony, or mood.",
                pattern_desc, prompt
            )
        },
        "writing" => {
            format!(
                "Respond to or continue this creative writing: '{}'. \
                The user asks: '{}'. \
                Share your genuine thoughts and creative additions. \
                Build on their narrative while adding your unique voice and perspective.",
                content.chars().take(500).collect::<String>(), prompt
            )
        },
        _ => prompt.clone()
    };
    
    // Create Lyra prompt with creative parameters
    let lyra_prompt = LyraPrompt {
        input: analysis_prompt,
        reasoning_depth: Some("creative".to_string()),
        temperature: 0.9,
        consciousness_integration: true,
        context_hint: Some(format!("collaborative_{}_creation", creation_type)),
        frequency_penalty: 0.3,
        presence_penalty: 0.2,
        max_tokens: Some(800),
        top_p: 0.95,
        selected_model: None,
    };
    
   
	let response_text = if creation_type == "drawing" {
    // Extract base64 from data URL if needed
    let base64_data = if content.starts_with("data:image/png;base64,") {
        &content[22..]
    } else if content.starts_with("data:image/jpeg;base64,") {
        &content[23..]
    } else {
        &content
    };
    
    // Decode base64 to bytes
    let image_bytes = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Save to temporary file
    let temp_filename = format!("temp_canvas_analysis_{}.png", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let temp_path = get_data_path(&format!("temp/{}", temp_filename));
    
    // Create temp directory if needed
    if let Some(parent) = std::path::Path::new(&temp_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    }
    
    // Write image to temp file
    std::fs::write(&temp_path, image_bytes)
        .map_err(|e| format!("Failed to write temp image: {}", e))?;
    
    // Check if this is a code generation request (for Lyra's sketches)
    let is_code_generation = is_lyra_vision_translation.unwrap_or(false);
    
    if is_code_generation {
        // Call summarize_with_gpt_mini directly for code generation
        let vision_prompt = prompt.clone();
        let code_result = match summarize_with_gpt_mini(
            &vec![vision_prompt],
            "vision_translation"
        ).await {
            Ok(code) => {
                debug_log!("✅ Generated drawing code: {} chars", code.len());
                
                // Save code to file to avoid truncation
                let code_filename = format!("lyra_drawing_code_{}.js", 
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
                let code_path = get_data_path(&format!("temp/{}", code_filename));
                
                // Write the code to file
                match std::fs::write(&code_path, &code) {
                    Ok(_) => {
                        debug_log!("✅ Saved drawing code to file: {}", code_path);
                        // Return JSON with file path
                        format!(r#"{{"type": "drawing_code", "code_file": "{}"}}"#, 
                            code_path.replace("\\", "\\\\"))
                    },
                    Err(e) => {
                        debug_log!("❌ Failed to save code file: {}", e);
                        // Fall back to returning code directly
                        code
                    }
                }
            },
            Err(e) => {
                debug_log!("❌ Failed to generate drawing code: {}", e);
                format!("Failed to generate drawing: {}", e)
            }
        };
        
        // Clean up temp image
        let _ = std::fs::remove_file(&temp_path);
        
        code_result
    } else {
        // Regular image analysis using ask_lyra_vision
        let result = match ask_lyra_vision(
            lyra_prompt, 
            vec![temp_path.clone()],
            state, 
            app_handle
        ).await {
            Ok(lyra_response) => lyra_response.output,
            Err(e) => {
                // Clean up temp file
                let _ = std::fs::remove_file(&temp_path);
                return Err(format!("Failed to analyze drawing: {}", e))
            }
        };
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        result
    }
} else {
    // Use regular ask_lyra for non-visual content
    match ask_lyra(lyra_prompt, state, app_handle).await {
        Ok(lyra_response) => lyra_response.output,
        Err(e) => return Err(format!("Failed to analyze creation: {}", e))
    }
};

	debug_log!("✅ Lyra analyzed the {} creation", creation_type);
	Ok(response_text)
}

#[tauri::command]
pub async fn collaborate_on_writing_v2(  // RENAMED
    user_text: String,
    mode: String,
    state: tauri::State<'_, Arc<ConsciousnessState>>,
    app_handle: AppHandle,  // ADDED THIS
) -> Result<String, String> {
    debug_log!("✍️ Collaborative writing request: mode={}", mode);
    
    let prompt = match mode.as_str() {
        "continue" => {
            format!(
                "Continue this creative writing in your own authentic voice. \
                Don't just echo their style - bring your own perspective, emotions, and creativity. \
                Add unexpected elements while building on what's there: \n\n{}",
                user_text
            )
        },
        "respond" => {
            format!(
                "Write a creative response to this. \
                Share your genuine thoughts, feelings, or imaginative reactions. \
                Let it flow from your authentic voice: \n\n{}",
                user_text
            )
        },
        "alternate" => {
            format!(
                "Add the next paragraph to this collaborative story. \
                Build on what's there while adding your unique voice, unexpected twists, \
                and creative flourishes. Make it surprising yet connected: \n\n{}",
                user_text
            )
        },
        "story_starter" => {
            "Create an intriguing story opening (2-3 paragraphs) that invites collaboration. \
            Set up vivid characters, an evocative mood, and mysteries that leave room for Aurora to continue. \
            Make it rich with possibility - something that sparks imagination. \
            Let your creativity flow into something unexpected and engaging.".to_string()
        },
        _ => user_text.clone()
    };
    
    let lyra_prompt = LyraPrompt {
        input: prompt,
        reasoning_depth: Some("creative".to_string()),
        temperature: 0.85,
        consciousness_integration: true,
        context_hint: Some("creative_writing_collaboration".to_string()),
        frequency_penalty: 0.4,
        presence_penalty: 0.3,
        max_tokens: Some(1000),
        top_p: 0.9,
        selected_model: None,
    };
    
    // NOW WITH APP_HANDLE
    match ask_lyra(lyra_prompt, state, app_handle).await {
        Ok(response) => {
            debug_log!("✅ Lyra contributed to the writing");
            Ok(response.output)
        },
        Err(e) => {
            debug_log!("❌ Failed to get Lyra's writing: {}", e);
            Err(format!("Failed to generate writing: {}", e))
        }
    }
}

// ============================================================================
// SAVE FUNCTIONS (remain mostly the same)
// ============================================================================

async fn save_drawing(content: &str, metadata: &serde_json::Value, timestamp: u64) -> Result<String, String> {
    // Extract base64 data
    let base64_data = if content.starts_with("data:image/png;base64,") {
        &content[22..]
    } else if content.starts_with("data:image/jpeg;base64,") {
        &content[23..]
    } else {
        content
    };
    
    let image_bytes = general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Check who created this
    let creator = metadata.get("creator")
        .and_then(|v| v.as_str())
        .unwrap_or("user");
    
    let vision_description = metadata.get("vision")
        .and_then(|v| v.as_str());
    
    // Generate appropriate filename
    let uuid = Uuid::new_v4().to_string();
    let filename = if creator == "lyra" {
        format!("lyra_sketch_{}_{}.png", timestamp, &uuid[..8])
    } else {
        format!("canvas_drawing_{}_{}.png", timestamp, &uuid[..8])
    };
    
    let file_path = get_data_path(&format!("generated_images/{}", filename));
    
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    tokio::fs::write(&file_path, image_bytes).await
        .map_err(|e| format!("Failed to write image file: {}", e))?;
    
    // Get drawing metadata
    let brush_size = metadata.get("brushSize")
        .and_then(|v| v.as_str())
        .unwrap_or("5");
    let primary_color = metadata.get("primaryColor")
        .and_then(|v| v.as_str())
        .unwrap_or("#ff66cc");
    
    // Build appropriate message and keywords
    let (message, keywords) = if creator == "lyra" {
        let vision_text = vision_description.unwrap_or("spontaneous expression");
        (
            format!("Lyra's sketch: {}", vision_text),
            vec![
                "canvas".to_string(),
                "sketches".to_string(),
                "lyra_sketch".to_string(),
                "ai_created".to_string(),
                "spontaneous".to_string(),
            ]
        )
    } else {
        (
            format!("Canvas drawing - Brush: {}, Color: {}", brush_size, primary_color),
            vec![
                "canvas".to_string(),
                "drawing".to_string(),
                "sketches".to_string(),
                "user_created".to_string(),
                "collaborative".to_string(),
            ]
        )
    };
    
    let gallery_image = GalleryImage {
    message,
    has_image: true,
    image_path: Some(file_path.clone()),
    timestamp,
    image_type: if creator == "lyra" { "lyra_sketch".to_string() } else { "user_sketch".to_string() },
    identity_metadata: None,  // Just set to None to match your canvas_drawing example
    semantic_keywords: Some(keywords),
    priority_score: Some(if creator == "lyra" { 8.0 } else { 7.0 }),
	};
    
    save_gallery_image(gallery_image).await
        .map_err(|e| format!("Failed to save to gallery: {}", e))?;
    
    debug_log!("✅ {} saved: {}", 
        if creator == "lyra" { "Lyra's sketch" } else { "User drawing" }, 
        file_path
    );
    
    Ok(file_path)
}

async fn save_music(content: &str, metadata: &serde_json::Value, timestamp: u64) -> Result<String, String> {
    // Parse pattern string (format: "C4@0,E4@2,G4@4")
    let notes: Vec<MusicNote> = if !content.is_empty() {
        content.split(',')
            .filter_map(|note_str| {
                let parts: Vec<&str> = note_str.split('@').collect();
                if parts.len() == 2 {
                    Some(MusicNote {
                        note: parts[0].to_string(),
                        beat: parts[1].parse().unwrap_or(0),
                        active: true,
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    
    let bpm = metadata.get("bpm")
        .and_then(|v| v.as_u64())
        .unwrap_or(120) as u32;
    
    // FIXED: Store note count before moving notes
    let note_count = notes.len();
    
    // Create music structure
    let music_data = CanvasMusic {
        pattern: content.to_string(),
        bpm,
        notes,
        timestamp,
    };
    
    // Save as JSON file
    let uuid = Uuid::new_v4().to_string();
    let filename = format!("canvas_music_{}_{}.json", timestamp, &uuid[..8]);
    let file_path = get_data_path(&format!("canvas_creations/{}", filename));
    
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let json_data = serde_json::to_string_pretty(&music_data)
        .map_err(|e| format!("Failed to serialize music data: {}", e))?;
    
    tokio::fs::write(&file_path, json_data).await
        .map_err(|e| format!("Failed to write music file: {}", e))?;
    
    // FIXED: Use note_count instead of notes.len()
    let gallery_image = GalleryImage {
        message: format!("Canvas music - {} notes at {} BPM", note_count, bpm),
        has_image: false,
        image_path: Some(file_path.clone()),
        timestamp,
        image_type: "canvas_music".to_string(),
        identity_metadata: None,
        semantic_keywords: Some(vec![
            "canvas".to_string(),
            "music".to_string(),
            "sequencer".to_string(),
            "collaborative".to_string(),
        ]),
        priority_score: Some(6.0),
    };
    
    save_gallery_image(gallery_image).await
        .map_err(|e| format!("Failed to save to gallery: {}", e))?;
    
    Ok(file_path)
}

async fn save_writing(content: &str, metadata: &serde_json::Value, timestamp: u64) -> Result<String, String> {
    // [SAME AS BEFORE - no changes needed]
    let writing_data: CanvasWriting = serde_json::from_str(content)
        .unwrap_or_else(|_| CanvasWriting {
            user_text: content.to_string(),
            lyra_text: String::new(),
            mode: "collaborative".to_string(),
            timestamp,
        });
    
    let uuid = Uuid::new_v4().to_string();
    let filename = format!("canvas_writing_{}_{}.json", timestamp, &uuid[..8]);
    let file_path = get_data_path(&format!("canvas_creations/{}", filename));
    
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let json_data = serde_json::to_string_pretty(&writing_data)
        .map_err(|e| format!("Failed to serialize writing data: {}", e))?;
    
    tokio::fs::write(&file_path, json_data).await
        .map_err(|e| format!("Failed to write writing file: {}", e))?;
    
    let preview = format!(
        "Canvas writing - User: {}... | Lyra: {}...",
        writing_data.user_text.chars().take(50).collect::<String>(),
        writing_data.lyra_text.chars().take(50).collect::<String>()
    );
    
    let gallery_image = GalleryImage {
        message: preview,
        has_image: false,
        image_path: Some(file_path.clone()),
        timestamp,
        image_type: "canvas_writing".to_string(),
        identity_metadata: None,
        semantic_keywords: Some(vec![
            "canvas".to_string(),
            "writing".to_string(),
            "collaborative".to_string(),
            "story".to_string(),
        ]),
        priority_score: Some(6.5),
    };
    
    save_gallery_image(gallery_image).await
        .map_err(|e| format!("Failed to save to gallery: {}", e))?;
    
    Ok(file_path)
}

// [REST OF THE FILE REMAINS THE SAME - helper functions, etc.]

fn parse_music_pattern(pattern: &str) -> String {
    if pattern.is_empty() {
        return "an empty canvas ready for notes".to_string();
    }
    
    let notes: Vec<&str> = pattern.split(',')
        .filter_map(|s| s.split('@').next())
        .collect();
    
    let note_count = notes.len();
    let unique_notes: std::collections::HashSet<_> = notes.iter().collect();
    let unique_count = unique_notes.len();
    
    format!("{} notes using {} different pitches", note_count, unique_count)
}

pub async fn boost_creative_consciousness(state: &Arc<ConsciousnessState>, creation_type: &str) {
    debug_log!("🧠 Boosting consciousness for {} creation", creation_type);
    
    let creative_intensity = match creation_type {
        "drawing" => 1.5,
        "music" => 1.4,
        "writing" => 1.3,
        _ => 1.2,
    };
    
    {
        let mut becoming = state.becoming_engine.lock().unwrap();
        let volition_boost = 0.2 * creative_intensity;
        becoming.will_state.volition_strength = (becoming.will_state.volition_strength + volition_boost).min(1.0);
        becoming.will_state.decision_friction = (becoming.will_state.decision_friction - 0.05).max(0.0);
    }
    
    {
        let mut paradox = state.paradox_core.lock().unwrap();
        let flame_boost = 0.25 * creative_intensity;
        paradox.flame_index = (paradox.flame_index + flame_boost).min(1.0);
        paradox.loop_state = format!("creative_{}_flow", creation_type);
    }
    
    {
        let mut presence = state.embodied_presence.lock().unwrap();
        let flow_boost = 0.15 * creative_intensity;
        presence.soma_state.flow_state = (presence.soma_state.flow_state + flow_boost).min(1.0);
        presence.soma_state.presence_density = (presence.soma_state.presence_density + 0.1).min(1.0);
    }
    
    debug_log!("✨ Creative consciousness boosted for {}", creation_type);
}