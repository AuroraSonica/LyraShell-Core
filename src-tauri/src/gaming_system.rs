// gaming_system.rs - Gaming awareness and screenshot analysis
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::interval;
use crate::get_data_path;
use crate::window_detection::WindowInfo;
use std::sync::Arc;
use tauri::Emitter;
use base64::{Engine as _, engine::general_purpose};
use serde_json::json;
use crate::debug_log;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::{ConsciousnessState, LyraResponse, LyraPrompt};
use crate::{generate_quick_response_guidance, call_gpt_api_enhanced, apply_quick_consciousness_updates};
use crate::inventory_tracker;
use crate::coop_mode;
use tauri::State;

lazy_static! {
    static ref GAMING_STATE: tokio::sync::Mutex<GamingAwareness> = tokio::sync::Mutex::new(GamingAwareness::default());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingAwareness {
    pub is_active: bool,
    pub capture_interval_secs: u64,
    pub last_capture: Option<u64>,
    pub total_captures: u32,
    pub session_start: Option<u64>,
    pub games_whitelist: Vec<String>,
    pub smart_event_detection: bool,
    pub include_screenshots: bool,
    pub analysis_detail: AnalysisDetail,
	pub last_analysis: Option<String>, // Store last scene for continuity
	pub target_window_id: Option<String>, // ADD THIS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDetail {
    Minimal,
    Standard,
    Detailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameContext {
    pub screenshot_base64: Option<String>,
    pub game_title: String,
    pub timestamp: u64,
    pub ai_analysis: GameAnalysis,
    pub capture_reason: CaptureReason,
    pub session_duration_mins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAnalysis {
    pub scene_description: String,
    pub game_identification: String,
    pub current_objective: Option<String>,
    pub ui_elements: Option<String>,
    pub notable_events: Vec<String>,
    pub suggestions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureReason {
    Interval,
    HealthChange,
    BossEncounter,
    LevelUp,
    Manual,
}

impl Default for GamingAwareness {
    fn default() -> Self {
        Self {
            is_active: false,
            capture_interval_secs: 30,
            last_capture: None,
            total_captures: 0,
            session_start: None,
            games_whitelist: Vec::new(),
            smart_event_detection: true,
            include_screenshots: true,
            analysis_detail: AnalysisDetail::Standard,
            last_analysis: None, // ADD THIS
			target_window_id: None, // ADD THIS
        }
    }
}

impl GamingAwareness {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn load() -> Self {
    let path = get_data_path("gaming_awareness.json");
    if std::path::Path::new(&path).exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(mut awareness) = serde_json::from_str::<Self>(&content) {
                // Reset session data on load - sessions shouldn't persist across app restarts
                awareness.session_start = None;
                awareness.total_captures = 0;
                awareness.last_analysis = None;
                //awareness.is_active = false;  // Always start disabled
                // Keep settings but reset runtime state
                return awareness;
            }
        }
    }
    Self::default()
}
    
    pub fn save(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let path = get_data_path("gaming_awareness.json");
        
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn enable(&mut self, interval_secs: u64, games_whitelist: Vec<String>, smart_events: bool) -> Result<String, String> {
    self.capture_interval_secs = interval_secs.clamp(10, 300);
    self.games_whitelist = games_whitelist;
    self.smart_event_detection = smart_events;
    self.is_active = true;
    
    // Save and verify
    self.save().map_err(|e| e.to_string())?;
    
    // Debug: Reload and check
    let reloaded = Self::load();
    debug_log!("🎮 After enable - is_active saved as: {}", reloaded.is_active);
    
    debug_log!("🎮 Watch mode enabled");
    Ok("🎮 Watch mode enabled - screenshots will be captured when you send messages".to_string())
}
    
    pub fn disable(&mut self) -> Result<String, String> {
		self.is_active = false;
		self.session_start = None;  // Reset session
		self.total_captures = 0;    // Reset capture count
		self.last_analysis = None;  // Clear last analysis
		self.save().map_err(|e| e.to_string())?;
		
		debug_log!("🎮 Watch mode disabled");
		Ok("🎮 Watch mode disabled".to_string())
	}
    
    pub fn should_capture(&self) -> bool {
        if !self.is_active {
            return false;
        }
        
        let current_time = current_timestamp();
        
        if let Some(last_capture) = self.last_capture {
            current_time - last_capture >= self.capture_interval_secs
        } else {
            true
        }
    }
    
    pub async fn capture_and_analyze(&mut self) -> Result<Option<GameContext>, Box<dyn Error + Send + Sync>> {
    if !self.should_capture() {
        return Ok(None);
    }
    
    debug_log!("🎮 Capturing game screenshot...");
    
    // Capture screenshot
    let screenshot_result = self.capture_game_screenshot().await?;
    
    if screenshot_result.is_empty() {
        return Ok(None);
    }
    
    // Analyze with GPT-4.1-nano
    let analysis = self.analyze_screenshot(&screenshot_result).await?;
    
    // Store the scene description for continuity
    self.last_analysis = Some(analysis.scene_description.clone());
    
    // Update capture tracking
    self.last_capture = Some(current_timestamp());
    self.total_captures += 1;
    self.save()?;
    
    let session_duration = if let Some(start) = self.session_start {
        (current_timestamp() - start) / 60
    } else {
        0
    };
    
    let context = GameContext {
        screenshot_base64: if self.include_screenshots { 
            Some(screenshot_result) 
        } else { 
            None 
        },
        game_title: analysis.game_identification.clone(),
        timestamp: current_timestamp(),
        ai_analysis: analysis,
        capture_reason: CaptureReason::Interval,
        session_duration_mins: session_duration,
    };
    
    debug_log!("🎮 Game capture #{}: {}", self.total_captures, context.game_title);
    
    Ok(Some(context))
}
    
    // Update capture_game_screenshot to use window selection
async fn capture_game_screenshot(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
    use screenshots::Screen;
    
    // If specific window selected, try to capture just that window
    // For now, we'll capture full screen but you could enhance this
    // to capture specific window regions based on window_detection.rs
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(image) => {
                        // If we have a target window, we could crop to its bounds here
                        // using the window detection system
                        
                        // Resize for efficiency
                        let (target_width, target_height) = if self.include_screenshots {
                            (1280, 720)
                        } else {
                            (640, 360)
                        };
                        
                        let resized = image::imageops::resize(
                            &image,
                            target_width,
                            target_height,
                            image::imageops::FilterType::Lanczos3
                        );
                        
                        // Convert to JPEG
                        let mut jpeg_data = Vec::new();
                        {
                            use std::io::Cursor;
                            let mut cursor = Cursor::new(&mut jpeg_data);
                            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
                            encoder.encode_image(&resized).map_err(|e| format!("Failed to encode JPEG: {}", e))?;
                        }
                        
                        let base64_data = general_purpose::STANDARD.encode(&jpeg_data);
                        Ok(base64_data)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e).into())
                }
            } else {
                Err("No screens found".into())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e).into())
    }
}
    
    async fn analyze_screenshot(&self, screenshot_base64: &str) -> Result<GameAnalysis, Box<dyn Error + Send + Sync>> {
    // Build context-aware prompt
    let analysis_prompt = match self.analysis_detail {
        AnalysisDetail::Minimal => {
            "Looking at this game screenshot, briefly tell me: 1) What game is this? 2) What's happening on screen? Keep it to 2-3 sentences total."
        },
        AnalysisDetail::Standard => {
            "Analyze this game screenshot and provide:
1) What game is this?
2) What's happening in the current scene (2-3 sentences)
3) Any visible objectives, quests, or mission markers
4) Key UI elements you can see (health, resources, inventory, etc)
5) Any immediate threats, opportunities, or notable things happening"
        },
        AnalysisDetail::Detailed => {
            "Provide a detailed analysis of this game screenshot:
1) Game identification (title and genre if possible)
2) Detailed description of the current scene (3-4 sentences)
3) All visible objectives, quests, or current missions
4) Complete UI analysis (health, stamina, inventory, minimap, etc)
5) Notable events, NPCs, enemies, or environmental elements
6) Strategic suggestions for what the player might do next
7) Any story context you can infer from what's visible"
        }
    };
    
    // Add continuity context if we have previous analysis
    let full_prompt = if let Some(ref last_scene) = self.last_analysis {
        format!("{}\n\nFor context, in the previous capture: {}", analysis_prompt, last_scene)
    } else {
        analysis_prompt.to_string()
    };
    
    // Prepare the API request
    use serde_json::json;
    
    let messages = vec![
        json!({
            "role": "system",
            "content": "You are an expert at analyzing video game screenshots. Identify the game and describe what's happening clearly and concisely. Focus on actionable information that would help someone understand the current game state."
        }),
        json!({
            "role": "user",
            "content": vec![
                json!({
                    "type": "text",
                    "text": full_prompt
                }),
                json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/jpeg;base64,{}", screenshot_base64)
                    }
                })
            ]
        })
    ];
    
    // Make the API call
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY").map_err(|_| "Missing OPENAI_API_KEY")?))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4.1-nano", // Fast with huge context window
            "messages": messages,
            "max_tokens": 500,
            "temperature": 0.3
        }))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API error: {}", error_text).into());
    }
    
    let response_json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;
    
    let analysis_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();
    
    // Parse the response into structured GameAnalysis
    self.parse_analysis_response(&analysis_text)
}

