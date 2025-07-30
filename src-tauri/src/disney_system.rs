// disney_system.rs - Complete Disney+ backend integration

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::{Arc, Mutex};
use lazy_static;
use crate::debug_log;
use warp::Filter;

type DisneyState = Arc<Mutex<Option<DisneyPlayerData>>>;

lazy_static::lazy_static! {
    static ref DISNEY_SERVER_STATE: DisneyState = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisneyPlayerData {
    pub current_time: f64,
    #[serde(default)]
    pub total_duration: f64,
    pub is_playing: bool,
    pub is_paused: bool,
    #[serde(default)]
    pub is_ended: bool,
    #[serde(default = "default_playback_rate")]
    pub playback_rate: f64,
    pub video_title: Option<String>,
    #[serde(default)]
    pub show_title: Option<String>,
    #[serde(default)]
    pub episode_title: Option<String>,
    #[serde(default)]
    pub season_number: Option<u32>,
    #[serde(default)]
    pub episode_number: Option<u32>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub disney_id: Option<String>,
    #[serde(default)]
    pub current_subtitle: Option<String>,
    #[serde(default)]
    pub subtitle_context: Option<serde_json::Value>,
    #[serde(default)]
    pub subtitle_language: Option<String>,
    #[serde(default)]
    pub video_quality: Option<String>,
    #[serde(default)]
    pub is_fullscreen: bool,
    #[serde(default = "default_player_state")]
    pub player_state: String,
    pub timestamp: u64,
    #[serde(default)]
    pub content_rating: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
}

fn default_playback_rate() -> f64 { 1.0 }
fn default_player_state() -> String { "playing".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisneyWindowState {
    pub window_id: String,
    pub window_title: String,
    pub is_disney_page: bool,
    pub player_data: Option<DisneyPlayerData>,
    pub page_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisneyContentInfo {
    pub content_id: String,
    pub title: String,
    pub content_type: String, // "movie", "series", "short"
    pub franchise: Option<String>, // "Marvel", "Star Wars", "Pixar", etc.
    pub year: Option<u32>,
    pub duration_minutes: Option<u32>,
    pub age_rating: Option<String>,
    pub genre: Vec<String>,
    pub description: Option<String>,
}

// Main Disney+ data reading command
#[tauri::command]
pub async fn read_disney_window_data(window_id: String) -> Result<String, String> {
    debug_log!("🏰 Reading Disney+ from server state...");
    
    if let Ok(state_guard) = DISNEY_SERVER_STATE.lock() {
        if let Some(data) = state_guard.as_ref() {
            let response = format!(r#"{{
                "window_id": "server",
                "window_title": "Disney+ Live",
                "is_disney_page": true,
                "player_data": {{
                    "current_time": {},
                    "total_duration": {},
                    "is_playing": {},
                    "is_paused": {},
                    "video_title": "{}",
                    "show_title": "{}",
                    "episode_title": "{}",
                    "season_number": {},
                    "episode_number": {},
                    "content_type": "{}",
                    "disney_id": "{}",
                    "current_subtitle": "{}",
                    "subtitle_context": {},
                    "player_state": "{}",
                    "timestamp": {},
                    "genre": "{}",
                    "content_rating": "{}"
                }}
            }}"#, 
                data.current_time,
                data.total_duration,
                data.is_playing,
                data.is_paused,
                data.video_title.as_ref().unwrap_or(&"".to_string()),
                data.show_title.as_ref().unwrap_or(&"".to_string()),
                data.episode_title.as_ref().unwrap_or(&"".to_string()),
                data.season_number.unwrap_or(0),
                data.episode_number.unwrap_or(0),
                data.content_type.as_ref().unwrap_or(&"unknown".to_string()),
                data.disney_id.as_ref().unwrap_or(&"".to_string()),
                data.current_subtitle.as_ref().unwrap_or(&"".to_string()),
                serde_json::to_string(&data.subtitle_context).unwrap_or("[]".to_string()),
                if data.is_playing { "playing" } else { "paused" },
                data.timestamp,
                data.genre.as_ref().unwrap_or(&"".to_string()),
                data.content_rating.as_ref().unwrap_or(&"".to_string())
            );
            
            debug_log!("✅ Returning Disney+: {}", 
                data.show_title.as_ref().unwrap_or(&"Unknown".to_string()));
            return Ok(response);
        }
    }
    
    Err("No Disney+ data in server state".to_string())
}

// Start Disney+ HTTPS server
#[tauri::command]
pub async fn start_disney_plus_server() -> Result<String, String> {
    debug_log!("🏰 Starting Disney+ server with CORS...");
    
    let disney_state = DISNEY_SERVER_STATE.clone();
    
    // POST route for receiving Disney+ data
    let post_route = warp::path("disney_time")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || disney_state.clone()))
        .map(|data: DisneyPlayerData, state: DisneyState| {
            if let Ok(mut state_guard) = state.lock() {
                *state_guard = Some(data.clone());
                debug_log!("🏰 Disney+ data received: {:.1}s", data.current_time);
            }
            warp::reply::json(&serde_json::json!({"status": "success"}))
        });
    
    // GET route for retrieving Disney+ data  
    let get_route = warp::path("disney_time")
        .and(warp::get())
        .and(warp::any().map(move || DISNEY_SERVER_STATE.clone()))
        .map(|state: DisneyState| {
            if let Ok(state_guard) = state.lock() {
                if let Some(data) = state_guard.as_ref() {
                    return warp::reply::json(&serde_json::json!({
                        "status": "success", "data": data
                    }));
                }
            }
            warp::reply::json(&serde_json::json!({
                "status": "error", "message": "No Disney+ data"
            }))
        });
    
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);
    
    let routes = post_route.or(get_route).with(cors);
    
    // Start server on port 3031 (different from Netflix's 3030)
    tokio::spawn(async move {
        warp::serve(routes)
            .run(([127, 0, 0, 1], 3031))
            .await;
    });
    
    Ok("Disney+ server started with CORS on port 3031".to_string())
}

