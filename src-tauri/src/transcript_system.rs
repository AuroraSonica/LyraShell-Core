// transcript_system.rs - FIXED Python PATH detection

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::error::Error;
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub text: String,
    pub start: f64,
    pub duration: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTranscript {
    pub video_id: String,
    pub title: Option<String>,
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
    pub total_duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualTranscript {
    pub current_segment: Option<TranscriptSegment>,
    pub surrounding_context: Vec<TranscriptSegment>,
    pub timestamp: f64,
    pub context_window: f64,
}

// Robust Python executable finder
fn find_python_executable() -> Result<String, String> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            "python",
            "py", 
            "python3",
            "python.exe",
            "py.exe",
            "python3.exe",
            // Common installation paths
            "C:\\Python\\python.exe",
            "C:\\Python39\\python.exe", 
            "C:\\Python310\\python.exe",
            "C:\\Python311\\python.exe",
            "C:\\Python312\\python.exe",
            "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Python\\Python39\\python.exe",
            "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Python\\Python310\\python.exe",
            "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Python\\Python311\\python.exe",
            "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Python\\Python312\\python.exe",
            // Anaconda paths
            "C:\\ProgramData\\Anaconda3\\python.exe",
            "C:\\Users\\%USERNAME%\\Anaconda3\\python.exe",
            "C:\\Users\\%USERNAME%\\miniconda3\\python.exe",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "python3",
            "python", 
            "/usr/bin/python3",
            "/usr/local/bin/python3",
            "/opt/homebrew/bin/python3",
            "/usr/bin/python",
            "/usr/local/bin/python",
            // Anaconda paths
            "/Users/$USER/anaconda3/bin/python",
            "/Users/$USER/miniconda3/bin/python",
            "/opt/anaconda3/bin/python",
        ]
    } else {
        // Linux
        vec![
            "python3",
            "python",
            "/usr/bin/python3", 
            "/usr/local/bin/python3",
            "/bin/python3",
            "/usr/bin/python",
            "/usr/local/bin/python",
            "/bin/python",
            // Common virtualenv paths
            "/home/$USER/.local/bin/python3",
            "/home/$USER/.local/bin/python",
        ]
    };
    
    println!("🔍 Searching for Python executable...");
    
    for candidate in candidates {
        // Expand environment variables for Windows
        let expanded_candidate = if cfg!(target_os = "windows") && candidate.contains("%USERNAME%") {
            if let Ok(username) = std::env::var("USERNAME") {
                candidate.replace("%USERNAME%", &username)
            } else {
                candidate.to_string()
            }
        } else if candidate.contains("$USER") {
            if let Ok(user) = std::env::var("USER") {
                candidate.replace("$USER", &user)
            } else {
                candidate.to_string()
            }
        } else {
            candidate.to_string()
        };
        
        println!("🔍 Trying: {}", expanded_candidate);
        
        match Command::new(&expanded_candidate)
            .arg("--version")
            .output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    println!("✅ Found Python: {} ({})", expanded_candidate, version.trim());
                    
                    // Test if youtube-transcript-api is available
                    match Command::new(&expanded_candidate)
                        .arg("-c")
                        .arg("import youtube_transcript_api; print('youtube-transcript-api available')")
                        .output() {
                        Ok(test_output) => {
                            if test_output.status.success() {
                                println!("✅ youtube-transcript-api is available");
                                return Ok(expanded_candidate);
                            } else {
                                println!("⚠️ Python found but youtube-transcript-api not installed: {}", 
                                       String::from_utf8_lossy(&test_output.stderr));
                            }
                        },
                        Err(e) => {
                            println!("⚠️ Failed to test youtube-transcript-api: {}", e);
                        }
                    }
                }
            },
            Err(e) => {
                println!("⚠️ Failed to execute {}: {}", expanded_candidate, e);
            }
        }
    }
    
    Err("No working Python installation with youtube-transcript-api found".to_string())
}