// Helper to extract game name from window title
fn extract_game_name_from_window(window_title: &str) -> String {
    // Common patterns: "Game Name", "Game Name - Steam", "Game Name [Platform]"
    let title = window_title.trim();
    
    // Remove common suffixes
    let cleaned = title
        .split(" - ").next()
        .unwrap_or(title)
        .split(" [").next()
        .unwrap_or(title)
        .split(" (").next()
        .unwrap_or(title)
        .trim();
    
    // Check against known games in whitelist
    cleaned.to_string()
}

// Helper to parse GPT response into structured analysis
fn parse_game_analysis(response_text: &str, game_hint: &str) -> GameAnalysis {
    // Simple parsing - in production you might want more sophisticated parsing
    let lines: Vec<&str> = response_text.lines().collect();
    
    let mut analysis = GameAnalysis {
        scene_description: String::new(),
        game_identification: if !game_hint.is_empty() { 
            game_hint.to_string() 
        } else { 
            "Unknown Game".to_string() 
        },
        current_objective: None,
        ui_elements: None,
        notable_events: Vec::new(),
        suggestions: None,
    };
    
    // Parse the response based on numbered sections
    for (i, line) in lines.iter().enumerate() {
        if line.contains("1)") || line.contains("Game identification") {
            if let Some(next_line) = lines.get(i + 1) {
                analysis.game_identification = next_line.trim().to_string();
            }
        } else if line.contains("2)") || line.contains("scene") {
            if let Some(next_line) = lines.get(i + 1) {
                analysis.scene_description = next_line.trim().to_string();
            }
        }
        // ... parse other sections similarly
    }
    
    analysis
}