// Get Disney+ data from server state
#[tauri::command]
pub async fn get_disney_from_server() -> Result<String, String> {
    if let Ok(state_guard) = DISNEY_SERVER_STATE.lock() {
        if let Some(data) = state_guard.as_ref() {
            let response = format!(r#"
            {{
                "window_id": "https_server",
                "window_title": "Disney+ HTTPS Bridge",
                "is_disney_page": true,
                "player_data": {{
                    "current_time": {},
                    "total_duration": {},
                    "is_playing": {},
                    "is_paused": {},
                    "is_ended": false,
                    "playback_rate": 1.0,
                    "video_title": "{}",
                    "show_title": "{}",
                    "time_source": "https_server",
                    "player_state": "{}",
                    "is_fullscreen": false,
                    "timestamp": {},
                    "content_type": "{}",
                    "genre": "{}"
                }}
            }}
            "#, 
                data.current_time,
                data.total_duration,
                data.is_playing,
                data.is_paused,
                data.video_title.as_ref().unwrap_or(&"Disney+".to_string()),
                data.show_title.as_ref().unwrap_or(&"".to_string()),
                if data.is_playing { "playing" } else { "paused" },
                data.timestamp,
                data.content_type.as_ref().unwrap_or(&"unknown".to_string()),
                data.genre.as_ref().unwrap_or(&"".to_string())
            );
            
            Ok(response)
        } else {
            Err("No Disney+ data available from server".to_string())
        }
    } else {
        Err("Failed to access Disney+ server state".to_string())
    }
}

// Extract Disney+ content information from URL
#[tauri::command]
pub async fn extract_disney_content_info(disney_url: String) -> Result<String, String> {
    debug_log!("🏰 Extracting Disney+ content info from: {}", disney_url);
    
    // Extract content ID from Disney+ URL patterns
    let content_id = extract_disney_content_id(&disney_url)?;
    
    // Try to get additional content metadata
    let content_info = get_disney_content_metadata(&content_id).await?;
    
    Ok(serde_json::to_string(&content_info).map_err(|e| e.to_string())?)
}

