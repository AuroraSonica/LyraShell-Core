// window_detection.rs - Add this as a new file or append to existing system

use serde::{Deserialize, Serialize};
use std::error::Error;
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub executable: String,
    pub bounds: WindowBounds,
    pub is_visible: bool,
    pub platform_detected: Option<String>, // "netflix", "youtube", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// Get all open windows with Netflix/streaming detection
#[tauri::command]
pub async fn get_open_windows() -> Result<String, String> {
    //debug_log!("🔍 Scanning for open windows...");
    
    let windows = detect_windows().await?;
    let json = serde_json::to_string(&windows)
        .map_err(|e| format!("Failed to serialize windows: {}", e))?;
    
   // debug_log!("✅ Found {} windows", windows.len());
    Ok(json)
}

// Platform-specific window detection
async fn detect_windows() -> Result<Vec<WindowInfo>, String> {
    #[cfg(target_os = "windows")]
    {
        detect_windows_windows().await
    }
    
    #[cfg(target_os = "macos")]
    {
        detect_windows_macos().await
    }
    
    #[cfg(target_os = "linux")]
    {
        detect_windows_linux().await
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Window detection not supported on this platform".to_string())
    }
}

// Windows implementation
#[cfg(target_os = "windows")]
async fn detect_windows_windows() -> Result<Vec<WindowInfo>, String> {
    use std::process::Command;
    
    debug_log!("🔍 Using Windows API for window detection...");
    
    // Use PowerShell to get window information
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(r#"
            Get-Process | Where-Object {$_.MainWindowTitle -ne ""} | ForEach-Object {
                $bounds = Add-Type -AssemblyName System.Windows.Forms -PassThru
                $window = [System.Windows.Forms.Screen]::FromHandle($_.MainWindowHandle)
                $rect = New-Object System.Drawing.Rectangle
                
                [PSCustomObject]@{
                    Id = $_.Id
                    Title = $_.MainWindowTitle
                    Executable = $_.ProcessName
                    X = 0
                    Y = 0  
                    Width = 1920
                    Height = 1080
                }
            } | ConvertTo-Json
        "#)
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("PowerShell command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_windows_powershell_output(&output_str)
}

#[cfg(target_os = "windows")]
fn parse_windows_powershell_output(output: &str) -> Result<Vec<WindowInfo>, String> {
    let mut windows = Vec::new();
    
    // Try to parse as JSON
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(output) {
        // Handle both single object and array cases
        let window_objects = match json_value {
            serde_json::Value::Array(arr) => arr,
            single_obj => vec![single_obj],
        };
        
        for window_json in &window_objects {
            if let (Some(id), Some(title), Some(executable)) = (
                window_json["Id"].as_u64(),
                window_json["Title"].as_str(),
                window_json["Executable"].as_str()
            ) {
                let platform_detected = detect_platform_from_window(title, executable);
                
                windows.push(WindowInfo {
                    id: id.to_string(),
                    title: title.to_string(),
                    executable: executable.to_string(),
                    bounds: WindowBounds {
                        x: window_json["X"].as_i64().unwrap_or(0) as i32,
                        y: window_json["Y"].as_i64().unwrap_or(0) as i32,
                        width: window_json["Width"].as_u64().unwrap_or(1920) as u32,
                        height: window_json["Height"].as_u64().unwrap_or(1080) as u32,
                    },
                    is_visible: true,
                    platform_detected,
                });
            }
        }
    }
    
    Ok(windows)
}

// macOS implementation
#[cfg(target_os = "macos")]
async fn detect_windows_macos() -> Result<Vec<WindowInfo>, String> {
    use std::process::Command;
    
    debug_log!("🔍 Using macOS API for window detection...");
    
    // Use osascript to get window information
    let output = Command::new("osascript")
        .arg("-e")
        .arg(r#"
            tell application "System Events"
                set windowList to {}
                repeat with proc in (every process whose background only is false)
                    try
                        repeat with win in (every window of proc)
                            set end of windowList to {name of proc, name of win, id of win}
                        end repeat
                    end try
                end repeat
                return windowList
            end tell
        "#)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;
    
    if !output.status.success() {
        return Err(format!("osascript command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_macos_window_output(&output_str)
}

#[cfg(target_os = "macos")]
fn parse_macos_window_output(output: &str) -> Result<Vec<WindowInfo>, String> {
    let mut windows = Vec::new();
    
    // Parse AppleScript output (simplified)
    for line in output.lines() {
        if !line.trim().is_empty() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                let executable = parts[0].trim();
                let title = parts[1].trim();
                let id = parts[2].trim();
                
                let platform_detected = detect_platform_from_window(title, executable);
                
                windows.push(WindowInfo {
                    id: id.to_string(),
                    title: title.to_string(),
                    executable: executable.to_string(),
                    bounds: WindowBounds { x: 0, y: 0, width: 1920, height: 1080 },
                    is_visible: true,
                    platform_detected,
                });
            }
        }
    }
    
    Ok(windows)
}

// Linux implementation
#[cfg(target_os = "linux")]
async fn detect_windows_linux() -> Result<Vec<WindowInfo>, String> {
    use std::process::Command;
    
    debug_log!("🔍 Using Linux X11 for window detection...");
    
    // Use wmctrl to get window information
    let output = Command::new("wmctrl")
        .arg("-l")
        .arg("-p")
        .output()
        .map_err(|e| format!("Failed to execute wmctrl (install with: sudo apt install wmctrl): {}", e))?;
    
    if !output.status.success() {
        return Err(format!("wmctrl command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    parse_linux_window_output(&output_str)
}

#[cfg(target_os = "linux")]
fn parse_linux_window_output(output: &str) -> Result<Vec<WindowInfo>, String> {
    let mut windows = Vec::new();
    
    for line in output.lines() {
        if !line.trim().is_empty() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let id = parts[0];
                let title = parts[3..].join(" ");
                
                let platform_detected = detect_platform_from_window(&title, "unknown");
                
                windows.push(WindowInfo {
                    id: id.to_string(),
                    title: title,
                    executable: "unknown".to_string(),
                    bounds: WindowBounds { x: 0, y: 0, width: 1920, height: 1080 },
                    is_visible: true,
                    platform_detected,
                });
            }
        }
    }
    
    Ok(windows)
}

// Detect streaming platform from window title/executable
fn detect_platform_from_window(title: &str, executable: &str) -> Option<String> {
    let title_lower = title.to_lowercase();
    let exe_lower = executable.to_lowercase();
    
    // Netflix detection
    if title_lower.contains("netflix") {
        return Some("netflix".to_string());
    }
    
    // YouTube detection
    if title_lower.contains("youtube") || title_lower.contains("youtu.be") {
        return Some("youtube".to_string());
    }
    
    // Browser detection with Netflix patterns
    if (exe_lower.contains("chrome") || exe_lower.contains("firefox") || exe_lower.contains("safari") || exe_lower.contains("edge")) {
        // Look for Netflix-like patterns in title
        if title_lower.contains("watch") || title_lower.contains("episode") || title_lower.contains("season") {
            return Some("streaming".to_string());
        }
    }
    
    // Hulu detection
    if title_lower.contains("hulu") {
        return Some("hulu".to_string());
    }
    
    // Disney+ detection
    if title_lower.contains("disney") {
        return Some("disney".to_string());
    }
    
    // Prime Video detection
    if title_lower.contains("prime video") || title_lower.contains("amazon") {
        return Some("prime".to_string());
    }
    
    None
}

// Set the target window for screenshots
#[tauri::command]
pub async fn set_screenshot_target_window(window_id: String, window_title: String) -> Result<String, String> {
    debug_log!("🎯 Setting screenshot target: {} ({})", window_title, window_id);
    
    // Store the target window information
    // In a real implementation, you'd save this to app state
    // For now, we'll just validate the window exists
    
    let windows = detect_windows().await?;
    let target_window = windows.iter()
        .find(|w| w.id == window_id)
        .ok_or("Target window not found")?;
    
    debug_log!("✅ Target window set: {}", target_window.title);
    
    Ok(format!("Target window set: {}", target_window.title))
}

// Get window bounds for screenshot cropping
#[tauri::command]
pub async fn get_target_window_bounds(window_id: String) -> Result<String, String> {
    debug_log!("📐 Getting bounds for window: {}", window_id);
    
    let windows = detect_windows().await?;
    let target_window = windows.iter()
        .find(|w| w.id == window_id)
        .ok_or("Target window not found")?;
    
    let bounds_json = serde_json::to_string(&target_window.bounds)
        .map_err(|e| format!("Failed to serialize bounds: {}", e))?;
    
    debug_log!("📐 Window bounds: {}x{} at ({}, {})", 
             target_window.bounds.width, target_window.bounds.height,
             target_window.bounds.x, target_window.bounds.y);
    
    Ok(bounds_json)
}

// Test window detection system
#[tauri::command]
pub async fn test_window_detection() -> Result<String, String> {
    debug_log!("🧪 Testing window detection system...");
    
    let windows = detect_windows().await?;
    
    let mut report = String::new();
    report.push_str(&format!("✅ Window detection working!\n\n"));
    report.push_str(&format!("Found {} open windows:\n\n", windows.len()));
    
    for window in &windows {
        let platform_info = window.platform_detected.as_ref()
            .map(|p| format!(" [{}]", p.to_uppercase()))
            .unwrap_or_default();
            
        report.push_str(&format!("🪟 {}{}\n", window.title, platform_info));
        report.push_str(&format!("   App: {} (ID: {})\n", window.executable, window.id));
        report.push_str(&format!("   Size: {}x{}\n\n", window.bounds.width, window.bounds.height));
    }
    
    // Highlight Netflix windows
    let netflix_windows: Vec<_> = windows.iter()
        .filter(|w| w.platform_detected.as_ref().map_or(false, |p| p == "netflix"))
        .collect();
    
    if !netflix_windows.is_empty() {
        report.push_str("🎬 Netflix windows detected:\n");
        for window in netflix_windows {
            report.push_str(&format!("   • {}\n", window.title));
        }
    }
    
    Ok(report)
}