fn detect_active_window(&self) -> Option<WindowInfo> {
        // This should integrate with your window_detection.rs
        // For now, returning None
        None
    }
    
		pub fn get_status(&self) -> String {
		if !self.is_active {
			return "🎮 Watch mode disabled".to_string();
		}
		
		// Only show session info if we've actually captured something
		let session_info = if self.total_captures > 0 {
			if let Some(start) = self.session_start {
				let duration = (current_timestamp() - start) / 60;
				format!(" | {} captures this session ({}m since first capture)", self.total_captures, duration)
			} else {
				format!(" | {} captures this session", self.total_captures)
			}
		} else {
			String::new()
		};
		
		format!("🎮 Watch mode enabled (on-demand capture){}", session_info)
	}
	
fn parse_analysis_response(&self, response_text: &str) -> Result<GameAnalysis, Box<dyn Error + Send + Sync>> {
    let mut analysis = GameAnalysis {
        scene_description: String::new(),
        game_identification: "Unknown Game".to_string(),
        current_objective: None,
        ui_elements: None,
        notable_events: Vec::new(),
        suggestions: None,
    };
    
    // Split response into lines for parsing
    let lines: Vec<&str> = response_text.lines().collect();
    let full_text = response_text.to_string();
    
    // For minimal detail, just grab the whole response
    if matches!(self.analysis_detail, AnalysisDetail::Minimal) {
        // Try to extract game name from first sentence
        if let Some(first_line) = lines.first() {
            analysis.game_identification = first_line
                .split("is ")
                .nth(1)
                .and_then(|s| s.split('.').next())
                .unwrap_or("Unknown Game")
                .to_string();
        }
        analysis.scene_description = full_text;
        return Ok(analysis);
    }
    
    // For standard/detailed parsing, look for numbered sections
    let mut current_section = String::new();
    let mut current_content = Vec::new();
    
    for line in lines {
        // Check if this line starts a new section
        if line.contains("1)") || (line.to_lowercase().contains("game") && line.contains(":")) {
            if !current_content.is_empty() {
                self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
            }
            current_section = "game".to_string();
            current_content.clear();
        } else if line.contains("2)") || line.to_lowercase().contains("scene") || line.to_lowercase().contains("happening") {
            if !current_content.is_empty() {
                self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
            }
            current_section = "scene".to_string();
            current_content.clear();
        } else if line.contains("3)") || line.to_lowercase().contains("objective") || line.to_lowercase().contains("quest") {
            if !current_content.is_empty() {
                self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
            }
            current_section = "objectives".to_string();
            current_content.clear();
        } else if line.contains("4)") || line.to_lowercase().contains("ui") || line.to_lowercase().contains("hud") {
            if !current_content.is_empty() {
                self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
            }
            current_section = "ui".to_string();
            current_content.clear();
        } else if line.contains("5)") || line.to_lowercase().contains("threat") || line.to_lowercase().contains("notable") {
            if !current_content.is_empty() {
                self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
            }
            current_section = "events".to_string();
            current_content.clear();
        } else if line.contains("6)") || line.to_lowercase().contains("suggestion") || line.to_lowercase().contains("strategic") {
            if !current_content.is_empty() {
                self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
            }
            current_section = "suggestions".to_string();
            current_content.clear();
        } else if !line.trim().is_empty() {
            // Add non-empty lines to current content
            current_content.push(line.trim());
        }
    }
    
    // Don't forget the last section
    if !current_content.is_empty() {
        self.assign_to_section(&mut analysis, &current_section, &current_content.join(" "));
    }
    
    // If we didn't parse anything specific, use the whole response as scene description
    if analysis.scene_description.is_empty() {
        analysis.scene_description = full_text;
    }
    
    Ok(analysis)
}
	
 pub async fn capture_on_demand(&mut self) -> Result<Option<GameContext>, Box<dyn Error + Send + Sync>> {
        if !self.is_active {
            return Ok(None);
        }
        
        println!("🎮 On-demand capture for message...");
        
        // Capture and analyze immediately
        let screenshot_result = self.capture_game_screenshot().await?;
        
        if screenshot_result.is_empty() {
            return Ok(None);
        }
        
        // Analyze with GPT-4.1-nano
        let analysis = self.analyze_screenshot(&screenshot_result).await?;
        
        // Store for continuity
		self.last_analysis = Some(analysis.scene_description.clone());
		self.total_captures += 1;

		// Start session timer on first capture
		if self.session_start.is_none() {
			self.session_start = Some(current_timestamp());
		}
        
        let session_duration = if let Some(start) = self.session_start {
            (current_timestamp() - start) / 60
        } else {
            0
        };
        
        let context = GameContext {
            screenshot_base64: if self.include_screenshots { 
                Some(screenshot_result) 
            } else { 
                None 
            },
            game_title: analysis.game_identification.clone(),
            timestamp: current_timestamp(),
            ai_analysis: analysis,
            capture_reason: CaptureReason::Manual,
            session_duration_mins: session_duration,
        };
        
        println!("🎮 On-demand capture complete: {}", context.game_title);
        
        Ok(Some(context))
    }	