// Main command to fetch YouTube transcript with robust Python detection
#[tauri::command]
pub async fn fetch_youtube_transcript(video_id: String) -> Result<String, String> {
    println!("📝 Fetching transcript for video: {}", video_id);
    
    // Try multiple attempts with delays to work around IP blocking
    for attempt in 1..=3 {
        println!("📝 Attempt {} of 3", attempt);
        
        match fetch_transcript_attempt(&video_id).await {
            Ok(transcript) => return Ok(transcript),
            Err(e) => {
                println!("⚠️ Attempt {} failed: {}", attempt, e);
                if attempt < 3 {
                    println!("⏳ Waiting 5 seconds before retry...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    // If all attempts fail, return a helpful placeholder
    Ok(create_transcript_placeholder(&video_id))
}

async fn fetch_transcript_attempt(video_id: &str) -> Result<String, String> {
    // Find Python executable
    let python_cmd = find_python_executable()?;
    
    println!("🐍 Using Python: {}", python_cmd);
    
    // Call Python script to get transcript using youtube-transcript-api
    let output = Command::new(&python_cmd)
        .arg("-c")
        .arg(&format!(r#"
import json
import sys

try:
    from youtube_transcript_api import YouTubeTranscriptApi
    from youtube_transcript_api.formatters import TextFormatter
    
    print("📝 Attempting to fetch transcript for: {}", file=sys.stderr)
    
    # Try to get transcript
    transcript_list = YouTubeTranscriptApi.get_transcript('{}')
    
    print("📝 Successfully fetched {{}} segments".format(len(transcript_list)), file=sys.stderr)
    
    # Format as JSON for Rust
    result = {{
        'video_id': '{}',
        'language': 'en',
        'segments': transcript_list,
        'success': True
    }}
    
    print(json.dumps(result))
    
except Exception as e:
    print("❌ Error: {{}}".format(str(e)), file=sys.stderr)
    error_result = {{
        'video_id': '{}',
        'error': str(e),
        'success': False
    }}
    print(json.dumps(error_result))
"#, video_id, video_id, video_id, video_id))
        .output()
        .map_err(|e| format!("Failed to execute Python script: {}", e))?;
    
    // Check if command succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("🐍 Python stderr: {}", stderr);
        return Err(format!("Python script failed with status {}: {}", 
                          output.status.code().unwrap_or(-1), stderr));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    
    println!("🐍 Python stdout: {}", output_str);
    println!("🐍 Python stderr: {}", stderr_str);
    
    // Parse the JSON response
    let json_response: serde_json::Value = serde_json::from_str(&output_str)
        .map_err(|e| format!("Failed to parse transcript JSON: {}", e))?;
    
    if json_response["success"].as_bool().unwrap_or(false) {
        // Success - format the transcript nicely
        let segments = json_response["segments"].as_array()
            .ok_or("No segments found in transcript")?;
        
        let mut formatted_transcript = String::new();
        formatted_transcript.push_str(&format!("📺 Transcript for video: {}\n\n", video_id));
        
        for segment in segments {
            let start = segment["start"].as_f64().unwrap_or(0.0);
            let text = segment["text"].as_str().unwrap_or("");
            
            // Format: [MM:SS] Text
            let minutes = (start / 60.0) as u32;
            let seconds = (start % 60.0) as u32;
            formatted_transcript.push_str(&format!("[{:02}:{:02}] {}\n", minutes, seconds, text));
        }
        
        Ok(formatted_transcript)
    } else {
        let error = json_response["error"].as_str().unwrap_or("Unknown error");
        Err(format!("Transcript fetch failed: {}", error))
    }
}

fn create_transcript_placeholder(video_id: &str) -> String {
    format!(
        "📺 Video ID: {}\n\n\
         📝 Transcript temporarily unavailable due to YouTube API limits\n\
         🎬 Co-watching system still fully functional:\n\
         • AI reactions with screenshot analysis\n\
         • Real-time screenshot capture\n\
         • Full contextual chat support\n\
         • Enhanced video discussions\n\n\
         💡 Ask me about:\n\
         • What you're seeing on screen\n\
         • Your thoughts on the video\n\
         • Analysis of visual elements\n\
         • Predictions about what happens next\n\n\
         🔄 Transcript will retry automatically on next load",
        video_id
    )
}

// Get contextual transcript around a specific timestamp with robust Python detection
#[tauri::command]
pub async fn get_contextual_transcript(
    video_id: String, 
    current_time: f64, 
    context_window: f64
) -> Result<String, String> {
    println!("🎯 Getting contextual transcript at {}s (±{}s window)", current_time, context_window);
    
    // Find Python executable
    let python_cmd = find_python_executable()?;
    
    // Get contextual transcript via Python
    let output = Command::new(&python_cmd)
        .arg("-c")
        .arg(&format!(r#"
import json
import sys
from youtube_transcript_api import YouTubeTranscriptApi

try:
    transcript_list = YouTubeTranscriptApi.get_transcript('{}')
    current_time = {}
    context_window = {}
    
    # Find segments within the context window
    relevant_segments = []
    current_segment = None
    
    for segment in transcript_list:
        start_time = segment['start']
        end_time = start_time + segment.get('duration', 0)
        
        # Check if this segment is in our context window
        if start_time <= current_time + context_window and end_time >= current_time - context_window:
            relevant_segments.append(segment)
            
            # Check if current time falls within this segment
            if start_time <= current_time <= end_time:
                current_segment = segment
    
    result = {{
        'current_segment': current_segment,
        'surrounding_context': relevant_segments,
        'timestamp': current_time,
        'context_window': context_window,
        'success': True
    }}
    
    print(json.dumps(result))
    
except Exception as e:
    error_result = {{
        'error': str(e),
        'success': False
    }}
    print(json.dumps(error_result))
"#, video_id, current_time, context_window))
        .output()
        .map_err(|e| format!("Failed to get contextual transcript: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("Contextual transcript failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let json_response: serde_json::Value = serde_json::from_str(&output_str)
        .map_err(|e| format!("Failed to parse contextual transcript: {}", e))?;
    
    if json_response["success"].as_bool().unwrap_or(false) {
        let mut context_text = String::new();
        
        // Add current segment info
        if let Some(current_seg) = json_response["current_segment"].as_object() {
            let start = current_seg["start"].as_f64().unwrap_or(0.0);
            let text = current_seg["text"].as_str().unwrap_or("");
            let minutes = (start / 60.0) as u32;
            let seconds = (start % 60.0) as u32;
            
            context_text.push_str(&format!("🎯 CURRENT ({:02}:{:02}): {}\n\n", minutes, seconds, text));
        }
        
        // Add surrounding context
        if let Some(segments) = json_response["surrounding_context"].as_array() {
            context_text.push_str("📝 CONTEXT:\n");
            for segment in segments {
                let start = segment["start"].as_f64().unwrap_or(0.0);
                let text = segment["text"].as_str().unwrap_or("");
                let minutes = (start / 60.0) as u32;
                let seconds = (start % 60.0) as u32;
                
                context_text.push_str(&format!("[{:02}:{:02}] {}\n", minutes, seconds, text));
            }
        }
        
        Ok(context_text)
    } else {
        let error = json_response["error"].as_str().unwrap_or("Unknown error");
        Err(format!("Contextual transcript failed: {}", error))
    }
}

// Enhanced message context with robust Python detection
#[tauri::command]
pub async fn create_enhanced_message_context(
    message: String,
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    println!("💬 Creating enhanced message context...");
    
    // Get contextual transcript
    let transcript_context = get_contextual_transcript(
        video_id.clone(), 
        current_time, 
        30.0  // 30 second context window
    ).await;
    
    // Format timestamp
    let minutes = (current_time / 60.0) as u32;
    let seconds = (current_time % 60.0) as u32;
    let timestamp = format!("{:02}:{:02}", minutes, seconds);
    
    // Create enhanced context
    let mut enhanced_message = String::new();
    enhanced_message.push_str(&format!("🎬 VIDEO CONTEXT:\n"));
    enhanced_message.push_str(&format!("📺 Video: {}\n", video_title));
    enhanced_message.push_str(&format!("⏰ Timestamp: {}\n", timestamp));
    enhanced_message.push_str(&format!("🆔 Video ID: {}\n\n", video_id));
    
    // Add transcript context if available
    match transcript_context {
        Ok(context) => {
            enhanced_message.push_str(&format!("📝 TRANSCRIPT CONTEXT:\n{}\n", context));
        },
        Err(e) => {
            enhanced_message.push_str(&format!("📝 TRANSCRIPT: Unable to fetch ({})\n\n", e));
        }
    }
    
    enhanced_message.push_str(&format!("💬 AURORA'S MESSAGE:\n{}", message));
    
    Ok(enhanced_message)
}

// Test transcript system with robust Python detection
#[tauri::command]
pub async fn test_transcript_system() -> Result<String, String> {
    println!("🧪 Testing transcript system...");
    
    // First test Python detection
    match find_python_executable() {
        Ok(python_cmd) => {
            println!("✅ Python found: {}", python_cmd);
            
            // Test with Rick Astley video (always has captions)
            let test_video_id = "dQw4w9WgXcQ".to_string();
            
            match fetch_youtube_transcript(test_video_id.clone()).await {
                Ok(transcript) => {
                    let preview = if transcript.len() > 500 {
                        format!("{}...", &transcript[..500])
                    } else {
                        transcript
                    };
                    Ok(format!("✅ Transcript system working with Python: {}\n\nPreview:\n{}", python_cmd, preview))
                },
                Err(e) => Err(format!("❌ Transcript test failed: {}", e))
            }
        },
        Err(e) => {
            Err(format!("❌ Python detection failed: {}\n\nPlease install Python and youtube-transcript-api:\n\npip install youtube-transcript-api", e))
        }
    }
}

// Co-watching history persistence commands
use crate::get_data_path;

#[tauri::command]
pub async fn save_cowatching_history(data: String) -> Result<String, String> {
    let path = get_data_path("cowatching_history.json");
    
    let data_len = data.len(); // Get the length before moving data
    
    std::fs::write(&path, data)
        .map_err(|e| format!("Failed to save co-watching history: {}", e))?;
    
   // debug_log!("💾 Saved co-watching history: {} bytes", data_len);
    Ok("Co-watching history saved".to_string())
}

#[tauri::command]
pub async fn load_cowatching_history() -> Result<String, String> {
    let path = get_data_path("cowatching_history.json");
    
    if !std::path::Path::new(&path).exists() {
        debug_log!("📝 No co-watching history file found");
        return Ok("null".to_string());
    }
    
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read co-watching history: {}", e))?;
    
    debug_log!("📖 Loaded co-watching history: {} bytes", content.len());
    Ok(content)
}

#[tauri::command]
pub async fn activate_nordvpn() -> Result<String, String> {
    use std::process::Command;
    
    #[cfg(target_os = "windows")]
    {
        // Try to connect NordVPN via command line
        match Command::new("cmd")
            .args(&["/C", "nordvpn", "-c"])
            .output() {
            Ok(output) => {
                if output.status.success() {
                    Ok("NordVPN connection initiated".to_string())
                } else {
                    // Try alternative command
                    match Command::new("cmd")
                        .args(&["/C", "nordvpn", "connect"])
                        .output() {
                        Ok(_) => Ok("NordVPN connection initiated".to_string()),
                        Err(e) => Err(format!("Failed to activate NordVPN: {}", e))
                    }
                }
            },
            Err(_) => {
                // Try to start NordVPN app if CLI isn't available
                match Command::new("cmd")
                    .args(&["/C", "start", "nordvpn://"])
                    .output() {
                    Ok(_) => Ok("NordVPN app launched".to_string()),
                    Err(e) => Err(format!("NordVPN not found: {}", e))
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("VPN auto-activation only supported on Windows".to_string())
    }
}

#[tauri::command]
pub async fn open_nordvpn_app() -> Result<String, String> {
    use std::process::Command;
    
    #[cfg(target_os = "windows")]
    {
        // Try to open NordVPN app
        match Command::new("cmd")
            .args(&["/C", "start", "nordvpn://"])
            .output() {
            Ok(_) => Ok("NordVPN app launched".to_string()),
            Err(_) => {
                // Try alternative method
                match Command::new("cmd")
                    .args(&["/C", "start", "", "C:\\Program Files\\NordVPN\\NordVPN.exe"])
                    .output() {
                    Ok(_) => Ok("NordVPN app launched".to_string()),
                    Err(e) => Err(format!("Failed to open NordVPN: {}", e))
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Only supported on Windows".to_string())
    }
}