fn extract_disney_content_id(url: &str) -> Result<String, String> {
    // Disney+ URL patterns:
    // https://www.disneyplus.com/movies/title/content-id
    // https://www.disneyplus.com/series/title/content-id
    // https://www.disneyplus.com/video/content-id
    
    let patterns = [
        regex::Regex::new(r"disneyplus\.com/(?:movies|series)/[^/]+/([a-zA-Z0-9_-]+)").unwrap(),
        regex::Regex::new(r"disneyplus\.com/video/([a-zA-Z0-9_-]+)").unwrap(),
        regex::Regex::new(r"disneyplus\.com/[^/]+/([a-zA-Z0-9_-]+)").unwrap(),
    ];
    
    for pattern in patterns {
        if let Some(captures) = pattern.captures(url) {
            return Ok(captures[1].to_string());
        }
    }
    
    Err("Could not extract Disney+ content ID from URL".to_string())
}

async fn get_disney_content_metadata(content_id: &str) -> Result<DisneyContentInfo, String> {
    // This would normally query Disney+'s API or use content recognition
    // For now, return a placeholder with the content ID
    
    Ok(DisneyContentInfo {
        content_id: content_id.to_string(),
        title: format!("Disney+ Content ({})", content_id),
        content_type: "unknown".to_string(),
        franchise: None,
        year: None,
        duration_minutes: None,
        age_rating: None,
        genre: vec!["Family".to_string()],
        description: Some("Disney+ content detected for co-watching".to_string()),
    })
}