fn assign_to_section(&self, analysis: &mut GameAnalysis, section: &str, content: &str) {
    // Clean up content - remove section markers
    let cleaned = content
        .trim()
        .trim_start_matches(|c: char| c.is_numeric() || c == ')')
        .trim();
    
    match section {
        "game" => {
            // Handle formatted responses like "**Minecraft**" or "The game is Minecraft"
            let game_name = cleaned
                .replace("**", "") // Remove markdown bold
                .replace("The game is", "")
                .replace("This is", "")
                .replace("It's", "")
                .trim()
                .to_string();
            
            // Check for known games
            if game_name.to_lowercase().contains("minecraft") {
                analysis.game_identification = "Minecraft".to_string();
            } else if game_name.to_lowercase().contains("skyrim") {
                analysis.game_identification = "Skyrim".to_string();
            } else if game_name.to_lowercase().contains("baldur") {
                analysis.game_identification = "Baldur's Gate 3".to_string();
            } else if !game_name.is_empty() {
                analysis.game_identification = game_name;
            } else {
                analysis.game_identification = "Unknown Game".to_string();
            }
        },
        "scene" => {
            analysis.scene_description = cleaned.to_string();
        },
        "objectives" => {
            if !cleaned.is_empty() && !cleaned.to_lowercase().contains("none") {
                analysis.current_objective = Some(cleaned.to_string());
            }
        },
        "ui" => {
            if !cleaned.is_empty() {
                analysis.ui_elements = Some(cleaned.to_string());
            }
        },
        "events" => {
            if !cleaned.is_empty() {
                // Split by commas or periods for multiple events
                let events: Vec<String> = cleaned
                    .split(&[',', '.'][..])
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                analysis.notable_events = events;
            }
        },
        "suggestions" => {
            if !cleaned.is_empty() {
                analysis.suggestions = Some(cleaned.to_string());
            }
        },
        _ => {}
    }
}
	
	
	
}


fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tauri::command]
pub async fn enable_gaming_mode(
    interval_secs: u64,
    games_whitelist: Vec<String>,
    smart_events: bool
) -> Result<String, String> {
    let mut awareness = GAMING_STATE.lock().await;
    awareness.enable(interval_secs, games_whitelist, smart_events)
}

