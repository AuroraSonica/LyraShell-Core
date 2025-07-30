// spotify_system.rs - Complete embedded Spotify system

use serde::{Deserialize, Serialize};
use std::error::Error;
use reqwest;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTrackData {
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub duration_ms: u64,
    pub current_position_ms: u64,
    pub is_playing: bool,
    pub is_paused: bool,
    pub track_uri: String,
    pub album_uri: String,
    pub artist_uri: String,
    pub progress_percentage: f64,
    pub timestamp: u64,
    pub album_artwork_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyLyricLine {
    pub text: String,
    pub start_time_ms: u64,
    pub end_time_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyLyrics {
    pub track_id: String,
    pub track_name: String,
    pub artist_name: String,
    pub language: String,
    pub lines: Vec<SpotifyLyricLine>,
    pub source: String,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyContextualLyrics {
    pub current_line: Option<SpotifyLyricLine>,
    pub surrounding_context: Vec<SpotifyLyricLine>,
    pub current_position_ms: u64,
    pub context_window_ms: u64,
    pub track_info: SpotifyTrackData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyAuthState {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub is_authenticated: bool,
    pub client_id: String,
    pub client_secret: String,
}

// Global state for Spotify auth with persistence
use std::sync::{Arc, Mutex};
use lazy_static;
use std::fs;
use std::path::PathBuf;

const CLIENT_ID: &str = "f2f4c547862e42a1b2601a4a72b86ac2";
const CLIENT_SECRET: &str = "77f5134586fc44c5b18ec3fcacdcacb4";

lazy_static::lazy_static! {
    static ref SPOTIFY_AUTH_STATE: Arc<Mutex<SpotifyAuthState>> = Arc::new(Mutex::new(SpotifyAuthState {
        access_token: None,
        refresh_token: None,
        expires_at: None,
        is_authenticated: false,
        client_id: CLIENT_ID.to_string(),
        client_secret: CLIENT_SECRET.to_string(),
    }));
}

// Load saved tokens on startup
fn load_saved_tokens() -> Result<SpotifyAuthState, String> {
    let token_path = get_token_file_path()?;
    
    if !token_path.exists() {
        return Ok(SpotifyAuthState {
            access_token: None,
            refresh_token: None,
            expires_at: None,
            is_authenticated: false,
            client_id: CLIENT_ID.to_string(),
            client_secret: CLIENT_SECRET.to_string(),
        });
    }
    
    let content = fs::read_to_string(&token_path)
        .map_err(|e| format!("Failed to read token file: {}", e))?;
    
    let mut auth_state: SpotifyAuthState = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse token file: {}", e))?;
    
    // Always use hardcoded credentials
    auth_state.client_id = CLIENT_ID.to_string();
    auth_state.client_secret = CLIENT_SECRET.to_string();
    
    Ok(auth_state)
}

// Save tokens to disk
fn save_tokens(auth_state: &SpotifyAuthState) -> Result<(), String> {
    let token_path = get_token_file_path()?;
    
    // Create directory if it doesn't exist
    if let Some(parent) = token_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create token directory: {}", e))?;
    }
    
    let content = serde_json::to_string_pretty(auth_state)
        .map_err(|e| format!("Failed to serialize tokens: {}", e))?;
    
    fs::write(&token_path, content)
        .map_err(|e| format!("Failed to save token file: {}", e))?;
    
    Ok(())
}

fn get_token_file_path() -> Result<PathBuf, String> {
    // Try to use system config directory, fallback to current directory
    let mut path = match std::env::var("APPDATA") {
        Ok(appdata) => PathBuf::from(appdata), // Windows
        Err(_) => match std::env::var("HOME") {
            Ok(home) => {
                let mut home_path = PathBuf::from(home);
                home_path.push(".config"); // Linux/macOS
                home_path
            },
            Err(_) => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), // Fallback
        }
    };
    
    path.push("LyraShell");
    path.push("spotify_tokens.json");
    
    Ok(path)
}

// Initialize the auth state (call this on app startup)
#[tauri::command]
pub async fn initialize_spotify_auth() -> Result<String, String> {
    println!("🎵 Initializing Spotify authentication...");
    
    let saved_state = load_saved_tokens()?;
    
    {
        let mut auth_state = SPOTIFY_AUTH_STATE.lock()
            .map_err(|e| format!("Failed to lock auth state: {}", e))?;
        *auth_state = saved_state;
    }
    
    Ok("Spotify auth initialized".to_string())
}

// Clear stored tokens for re-authorization
#[tauri::command]
pub async fn clear_spotify_tokens() -> Result<String, String> {
    println!("🗑️ Clearing Spotify tokens...");
    
    // Clear in-memory state
    {
        let mut auth_state = SPOTIFY_AUTH_STATE.lock()
            .map_err(|e| format!("Failed to lock auth state: {}", e))?;
        
        auth_state.access_token = None;
        auth_state.refresh_token = None;
        auth_state.expires_at = None;
        auth_state.is_authenticated = false;
    }
    
    // Delete token file
    let token_path = get_token_file_path()?;
    if token_path.exists() {
        std::fs::remove_file(&token_path)
            .map_err(|e| format!("Failed to delete token file: {}", e))?;
    }
    
    Ok("Spotify tokens cleared successfully".to_string())
}

// Setup initial tokens from auth code (one-time)
#[tauri::command]
pub async fn setup_spotify_tokens(auth_code: String, client_id: String, client_secret: String) -> Result<String, String> {

    println!("🎵 Setting up Spotify tokens from auth code...");
    
    let client = reqwest::Client::new();
    
    let params = [
        ("grant_type", "authorization_code"),
        ("code", &auth_code),
        ("redirect_uri", "https://developer.spotify.com/documentation/"),
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
    ];
    
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status_code = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Ok(serde_json::json!({
            "success": false,
            "error": format!("HTTP {}: {}", status_code, error_text)
        }).to_string());
    }
    
    let token_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    
    let access_token = token_data["access_token"].as_str()
        .ok_or("No access token in response")?
        .to_string();
    
    let refresh_token = token_data["refresh_token"].as_str()
        .ok_or("No refresh token in response")?
        .to_string();
    
    let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + expires_in;
    
    // Store tokens
    let new_auth_state = SpotifyAuthState {
        access_token: Some(access_token),
        refresh_token: Some(refresh_token),
        expires_at: Some(expires_at),
        is_authenticated: true,
        client_id: CLIENT_ID.to_string(),
        client_secret: CLIENT_SECRET.to_string(),
    };
    
    // Save to disk
    save_tokens(&new_auth_state)?;
    
    // Update in-memory state
    {
        let mut auth_state = SPOTIFY_AUTH_STATE.lock()
            .map_err(|e| format!("Failed to lock auth state: {}", e))?;
        *auth_state = new_auth_state;
    }
    
    println!("✅ Spotify tokens saved permanently!");
    
    Ok(serde_json::json!({
        "success": true,
        "message": "Tokens saved permanently"
    }).to_string())
}

// Auto-refresh access token if needed
async fn ensure_valid_token() -> Result<String, String> {
    let (needs_refresh, refresh_token) = {
        let auth_state = SPOTIFY_AUTH_STATE.lock()
            .map_err(|e| format!("Failed to lock auth state: {}", e))?;
        
        if !auth_state.is_authenticated {
            return Err("Not authenticated".to_string());
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let expires_at = auth_state.expires_at.unwrap_or(0);
        let needs_refresh = expires_at <= now + 300; // Refresh 5 minutes early
        
        if !needs_refresh {
            return Ok(auth_state.access_token.as_ref().unwrap().clone());
        }
        
        (needs_refresh, auth_state.refresh_token.as_ref().unwrap().clone())
    };
    
    if !needs_refresh {
        return Ok("Token still valid".to_string());
    }
    
    println!("🔄 Refreshing Spotify access token...");
    
    // Refresh the token
    let client = reqwest::Client::new();
    
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
    ];
    
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Token refresh HTTP error: {}", response.status()));
    }
    
    let token_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;
    
    let new_access_token = token_data["access_token"].as_str()
        .ok_or("No access token in refresh response")?
        .to_string();
    
    let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);
    let new_expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + expires_in;
    
    // Update tokens
    {
        let mut auth_state = SPOTIFY_AUTH_STATE.lock()
            .map_err(|e| format!("Failed to lock auth state: {}", e))?;
        
        auth_state.access_token = Some(new_access_token.clone());
        auth_state.expires_at = Some(new_expires_at);
        
        // Save to disk
        save_tokens(&auth_state).ok(); // Don't fail if save fails
    }
    
    println!("✅ Access token refreshed successfully!");
    
    Ok(new_access_token)
}