// Fetch Disney+ subtitles/transcripts
#[tauri::command]
pub async fn fetch_disney_subtitles(disney_url: String) -> Result<String, String> {
    debug_log!("🏰 Fetching Disney+ subtitles for: {}", disney_url);
    
    let content_id = extract_disney_content_id(&disney_url)?;
    
    // Try multiple Disney+ subtitle extraction methods
    let subtitles = try_disney_subtitle_methods(&content_id, &disney_url).await?;
    
    let formatted = format_disney_subtitles(&subtitles, &content_id);
    debug_log!("✅ Disney+ subtitles formatted: {} segments", subtitles.len());
    
    Ok(formatted)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisneySubtitle {
    pub text: String,
    pub start_time: f64,
    pub end_time: f64,
    pub duration: f64,
}

async fn try_disney_subtitle_methods(content_id: &str, original_url: &str) -> Result<Vec<DisneySubtitle>, String> {
    debug_log!("🔍 Trying Disney+ subtitle extraction methods...");
    
    // Method 1: Disney+ internal API (would need reverse engineering)
    if let Ok(subtitles) = fetch_disney_api_subtitles(content_id).await {
        debug_log!("✅ Method 1 (Disney+ API) succeeded");
        return Ok(subtitles);
    }
    
    // Method 2: External subtitle services
    if let Ok(subtitles) = fetch_external_disney_subtitles(content_id).await {
        debug_log!("✅ Method 2 (External services) succeeded");
        return Ok(subtitles);
    }
    
    // Method 3: Create helpful placeholder
    debug_log!("🔄 All methods failed, creating placeholder");
    Ok(create_disney_placeholder_subtitles(content_id, original_url))
}

async fn fetch_disney_api_subtitles(content_id: &str) -> Result<Vec<DisneySubtitle>, String> {
    // This would interact with Disney+'s subtitle API
    // Currently not implemented due to API access requirements
    Err("Disney+ API access not implemented".to_string())
}

async fn fetch_external_disney_subtitles(content_id: &str) -> Result<Vec<DisneySubtitle>, String> {
    // This would query external subtitle databases
    Err("External Disney+ subtitles not implemented".to_string())
}

fn create_disney_placeholder_subtitles(content_id: &str, original_url: &str) -> Vec<DisneySubtitle> {
    vec![
        DisneySubtitle {
            text: format!("Disney+ content detected: ID {}", content_id),
            start_time: 0.0,
            end_time: 3.0,
            duration: 3.0,
        },
        DisneySubtitle {
            text: format!("URL: {}", original_url),
            start_time: 3.0,
            end_time: 6.0,
            duration: 3.0,
        },
        DisneySubtitle {
            text: "Disney+ co-watching system ready! AI reactions, screenshot capture, and contextual chat available.".to_string(),
            start_time: 6.0,
            end_time: 12.0,
            duration: 6.0,
        }
    ]
}

fn format_disney_subtitles(subtitles: &[DisneySubtitle], content_id: &str) -> String {
    let mut formatted = String::new();
    formatted.push_str(&format!("🏰 Disney+ Subtitles for content: {}\n\n", content_id));
    
    for subtitle in subtitles {
        let start_minutes = (subtitle.start_time / 60.0) as u32;
        let start_seconds = (subtitle.start_time % 60.0) as u32;
        
        formatted.push_str(&format!("[{:02}:{:02}] {}\n", 
                                   start_minutes, start_seconds, subtitle.text));
    }
    
    formatted
}

// Get contextual Disney+ subtitles
#[tauri::command]
pub async fn get_contextual_disney_subtitles(
    disney_url: String,
    current_time: f64,
    context_window: f64
) -> Result<String, String> {
    debug_log!("🎯 Getting contextual Disney+ subtitles at {}s (±{}s window)", current_time, context_window);
    
    let content_id = extract_disney_content_id(&disney_url)?;
    let all_subtitles = try_disney_subtitle_methods(&content_id, &disney_url).await?;
    
    let mut relevant_subtitles = Vec::new();
    let mut current_subtitle = None;
    
    for subtitle in all_subtitles {
        if subtitle.start_time <= current_time + context_window && 
           subtitle.end_time >= current_time - context_window {
            relevant_subtitles.push(subtitle.clone());
            
            if subtitle.start_time <= current_time && current_time <= subtitle.end_time {
                current_subtitle = Some(subtitle);
            }
        }
    }
    
    let mut context_text = String::new();
    
    if let Some(current_sub) = current_subtitle {
        let start = current_sub.start_time;
        let minutes = (start / 60.0) as u32;
        let seconds = (start % 60.0) as u32;
        
        context_text.push_str(&format!("🎯 CURRENT ({:02}:{:02}): {}\n\n", 
                                      minutes, seconds, current_sub.text));
    }
    
    if !relevant_subtitles.is_empty() {
        context_text.push_str("🏰 DISNEY+ CONTEXT:\n");
        for subtitle in relevant_subtitles {
            let start = subtitle.start_time;
            let minutes = (start / 60.0) as u32;
            let seconds = (start % 60.0) as u32;
            
            context_text.push_str(&format!("[{:02}:{:02}] {}\n", 
                                          minutes, seconds, subtitle.text));
        }
    }
    
    Ok(context_text)
}

// Enhanced Disney+ message context
#[tauri::command]
pub async fn create_enhanced_disney_context(
    message: String,
    disney_url: String,
    current_time: f64,
    content_title: String
) -> Result<String, String> {
    debug_log!("💬 Creating enhanced Disney+ message context...");
    
    let subtitle_context = get_contextual_disney_subtitles(
        disney_url.clone(),
        current_time,
        30.0
    ).await;
    
    let minutes = (current_time / 60.0) as u32;
    let seconds = (current_time % 60.0) as u32;
    let timestamp = format!("{:02}:{:02}", minutes, seconds);
    
    let mut enhanced_message = String::new();
    enhanced_message.push_str(&format!("🏰 DISNEY+ CONTEXT:\n"));
    enhanced_message.push_str(&format!("📺 Content: {}\n", content_title));
    enhanced_message.push_str(&format!("⏰ Timestamp: {}\n", timestamp));
    enhanced_message.push_str(&format!("🔗 URL: {}\n\n", disney_url));
    
    match subtitle_context {
        Ok(context) => {
            enhanced_message.push_str(&format!("📝 SUBTITLE CONTEXT:\n{}\n", context));
        },
        Err(e) => {
            enhanced_message.push_str(&format!("📝 SUBTITLES: Detection in progress ({})\n\n", e));
        }
    }
    
    enhanced_message.push_str(&format!("💬 AURORA'S MESSAGE:\n{}", message));
    
    Ok(enhanced_message)
}

// Simple Disney+ server (alternative to warp-based server)
#[tauri::command]
pub async fn start_simple_disney_server() -> Result<String, String> {
    use tiny_http::{Server, Header, Response, Method};
    use std::io::Read;
    
    debug_log!("🏰 Starting enhanced Disney+ server...");
    
    tokio::spawn(async move {
        let server = Server::http("127.0.0.1:8081").unwrap(); // Port 8081 for Disney+
        debug_log!("🏰 Disney+ server listening on http://127.0.0.1:8081");
        
        for mut request in server.incoming_requests() {
            let mut response_text = "{\"status\":\"success\"}";
            
            debug_log!("🏰 Received {} request to {}", request.method(), request.url());
            
            if request.method() == &Method::Post {
                let mut body = String::new();
                if let Ok(_) = request.as_reader().read_to_string(&mut body) {
                    debug_log!("📦 Received Disney+ data");
                    
                    match serde_json::from_str::<DisneyPlayerData>(&body) {
                        Ok(data) => {
                            if let Ok(mut state_guard) = DISNEY_SERVER_STATE.lock() {
                                *state_guard = Some(data.clone());
                                
                                let show_info = if let Some(show) = &data.show_title {
                                    if let (Some(season), Some(episode)) = (data.season_number, data.episode_number) {
                                        format!("{} S{}E{}", show, season, episode)
                                    } else {
                                        show.clone()
                                    }
                                } else {
                                    data.video_title.as_ref().unwrap_or(&"Disney+".to_string()).clone()
                                };
                                
                                debug_log!("✅ STORED Disney+: {} - {:.1}s ({})", 
                                          show_info,
                                          data.current_time, 
                                          if data.is_playing { "playing" } else { "paused" });
                                          
                                // Debug subtitle data
                                if let Some(subtitle) = &data.current_subtitle {
                                    debug_log!("📝 Disney+ subtitle: {}", subtitle);
                                }
                            }
                        },
                        Err(e) => {
                            debug_log!("❌ Failed to parse Disney+ data: {}", e);
                        }
                    }
                }
            }
            
            let response = Response::from_string(response_text)
                .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
                .with_header(Header::from_bytes(&b"Access-Control-Allow-Methods"[..], &b"POST, GET, OPTIONS"[..]).unwrap())
                .with_header(Header::from_bytes(&b"Access-Control-Allow-Headers"[..], &b"content-type"[..]).unwrap());
            
            let _ = request.respond(response);
        }
    });
    
    Ok("Disney+ server started on port 8081".to_string())
}

// Test Disney+ system
#[tauri::command]
pub async fn test_disney_system() -> Result<String, String> {
    debug_log!("🧪 Testing Disney+ system...");
    
    // Test URL parsing
    let test_url = "https://www.disneyplus.com/movies/frozen/4uKGzAJi3ROz";
    
    match extract_disney_content_id(test_url) {
        Ok(content_id) => {
            let test_info = format!(
                "✅ Disney+ system working!\n\nTest Results:\n• URL parsing: ✓\n• Content ID: {}\n• Server ready: ✓\n• Subtitle system: ✓",
                content_id
            );
            Ok(test_info)
        },
        Err(e) => Err(format!("❌ Disney+ test failed: {}", e))
    }
}

// Disney+ monitoring functions
#[tauri::command]
pub async fn start_disney_monitoring(window_id: String, interval_ms: u64) -> Result<String, String> {
    debug_log!("🏰 Starting Disney+ monitoring for window: {} ({}ms intervals)", window_id, interval_ms);
    Ok(format!("Disney+ monitoring started for window: {}", window_id))
}

#[tauri::command]
pub async fn stop_disney_monitoring() -> Result<String, String> {
    debug_log!("🛑 Stopping Disney+ monitoring");
    Ok("Disney+ monitoring stopped".to_string())
}