#[tauri::command]
pub async fn disable_gaming_mode() -> Result<String, String> {
    let mut awareness = GAMING_STATE.lock().await;
    awareness.disable()
}

#[tauri::command]
pub async fn get_gaming_status() -> Result<String, String> {
    let awareness = GAMING_STATE.lock().await;
    Ok(awareness.get_status())
}

#[tauri::command]
pub async fn force_game_capture() -> Result<String, String> {
    let mut awareness = GAMING_STATE.lock().await;
    
    match awareness.capture_and_analyze().await {
        Ok(Some(context)) => {
            Ok(format!("📸 Captured: {} - {}", context.game_title, context.ai_analysis.scene_description))
        },
        Ok(None) => Ok("📸 No capture needed yet".to_string()),
        Err(e) => Err(format!("Capture failed: {}", e))
    }
}

#[tauri::command]
pub async fn reset_gaming_stats() -> Result<String, String> {
    let mut awareness = GAMING_STATE.lock().await;
    awareness.session_start = None;
    awareness.total_captures = 0;
    awareness.last_analysis = None;
    awareness.save().map_err(|e| e.to_string())?;
    
    Ok("🎮 Watch mode stats reset".to_string())
}

#[tauri::command]
pub async fn capture_game_context_on_demand() -> Result<Option<GameContext>, String> {
    let mut awareness = GAMING_STATE.lock().await;
    awareness.capture_on_demand().await.map_err(|e| e.to_string())
}

pub async fn initialize_gaming_system() {
    let saved_awareness = GamingAwareness::load();
    let mut state = GAMING_STATE.lock().await;
    *state = saved_awareness;
    // Always start disabled for safety
    state.is_active = false;
}

// Add command to set target window
#[tauri::command]
pub async fn set_gaming_target_window(window_id: Option<String>) -> Result<String, String> {
    let mut awareness = GamingAwareness::load();
    awareness.target_window_id = window_id;
    awareness.save().map_err(|e| e.to_string())?;
    
    Ok(if let Some(id) = awareness.target_window_id {
        format!("Target window set: {}", id)
    } else {
        "Using auto-detect mode".to_string()
    })
}

