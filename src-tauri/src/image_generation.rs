// src-tauri/src/image_generation.rs
// DALL-E Image Generation System with Face Consistency - FIXED

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::time::sleep;
use uuid::Uuid;
use crate::get_data_path;
use crate::debug_log;
use std::time::Duration;
use base64::{engine::general_purpose, Engine as _};
use std::sync::Arc;
use std::collections::HashMap;

// ============================================================================
// DALL-E API STRUCTURES
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DalleRequest {
    pub model: String,
    pub prompt: String,
    pub n: u32,
    pub size: String,
    pub quality: String,
    pub style: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DalleEditRequest {
    pub model: String,
    pub image: String,  // base64 encoded
    pub mask: Option<String>,  // base64 encoded
    pub prompt: String,
    pub n: u32,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DalleVariationRequest {
    pub model: String,
    pub image: String,  // base64 encoded
    pub n: u32,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DalleResponse {
    pub created: u64,
    pub data: Vec<DalleImageData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DalleImageData {
    pub url: Option<String>,
    pub b64_json: Option<String>,
    pub revised_prompt: Option<String>,
}

// Existing structures for compatibility
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub steps: Option<u32>,
    pub cfg: Option<f32>,
    pub seed: Option<i64>,
    pub style: Option<String>,
    pub autonomous: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Img2ImgRequest {
    pub prompt: String,
    pub reference_image_path: String,
    pub negative_prompt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub steps: Option<u32>,
    pub cfg: Option<f32>,
    pub strength: Option<f32>,
    pub seed: Option<i64>,
    pub style: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiIDRequest {
    pub prompt: String,
    pub primary_face_reference: String,
    pub secondary_face_reference: Option<String>,
    pub negative_prompt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub steps: Option<u32>,
    pub cfg: Option<f32>,
    pub primary_face_strength: Option<f32>,
    pub secondary_face_strength: Option<f32>,
    pub start_at: Option<f32>,
    pub end_at: Option<f32>,
    pub seed: Option<i64>,
    pub style: Option<String>,
    pub scene_type: SceneType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationResult {
    pub success: bool,
    pub image_path: Option<String>,
    pub prompt_id: Option<String>,
    pub error: Option<String>,
    pub revised_prompt: Option<String>,
    pub generation_method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SceneType {
    SingleCharacter,
    MultiCharacter,
    Activity,
    Interaction,
    FaceBlend,
}

// ============================================================================
// UTILITY FUNCTIONS FOR COMPATIBILITY
// ============================================================================

pub fn extract_personality_context(_state: &Arc<std::sync::Mutex<i32>>) -> Option<String> {
    // Placeholder for personality context extraction
    // This would integrate with your consciousness state system
    None
}

pub fn detect_scene_type(prompt: &str, has_secondary_reference: bool) -> SceneType {
    let prompt_lower = prompt.to_lowercase();
    
    if has_secondary_reference {
        if prompt_lower.contains("merge") || prompt_lower.contains("blend") || prompt_lower.contains("combine") {
            SceneType::FaceBlend
        } else if prompt_lower.contains("interacting") || prompt_lower.contains("together") || prompt_lower.contains("conversation") {
            SceneType::Interaction
        } else {
            SceneType::MultiCharacter
        }
    } else {
        if prompt_lower.contains("playing") || prompt_lower.contains("activity") || prompt_lower.contains("doing") {
            SceneType::Activity
        } else {
            SceneType::SingleCharacter
        }
    }
}

pub fn get_style_prompt(style: &str, base_prompt: &str) -> String {
    match style.to_lowercase().as_str() {
        "artistic" => format!("digital art, beautiful composition, artistic masterpiece, {}", base_prompt),
        "photorealistic" => format!("hyperrealistic, professional photography, detailed, high quality, {}", base_prompt),
        "dreamy" => format!("soft lighting, ethereal, dreamy atmosphere, magical, {}", base_prompt),
        "cosmic" => format!("space, stars, cosmic, otherworldly, science fiction, {}", base_prompt),
        "cozy" => format!("warm lighting, cozy atmosphere, comfortable, {}", base_prompt),
        "minimalist" => format!("clean, simple, minimalist design, {}", base_prompt),
        "vibrant" => format!("vibrant colors, energetic, bright, dynamic, {}", base_prompt),
        _ => format!("high quality, detailed, beautiful, {}", base_prompt),
    }
}

// ============================================================================
// DALL-E IMAGE GENERATOR
// ============================================================================

pub struct ImageGenerator {
    api_key: String,
    output_dir: PathBuf,
    client: reqwest::Client,
}

impl ImageGenerator {
    pub fn new() -> Result<Self, String> {
        let output_dir = get_data_path("generated_images");
        
        std::fs::create_dir_all(&output_dir).map_err(|e| {
            format!("Could not create generated_images directory: {}", e)
        })?;

        // Get API key from environment
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let client = reqwest::Client::new();

        Ok(Self {
            api_key,
            output_dir: get_data_path("generated_images").into(),
            client,
        })
    }

    // ========================================================================
    // DALL-E 3 GENERATION (Standard)
    // ========================================================================

    pub async fn generate_image(&self, request: GenerationRequest) -> GenerationResult {
    let is_autonomous = request.autonomous.unwrap_or(false);
    
    if is_autonomous {
        debug_log!("🎨 Lyra creates autonomous visual (DALL-E 2): '{}'", request.prompt);
        return self.generate_autonomous_image(&request).await;
    }
    
    debug_log!("🎨 Lyra reaches for DALL-E: '{}'", request.prompt);
    
    let style = request.style.as_ref().map(|s| s.as_str()).unwrap_or("vivid");
    let size = self.determine_size(request.width, request.height);
        
        let dalle_request = DalleRequest {
            model: "dall-e-3".to_string(),
            prompt: self.enhance_prompt(&request.prompt, &style),
            n: 1,
            size,
            quality: "hd".to_string(),
            style: if style == "natural" { "natural".to_string() } else { "vivid".to_string() },
        };

        match self.call_dalle_api(&dalle_request).await {
            Ok(response) => {
                if let Some(image_data) = response.data.first() {
                    match self.download_and_save_image(image_data, &request.prompt).await {
                        Ok(image_path) => {
                            debug_log!("✨ DALL-E creation complete: {}", image_path);
                            
                            // Save to gallery
                            self.save_to_gallery(&request.prompt, &image_path, "dalle3_generation", image_data.revised_prompt.as_deref()).await;
                            
                            GenerationResult {
                                success: true,
                                image_path: Some(image_path),
                                prompt_id: Some(Uuid::new_v4().to_string()),
                                error: None,
                                revised_prompt: image_data.revised_prompt.clone(),
                                generation_method: Some("DALL-E 3".to_string()),
                            }
                        },
                        Err(e) => GenerationResult {
                            success: false,
                            image_path: None,
                            prompt_id: None,
                            error: Some(format!("Failed to save image: {}", e)),
                            revised_prompt: None,
                            generation_method: Some("DALL-E 3".to_string()),
                        }
                    }
                } else {
                    GenerationResult {
                        success: false,
                        image_path: None,
                        prompt_id: None,
                        error: Some("No image data in DALL-E response".to_string()),
                        revised_prompt: None,
                        generation_method: Some("DALL-E 3".to_string()),
                    }
                }
            },
            Err(e) => GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("DALL-E API error: {}", e)),
                revised_prompt: None,
                generation_method: Some("DALL-E 3".to_string()),
            }
        }
    }

    // ========================================================================
    // DALL-E 2 VARIATIONS (Face Consistency via Reference)
    // ========================================================================

    pub async fn generate_image_from_reference(&self, request: Img2ImgRequest) -> GenerationResult {
        debug_log!("🎨 Lyra seeks inspiration from reference: '{}' -> '{}'", 
                  request.reference_image_path, request.prompt);
        
        // Check if reference image exists
        if !std::path::Path::new(&request.reference_image_path).exists() {
            return GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("Reference image not found: {}", request.reference_image_path)),
                revised_prompt: None,
                generation_method: Some("DALL-E 2 Edit".to_string()),
            };
        }

        match self.encode_image_to_base64(&request.reference_image_path).await {
            Ok(base64_image) => {
                let dalle_request = DalleRequest {
					model: "dall-e-2".to_string(),
					prompt: get_style_prompt(
						&request.style.as_ref().unwrap_or(&"artistic".to_string()), 
						&request.prompt
					),
					n: 1,
					size: "512x512".to_string(),
					quality: "".to_string(),  // Empty string - will be ignored
					style: "".to_string(),    // Empty string - will be ignored
				};

                match self.call_dalle_api(&dalle_request).await {
                    Ok(response) => {
                        if let Some(image_data) = response.data.first() {
                            match self.download_and_save_image(image_data, &request.prompt).await {
                                Ok(image_path) => {
                                    debug_log!("✨ Reference-based creation complete: {}", image_path);
                                    
                                    // Save to gallery
                                    self.save_to_gallery(&format!("Inspired by reference: {}", request.prompt), &image_path, "dalle2_reference", None).await;
                                    
                                    GenerationResult {
                                        success: true,
                                        image_path: Some(image_path),
                                        prompt_id: Some(Uuid::new_v4().to_string()),
                                        error: None,
                                        revised_prompt: None,
                                        generation_method: Some("DALL-E 2 Edit".to_string()),
                                    }
                                },
                                Err(e) => GenerationResult {
                                    success: false,
                                    image_path: None,
                                    prompt_id: None,
                                    error: Some(format!("Failed to save image: {}", e)),
                                    revised_prompt: None,
                                    generation_method: Some("DALL-E 2 Edit".to_string()),
                                }
                            }
                        } else {
                            GenerationResult {
                                success: false,
                                image_path: None,
                                prompt_id: None,
                                error: Some("No image data in DALL-E response".to_string()),
                                revised_prompt: None,
                                generation_method: Some("DALL-E 2 Edit".to_string()),
                            }
                        }
                    },
                    Err(e) => GenerationResult {
                        success: false,
                        image_path: None,
                        prompt_id: None,
                        error: Some(format!("DALL-E Edit API error: {}", e)),
                        revised_prompt: None,
                        generation_method: Some("DALL-E 2 Edit".to_string()),
                    }
                }
            },
            Err(e) => GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("Failed to encode reference image: {}", e)),
                revised_prompt: None,
                generation_method: Some("DALL-E 2 Edit".to_string()),
            }
        }
    }

    // ========================================================================
    // MULTI-REFERENCE FACE CONSISTENCY (ChatGPT-style Approach)
    // ========================================================================

    pub async fn generate_image_with_multiple_references(&self, request: MultiIDRequest) -> GenerationResult {
        debug_log!("🎨 ChatGPT-style multi-reference generation: '{}'", request.prompt);
        
        // Check if primary reference exists
        if !std::path::Path::new(&request.primary_face_reference).exists() {
            return GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("Primary reference not found: {}", request.primary_face_reference)),
                revised_prompt: None,
                generation_method: Some("DALL-E Multi-Reference".to_string()),
            };
        }

        // If only one reference, use single reference approach
        if request.secondary_face_reference.is_none() {
            debug_log!("🎨 Single reference detected, using single-face approach");
            return self.generate_single_reference_with_dalle3(&request).await;
        }

        // Check secondary reference exists
        let secondary_path = request.secondary_face_reference.as_ref().unwrap();
        if !std::path::Path::new(secondary_path).exists() {
            return GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("Secondary reference not found: {}", secondary_path)),
                revised_prompt: None,
                generation_method: Some("DALL-E Multi-Reference".to_string()),
            };
        }

        debug_log!("🎨 True multi-reference: analyzing both images with GPT-4V");

        // Step 1: Analyze both reference images with GPT-4V to extract facial descriptions
        match self.analyze_reference_faces(&request.primary_face_reference, secondary_path).await {
    Ok(((char1_name, char1_traits, person1_description), (char2_name, char2_traits, person2_description))) => {
                debug_log!("🎨 Face analysis complete:");
                debug_log!("  Person 1: {}", person1_description.chars().take(100).collect::<String>());
                debug_log!("  Person 2: {}", person2_description.chars().take(100).collect::<String>());

					// Step 2: Build ChatGPT-style prompt with detailed facial descriptions
					let enhanced_prompt = self.build_multi_face_prompt(
						&request.prompt,
						&char1_name, &char1_traits, &person1_description,
						&char2_name, &char2_traits, &person2_description,
						&request.scene_type
					);

                debug_log!("🎨 Enhanced multi-face prompt ({} chars): {}", enhanced_prompt.len(), enhanced_prompt);

                // Step 3: Generate with DALL-E 3 using the enhanced prompt
                let dalle_request = DalleRequest {
                    model: "dall-e-3".to_string(),
                    prompt: enhanced_prompt.clone(),
                    n: 1,
                    size: self.determine_size(request.width, request.height),
                    quality: "hd".to_string(),
                    style: "vivid".to_string(),
                };

                match self.call_dalle_api(&dalle_request).await {
                    Ok(response) => {
                        if let Some(image_data) = response.data.first() {
                            match self.download_and_save_image(image_data, &request.prompt).await {
                                Ok(image_path) => {
                                    debug_log!("✨ Multi-reference generation complete: {}", image_path);
                                    
                                    // Save to gallery with multi-reference metadata
                                    self.save_to_gallery(
                                        &format!("Multi-reference: {}", request.prompt), 
                                        &image_path, 
                                        "dalle3_multi_reference", 
                                        image_data.revised_prompt.as_deref()
                                    ).await;
									

                                    
                                    GenerationResult {
                                        success: true,
                                        image_path: Some(image_path),
                                        prompt_id: Some(uuid::Uuid::new_v4().to_string()),
                                        error: None,
                                        revised_prompt: image_data.revised_prompt.clone(),
                                        generation_method: Some("DALL-E 3 Multi-Reference".to_string()),
                                    }
                                },
                                Err(e) => GenerationResult {
                                    success: false,
                                    image_path: None,
                                    prompt_id: None,
                                    error: Some(format!("Failed to save multi-reference image: {}", e)),
                                    revised_prompt: None,
                                    generation_method: Some("DALL-E 3 Multi-Reference".to_string()),
                                }
                            }
                        } else {
                            GenerationResult {
                                success: false,
                                image_path: None,
                                prompt_id: None,
                                error: Some("No image data in DALL-E response".to_string()),
                                revised_prompt: None,
                                generation_method: Some("DALL-E 3 Multi-Reference".to_string()),
                            }
                        }
                    },
                    Err(e) => GenerationResult {
                        success: false,
                        image_path: None,
                        prompt_id: None,
                        error: Some(format!("DALL-E multi-reference API error: {}", e)),
                        revised_prompt: None,
                        generation_method: Some("DALL-E 3 Multi-Reference".to_string()),
                    }
                }
            },
            Err(e) => GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("Failed to analyze reference faces: {}", e)),
                revised_prompt: None,
                generation_method: Some("DALL-E 3 Multi-Reference".to_string()),
            }
        }
    }
	
	async fn generate_autonomous_image(&self, request: &GenerationRequest) -> GenerationResult {
    // Use DALL-E 3 for higher quality autonomous creations
    let enhanced_prompt = get_style_prompt(
        &request.style.as_ref().unwrap_or(&"artistic".to_string()), 
        &request.prompt
    );
    
    let dalle_request = json!({
        "model": "dall-e-3",
        "prompt": enhanced_prompt,
        "n": 1,
        "size": "1024x1024",
        "quality": "hd",
        "style": "vivid"
    });

    let response = self.client
        .post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", self.api_key))
        .header("Content-Type", "application/json")
        .json(&dalle_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if !status.is_success() {
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return GenerationResult {
                    success: false,
                    image_path: None,
                    prompt_id: None,
                    error: Some(format!("DALL-E 3 API error {}: {}", status, error_text)),
                    revised_prompt: None,
                    generation_method: Some("DALL-E 3 Autonomous".to_string()),
                };
            }

            let dalle_response: DalleResponse = match resp.json().await {
                Ok(response) => response,
                Err(e) => return GenerationResult {
                    success: false,
                    image_path: None,
                    prompt_id: None,
                    error: Some(format!("Failed to parse DALL-E 3 response: {}", e)),
                    revised_prompt: None,
                    generation_method: Some("DALL-E 3 Autonomous".to_string()),
                }
            };

            if let Some(image_data) = dalle_response.data.first() {
                match self.download_and_save_image(image_data, &request.prompt).await {
                    Ok(image_path) => {
                        debug_log!("✨ DALL-E 3 autonomous creation complete: {}", image_path);
                        
                        // Save to gallery with revised prompt if available
                        let display_prompt = if let Some(ref revised) = image_data.revised_prompt {
                            format!("Auto: {} (DALL-E revised: {})", request.prompt, revised)
                        } else {
                            format!("Auto: {}", request.prompt)
                        };
                        
                        self.save_to_gallery(&display_prompt, &image_path, "dalle3_autonomous", image_data.revised_prompt.as_deref()).await;
                        
                        GenerationResult {
                            success: true,
                            image_path: Some(image_path),
                            prompt_id: Some(uuid::Uuid::new_v4().to_string()),
                            error: None,
                            revised_prompt: image_data.revised_prompt.clone(),
                            generation_method: Some("DALL-E 3 Autonomous".to_string()),
                        }
                    },
                    Err(e) => GenerationResult {
                        success: false,
                        image_path: None,
                        prompt_id: None,
                        error: Some(format!("Failed to save autonomous image: {}", e)),
                        revised_prompt: None,
                        generation_method: Some("DALL-E 3 Autonomous".to_string()),
                    }
                }
            } else {
                GenerationResult {
                    success: false,
                    image_path: None,
                    prompt_id: None,
                    error: Some("No image data in autonomous response".to_string()),
                    revised_prompt: None,
                    generation_method: Some("DALL-E 3 Autonomous".to_string()),
                }
            }
        },
        Err(e) => GenerationResult {
            success: false,
            image_path: None,
            prompt_id: None,
            error: Some(format!("DALL-E 3 network error: {}", e)),
            revised_prompt: None,
            generation_method: Some("DALL-E 3 Autonomous".to_string()),
        }
    }
}

    // ========================================================================
    // SINGLE REFERENCE WITH DALL-E 3 (For Consistency)
    // ========================================================================

    async fn generate_single_reference_with_dalle3(&self, request: &MultiIDRequest) -> GenerationResult {
        debug_log!("🎨 Single reference with DALL-E 3: '{}'", request.prompt);

        match self.analyze_single_reference_face(&request.primary_face_reference).await {
            Ok(person_description) => {
                debug_log!("🎨 Single face analysis: {}", person_description.chars().take(100).collect::<String>());

                let enhanced_prompt = self.build_single_face_prompt(
                    &request.prompt,
                    &person_description,
                    &request.scene_type
                );

                let dalle_request = DalleRequest {
                    model: "dall-e-3".to_string(),
                    prompt: enhanced_prompt,
                    n: 1,
                    size: self.determine_size(request.width, request.height),
                    quality: "hd".to_string(),
                    style: "vivid".to_string(),
                };

                match self.call_dalle_api(&dalle_request).await {
                    Ok(response) => {
                        if let Some(image_data) = response.data.first() {
                            match self.download_and_save_image(image_data, &request.prompt).await {
                                Ok(image_path) => {
                                    self.save_to_gallery(
                                        &format!("Single reference: {}", request.prompt), 
                                        &image_path, 
                                        "dalle3_single_reference", 
                                        image_data.revised_prompt.as_deref()
                                    ).await;
                                    
                                    GenerationResult {
                                        success: true,
                                        image_path: Some(image_path),
                                        prompt_id: Some(uuid::Uuid::new_v4().to_string()),
                                        error: None,
                                        revised_prompt: image_data.revised_prompt.clone(),
                                        generation_method: Some("DALL-E 3 Single Reference".to_string()),
                                    }
                                },
                                Err(e) => GenerationResult {
                                    success: false,
                                    image_path: None,
                                    prompt_id: None,
                                    error: Some(format!("Failed to save single reference image: {}", e)),
                                    revised_prompt: None,
                                    generation_method: Some("DALL-E 3 Single Reference".to_string()),
                                }
                            }
                        } else {
                            GenerationResult {
                                success: false,
                                image_path: None,
                                prompt_id: None,
                                error: Some("No image data in DALL-E response".to_string()),
                                revised_prompt: None,
                                generation_method: Some("DALL-E 3 Single Reference".to_string()),
                            }
                        }
                    },
                    Err(e) => GenerationResult {
                        success: false,
                        image_path: None,
                        prompt_id: None,
                        error: Some(format!("DALL-E single reference API error: {}", e)),
                        revised_prompt: None,
                        generation_method: Some("DALL-E 3 Single Reference".to_string()),
                    }
                }
            },
            Err(e) => GenerationResult {
                success: false,
                image_path: None,
                prompt_id: None,
                error: Some(format!("Failed to analyze single reference face: {}", e)),
                revised_prompt: None,
                generation_method: Some("DALL-E 3 Single Reference".to_string()),
            }
        }
    }

    // ========================================================================
    // CHATGPT-STYLE FACE ANALYSIS AND PROMPT BUILDING
    // ========================================================================

    async fn analyze_reference_faces(&self, primary_path: &str, secondary_path: &str) -> Result<((String, Vec<String>, String), (String, Vec<String>, String)), String> {
    debug_log!("🔍 Analyzing both reference faces with GPT-4V");
    
    // Get character names and traits from metadata
    let (char1_name, char1_traits) = self.get_character_details_from_path(primary_path).await;
    let (char2_name, char2_traits) = self.get_character_details_from_path(secondary_path).await;
    
    debug_log!("🎭 Characters identified: {} with traits {:?} and {} with traits {:?}", 
               char1_name, char1_traits, char2_name, char2_traits);

    // Read both images as base64
    let primary_base64 = self.encode_image_to_base64(primary_path).await?;
    let secondary_base64 = self.encode_image_to_base64(secondary_path).await?;

    // Create analysis prompt with actual character names
    let analysis_prompt = format!(r#"These are character reference images for digital art creation. Please describe the visual design characteristics of each character:

For the character in IMAGE 1 (this is {}):
- Overall character appearance and style
- Hair: color, length, texture, and styling
- Eyes: color, shape, and expression style
- Face: general shape, expression, and character design
- Clothing: style, colors, and fashion choices
- Build and posture
- Art style (realistic, stylized, etc.)
- Color palette and aesthetic
- Any distinctive design elements or accessories

For the character in IMAGE 2 (this is {}):
- Overall character appearance and style
- Hair: color, length, texture, and styling
- Eyes: color, shape, and expression style
- Face: general shape, expression, and character design
- Clothing: style, colors, and fashion choices
- Build and posture
- Art style (realistic, stylized, etc.)
- Color palette and aesthetic
- Any distinctive design elements or accessories

This is for creating consistent character art, so include enough visual detail for an artist to recreate the character design.

Format your response as:
{}: [complete visual design description]
{}: [complete visual design description]"#, 
        char1_name.to_uppercase(), char2_name.to_uppercase(),
        char1_name.to_uppercase(), char2_name.to_uppercase());

    // Update parsing to use actual character names
    let char1_key = format!("{}:", char1_name.to_uppercase());
    let char2_key = format!("{}:", char2_name.to_uppercase());
    
    match self.call_gpt_4v_for_analysis(&analysis_prompt, &[primary_base64, secondary_base64]).await {
        Ok(response) => {
            debug_log!("🔍 FULL GPT-4V CHARACTER RESPONSE: {}", response);
            
            if let Some(char1_start) = response.find(&char1_key) {
                if let Some(char2_start) = response.find(&char2_key) {
                    let char1_desc = response[char1_start + char1_key.len()..char2_start]
                        .trim()
                        .to_string();
                    let char2_desc = response[char2_start + char2_key.len()..]
                        .trim()
                        .to_string();
                    
                    // Return character details with descriptions
                    Ok(((char1_name, char1_traits, char1_desc), (char2_name, char2_traits, char2_desc)))
                } else {
                    Err(format!("Could not parse {} description from GPT-4V response", char2_name))
                }
            } else {
                Err(format!("Could not parse {} description from GPT-4V response", char1_name))
            }
        },
        Err(e) => Err(format!("GPT-4V face analysis failed: {}", e))
    }
}

    async fn analyze_single_reference_face(&self, image_path: &str) -> Result<String, String> {
        debug_log!("🔍 Analyzing single reference face with GPT-4V");

        let image_base64 = self.encode_image_to_base64(image_path).await?;

        let analysis_prompt = r#"Analyze this reference photo and provide a detailed facial description.

Describe:
- Overall facial structure and bone structure
- Eye shape, color, and distinctive features
- Nose shape and characteristics  
- Mouth and lip characteristics
- Hair color, texture, and style
- Skin tone and complexion
- Age appearance and any distinctive features
- Overall facial expression or demeanor

Be specific and detailed enough that someone could generate an accurate image from your description."#;

        match self.call_gpt_4v_for_analysis(&analysis_prompt, &[image_base64]).await {
            Ok(description) => Ok(description.trim().to_string()),
            Err(e) => Err(format!("GPT-4V single face analysis failed: {}", e))
        }
    }

    async fn call_gpt_4v_for_analysis(&self, prompt: &str, image_base64_list: &[String]) -> Result<String, String> {
        debug_log!("🔗 Calling GPT-4V for face analysis with {} images", image_base64_list.len());

        // Build messages array for GPT-4V
        let mut content = vec![
            json!({
                "type": "text",
                "text": prompt
            })
        ];

        // Add each image
        for (i, image_data) in image_base64_list.iter().enumerate() {
            content.push(json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/png;base64,{}", image_data),
                    "detail": "high"
                }
            }));
            debug_log!("🖼️ Added image {} to GPT-4V request", i + 1);
        }

        let request_body = json!({
            "model": "gpt-4.1-mini", // Keep hardcoded for analysis functions // "gpt-4.1-mini", //"ft:gpt-4o-2024-08-06:personal:lyra-03:BrO9sB6G",
            "messages": [
                {
                    "role": "user",
                    "content": content
                }
            ],
            "max_tokens": 1000
        });

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("GPT-4V request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("GPT-4V API error {}: {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse GPT-4V response: {}", e))?;

        if let Some(message) = response_json["choices"][0]["message"]["content"].as_str() {
            debug_log!("✅ GPT-4V analysis complete");
            Ok(message.to_string())
        } else {
            Err("No content in GPT-4V response".to_string())
        }
    }

    fn build_multi_face_prompt(&self, base_prompt: &str, char1_name: &str, char1_traits: &[String], person1_desc: &str, char2_name: &str, char2_traits: &[String], person2_desc: &str, scene_type: &SceneType) -> String {
    let scene_context = match scene_type {
            SceneType::SingleCharacter => "featuring one person prominently",
            SceneType::MultiCharacter => "featuring two distinct people in the same scene",
            SceneType::Activity => "showing people engaged in an activity together",
            SceneType::Interaction => "showing two people interacting with each other",
            SceneType::FaceBlend => "blending characteristics of both people",
        };

        // Truncate character descriptions to prevent DALL-E prompt limit issues
        let max_desc_length = 300;
        let trimmed_person1 = if person1_desc.len() > max_desc_length {
            format!("{}...", &person1_desc[..max_desc_length])
        } else {
            person1_desc.to_string()
        };
        
        let trimmed_person2 = if person2_desc.len() > max_desc_length {
            format!("{}...", &person2_desc[..max_desc_length])
        } else {
            person2_desc.to_string()
        };

        // Build trait descriptions using the semantic dictionary
        let char1_trait_desc = if char1_traits.is_empty() { 
			"character".to_string() 
		} else { 
			self.build_natural_character_description(char1_name, char1_traits)
		};
        
        let char2_trait_desc = if char2_traits.is_empty() { 
			"character".to_string() 
		} else { 
			self.build_natural_character_description(char2_name, char2_traits)
		};

        let full_prompt = format!(
			"{}. This scene features two distinct people: 

		FIRST PERSON - {}: {}. Additional visual notes: {}

		SECOND PERSON - {}: {}. Additional visual notes: {}

		Both people should be clearly visible and recognizable in the scene. {}. High quality, photorealistic, detailed faces, professional photography.",
			base_prompt,                    // 1
			char1_name.to_uppercase(),      // 2
			char1_trait_desc,               // 3
			if trimmed_person1.len() > 50 { &trimmed_person1[..50] } else { &trimmed_person1 },  // 4
			char2_name.to_uppercase(),      // 5
			char2_trait_desc,               // 6
			if trimmed_person2.len() > 50 { &trimmed_person2[..50] } else { &trimmed_person2 },  // 7
			scene_context                   // 8
		);

        debug_log!("🎨 Final DALL-E prompt length: {} characters", full_prompt.len());
        
        if full_prompt.len() > 3800 {
            debug_log!("⚠️ Prompt too long, truncating to fit DALL-E limits");
            format!("{}...", &full_prompt[..3800])
        } else {
            full_prompt
        }
    }


    fn build_single_face_prompt(&self, base_prompt: &str, person_desc: &str, scene_type: &SceneType) -> String {
        let scene_context = match scene_type {
            SceneType::SingleCharacter => "featuring this person prominently",
            SceneType::Activity => "showing this person engaged in the described activity",
            _ => "featuring this specific person",
        };

        format!(
            "{}. The scene should feature a specific person who has: {}. The person should be clearly visible and recognizable. {}. High quality, photorealistic, detailed face, professional photography.",
            base_prompt,
            person_desc,
            scene_context
        )
    }

    async fn call_dalle_api(&self, request: &DalleRequest) -> Result<DalleResponse, String> {
        debug_log!("🔗 Calling DALL-E 3 API...");
        
        let response = self.client
            .post("https://api.openai.com/v1/images/generations")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&if request.model == "dall-e-2" {
				// Strip unsupported parameters for DALL-E 2
				serde_json::json!({
					"model": request.model,
					"prompt": request.prompt,
					"n": request.n,
					"size": request.size
				})
			} else {
				// Convert DalleRequest to JSON for DALL-E 3
				serde_json::to_value(request).unwrap()
			})
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error {}: {}", status, error_text));
        }

        let dalle_response: DalleResponse = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        debug_log!("✅ DALL-E API responded with {} images", dalle_response.data.len());
        Ok(dalle_response)
    }

    async fn call_dalle_edit_api(&self, request: &DalleEditRequest) -> Result<DalleResponse, String> {
        debug_log!("🔗 Calling DALL-E 2 Edit API...");
        
        // Create multipart form
        let form = reqwest::multipart::Form::new()
            .text("model", request.model.clone())
            .text("prompt", request.prompt.clone())
            .text("n", request.n.to_string())
            .text("size", request.size.clone())
            .part("image", reqwest::multipart::Part::bytes(
                general_purpose::STANDARD.decode(&request.image)
                    .map_err(|e| format!("Failed to decode base64 image: {}", e))?
            ).file_name("image.png").mime_str("image/png").unwrap());

        let response = self.client
            .post("https://api.openai.com/v1/images/edits")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API error {}: {}", status, error_text));
        }

        let dalle_response: DalleResponse = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        debug_log!("✅ DALL-E Edit API responded with {} images", dalle_response.data.len());
        Ok(dalle_response)
    }

    // ========================================================================
    // IMAGE PROCESSING
    // ========================================================================

    async fn download_and_save_image(&self, image_data: &DalleImageData, prompt: &str) -> Result<String, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let filename = format!("DALLE_{}_{}.png", timestamp, Uuid::new_v4().to_string()[..8].to_string());
        let file_path = self.output_dir.join(&filename);

        if let Some(url) = &image_data.url {
            // Download from URL
            debug_log!("📥 Downloading image from URL...");
            
            let response = self.client
                .get(url)
                .send()
                .await
                .map_err(|e| format!("Failed to download image: {}", e))?;

            let bytes = response.bytes().await
                .map_err(|e| format!("Failed to read image bytes: {}", e))?;

            tokio::fs::write(&file_path, bytes).await
                .map_err(|e| format!("Failed to save image: {}", e))?;

        } else if let Some(b64_data) = &image_data.b64_json {
            // Decode base64
            debug_log!("🔄 Decoding base64 image...");
            
            let bytes = general_purpose::STANDARD.decode(b64_data)
                .map_err(|e| format!("Failed to decode base64: {}", e))?;

            tokio::fs::write(&file_path, bytes).await
                .map_err(|e| format!("Failed to save image: {}", e))?;

        } else {
            return Err("No image URL or base64 data in response".to_string());
        }

        debug_log!("💾 Image saved: {}", file_path.to_string_lossy());
        Ok(file_path.to_string_lossy().to_string())
    }

    async fn encode_image_to_base64(&self, image_path: &str) -> Result<String, String> {
        debug_log!("🔄 Encoding image to base64: {}", image_path);
        
        let bytes = tokio::fs::read(image_path).await
            .map_err(|e| format!("Failed to read image file: {}", e))?;
        
        Ok(general_purpose::STANDARD.encode(bytes))
    }

    // ========================================================================
    // DALL-E API CALLS
    // ========================================================================

    fn enhance_prompt(&self, base_prompt: &str, style: &str) -> String {
        let quality_terms = match style {
            "photorealistic" => "hyperrealistic, professional photography, detailed, high quality",
            "artistic" => "digital art, beautiful composition, artistic masterpiece",
            "dreamy" => "soft lighting, ethereal, dreamy atmosphere, magical",
            "cosmic" => "space, stars, cosmic, otherworldly, science fiction",
            "vibrant" => "vibrant colors, energetic, bright, dynamic",
            _ => "high quality, detailed, beautiful"
        };

        format!("{}, {}", base_prompt, quality_terms)
    }

    fn enhance_prompt_for_reference(&self, base_prompt: &str) -> String {
        format!("Modify this image: {}, maintain the person's face and key features, photorealistic, high quality", base_prompt)
    }

    fn enhance_prompt_for_scene_type(&self, base_prompt: &str, scene_type: &SceneType) -> String {
        let scene_enhancement = match scene_type {
            SceneType::SingleCharacter => "single person portrait, detailed character",
            SceneType::MultiCharacter => "multiple people, group scene, interaction",
            SceneType::Activity => "person engaged in activity, dynamic scene",
            SceneType::Interaction => "people interacting, social scene",
            SceneType::FaceBlend => "blended features, artistic fusion",
        };

        format!("{}, {}, photorealistic, high quality", base_prompt, scene_enhancement)
    }

    // ========================================================================
    // SIZE HANDLING
    // ========================================================================

    fn determine_size(&self, width: Option<u32>, height: Option<u32>) -> String {
    match (width.unwrap_or(1024), height.unwrap_or(1024)) {
        (w, h) if w > h => "1792x1024".to_string(),
        (w, h) if h > w => "1024x1792".to_string(),
        _ => "1024x1024".to_string(),  // Back to 1024x1024 for DALL-E 3
    }
}

    fn determine_size_dalle2(&self, width: Option<u32>, height: Option<u32>) -> String {
    // DALL-E 2 only supports 256x256, 512x512, 1024x1024
    match (width.unwrap_or(512), height.unwrap_or(512)) {
        (w, h) if w <= 256 && h <= 256 => "256x256".to_string(),
        (w, h) if w <= 512 && h <= 512 => "512x512".to_string(),
        _ => "512x512".to_string(),  // Default to 512x512 instead of 1024x1024
    }
}

    // ========================================================================
    // GALLERY INTEGRATION
    // ========================================================================

    async fn save_to_gallery(&self, prompt: &str, image_path: &str, generation_type: &str, revised_prompt: Option<&str>) {
        let display_prompt = if let Some(revised) = revised_prompt {
            format!("{} (DALL-E revised: {})", prompt, revised)
        } else {
            prompt.to_string()
        };

        let gallery_image = crate::GalleryImage {
            message: display_prompt,
            has_image: true,
            image_path: Some(image_path.to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            image_type: generation_type.to_string(),
            identity_metadata: None,
            semantic_keywords: Some(vec!["dalle".to_string(), generation_type.to_string()]),
            priority_score: Some(8.0),
        };
        
        tokio::spawn(async move {
            if let Err(e) = crate::save_gallery_image(gallery_image).await {
                debug_log!("⚠️ Failed to save DALL-E image to gallery: {}", e);
            }
        });
    }

    // ========================================================================
    // COMPATIBILITY FUNCTIONS FOR EXISTING CODEBASE
    // ========================================================================

    pub async fn check_dalle_status(&self) -> bool {
        match std::env::var("OPENAI_API_KEY") {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    // Alias for existing code compatibility
    pub async fn generate_image_with_personality_context(&self, request: MultiIDRequest, _personality: Option<&str>) -> GenerationResult {
        // Route to appropriate method based on reference count
        if request.secondary_face_reference.is_some() {
            self.generate_image_with_multiple_references(request).await
        } else {
            self.generate_single_reference_with_dalle3(&request).await
        }
    }

    pub async fn generate_image_with_research_enhancements(&self, request: MultiIDRequest, _personality: Option<&str>) -> GenerationResult {
        // Alias for compatibility
        self.generate_image_with_personality_context(request, _personality).await
    }
	
	async fn get_character_details_from_path(&self, image_path: &str) -> (String, Vec<String>) {
    let gallery_path = crate::get_data_path("generated_images/gallery_metadata.json");
    
    if let Ok(content) = std::fs::read_to_string(&gallery_path) {
        if let Ok(gallery_items) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            for item in gallery_items {
                if let Some(item_path) = item.get("image_path").and_then(|p| p.as_str()) {
                    let normalized_item = item_path.replace('\\', "/");
                    let normalized_search = image_path.replace('\\', "/");
                    
                    if normalized_item == normalized_search {
                        let character_name = if let Some(identity) = item.get("identity_metadata") {
                            if let Some(represents) = identity.get("represents").and_then(|r| r.as_array()) {
                                if let Some(name) = represents.first().and_then(|n| n.as_str()) {
                                    name.to_string()
                                } else { "Character".to_string() }
                            } else { "Character".to_string() }
                        } else { "Character".to_string() };
                        
                        let trait_tags = if let Some(keywords) = item.get("semantic_keywords") {
                            keywords.as_array().unwrap_or(&vec![])
                                .iter()
                                .filter_map(|k| k.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        } else { vec![] };
                        
                        debug_log!("🎭 Found character: {} with traits: {:?}", character_name, trait_tags);
                        return (character_name, trait_tags);
                    }
                }
            }
        }
    }
    
    ("Character".to_string(), vec![])
}

fn create_semantic_dictionary(&self) -> HashMap<&'static str, &'static str> {
    HashMap::from([
        // === SPECIES & ANATOMY ===
		("human", "human character with normal human anatomy, no fantasy features, no cat ears, no non-human characteristics, regular human proportions"),
		("miqo'te", "Final Fantasy XIV miqo'te character with natural cat ears integrated into fantasy character design, not costume accessories"),
		("miqote", "Final Fantasy XIV miqo'te character with natural cat ears integrated into fantasy character design, not costume accessories"),
		("cat-ears", "natural feline ears seamlessly part of character design"),
		("feline-features", "subtle cat-like facial features, graceful and elegant"),
		("no-animal-features", "completely human appearance with no animal characteristics whatsoever"),
		("normal-ears", "regular human ears, no pointed ears, no cat ears, no fantasy ear shapes"),
		("whisker-marks", "miqo'te style whisker tattoos - line tattoos around the nose, across the cheeks, that look like whiskers"),

		// === HUMAN ETHNICITIES & RACES ===
		("caucasian", "Caucasian ethnicity with European ancestry features and skin tone"),
		("white", "white ethnicity with fair skin tone and European features"),
		("black", "Black ethnicity with African ancestry features and darker skin tone"),
		("african-american", "African-American ethnicity with rich cultural heritage and features"),
		("hispanic", "Hispanic ethnicity with Latin American cultural features"),
		("latino", "Latino ethnicity with diverse Latin American ancestry"),
		("asian", "Asian ethnicity with East Asian features and characteristics"),
		("east-asian", "East Asian ethnicity including Chinese, Japanese, Korean ancestry"),
		("south-asian", "South Asian ethnicity including Indian, Pakistani, Bangladeshi ancestry"),
		("southeast-asian", "Southeast Asian ethnicity including Thai, Vietnamese, Filipino ancestry"),
		("middle-eastern", "Middle Eastern ethnicity with regional cultural features"),
		("native-american", "Native American ethnicity with indigenous American features"),
		("indigenous", "indigenous ethnicity with tribal and cultural characteristics"),
		("mixed-race", "mixed ethnicity with blended cultural and physical characteristics"),
		("multiracial", "multiracial background with diverse ethnic heritage"),
		("pacific-islander", "Pacific Islander ethnicity with Polynesian or Melanesian features"),
		("polynesian", "Polynesian ethnicity with Pacific Island cultural characteristics"),
		("caribbean", "Caribbean ethnicity with island cultural heritage"),
		("mediterranean", "Mediterranean ethnicity with Southern European features"),
		("scandinavian", "Scandinavian ethnicity with Northern European characteristics"),
		("slavic", "Slavic ethnicity with Eastern European cultural features"),

		// === GENDER IDENTITY & EXPRESSION ===
		("female", "feminine gender expression and female characteristics"),
		("male", "masculine gender expression and male characteristics"),
		("woman", "adult female person with feminine characteristics"),
		("man", "adult male person with masculine characteristics"),
		("girl", "young female person with youthful feminine features"),
		("boy", "young male person with youthful masculine features"),
		("non-binary", "non-binary gender expression with gender-neutral characteristics"),
		("genderfluid", "gender-fluid expression with flexible gender characteristics"),
		("androgynous", "androgynous appearance with balanced masculine and feminine features"),
		("masculine", "masculine gender expression regardless of assigned gender"),
		("feminine", "feminine gender expression regardless of assigned gender"),
		("gender-neutral", "gender-neutral appearance with ambiguous gender characteristics"),
		("trans-woman", "transgender woman with feminine expression and identity"),
		("trans-man", "transgender man with masculine expression and identity"),
		("agender", "agender expression with minimal gender characteristics"),

		// === BODY CHARACTERISTICS ===
		("fair-skin", "fair light skin tone with pale complexion"),
		("medium-skin", "medium skin tone with balanced complexion"),
		("dark-skin", "dark skin tone with rich deep complexion"),
		("olive-skin", "olive skin tone with warm Mediterranean complexion"),
		("tan-skin", "tanned skin tone with sun-kissed appearance"),
		("pale-skin", "very pale skin tone with porcelain-like complexion"),
		("bronze-skin", "bronze skin tone with warm golden complexion"),
		("ebony-skin", "deep ebony skin tone with rich dark complexion"),
		("caramel-skin", "caramel skin tone with warm brown complexion"),
		("honey-skin", "honey-colored skin tone with golden brown complexion"),

		// === CULTURAL STYLING ===
		("traditional-dress", "traditional cultural clothing appropriate to character's heritage"),
		("modern-style", "contemporary fashion reflecting current cultural trends"),
		("cultural-jewelry", "traditional jewelry and accessories from character's culture"),
		("ethnic-patterns", "traditional patterns and designs from character's cultural background"),
		("cultural-hairstyle", "traditional hairstyle reflecting character's cultural heritage"),
        
        // === D&D FANTASY RACES ===
        ("elf", "elven character with pointed ears, ethereal beauty, graceful features, and otherworldly elegance"),
        ("high-elf", "high elf with refined aristocratic features, pale skin, and sophisticated bearing"),
        ("wood-elf", "wood elf with natural earthy appearance, connection to nature, and ranger-like qualities"),
        ("dark-elf", "dark elf with dusky skin, white or silver hair, and elegant gothic features"),
        ("drow", "drow with obsidian dark skin, white hair, and striking crimson or violet eyes"),
        ("half-elf", "half-elf with subtly pointed ears, human-elf hybrid features, approachable yet mystical"),
        ("dwarf", "dwarven character with stocky build, braided beard, and sturdy mountain folk appearance"),
        ("halfling", "halfling with small stature, cheerful round face, and hobbit-like rural charm"),
        ("gnome", "gnome with diminutive size, large expressive eyes, and tinker-inventor aesthetic"),
        ("tiefling", "tiefling with small horns, devil tail, and infernal heritage but humanoid appearance"),
        ("dragonborn", "dragonborn with draconic features, scaled skin, and dragon-like head structure"),
        ("aasimar", "aasimar with celestial beauty, subtle divine glow, and angelic heritage features"),
        ("genasi", "genasi with elemental features reflecting their elemental plane heritage"),
        ("firbolg", "firbolg with giant heritage, nature connection, and gentle giant appearance"),
        ("tabaxi", "tabaxi with feline humanoid features, cat-like grace, and natural curiosity"),
        ("aarakocra", "aarakocra with bird-like features, feathered appearance, and avian grace"),
        ("warforged", "warforged with constructed body, metal and wood components, but humanoid shape"),
        ("pointed-ears", "distinctly pointed ears characteristic of elven heritage"),
        ("small-horns", "small curved horns growing from forehead, tiefling characteristic"),
        ("devil-tail", "long thin tail with spaded tip, tiefling infernal heritage"),
        
        // === HAIR COLORS & STYLES ===
        ("rainbow-hair", "naturally multicolored hair flowing through vibrant rainbow spectrum, not artificial or costume-like"),
        ("green-hair", "vibrant emerald green hair color, naturally colored not dyed-looking"),
        ("pink-highlights", "bright pink hair highlights and streaks woven naturally throughout the hair"),
        ("blue-hair", "deep blue hair color, natural-looking blue tone"),
        ("purple-hair", "rich purple hair color, naturally vibrant purple"),
        ("blonde-hair", "golden blonde hair color, natural blonde tones"),
        ("brown-hair", "rich brown hair color, natural chestnut or chocolate brown"),
        ("black-hair", "deep black hair color, naturally dark"),
        ("red-hair", "vibrant red or auburn hair color, natural ginger tones"),
        ("white-hair", "platinum white or silver hair color, ethereal not aged"),
        ("silver-hair", "lustrous silver hair with metallic sheen, mystical appearance"),
        ("gray-hair", "natural gray hair color, distinguished silver-gray tones"),
        ("pastel-hair", "soft pastel colored hair in gentle cotton candy tones"),
        ("ombre-hair", "gradient hair color transitioning from dark roots to light tips"),
        ("streaked-hair", "hair with colorful streaks or highlights throughout"),
        ("two-tone-hair", "hair with two distinct colors in sections or layers"),
        
        // === HAIR LENGTHS ===
        ("very-long-hair", "extremely long hair reaching waist or below, flowing dramatically"),
        ("long-hair", "long flowing hair cascading down past shoulders to mid-back"),
        ("medium-length-hair", "shoulder-length hair, versatile medium cut"),
        ("short-hair", "stylish short haircut above shoulders, chin to ear length"),
        ("very-short-hair", "very short hair, pixie cut or buzz cut style"),
        ("bob-cut", "classic bob haircut, even length around jawline"),
        ("shoulder-length", "hair cut to shoulder length, classic medium style"),
        ("waist-length", "hair reaching to waist, dramatically long and flowing"),
        ("ankle-length", "extremely long hair reaching to ankles, fantasy-length"),
        
        // === HAIR STYLES & TEXTURES ===
        ("curly-hair", "naturally curly or wavy hair texture, bouncy ringlets"),
        ("wavy-hair", "gentle waves throughout hair, soft flowing texture"),
        ("straight-hair", "straight smooth hair texture, sleek and polished"),
        ("coily-hair", "tight coils and natural texture, beautiful textured hair"),
        ("kinky-hair", "natural kinky texture with tight curl pattern"),
        ("messy-hair", "tousled casual hairstyle, artfully disheveled bedhead"),
        ("sleek-hair", "perfectly smooth and straight, polished appearance"),
        ("voluminous-hair", "full thick hair with lots of body and volume"),
        ("fine-hair", "delicate fine-textured hair, silky and lightweight"),
        ("thick-hair", "dense thick hair with substantial body and weight"),
        
        // === HAIR STYLING ===
        ("braided-hair", "hair styled in braids or plaits, intricate woven patterns"),
        ("ponytail", "hair pulled back into ponytail, practical and stylish"),
        ("pigtails", "hair divided into two side ponytails, youthful style"),
        ("bun", "hair twisted or gathered into neat bun, elegant updo"),
        ("half-up", "partial updo with top section pulled up, bottom flowing"),
        ("side-swept", "hair swept dramatically to one side, asymmetrical style"),
        ("bangs", "front fringe cut across forehead, frames the face"),
        ("side-bangs", "angled bangs swept to the side, soft face framing"),
        ("no-bangs", "forehead completely visible, hair pulled back from face"),
        ("layered-hair", "hair cut in layers for movement and texture"),
        ("blunt-cut", "hair cut in straight even line, sharp clean edges"),
        ("shag-cut", "layered shag haircut with feathered texture"),
        ("undercut", "hair with shaved or very short sides and back"),
        ("mohawk", "hair styled with center strip longer than shaved sides"),
        ("dreadlocks", "hair styled in dreadlocks, natural or formed locs"),
        ("cornrows", "hair braided close to scalp in geometric patterns"),
        ("space-buns", "hair in two buns positioned high on head, playful style"),
        ("crown-braid", "braid wrapped around head like a crown, regal styling"),
        ("fishtail-braid", "intricate fishtail braid pattern, complex weaving"),
        ("french-braid", "classic french braid starting from crown of head"),
        ("loose-curls", "relaxed loose curls, soft and flowing"),
        ("tight-curls", "defined tight curls, bouncy and structured"),
        ("beach-waves", "relaxed wavy texture like natural beach hair"),
        ("vintage-waves", "classic vintage finger waves, old Hollywood glamour"),
        
        // === EYE COLORS & FEATURES ===
        ("purple-eyes", "striking violet purple colored eyes, natural eye color"),
        ("green-eyes", "bright emerald green eyes, natural green iris"),
        ("blue-eyes", "deep blue colored eyes, natural blue iris"),
        ("brown-eyes", "warm brown colored eyes, natural brown iris"),
        ("hazel-eyes", "hazel colored eyes with mixed brown and green tones"),
        ("gray-eyes", "silver gray colored eyes, natural gray iris"),
        ("light-eyes", "pale light colored eyes with gentle expression"),
        ("dark-eyes", "deep dark colored eyes, intense gaze"),
        ("normal-eyes", "regular human eyes, no glowing effects, no unusual pupil shapes"),
        
        // === FASHION STYLES ===
        ("punk-style", "punk rock fashion with leather jacket, studded accessories, edgy makeup, rebellious aesthetic"),
        ("bohemian-style", "bohemian flowing clothing, festival fashion, artistic patterns, free-spirited look"),
        ("gothic-style", "gothic fashion with dark clothing and dramatic makeup, elegant darkness"),
        ("casual-style", "relaxed casual clothing, comfortable everyday wear, approachable style"),
        ("elegant-style", "sophisticated elegant clothing and refined appearance, formal grace"),
        ("vintage-style", "retro vintage clothing from past decades, classic timeless look"),
        ("modern-style", "contemporary modern fashion, current trends and clean lines"),
        ("artistic-style", "creative artistic clothing with unique patterns and expressive design"),
        ("professional-style", "business professional attire, workplace appropriate clothing"),
        ("streetwear-style", "urban streetwear fashion, hip-hop influenced casual wear"),
        
        // === CLOTHING ITEMS ===
        ("spiral-pattern", "clothing with colorful spiral patterns and swirling designs"),
        ("leather-jacket", "black leather jacket with punk aesthetic, worn leather texture"),
        ("flowing-top", "loose flowing shirt or blouse, comfortable and airy"),
        ("band-tshirt", "graphic band t-shirt with music group logo or design"),
        ("hoodie", "comfortable hooded sweatshirt, casual streetwear"),
        ("dress", "feminine dress appropriate to character style"),
        ("jeans", "denim jeans, casual everyday pants"),
        ("skirt", "skirt appropriate to character style and length"),
        ("boots", "sturdy boots matching character aesthetic"),
        ("sneakers", "casual athletic shoes or sneakers"),
        ("jewelry", "accessories like necklaces, bracelets, or rings fitting character style"),
        
        // === PERSONALITY TRAITS ===
        ("confident", "confident posture and self-assured expression, strong presence"),
        ("shy", "gentle shy expression with soft demeanor, reserved nature"),
        ("energetic", "dynamic energetic pose and bright expression, full of life"),
        ("mysterious", "enigmatic expression with subtle allure, intriguing presence"),
        ("friendly", "warm friendly expression, approachable and kind demeanor"),
        ("serious", "focused serious expression, determined and thoughtful"),
        ("playful", "playful mischievous expression, fun-loving energy"),
        ("creative", "artistic creative expression, imaginative and inspired"),
        ("rebellious", "defiant rebellious attitude, non-conformist spirit"),
        ("gentle", "soft gentle demeanor, kind and nurturing presence"),
        
        // === INSTRUMENT RELATED ===
        ("guitar-player", "skilled electric guitar player with natural musician posture, holding guitar properly"),
        ("acoustic-guitarist", "acoustic guitar player with folk or classical styling"),
        ("drummer", "talented drummer with rhythmic energy and focus, behind drum kit"),
        ("singer", "vocalist with expressive performance presence, microphone or singing pose"),
        ("bassist", "bass guitar player with steady rhythm focus, holding bass guitar"),
        ("pianist", "piano player with elegant finger positioning and musical expression"),
        ("musician", "general musician with musical instrument and performance presence"),
        
        // === BODY LANGUAGE & POSES ===
        ("standing", "standing upright in natural pose"),
        ("sitting", "sitting in comfortable position appropriate to scene"),
        ("walking", "walking with natural gait and movement"),
        ("dancing", "dancing with rhythmic movement and musical expression"),
        ("relaxed-pose", "relaxed casual body language, at ease"),
        ("dynamic-pose", "energetic dynamic pose with movement and action"),
        ("portrait-pose", "posed for portrait photography, facing camera"),
        ("candid-pose", "natural candid moment, unposed and authentic"),
        
        // === AGE RANGES ===
        ("young-adult", "young adult appearance, early twenties age range"),
        ("teenager", "teenage appearance, adolescent features"),
        ("adult", "mature adult appearance, established adult features"),
        ("youthful", "youthful appearance regardless of actual age"),
        
        // === FACIAL FEATURES ===
        ("heart-shaped-face", "heart-shaped facial structure with wider forehead"),
        ("oval-face", "oval facial structure, balanced proportions"),
        ("round-face", "round facial structure with soft curves"),
        ("square-face", "square facial structure with defined angles"),
        ("angular-face", "angular defined facial features with sharp lines"),
        ("soft-features", "gentle soft facial features with rounded edges"),
        ("strong-jawline", "defined strong jawline with masculine or striking definition"),
        ("delicate-features", "delicate refined facial features, subtle and graceful"),
        ("expressive-eyebrows", "expressive eyebrows that show emotion and character"),
        ("natural-makeup", "natural makeup look, enhanced but not overdone"),
        ("bold-makeup", "bold dramatic makeup, striking and artistic"),
        ("no-makeup", "natural appearance without makeup, authentic skin"),
        
        // === BODY TYPE ===
        ("tall", "tall height and elegant stature, above average height"),
        ("average-height", "average height, typical proportions"),
        ("petite", "petite smaller frame, delicate build"),
        ("athletic", "athletic build and toned physique, physically fit"),
        ("slender", "slender graceful build, naturally thin"),
        ("curvy", "curvy feminine figure with natural curves"),
        ("muscular", "muscular build with defined muscle tone"),
        ("average-build", "average body type, typical proportions"),
        
        // === EXPRESSIONS ===
        ("smiling", "warm genuine smile, happy expression"),
        ("laughing", "laughing with joy, animated happy expression"),
        ("serious", "serious focused expression, concentrated"),
        ("contemplative", "thoughtful contemplative expression, reflective"),
        ("surprised", "surprised expression with wide eyes"),
        ("excited", "excited enthusiastic expression, full of energy"),
        ("calm", "calm peaceful expression, serene and tranquil"),
        ("determined", "determined focused expression, strong will"),
        ("dreamy", "dreamy contemplative expression, lost in thought"),
        ("mischievous", "playful mischievous expression, hint of trouble"),
        
        // === SCENE MODIFIERS ===
        ("visualanchor", "primary visual focus of the scene, main character"),
        ("background-character", "supporting character in the background"),
        ("center-stage", "prominently featured in center of composition"),
        ("close-up", "close-up view focusing on face and upper body"),
        ("full-body", "full body view showing entire character"),
        ("portrait", "portrait style focusing on head and shoulders"),
        
        // === LIGHTING & ATMOSPHERE ===
        ("natural-lighting", "natural lighting conditions, realistic illumination"),
        ("dramatic-lighting", "dramatic lighting with strong shadows and highlights"),
        ("soft-lighting", "soft diffused lighting, gentle and flattering"),
        ("bright-lighting", "bright cheerful lighting, well-illuminated"),
        ("moody-lighting", "moody atmospheric lighting, creates emotion"),
        
        // === QUALITY MODIFIERS ===
        ("photorealistic", "hyperrealistic photography quality, looks like real photo"),
        ("artistic", "digital art style with painterly quality, artistic interpretation"),
        ("anime-style", "anime character design with stylized features"),
        ("cartoon-style", "cartoon or animated character design"),
        ("realistic", "realistic human proportions and features"),
        ("stylized", "stylized artistic interpretation while maintaining realism"),
        ("detailed", "highly detailed with intricate features and textures"),
        ("high-quality", "high quality rendering with professional finish"),
        
        // === COMMON MISTAKES TO PREVENT ===
        ("not-anime", "not anime style, realistic human features and proportions"),
        ("not-cartoon", "not cartoon style, realistic human appearance"),
        ("not-fantasy", "not fantasy character, regular human without magical features"),
        ("not-cosplay", "not cosplay or costume, natural everyday appearance"),
        ("not-artificial", "not artificial or fake looking, natural human appearance"),
        ("no-horns", "no horns or demonic features, completely human"),
        ("no-wings", "no wings or angelic features, regular human back"),
        ("no-tail", "no tail or animal appendages, normal human anatomy"),
        ("no-fangs", "no fangs or vampire teeth, normal human teeth"),
        ("no-claws", "no claws or animal nails, regular human fingernails"),
    ])
}

fn translate_semantic_tags(&self, tags: &[String]) -> Vec<String> {
        let dictionary = self.create_semantic_dictionary();
        
        tags.iter().map(|tag| {
            let clean_tag = tag.trim().to_lowercase();
            
            // Check if we have a translation for this tag
            if let Some(&translation) = dictionary.get(clean_tag.as_str()) {
                translation.to_string()
            } else {
                // Keep original tag if no translation exists
                tag.clone()
            }
        }).collect()
    }
	
fn build_natural_character_description(&self, char_name: &str, tags: &[String]) -> String {
    let translated_tags = self.translate_semantic_tags(tags);
    
    // Filter out metadata tags AND deduplicate
    let mut visual_tags: Vec<String> = translated_tags.iter()
    .filter(|tag| {
        // Keep important visual traits, filter out only metadata
        !tag.contains("manually_tagged") && 
        !tag.contains("reference") && 
        !tag.contains("self") && 
        !tag.contains("visual anchor") &&
        !tag.contains("primary visual focus") &&
        !tag.contains("appearance") &&
        !tag.contains("visualanchor") &&  // Add this
        *tag != "female" &&  // Keep other female-related tags, just not standalone "female"
		*tag != "lyra" &&    // Don't include the character name as a trait
        tag.len() > 3
    })
    .cloned()
    .collect();
	
    // DEDUPLICATE - remove exact duplicates
    visual_tags.dedup();
    
    if visual_tags.is_empty() {
        return format!("{} character", char_name);
    }
    
    format!("CRITICAL CHARACTER DESIGN: {} must have these exact features: {}. DO NOT change these character details.", 
            char_name.to_uppercase(), 
            visual_tags.join(". ALSO: "))
}
	
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
pub async fn generate_image_command(
    prompt: String,
    style: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    autonomous: Option<bool>,
) -> Result<GenerationResult, String> {
    let generator = ImageGenerator::new().map_err(|e| format!("Failed to initialize DALL-E: {}", e))?;
    
    let request = GenerationRequest {
    prompt,
    negative_prompt: None,
    width,
    height,
    steps: None,
    cfg: None,
    seed: None,
    style,
    autonomous,
};

    Ok(generator.generate_image(request).await)
}

#[tauri::command]
pub async fn generate_image_from_reference_command(
    prompt: String,
    reference_image_path: String,
    strength: Option<f32>,
    style: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<GenerationResult, String> {
    let generator = ImageGenerator::new().map_err(|e| format!("Failed to initialize DALL-E: {}", e))?;

    let request = Img2ImgRequest {
        prompt,
        reference_image_path,
        negative_prompt: None,
        width,
        height,
        steps: None,
        cfg: None,
        strength,
        seed: None,
        style,
    };

    Ok(generator.generate_image_from_reference(request).await)
}

#[tauri::command]
pub async fn generate_image_with_universal_multi_id_command(
    prompt: String,
    primary_face_reference: String,
    secondary_face_reference: Option<String>,
    primary_face_strength: Option<f32>,
    secondary_face_strength: Option<f32>,
    style: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<GenerationResult, String> {
    let generator = ImageGenerator::new().map_err(|e| format!("Failed to initialize DALL-E: {}", e))?;

    let scene_type = if secondary_face_reference.is_some() {
        SceneType::MultiCharacter
    } else {
        SceneType::SingleCharacter
    };

    let request = MultiIDRequest {
        prompt,
        primary_face_reference,
        secondary_face_reference,
        negative_prompt: None,
        width,
        height,
        steps: None,
        cfg: None,
        primary_face_strength,
        secondary_face_strength,
        start_at: None,
        end_at: None,
        seed: None,
        style,
        scene_type,
    };

    Ok(generator.generate_image_with_multiple_references(request).await)
}

#[tauri::command]
pub async fn check_dalle_status() -> Result<bool, String> {
    let generator = ImageGenerator::new().map_err(|e| format!("Failed to initialize DALL-E: {}", e))?;
    Ok(generator.check_dalle_status().await)
}

