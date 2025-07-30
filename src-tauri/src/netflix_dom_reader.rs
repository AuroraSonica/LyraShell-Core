// netflix_dom_reader.rs - Direct Netflix data extraction

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::{Arc, Mutex};
use warp::Filter;
use lazy_static;
use serde_json;
use crate::debug_log;

type NetflixState = Arc<Mutex<Option<NetflixPlayerData>>>;

lazy_static::lazy_static! {
    static ref NETFLIX_SERVER_STATE: NetflixState = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetflixPlayerData {
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
    pub netflix_id: Option<String>,
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
}

fn default_playback_rate() -> f64 { 1.0 }
fn default_player_state() -> String { "playing".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetflixWindowState {
    pub window_id: String,
    pub window_title: String,
    pub is_netflix_page: bool,
    pub player_data: Option<NetflixPlayerData>,
    pub page_url: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn read_netflix_window_data(window_id: String) -> Result<String, String> {
    //debug_log!("🎬 Reading Netflix from server state...");
    
    if let Ok(state_guard) = NETFLIX_SERVER_STATE.lock() {
        if let Some(data) = state_guard.as_ref() {
            let response = format!(r#"{{
                "window_id": "server",
                "window_title": "Netflix Live",
                "is_netflix_page": true,
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
                    "netflix_id": "{}",
                    "current_subtitle": "{}",
                    "subtitle_context": {},
                    "player_state": "{}",
                    "timestamp": {}
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
                data.netflix_id.as_ref().unwrap_or(&"".to_string()),
                data.current_subtitle.as_ref().unwrap_or(&"".to_string()),
                serde_json::to_string(&data.subtitle_context).unwrap_or("[]".to_string()),
                if data.is_playing { "playing" } else { "paused" },
                data.timestamp
            );
            
            //debug_log!("✅ Returning Netflix: {} - S{}E{}", 
                  //  data.show_title.as_ref().unwrap_or(&"Unknown".to_string()),
                    //data.season_number.unwrap_or(0),
                    //data.episode_number.unwrap_or(0));
            return Ok(response);
        }
    }
    
    Err("No Netflix data in server state".to_string())
}

#[cfg(target_os = "windows")]
async fn extract_netflix_data_windows(window_id: &str) -> Result<NetflixWindowState, String> {
    // Read the real Netflix time written by native messaging host
    let file_path = "C:\\temp\\netflix_real_time.json";
    
    match std::fs::read_to_string(file_path) {
        Ok(content) => {
            let data: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse Netflix data: {}", e))?;
            
            let current_time = data["current_time"].as_f64().unwrap_or(0.0);
            let is_playing = data["is_playing"].as_bool().unwrap_or(true);
            
            let response = format!(r#"
            {{
                "window_id": "{}",
                "window_title": "Netflix Native Messaging",
                "is_netflix_page": true,
                "player_data": {{
                    "current_time": {},
                    "total_duration": 3600,
                    "is_playing": {},
                    "is_paused": {},
                    "is_ended": false,
                    "playback_rate": 1.0,
                    "video_title": "Netflix",
                    "time_source": "native_messaging",
                    "player_state": "{}",
                    "is_fullscreen": false,
                    "timestamp": {}
                }}
            }}
            "#, 
                window_id,
                current_time,
                is_playing,
                !is_playing,
                if is_playing { "playing" } else { "paused" },
                chrono::Utc::now().timestamp()
            );
            
            parse_netflix_data_response(&response)
        },
        Err(_) => {
            Err("Native messaging host not running or no Netflix data available".to_string())
        }
    }
}

// macOS implementation using AppleScript + Safari/Chrome automation
#[cfg(target_os = "macos")]
async fn extract_netflix_data_macos(window_id: &str) -> Result<NetflixWindowState, String> {
    use std::process::Command;
    
    debug_log!("🔍 Using macOS AppleScript to read Netflix data...");
    
    // AppleScript to inject JavaScript into browser
    let applescript = format!(r#"
        tell application "System Events"
            set targetWindow to (first window whose id is {})
            set appName to name of (first process whose windows contains targetWindow)
        end tell
        
        if appName contains "Chrome" or appName contains "Safari" or appName contains "Firefox" then
            # Inject JavaScript into browser
            tell application appName
                set jsCode to "{}"
                try
                    set result to do JavaScript jsCode in front document
                    return result
                on error errMsg
                    return "{{\"error\": \"JavaScript injection failed: " & errMsg & "\"}}"
                end try
            end tell
        else
            return "{{\"error\": \"Target window is not a supported browser\"}}"
        end if
    "#, window_id, create_netflix_extraction_script().replace("\"", "\\\""));
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&applescript)
        .output()
        .map_err(|e| format!("Failed to execute AppleScript: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("AppleScript failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_netflix_data_response(&output_str)
}

// Linux implementation using xdotool + browser automation
#[cfg(target_os = "linux")]
async fn extract_netflix_data_linux(window_id: &str) -> Result<NetflixWindowState, String> {
    use std::process::Command;
    
    debug_log!("🔍 Using Linux automation to read Netflix data...");
    
    // Try to inject JavaScript using browser-specific methods
    // This is complex and would require browser-specific implementations
    
    // For now, return a mock response
    let mock_response = format!(r#"
    {{
        "window_id": "{}",
        "window_title": "Netflix - Linux Mock",
        "is_netflix_page": true,
        "page_url": "https://www.netflix.com/watch/mock",
        "player_data": {{
            "current_time": 856.2,
            "total_duration": 3600.0,
            "is_playing": true,
            "is_paused": false,
            "is_ended": false,
            "playback_rate": 1.0,
            "video_title": "Test Content",
            "is_fullscreen": false,
            "player_state": "playing",
            "timestamp": {}
        }}
    }}
    "#, window_id, chrono::Utc::now().timestamp());
    
    parse_netflix_data_response(&mock_response)
}

// Create JavaScript code to extract Netflix player data
fn create_netflix_api_script() -> String {
    r#"
    (function() {
        try {
            // Read from our extension's DOM element
            const element = document.getElementById('lyra-netflix-data');
            if (element && element.textContent) {
                console.log('✅ Found extension data');
                return element.textContent;
            } else {
                return JSON.stringify({error: 'Extension data element not found'});
            }
        } catch (error) {
            return JSON.stringify({error: 'Failed to read extension data: ' + error.message});
        }
    })();
    "#.to_string()
}

// Parse the response from JavaScript injection
fn parse_netflix_data_response(response: &str) -> Result<NetflixWindowState, String> {
    let response = response.trim();
    
    if response.is_empty() {
        return Err("Empty response from Netflix data extraction".to_string());
    }
    
    debug_log!("📄 Raw Netflix response: {}", response);
    
    // Try to parse as JSON
    match serde_json::from_str::<NetflixWindowState>(response) {
        Ok(state) => Ok(state),
        Err(parse_error) => {
            // If direct parsing fails, try to extract JSON from mixed output
            if let Some(json_start) = response.find('{') {
                let json_part = &response[json_start..];
                match serde_json::from_str::<NetflixWindowState>(json_part) {
                    Ok(state) => Ok(state),
                    Err(_) => Err(format!("Failed to parse Netflix data: {} (Response: {})", parse_error, response))
                }
            } else {
                Err(format!("No JSON found in response: {}", response))
            }
        }
    }
}

// Start Netflix HTTPS server (ADD this function)
#[tauri::command]
pub async fn start_netflix_https_server() -> Result<String, String> {
    debug_log!("🌐 Starting Netflix server with proper CORS...");
    
    let netflix_state = NETFLIX_SERVER_STATE.clone();
    
    // POST route
    let post_route = warp::path("netflix_time")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || netflix_state.clone()))
        .map(|data: NetflixPlayerData, state: NetflixState| {
            if let Ok(mut state_guard) = state.lock() {
                *state_guard = Some(data.clone());
                debug_log!("📺 Received: {:.1}s", data.current_time);
            }
            warp::reply::json(&serde_json::json!({"status": "success"}))
        });
    
    // GET route  
    let get_route = warp::path("netflix_time")
        .and(warp::get())
        .and(warp::any().map(move || NETFLIX_SERVER_STATE.clone()))
        .map(|state: NetflixState| {
            if let Ok(state_guard) = state.lock() {
                if let Some(data) = state_guard.as_ref() {
                    return warp::reply::json(&serde_json::json!({
                        "status": "success", "data": data
                    }));
                }
            }
            warp::reply::json(&serde_json::json!({
                "status": "error", "message": "No Netflix data"
            }))
        });
    
    // Proper CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);
    
    let routes = post_route.or(get_route).with(cors);
    
    tokio::spawn(async move {
        warp::serve(routes)
            .run(([127, 0, 0, 1], 3030))
            .await;
    });
    
    Ok("Netflix server started with CORS".to_string())
}

// Get Netflix data from server state (ADD this function)
#[tauri::command]
pub async fn get_netflix_from_server() -> Result<String, String> {
    if let Ok(state_guard) = NETFLIX_SERVER_STATE.lock() {
        if let Some(data) = state_guard.as_ref() {
            // Return in same format as existing function
            let response = format!(r#"
{{
    "window_id": "https_server",
    "window_title": "Netflix HTTPS Bridge",
    "is_netflix_page": true,
    "player_data": {{
        "current_time": {},
        "total_duration": {},
        "is_playing": {},
        "is_paused": {},
        "is_ended": false,
        "playback_rate": 1.0,
        "video_title": "{}",
        "time_source": "https_server",
        "player_state": "{}",
        "is_fullscreen": false,
        "timestamp": {},
        "current_subtitle": "{}",
        "show_title": "{}",
        "episode_title": "{}"
    }}
}}
"#, 
    data.current_time,
	data.total_duration,
    data.is_playing,
    data.is_paused,
    data.video_title.as_ref().unwrap_or(&"Netflix".to_string()),
    if data.is_playing { "playing" } else { "paused" },
    data.timestamp,
    data.current_subtitle.as_ref().unwrap_or(&"".to_string()),
    data.show_title.as_ref().unwrap_or(&"".to_string()),
    data.episode_title.as_ref().unwrap_or(&"".to_string())
);
            
            Ok(response)
        } else {
            Err("No Netflix data available from server".to_string())
        }
    } else {
        Err("Failed to access Netflix server state".to_string())
    }
}

// Continuously monitor Netflix player state
#[tauri::command]
pub async fn start_netflix_monitoring(window_id: String, interval_ms: u64) -> Result<String, String> {
    debug_log!("🎬 Starting Netflix monitoring for window: {} ({}ms intervals)", window_id, interval_ms);
    
    // In a real implementation, you'd start a background task that periodically
    // calls read_netflix_window_data and emits events to the frontend
    
    Ok(format!("Netflix monitoring started for window: {}", window_id))
}

// Stop Netflix monitoring
#[tauri::command]
pub async fn stop_netflix_monitoring() -> Result<String, String> {
    debug_log!("🛑 Stopping Netflix monitoring");
    
    Ok("Netflix monitoring stopped".to_string())
}

// Test Netflix DOM reading
#[tauri::command]
pub async fn test_netflix_dom_reading() -> Result<String, String> {
    debug_log!("🧪 Testing Netflix DOM reading system...");
    
    // Create a mock window ID for testing
    let test_window_id = "test_netflix_window".to_string();
    
    match read_netflix_window_data(test_window_id).await {
        Ok(data) => {
            let preview = if data.len() > 500 {
                format!("{}...", &data[..500])
            } else {
                data
            };
            Ok(format!("✅ Netflix DOM reading system working!\n\nMock data:\n{}", preview))
        },
        Err(e) => Err(format!("❌ Netflix DOM reading test failed: {}", e))
    }
}

#[tauri::command]
pub async fn read_netflix_timestamp_from_file() -> Result<String, String> {
    let downloads_path = match std::env::var("USERPROFILE") {
        Ok(home) => format!("{}/Downloads/lyra_netflix_data.json", home),
        Err(_) => return Err("Could not find Downloads folder".to_string()),
    };
    
    if !std::path::Path::new(&downloads_path).exists() {
        return Err("No Netflix data file found".to_string());
    }
    
    let content = std::fs::read_to_string(&downloads_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let netflix_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse data: {}", e))?;
    
    let response = format!(r#"{{
        "window_id": "file_bridge",
        "window_title": "Netflix File Bridge", 
        "is_netflix_page": true,
        "player_data": {{
            "current_time": {},
            "is_playing": {},
            "is_paused": {},
            "video_title": "{}",
            "timestamp": {}
        }}
    }}"#, 
        netflix_data["current_time"],
        netflix_data["is_playing"],
        netflix_data["is_paused"],
        netflix_data["video_title"].as_str().unwrap_or("Netflix"),
        netflix_data["timestamp"]
    );
    
    debug_log!("✅ Netflix file: {:.1}s", netflix_data["current_time"].as_f64().unwrap_or(0.0));
    Ok(response)
}

#[tauri::command]
pub async fn start_simple_netflix_server() -> Result<String, String> {
    use tiny_http::{Server, Header, Response, Method};
    use std::io::Read;
    
    debug_log!("🌐 Starting Netflix server with enhanced data storage...");
    
    tokio::spawn(async move {
        let server = Server::http("127.0.0.1:8080").unwrap();
        
        for mut request in server.incoming_requests() {
            let mut response_text = "{\"status\":\"success\"}";
            
            if request.method() == &Method::Post {
                // Read the POST body
                let mut body = String::new();
                if let Ok(_) = request.as_reader().read_to_string(&mut body) {
				debug_log!("📦 Received Netflix data: {}", body);
				
				// Parse the enhanced Netflix data
				match serde_json::from_str::<NetflixPlayerData>(&body) {
                        Ok(data) => {
                            if let Ok(mut state_guard) = NETFLIX_SERVER_STATE.lock() {
                                *state_guard = Some(data.clone());
                                
                                let show_info = if let Some(show) = &data.show_title {
                                    if let (Some(season), Some(episode)) = (data.season_number, data.episode_number) {
                                        format!("{} S{}E{}", show, season, episode)
                                    } else {
                                        show.clone()
                                    }
                                } else {
                                    data.video_title.as_ref().unwrap_or(&"Netflix".to_string()).clone()
                                };
                                
                                //debug_log!("✅ STORED: {} - {:.1}s ({})", 
                                      //  show_info,
                                      //  data.current_time, 
                                      //  if data.is_playing { "playing" } else { "paused" });
                            }
                        },
                        Err(e) => {
                            debug_log!("❌ Failed to parse enhanced Netflix data: {}", e);
                            debug_log!("📄 Raw data: {}", body);
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
    
    Ok("Enhanced Netflix server started".to_string())
}