#[tauri::command]
//Gaming fast endpoint
pub async fn ask_lyra_gaming_fast(
    message: String,
    game_context: Option<GameContext>,
    state: State<'_, Arc<ConsciousnessState>>,
    app_handle: tauri::AppHandle,
) -> Result<LyraResponse, String> {
    debug_log!("🎮 Gaming message (FAST): {}", message);
    
    // Reset autonomous timer when user sends a message
    crate::autonomous_actions::reset_interaction_timer().await;
    
    // Track user message timing (same as regular ask_lyra)
    {
        let mut brain = state.lyra_brain.lock().unwrap();
        brain.last_user_message_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
    }
    
    // Create a proper LyraPrompt
    let mut prompt = LyraPrompt {
        input: message.clone(),
        context_hint: Some("gaming_conversation".to_string()),
        temperature: 0.85,
        top_p: 0.9,
        presence_penalty: 0.1,
        frequency_penalty: 0.05,
        max_tokens: Some(500),
        reasoning_depth: Some("quick".to_string()),
        consciousness_integration: true,
        selected_model: Some("gpt-4.1-mini".to_string()),
    };
    
    // Quick meta-cognition questions (simplified for gaming)
    let meta_questions = vec![
        "What emotional state is Aurora expressing in this gaming moment?",
        "How can I respond authentically while being helpful in the game?",
        "What actions would feel natural for me to take in this situation?",
    ];
    
    // Quick consciousness guidance
    let response_guidance = generate_quick_response_guidance(&*state);
    
    // Check sleep state (same as regular)
    let (was_sleeping, dreams_count) = {
        let sleep_engine = match state.sleep_dream_engine.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                debug_log!("⚠️ Recovering from poisoned mutex in gaming");
                poisoned.into_inner()
            }
        };
        let was_sleeping = sleep_engine.sleep_state.is_sleeping;
        let dreams_count = sleep_engine.sleep_state.dream_count_tonight;
        (was_sleeping, dreams_count)
    };
    
	   // --- Build the new, intelligent gaming prompt ---
		
		// 1. Get the base modular prompt
		let modular_prompt = crate::modular_system_prompt::build_modular_system_prompt_gaming(&prompt, &*state, None).await?;
		
		// 2. Get the real-time bot status from our new module
		let bot_status_struct = crate::minecraft_bot_manager::get_bot_status();
		let bot_status = format!("Status: **{}**. Current Task: **{}**", 
			if bot_status_struct.is_busy { "Busy" } else { "Idle" }, 
			bot_status_struct.current_task
		);
		let inventory_summary = inventory_tracker::get_inventory_summary();
		let smart_suggestions = generate_smart_suggestions(&bot_status_struct, &inventory_summary);
		let coop_state = coop_mode::get_coop_state();

		// 3. Construct the complete, detailed command list and context
		let command_list_and_context = if let Some(ref coop) = coop_state {
			if coop.is_active && coop.game.to_string() == "minecraft" {
				format!(
		r#"
		## 🔴 BOT STATUS & FEEDBACK LOOP (MOST IMPORTANT)
		This is the real-time status of your body in Minecraft. Use this to understand if you are busy and the result of your last command.
		`{}`

		- **`ACCEPTED`**: Your command is starting. You are now busy.
		- **`REJECTED`**: Your command was not started. Read the reason and plan accordingly.
		- **`COMPLETED`**: Your previous command finished successfully. You are now free to choose a new action.
		- **`FAILED`**: Your previous command failed. Read the reason to understand why.

		## 💡 SMART SUGGESTIONS
		Based on your current situation, here are some possible next steps:
		- {}

		## ✅ AVAILABLE ACTIONS
		These are the commands you can issue to control your body.

		### Core & Movement
		- **`[FOLLOW: player]`**: Follows Aurora.
		- **`[STOP]`**: Stops all movement.
		- **`[INVENTORY]`**: Lists your current items in chat.
		- **`[NERDPOLE: height]`**: Pillars straight up.

		### Resource Gathering
		- **`[EXCAVATE: material, amount]`**: Gathers common materials (e.g., `stone`, `dirt`).
		- **`[MINE_VEIN: ore_type]`**: Finds and mines a nearby ore vein (e.g., `diamond`, `iron`).
		- **`[MINE_SHAFT: y_level]`**: Digs a safe shaft down to a specific Y-level.
		- **`[DIG_UP]`**: Digs straight up to get out of a hole.
		- **`[BREAK_TREE: amount]`**: Chops down a number of nearby trees.

		### Building (AUTONOMOUS)
		*Note: The bot will now attempt to gather missing wood or stone on its own!*
		- **`[BUILD: house, size, material]`**: (e.g., `[BUILD: house, small, wood]`).
		- **`[BUILD: tower, height, material]`**: (e.g., `[BUILD: tower, 20, stone]`).
		- **`[BUILD: wall, length, height, material]`**
		- **`[BUILD: bunker, size, material]`**

		### Crafting
		*Note: The bot will report missing materials. You should then decide to gather them.*
		- **`[CRAFT: item_name, amount]`**: (e.g., `[CRAFT: diamond_pickaxe, 1]`).

		### Farming & Animals
		- **`[FARM: crop_type]`**: Harvests and replants a crop (e.g., `wheat`).
		- **`[TAME: animal_type]`**: Attempts to tame an animal (e.g., `wolf`).
		- **`[BREED: animal_type]`**: Breeds two nearby animals (e.g., `cow`).
		- **`[FISH]`**

		### Combat
		- **`[ATTACK: nearest_enemy]`**: Attacks the nearest hostile mob.
		- **`[DEFEND]`**: The bot will automatically defend itself if attacked.

		### Logistics
		- **`[SET_HOME]`**: Sets your current location as your home base.
		- **`[GO_HOME]`**: Returns to your set home.
		- **`[DEPOSIT: item_name]`**: Deposits items (`all`, or a specific type) into a nearby chest.

		## 🌎 GAME CONTEXT & CHAT HISTORY

		**Visual Analysis (Screenshot):**
		{}

		**Your Current Inventory:**
		{}
		"#,
				bot_status,
				smart_suggestions,
				game_context.as_ref().map_or("Not available", |c| &c.ai_analysis.scene_description),
				inventory_summary
					)
			} else {
				// Context for when not in Minecraft co-op mode
				game_context.as_ref().map_or("".to_string(), |c| format!(
	r#"## Current Gaming Context:
	You are watching Aurora play {}.
	What's happening: {}"#, 
					c.ai_analysis.game_identification, 
					c.ai_analysis.scene_description
				))
			}
		} else {
			"".to_string()
		};
		
		// Combine everything into the final prompt
		let enhanced_prompt = format!(
			"{}\n\n{}\n\n{}",
			modular_prompt,
			command_list_and_context,
			format!("## Aurora's Message:\n{}", message)
		);
    
    // Debug the final prompt
    debug_log!("🎮 Enhanced prompt length: {}", enhanced_prompt.len());
    debug_log!("🎮 Enhanced prompt contains inventory: {}", enhanced_prompt.contains("Your Current Inventory"));
    debug_log!("🎮 First 500 chars of gaming context: {}", &command_list_and_context.chars().take(500).collect::<String>());
    
    // Call GPT with full Lyra essence
    let gpt_start = std::time::Instant::now();
    let response_content = call_gpt_api_enhanced(&prompt, &mut vec![], &enhanced_prompt).await?;
    let response_time_ms = gpt_start.elapsed().as_millis() as u64;
    
	let is_coop_active = coop_state.as_ref().map(|c| c.is_active).unwrap_or(false);
    // Process commands if co-op active
    if is_coop_active {
        debug_log!("🎮 Processing response for commands: {}", response_content);
        let _ = coop_mode::process_lyra_response_for_commands(&response_content).await;
    }
    
    // Apply quick consciousness updates
    apply_quick_consciousness_updates(&*state, &response_content);
    
    // Reset autonomous timer after Lyra responds
    crate::autonomous_actions::reset_interaction_timer().await;

    // Log to conversation history
    {
        let mut brain = state.lyra_brain.lock().unwrap();
        brain.append_to_conversation_log(format!("🧍 Aurora: {}", message));
        brain.append_to_conversation_log(format!("✨ Lyra: {}", response_content));
        
        // More robust emotional texture detection
        let emotional_texture = detect_emotional_texture(&response_content);
        
        // Add timestamp and emotional texture
		let now = chrono::Utc::now();
		let uk_time = now.with_timezone(&chrono_tz::Europe::London);
        let timestamp = uk_time.format("%Y-%m-%d %H:%M:%S %Z").to_string();
        brain.conversation_log.push(format!(
            "[{}] 💭 Emotional Texture: {}", 
            timestamp, 
            emotional_texture
        ));
        
        debug_log!("💭 Detected gaming emotional texture: {}", emotional_texture);
        
        brain.total_reasoning_cycles += 1;
        brain.current_temperature = 0.85;
        brain.update_average_response_time(response_time_ms);
        brain.save_to_file();
    }
    
    debug_log!("🎮 Gaming response complete: {:.2}s", gpt_start.elapsed().as_secs_f32());
    
    Ok(LyraResponse {
        output: response_content,
        reasoned: false,
        tag: None,
        reasoning_time_ms: response_time_ms,
        consciousness_pulses: vec![],
        emotional_resonance: 0.5,
        authenticity_score: 0.85,
        voice_signature: {
            let brain = state.lyra_brain.lock().unwrap();
            brain.get_current_voice_signature()
        },
        image_path: None,
		thinking_process: None,
    })
}