// Get current playing track with auto-refresh
#[tauri::command]
pub async fn get_current_spotify_track() -> Result<String, String> {
    let token = ensure_valid_token().await?;
    
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch current track: {}", e))?;
    
    if response.status() == 204 {
        return Err("No track currently playing".to_string());
    }
    
    if !response.status().is_success() {
        return Err(format!("Spotify API error: {}", response.status()));
    }
    
    let spotify_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Spotify response: {}", e))?;
    
    let track_data = parse_spotify_track_data(&spotify_data)?;
    
    Ok(serde_json::to_string(&track_data).map_err(|e| e.to_string())?)
}

fn parse_spotify_track_data(data: &serde_json::Value) -> Result<SpotifyTrackData, String> {
    let item = data["item"].as_object().ok_or("No track item found")?;
    let track_name = item["name"].as_str().ok_or("No track name")?.to_string();
    let track_id = item["id"].as_str().ok_or("No track ID")?.to_string();
    let track_uri = item["uri"].as_str().ok_or("No track URI")?.to_string();
    
    let artists = item["artists"].as_array().ok_or("No artists found")?;
    let artist_name = artists.get(0)
        .and_then(|a| a["name"].as_str())
        .ok_or("No artist name")?
        .to_string();
    let artist_uri = artists.get(0)
        .and_then(|a| a["uri"].as_str())
        .unwrap_or("")
        .to_string();
    
    let album = item["album"].as_object().ok_or("No album found")?;
    let album_name = album["name"].as_str().ok_or("No album name")?.to_string();
    let album_uri = album["uri"].as_str().unwrap_or("").to_string();
    
    // Get album artwork
    let album_artwork_url = album["images"].as_array()
        .and_then(|images| images.get(0))
        .and_then(|image| image["url"].as_str())
        .map(|url| url.to_string());
    
    let duration_ms = item["duration_ms"].as_u64().ok_or("No duration")?;
    let current_position_ms = data["progress_ms"].as_u64().unwrap_or(0);
    let is_playing = data["is_playing"].as_bool().unwrap_or(false);
    
    let progress_percentage = if duration_ms > 0 {
        (current_position_ms as f64 / duration_ms as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(SpotifyTrackData {
        track_id,
        track_name,
        artist_name,
        album_name,
        duration_ms,
        current_position_ms,
        is_playing,
        is_paused: !is_playing,
        track_uri,
        album_uri,
        artist_uri,
        progress_percentage,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        album_artwork_url,
    })
}

// Fetch lyrics for a track (placeholder implementation)
#[tauri::command]
pub async fn fetch_spotify_lyrics(track_id: String, track_name: String, artist_name: String) -> Result<String, String> {
    println!("🎵 Fetching real lyrics for: {} - {}", artist_name, track_name);
    
    // Try Genius API for real lyrics
    match fetch_genius_lyrics(&artist_name, &track_name).await {
        Ok(lyrics) => Ok(serde_json::to_string(&lyrics).map_err(|e| e.to_string())?),
        Err(_) => {
            // Fallback to contextual lyrics if real lyrics fail
            let lyrics = create_contextual_lyrics(&track_id, &track_name, &artist_name);
            Ok(serde_json::to_string(&lyrics).map_err(|e| e.to_string())?)
        }
    }
}

async fn fetch_genius_lyrics(artist: &str, song: &str) -> Result<SpotifyLyrics, String> {
    // Add Genius API integration here
    // For now, return error to use fallback
    Err("Real lyrics API not implemented yet".to_string())
}

fn create_contextual_lyrics(track_id: &str, track_name: &str, artist_name: &str) -> SpotifyLyrics {
    let lyrics_lines = vec![
        SpotifyLyricLine {
            text: format!("🎵 Now playing: {} - {}", artist_name, track_name),
            start_time_ms: 0,
            end_time_ms: Some(4000),
            duration_ms: Some(4000),
        },
        SpotifyLyricLine {
            text: "🎼 Full lyrics integration coming soon with real-time sync".to_string(),
            start_time_ms: 4000,
            end_time_ms: Some(8000),
            duration_ms: Some(4000),
        },
        SpotifyLyricLine {
            text: "🎧 Co-listening with Lyra - AI reactions and full context available".to_string(),
            start_time_ms: 8000,
            end_time_ms: Some(12000),
            duration_ms: Some(4000),
        },
        SpotifyLyricLine {
            text: "✨ Ask me about the music, the artist, or how this song makes you feel".to_string(),
            start_time_ms: 12000,
            end_time_ms: Some(16000),
            duration_ms: Some(4000),
        },
    ];
    
    SpotifyLyrics {
        track_id: track_id.to_string(),
        track_name: track_name.to_string(),
        artist_name: artist_name.to_string(),
        language: "en".to_string(),
        lines: lyrics_lines,
        source: "lyra_contextual".to_string(),
        total_duration_ms: 16000,
    }
}

// Get contextual lyrics around current track position
#[tauri::command]
pub async fn get_contextual_spotify_lyrics(
    track_id: String,
    current_position_ms: u64,
    context_window_ms: u64
) -> Result<String, String> {
    println!("🎯 Getting contextual lyrics at {}ms (±{}ms window)", current_position_ms, context_window_ms);
    
    let track_data_str = get_current_spotify_track().await?;
    let track_data: SpotifyTrackData = serde_json::from_str(&track_data_str)
        .map_err(|e| format!("Failed to parse track data: {}", e))?;
    
    let lyrics_str = fetch_spotify_lyrics(
        track_data.track_id.clone(),
        track_data.track_name.clone(),
        track_data.artist_name.clone()
    ).await?;
    
    let lyrics: SpotifyLyrics = serde_json::from_str(&lyrics_str)
        .map_err(|e| format!("Failed to parse lyrics: {}", e))?;
    
    let window_start = current_position_ms.saturating_sub(context_window_ms);
    let window_end = current_position_ms + context_window_ms;
    
    let mut relevant_lines = Vec::new();
    let mut current_line = None;
    
    for line in &lyrics.lines {
        let line_start = line.start_time_ms;
        let line_end = line.end_time_ms.unwrap_or(line_start + 4000);
        
        if line_start <= window_end && line_end >= window_start {
            relevant_lines.push(line.clone());
            
            if current_position_ms >= line_start && current_position_ms <= line_end {
                current_line = Some(line.clone());
            }
        }
    }
    
    let contextual_lyrics = SpotifyContextualLyrics {
        current_line,
        surrounding_context: relevant_lines,
        current_position_ms,
        context_window_ms,
        track_info: track_data,
    };
    
    let formatted = format_contextual_lyrics(&contextual_lyrics);
    Ok(formatted)
}

fn format_contextual_lyrics(contextual: &SpotifyContextualLyrics) -> String {
    let mut context_text = String::new();
    
    if let Some(current_line) = &contextual.current_line {
        let timestamp = format_ms_to_time(current_line.start_time_ms);
        context_text.push_str(&format!("🎵 CURRENT ({}): {}\n\n", timestamp, current_line.text));
    }
    
    if !contextual.surrounding_context.is_empty() {
        context_text.push_str("🎼 LYRA CONTEXT:\n");
        for line in &contextual.surrounding_context {
            let timestamp = format_ms_to_time(line.start_time_ms);
            context_text.push_str(&format!("[{}] {}\n", timestamp, line.text));
        }
    }
    
    context_text
}

// Create enhanced message context for Spotify
#[tauri::command]
pub async fn create_enhanced_spotify_context(
    message: String,
    track_id: String,
    current_position_ms: u64,
    track_name: String,
    artist_name: String
) -> Result<String, String> {
    println!("💬 Creating enhanced Spotify message context...");
    
    let lyrics_context = get_contextual_spotify_lyrics(
        track_id.clone(),
        current_position_ms,
        30000
    ).await;
    
    let timestamp = format_ms_to_time(current_position_ms);
    
    let mut enhanced_message = String::new();
    enhanced_message.push_str(&format!("🎵 SPOTIFY CONTEXT:\n"));
    enhanced_message.push_str(&format!("🎧 Track: {} - {}\n", artist_name, track_name));
    enhanced_message.push_str(&format!("⏰ Position: {}\n", timestamp));
    enhanced_message.push_str(&format!("🆔 Track ID: {}\n\n", track_id));
    
    match lyrics_context {
        Ok(context) => {
            enhanced_message.push_str(&format!("🎼 LYRICS CONTEXT:\n{}\n", context));
        },
        Err(e) => {
            enhanced_message.push_str(&format!("🎼 LYRICS: Loading... ({})\n\n", e));
        }
    }
    
    enhanced_message.push_str(&format!("💬 AURORA'S MESSAGE:\n{}", message));
    
    Ok(enhanced_message)
}

fn format_ms_to_time(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}

// Check if Spotify is authenticated
#[tauri::command]
pub async fn check_spotify_auth() -> Result<String, String> {
    let auth_info = {
        let auth_state = SPOTIFY_AUTH_STATE.lock()
            .map_err(|e| format!("Failed to check auth: {}", e))?;
        
        serde_json::json!({
            "is_authenticated": auth_state.is_authenticated,
            "has_token": auth_state.access_token.is_some(),
            "expires_at": auth_state.expires_at
        })
    };
    
    Ok(auth_info.to_string())
}

// Test Spotify system
#[tauri::command]
pub async fn test_spotify_system() -> Result<String, String> {
    println!("🧪 Testing Spotify system...");
    
    let auth_status = check_spotify_auth().await?;
    let auth_data: serde_json::Value = serde_json::from_str(&auth_status)
        .map_err(|e| format!("Failed to parse auth status: {}", e))?;
    
    if !auth_data["is_authenticated"].as_bool().unwrap_or(false) {
        return Ok("❌ Spotify not authenticated. Please connect your Spotify account first.".to_string());
    }
    
    match get_current_spotify_track().await {
        Ok(track_data) => {
            let track: SpotifyTrackData = serde_json::from_str(&track_data)
                .map_err(|e| format!("Failed to parse track data: {}", e))?;
            
            Ok(format!(
                "✅ Spotify system working!\n\nCurrently playing:\n{} - {}\nPosition: {}/{}\nPlaying: {}",
                track.artist_name,
                track.track_name,
                format_ms_to_time(track.current_position_ms),
                format_ms_to_time(track.duration_ms),
                track.is_playing
            ))
        },
        Err(e) => Ok(format!("⚠️ Spotify system ready but no track playing: {}", e))
    }
}

// Add this function to spotify_system.rs
#[tauri::command]
pub async fn get_spotify_access_token() -> Result<String, String> {
    let auth_state = SPOTIFY_AUTH_STATE.lock()
        .map_err(|e| format!("Failed to lock auth state: {}", e))?;
    
    if let Some(token) = &auth_state.access_token {
        Ok(serde_json::json!({
            "access_token": token
        }).to_string())
    } else {
        Err("No access token available".to_string())
    }
}

#[tauri::command]
pub async fn start_spotify_playback(track_uri: String, device_id: String) -> Result<String, String> {
    println!("🎵 Starting playback: {} on device: {}", track_uri, device_id);
    
    let token = ensure_valid_token().await?;
    let client = reqwest::Client::new();
    
    let play_request = serde_json::json!({
        "uris": [track_uri],
        "position_ms": 0
    });
    
    let response = client
        .put(&format!("https://api.spotify.com/v1/me/player/play?device_id={}", device_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&play_request)
        .send()
        .await
        .map_err(|e| format!("Failed to start playback: {}", e))?;
    
    if response.status().is_success() {
        Ok("Playback started successfully".to_string())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Spotify API error: {}", error_text))
    }
}

#[tauri::command]
pub async fn ensure_valid_spotify_token() -> Result<String, String> {
    // This will automatically refresh if needed
    let token = ensure_valid_token().await?;
    
    Ok(serde_json::json!({
        "access_token": token
    }).to_string())
}

#[tauri::command]
pub async fn fetch_lyrics_backend(artist: String, song: String) -> Result<String, String> {
    let clean_artist = artist.to_lowercase().chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let clean_song = song.to_lowercase().chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    
    let url = format!("https://www.azlyrics.com/lyrics/{}/{}.html", clean_artist, clean_song);
    
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()
        .map_err(|e| format!("Failed to create client: {}", e))?;
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch lyrics: {}", e))?;
    
    if !response.status().is_success() {
        return Err("Lyrics not found".to_string());
    }
    
    let html = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // Extract lyrics from HTML
    if let Some(start_pos) = html.find("<!-- Usage of azlyrics.com content") {
        if let Some(div_start) = html[start_pos..].find(">") {
            let lyrics_start = start_pos + div_start + 1;
            if let Some(div_end) = html[lyrics_start..].find("</div>") {
                let lyrics = html[lyrics_start..lyrics_start + div_end]
                    .replace("<br>", "\n")
                    .replace("<br/>", "\n")
                    .replace("<br />", "\n")
                    .replace("<i>", "")
                    .replace("</i>", "")
                    .replace("<b>", "")
                    .replace("</b>", "")
                    .trim()
                    .to_string();
                
                return Ok(lyrics);
            }
        }
    }
    
    Err("Could not extract lyrics from page".to_string())
}

#[tauri::command]
pub async fn fetch_lrc_lyrics(artist: String, song: String) -> Result<String, String> {
    // Try multiple LRC sources
    
    // Source 1: LRCLib.net (free LRC database)
    let search_query = format!("{} {}", artist, song);
    let encoded_query = urlencoding::encode(&search_query);
    
    let client = reqwest::Client::new();
    
    // Search for the song
    let search_url = format!("https://lrclib.net/api/search?q={}", encoded_query);
    
    let response = client.get(&search_url)
        .header("User-Agent", "LyraShell/1.0")
        .send()
        .await
        .map_err(|e| format!("LRC search failed: {}", e))?;
    
    if response.status().is_success() {
        let search_results: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse LRC search: {}", e))?;
        
        // Get the first matching result
        if let Some(first_result) = search_results.as_array()
            .and_then(|arr| arr.first()) {
            
            if let Some(synced_lyrics) = first_result["syncedLyrics"].as_str() {
                return Ok(synced_lyrics.to_string());
            }
        }
    }
    
    Err("No LRC lyrics found".to_string())
}
#[tauri::command]
pub async fn fetch_spotify_track_lyrics(track_id: String) -> Result<String, String> {
    println!("🎵 Attempting to fetch Spotify native lyrics for track: {}", track_id);
    
    let token = ensure_valid_token().await?;
    let client = reqwest::Client::new();
    
    // Try Spotify's internal lyrics endpoint (undocumented)
    // This may not work as it's not part of the public API
    let lyrics_url = format!("https://spclient.wg.spotify.com/color-lyrics/v2/track/{}", track_id);
    
    let response = client
        .get(&lyrics_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("App-Platform", "WebPlayer")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Spotify lyrics: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Spotify lyrics API error: {}", response.status()));
    }
    
    let lyrics_data: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Spotify lyrics response: {}", e))?;
    
    // Extract lyrics from Spotify's response format
    if let Some(lyrics_obj) = lyrics_data["lyrics"].as_object() {
        if let Some(lines) = lyrics_obj["lines"].as_array() {
            let mut formatted_lyrics = String::new();
            
            for line in lines {
                if let Some(words) = line["words"].as_str() {
                    formatted_lyrics.push_str(words);
                    formatted_lyrics.push('\n');
                }
            }
            
            if !formatted_lyrics.trim().is_empty() {
                println!("✅ Found Spotify native lyrics: {} characters", formatted_lyrics.len());
                return Ok(formatted_lyrics.trim().to_string());
            }
        }
    }
    
    Err("No lyrics found in Spotify response".to_string())
}

#[tauri::command]
pub async fn fetch_musixmatch_lyrics(artist: String, song: String) -> Result<String, String> {
    println!("🎵 Fetching Musixmatch synchronized lyrics...");
    
    let client = reqwest::Client::new();
    
    // Musixmatch has a free tier - you'd need to get an API key from developer.musixmatch.com
    // For now, using their public search (limited but functional)
    let search_query = format!("{} {}", artist, song);
    let encoded_query = urlencoding::encode(&search_query);
    
    // Try their public API endpoint
    let search_url = format!(
        "https://apic-desktop.musixmatch.com/ws/1.1/macro.subtitles.get?format=json&q_track={}&q_artist={}&user_language=en",
        urlencoding::encode(&song),
        urlencoding::encode(&artist)
    );
    
    let response = client
        .get(&search_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Musixmatch request failed: {}", e))?;
    
    if response.status().is_success() {
        let json_response: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Musixmatch response: {}", e))?;
        
        // Extract synchronized lyrics from Musixmatch response
        if let Some(subtitle_body) = json_response["message"]["body"]["macro_calls"]["track.subtitles.get"]["message"]["body"]["subtitle_list"].as_array() {
            if let Some(first_subtitle) = subtitle_body.first() {
                if let Some(subtitle_body_text) = first_subtitle["subtitle"]["subtitle_body"].as_str() {
                    println!("✅ Found Musixmatch synchronized lyrics");
                    return Ok(subtitle_body_text.to_string());
                }
            }
        }
    }
    
    Err("No Musixmatch synchronized lyrics found".to_string())
}

#[tauri::command]
pub async fn fetch_syncedlyrics_api(artist: String, song: String) -> Result<String, String> {
    println!("🎵 Fetching from SyncedLyrics API...");
    
    let client = reqwest::Client::new();
    
    // SyncedLyrics is a Python library, but there are web APIs that mirror it
    let search_url = format!(
        "https://lrclib.net/api/search?artist_name={}&track_name={}",
        urlencoding::encode(&artist),
        urlencoding::encode(&song)
    );
    
    let response = client
        .get(&search_url)
        .header("User-Agent", "LyraShell/1.0")
        .send()
        .await
        .map_err(|e| format!("SyncedLyrics request failed: {}", e))?;
    
    if response.status().is_success() {
        let search_results: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse SyncedLyrics response: {}", e))?;
        
        // Get the best match (first result)
        if let Some(first_result) = search_results.as_array().and_then(|arr| arr.first()) {
            if let Some(synced_lyrics) = first_result["syncedLyrics"].as_str() {
                if !synced_lyrics.is_empty() {
                    println!("✅ Found SyncedLyrics synchronized lyrics");
                    return Ok(synced_lyrics.to_string());
                }
            }
        }
    }
    
    Err("No SyncedLyrics synchronized lyrics found".to_string())
}

#[tauri::command]
pub async fn fetch_genius_timed_lyrics(artist: String, song: String) -> Result<String, String> {
    println!("🎵 Fetching Genius timed lyrics...");
    
    let client = reqwest::Client::new();
    
    // Search for the song on Genius
    let search_query = format!("{} {}", artist, song);
    let encoded_query = urlencoding::encode(&search_query);
    
    let search_url = format!(
        "https://genius.com/api/search/multi?per_page=5&q={}",
        encoded_query
    );
    
    let response = client
        .get(&search_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Genius search failed: {}", e))?;
    
    if response.status().is_success() {
        let search_data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Genius search: {}", e))?;
        
        // Look for songs with timed lyrics
        if let Some(sections) = search_data["response"]["sections"].as_array() {
            for section in sections {
                if section["type"].as_str() == Some("song") {
                    if let Some(hits) = section["hits"].as_array() {
                        for hit in hits {
                            if let Some(result) = hit["result"].as_object() {
                                if let Some(song_id) = result["id"].as_u64() {
                                    // Try to get timed lyrics for this song
                                    let lyrics_url = format!("https://genius.com/api/songs/{}/timed_lyrics", song_id);
                                    
                                    if let Ok(lyrics_response) = client.get(&lyrics_url)
                                        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                                        .send()
                                        .await {
                                        
                                        if let Ok(lyrics_data) = lyrics_response.json::<serde_json::Value>().await {
                                            if let Some(timed_lyrics) = lyrics_data["response"]["timed_lyrics"].as_object() {
                                                if let Some(lyrics_array) = timed_lyrics["lyrics"].as_array() {
                                                    // Convert Genius timed format to LRC format
                                                    let mut lrc_lyrics = String::new();
                                                    
                                                    for lyric_line in lyrics_array {
                                                        if let (Some(start_time), Some(text)) = (
                                                            lyric_line["start_time_ms"].as_u64(),
                                                            lyric_line["text"].as_str()
                                                        ) {
                                                            let minutes = start_time / 60000;
                                                            let seconds = (start_time % 60000) / 1000;
                                                            let centiseconds = (start_time % 1000) / 10;
                                                            
                                                            lrc_lyrics.push_str(&format!(
                                                                "[{:02}:{:02}.{:02}] {}\n",
                                                                minutes, seconds, centiseconds, text
                                                            ));
                                                        }
                                                    }
                                                    
                                                    if !lrc_lyrics.is_empty() {
                                                        println!("✅ Found Genius timed lyrics");
                                                        return Ok(lrc_lyrics);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Err("No Genius timed lyrics found".to_string())
}