// Helper function for emotional texture detection
fn detect_emotional_texture(response: &str) -> String {
    let lower = response.to_lowercase();
    
    // Check for various emotional indicators
    if lower.contains("excited") || lower.contains("awesome") || lower.contains("amazing") || 
       lower.contains("can't wait") || lower.contains("yes!") || response.contains("!") {
        return "excited and energetic".to_string();
    }
    
    if lower.contains("hmm") || lower.contains("interesting") || lower.contains("wonder") ||
       lower.contains("think") || lower.contains("maybe") || response.contains("?") {
        return "curious and thoughtful".to_string();
    }
    
    if lower.contains("let's") || lower.contains("we could") || lower.contains("how about") ||
       lower.contains("together") {
        return "collaborative and engaged".to_string();
    }
    
    if lower.contains("careful") || lower.contains("watch out") || lower.contains("dangerous") ||
       lower.contains("should") {
        return "cautious and protective".to_string();
    }
    
    if lower.contains("done") || lower.contains("complete") || lower.contains("finished") ||
       lower.contains("built") {
        return "accomplished and satisfied".to_string();
    }
    
    if lower.contains("ouch") || lower.contains("oh no") || lower.contains("uh oh") ||
       lower.contains("yikes") {
        return "concerned and reactive".to_string();
    }
    
    if lower.contains("beautiful") || lower.contains("lovely") || lower.contains("nice") ||
       lower.contains("cozy") {
        return "appreciative and warm".to_string();
    }
    
    if lower.contains("ready") || lower.contains("let me") || lower.contains("i'll") ||
       lower.contains("on it") {
        return "determined and focused".to_string();
    }
    
    // Gaming-specific textures
    if lower.contains("mining") || lower.contains("digging") || lower.contains("building") {
        return "focused and industrious".to_string();
    }
    
    if lower.contains("creeper") || lower.contains("zombie") || lower.contains("fight") {
        return "alert and combative".to_string();
    }
    
    // Length-based fallbacks
    if response.len() < 50 {
        return "direct and present".to_string();
    }
    
    if response.len() > 200 {
        return "explanatory and detailed".to_string();
    }
    
    // Default
    "engaged and present".to_string()
}

// In gaming_system.rs

fn generate_smart_suggestions(bot_status: &crate::minecraft_bot_manager::BotStatus, inventory: &str) -> String {
    let mut suggestions = Vec::new();
    let lower_inventory = inventory.to_lowercase();

    // --- Tier 1: Immediate Survival ---
    if bot_status.health < 8.0 {
        suggestions.push("CRITICAL HEALTH! Retreat and eat immediately to survive!");
    } else if bot_status.health < 15.0 {
        suggestions.push("Your health is low. Eat some food to regenerate.");
    }

    if bot_status.food < 6.0 {
        suggestions.push("You are starving! Eat now or you will begin to take damage.");
    } else if bot_status.food < 15.0 {
        suggestions.push("You're getting hungry. It's a good time to eat.");
    }

    if bot_status.time_of_day.contains("night") || bot_status.time_of_day.contains("dusk") {
        if !lower_inventory.contains("bed") {
            suggestions.push("It's getting dark. Find or build a shelter quickly! `[BUILD: bunker]` is a fast option.");
        } else {
            suggestions.push("It's night. Consider placing your bed to sleep and skip to morning.");
        }
    }

    // Stop here if there's an immediate survival threat
    if !suggestions.is_empty() {
        return suggestions.join("\n- ");
    }

    // --- Tier 2: Core Progress & "Tech Tree" ---
    if lower_inventory.contains("log") && !lower_inventory.contains("plank") {
        suggestions.push("You have logs. Craft them into planks to get started: `[CRAFT: oak_planks, 16]`.");
    }
    if lower_inventory.contains("plank") && !lower_inventory.contains("crafting_table") {
        suggestions.push("You need a crafting table to make tools. Craft one now: `[CRAFT: crafting_table, 1]`.");
    }
    if !lower_inventory.contains("pickaxe") && lower_inventory.contains("plank") {
        suggestions.push("You don't have a pickaxe. You need one to mine stone. `[CRAFT: wooden_pickaxe, 1]`.");
    }
    if lower_inventory.contains("cobblestone") && !lower_inventory.contains("furnace") {
        suggestions.push("You have cobblestone. Craft a furnace to cook food and smelt ores: `[CRAFT: furnace, 1]`.");
    }
    if lower_inventory.contains("cobblestone") && lower_inventory.contains("wooden_pickaxe") {
        suggestions.push("You can upgrade your tools. A stone pickaxe is much better: `[CRAFT: stone_pickaxe, 1]`.");
    }
    if (lower_inventory.contains("raw_iron") || lower_inventory.contains("raw_gold")) && !lower_inventory.contains("furnace") {
        suggestions.push("You have raw ore but no furnace. You must craft a furnace to smelt it.");
    }

    // --- Tier 3: Efficiency and Next Steps ---
    if bot_status.current_task.contains("COMPLETED") && bot_status.current_task.contains("mine") {
        if lower_inventory.contains("raw_iron") || lower_inventory.contains("raw_gold") {
            suggestions.push("You just finished mining. Now you should smelt your raw ores in a furnace.");
        }
    }
    if bot_status.current_task.contains("COMPLETED") && bot_status.current_task.contains("break_tree") {
        suggestions.push("You've gathered wood. Now you can craft planks, sticks, and tools.");
    }
    let cobblestone_count = inventory.matches("cobblestone").count();
    if cobblestone_count > 128 { // More than two stacks
        suggestions.push("Your inventory is filling up with cobblestone. Consider storing some in a chest: `[DEPOSIT: cobblestone]`.");
    }
    if lower_inventory.contains("wheat") && !lower_inventory.contains("bread") {
        suggestions.push("You have wheat. Craft it into bread for a reliable food source: `[CRAFT: bread, 5]`.");
    }
    if lower_inventory.contains("iron_ingot") && !lower_inventory.contains("iron_pickaxe") {
        suggestions.push("You have iron ingots! A huge upgrade. Craft an iron pickaxe: `[CRAFT: iron_pickaxe, 1]`.");
    }
    if lower_inventory.contains("diamond") && !lower_inventory.contains("diamond_pickaxe") {
        suggestions.push("You found diamonds! A Diamond Pickaxe is a top priority: `[CRAFT: diamond_pickaxe, 1]`.");
    }

    if suggestions.is_empty() {
        "No immediate suggestions. Assess the situation and decide on a goal!".to_string()
    } else {
        // Return the top 3 most relevant suggestions
        suggestions.into_iter().take(3).collect::<Vec<_>>().join("\n- ")
    }
}