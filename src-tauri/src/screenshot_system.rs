// screenshot_system.rs - Fixed API compatibility

use serde::{Deserialize, Serialize};
use std::error::Error;
use base64::{Engine as _, engine::general_purpose};
use std::io::Cursor;
use std::fs;
use std::path::Path;
use image::{ImageBuffer, RgbaImage, SubImage};
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotContext {
    pub screenshot_data: String, // Base64 encoded image
    pub timestamp: f64,
    pub video_id: String,
    pub video_title: String,
    pub capture_method: String,
    pub image_format: String,
    pub resolution: String,
}

// APPROACH 2: Use screenshots crate directly (FIXED)
#[tauri::command]
pub async fn capture_youtube_screenshot_v2(
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Attempting screenshot capture (method 2)...");
    
    use screenshots::Screen;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(image) => {
                        // FIXED: Use correct ImageBuffer API from image crate
                        let width = image.width();
                        let height = image.height();
                        
                        // Convert ImageBuffer to PNG bytes
                        use image::ImageFormat;
                        
                        let mut png_data = Vec::new();
                        {
                            let mut cursor = Cursor::new(&mut png_data);
                            image.write_to(&mut cursor, ImageFormat::Png)
                                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
                        }
                        
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        
                        let context = ScreenshotContext {
                            screenshot_data: base64_data,
                            timestamp: current_time,
                            video_id,
                            video_title,
                            capture_method: "screenshots-crate".to_string(),
                            image_format: "png".to_string(),
                            resolution: format!("{}x{}", width, height),
                        };
                        
                        debug_log!("✅ Screenshot captured: {}x{}", width, height);
                        Ok(serde_json::to_string(&context).map_err(|e| e.to_string())?)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}

// APPROACH 4: Web-based canvas capture (called from frontend)
#[tauri::command]
pub async fn process_canvas_screenshot(
    canvas_data: String, // Base64 canvas data from frontend
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Processing canvas screenshot from frontend...");
    
    // Validate base64 data
    if canvas_data.starts_with("data:image/") {
        // Remove data:image/png;base64, prefix
        let base64_data = canvas_data
            .split(',')
            .nth(1)
            .ok_or("Invalid canvas data format")?;
        
        // Verify it's valid base64
        general_purpose::STANDARD.decode(base64_data)
            .map_err(|_| "Invalid base64 data")?;
        
        let context = ScreenshotContext {
            screenshot_data: base64_data.to_string(),
            timestamp: current_time,
            video_id,
            video_title,
            capture_method: "canvas-capture".to_string(),
            image_format: "png".to_string(),
            resolution: "from-canvas".to_string(),
        };
        
        debug_log!("✅ Canvas screenshot processed");
        Ok(serde_json::to_string(&context).map_err(|e| e.to_string())?)
    } else {
        Err("Invalid canvas data format".to_string())
    }
}

// UNIFIED CAPTURE COMMAND - tries multiple methods
#[tauri::command]
pub async fn capture_youtube_context_with_screenshot(
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Unified screenshot capture starting...");
    
    // Try methods in order of preference
    let methods = [
        "screenshots-crate",
        "placeholder",
    ];
    
    for method in methods {
        let result = match method {
            "screenshots-crate" => {
                capture_youtube_screenshot_v2(
                    video_id.clone(),
                    current_time,
                    video_title.clone()
                ).await
            },
            "placeholder" => {
                create_placeholder_screenshot(
                    video_id.clone(),
                    current_time,
                    video_title.clone()
                ).await
            },
            _ => Err("Unknown method".to_string())
        };
        
        if result.is_ok() {
            debug_log!("✅ Screenshot captured using: {}", method);
            return result;
        } else {
            debug_log!("⚠️ Method {} failed: {:?}", method, result);
        }
    }
    
    Err("All screenshot methods failed".to_string())
}

// Placeholder screenshot creator
#[tauri::command]
pub async fn create_placeholder_screenshot(
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Creating placeholder screenshot...");
    
    // Create a simple placeholder image as base64
    // This is a minimal 1x1 PNG in base64 - replace with actual placeholder generation if needed
    let placeholder_png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChAGA4fELYwAAAABJRU5ErkJggg==";
    
    let context = ScreenshotContext {
        screenshot_data: placeholder_png_base64.to_string(),
        timestamp: current_time,
        video_id,
        video_title,
        capture_method: "placeholder".to_string(),
        image_format: "png".to_string(),
        resolution: "1x1".to_string(),
    };
    
    debug_log!("✅ Placeholder screenshot created");
    Ok(serde_json::to_string(&context).map_err(|e| e.to_string())?)
}

// Test command for debugging
#[tauri::command]
pub async fn test_screenshot_capabilities() -> Result<String, String> {
    debug_log!("🧪 Testing screenshot capabilities...");
    
    let mut results = Vec::new();
    
    // Test screenshots crate
    match screenshots::Screen::all() {
        Ok(screens) => {
            results.push(format!("✅ screenshots crate: {} screens found", screens.len()));
            if let Some(screen) = screens.first() {
                results.push(format!("   Primary screen: {}x{}", screen.display_info.width, screen.display_info.height));
                
                // Test actual capture
                match screen.capture() {
                    Ok(image) => {
                        results.push(format!("   ✅ Test capture successful: {}x{}", image.width(), image.height()));
                    },
                    Err(e) => {
                        results.push(format!("   ❌ Test capture failed: {}", e));
                    }
                }
            }
        },
        Err(e) => {
            results.push(format!("❌ screenshots crate failed: {}", e));
        }
    }
    
    // Test base64 encoding
    results.push("✅ base64 encoding: available".to_string());
    
    // Test JSON serialization
    let test_context = ScreenshotContext {
        screenshot_data: "test".to_string(),
        timestamp: 123.45,
        video_id: "test123".to_string(),
        video_title: "Test Video".to_string(),
        capture_method: "test".to_string(),
        image_format: "png".to_string(),
        resolution: "640x360".to_string(),
    };
    
    match serde_json::to_string(&test_context) {
        Ok(_) => results.push("✅ JSON serialization: working".to_string()),
        Err(e) => results.push(format!("❌ JSON serialization failed: {}", e)),
    }
    
    Ok(results.join("\n"))
}
// Save screenshot to file for debugging
#[tauri::command]
pub async fn debug_save_screenshot_to_file(
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Debug: Saving screenshot to file...");
    
    use screenshots::Screen;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(image) => {
                        let width = image.width();
                        let height = image.height();
                        
                        // Create debug directory
                        let debug_dir = "debug_screenshots";
                        if !Path::new(debug_dir).exists() {
                            fs::create_dir_all(debug_dir).map_err(|e| format!("Failed to create debug directory: {}", e))?;
                        }
                        
                        // Generate filename with timestamp
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        let filename = format!("{}/screenshot_{}_{:.1}s_{}.png", 
                                             debug_dir, video_id, current_time, timestamp);
                        
                        // Save to file
                        use image::ImageFormat;
                        image.save_with_format(&filename, ImageFormat::Png)
                            .map_err(|e| format!("Failed to save image: {}", e))?;
                        
                        let absolute_path = std::env::current_dir()
                            .map_err(|e| format!("Failed to get current directory: {}", e))?
                            .join(&filename);
                        
                        debug_log!("✅ Screenshot saved to: {:?}", absolute_path);
                        
                        Ok(format!("Screenshot saved to: {}\nSize: {}x{}", 
                                 absolute_path.display(), width, height))
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}

// Enhanced capture with both file save and base64 return
#[tauri::command]
pub async fn debug_capture_with_file_save(
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Debug: Capturing with file save...");
    
    use screenshots::Screen;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(image) => {
                        let width = image.width();
                        let height = image.height();
                        
                        // Save to file for debugging
                        let debug_dir = "debug_screenshots";
                        if !Path::new(debug_dir).exists() {
                            fs::create_dir_all(debug_dir).map_err(|e| format!("Failed to create debug directory: {}", e))?;
                        }
                        
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        let filename = format!("{}/debug_{}_{:.1}s_{}.png", 
                                             debug_dir, video_id, current_time, timestamp);
                        
                        // Save to file
                        use image::ImageFormat;
                        image.save_with_format(&filename, ImageFormat::Png)
                            .map_err(|e| format!("Failed to save debug image: {}", e))?;
                        
                        // Also create base64 for normal use
                        let mut png_data = Vec::new();
                        {
                            use std::io::Cursor;
                            let mut cursor = Cursor::new(&mut png_data);
                            image.write_to(&mut cursor, ImageFormat::Png)
                                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
                        }
                        
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        
                        let context = ScreenshotContext {
                            screenshot_data: base64_data,
                            timestamp: current_time,
                            video_id: video_id.clone(),
                            video_title: video_title.clone(),
                            capture_method: "debug-with-file".to_string(),
                            image_format: "png".to_string(),
                            resolution: format!("{}x{}", width, height),
                        };
                        
                        let absolute_path = std::env::current_dir()
                            .map_err(|e| format!("Failed to get current directory: {}", e))?
                            .join(&filename);
                        
                        debug_log!("✅ Debug screenshot saved to: {:?}", absolute_path);
                        debug_log!("📊 Screenshot data size: {} bytes (base64: {} chars)", png_data.len(), context.screenshot_data.len());
                        
                        // Return both the JSON context AND file path info
                        let json_context = serde_json::to_string(&context).map_err(|e| e.to_string())?;
                        let debug_info = format!("FILE_SAVED: {}\nSIZE: {}x{}\nDATA: {}", 
                                               absolute_path.display(), width, height, json_context);
                        
                        Ok(debug_info)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}

// Show screenshot as data URL in browser
#[tauri::command]
pub async fn debug_screenshot_as_data_url(
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Debug: Creating data URL for browser display...");
    
    use screenshots::Screen;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(image) => {
                        let width = image.width();
                        let height = image.height();
                        
                        // Convert to PNG bytes
                        let mut png_data = Vec::new();
                        {
                            use std::io::Cursor;
                            let mut cursor = Cursor::new(&mut png_data);
                            image.write_to(&mut cursor, image::ImageFormat::Png)
                                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
                        }
                        
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        let data_url = format!("data:image/png;base64,{}", base64_data);
                        
                        debug_log!("✅ Screenshot data URL created: {}x{}, {} bytes", width, height, png_data.len());
                        
                        Ok(data_url)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}
// Add this to your screenshot_system.rs

// Cropped screenshot capture
#[tauri::command]
pub async fn capture_cropped_youtube_screenshot(
    video_id: String,
    current_time: f64,
    video_title: String,
    crop_x: i32,
    crop_y: i32,
    crop_width: u32,
    crop_height: u32
) -> Result<String, String> {
    debug_log!("📸 Capturing cropped screenshot at ({}, {}) {}x{}", crop_x, crop_y, crop_width, crop_height);
    
    use screenshots::Screen;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(full_image) => {
                        let full_width = full_image.width();
                        let full_height = full_image.height();
                        
                        debug_log!("📸 Full screen: {}x{}, cropping to: {}x{} at ({}, {})", 
                               full_width, full_height, crop_width, crop_height, crop_x, crop_y);
                        
                        // Validate crop bounds
                        let safe_x = crop_x.max(0) as u32;
                        let safe_y = crop_y.max(0) as u32;
                        let safe_width = crop_width.min(full_width.saturating_sub(safe_x));
                        let safe_height = crop_height.min(full_height.saturating_sub(safe_y));
                        
                        if safe_width == 0 || safe_height == 0 {
                            return Err("Invalid crop dimensions".to_string());
                        }
                        
                        // Crop the image using image crate
                        use image::{ImageBuffer, RgbaImage};
                        
                        // Create new image buffer for cropped area
                        let mut cropped_data = Vec::new();
                        
                        // Extract pixel data row by row
                        for y in 0..safe_height {
                            for x in 0..safe_width {
                                let src_x = safe_x + x;
                                let src_y = safe_y + y;
                                
                                if src_x < full_width && src_y < full_height {
                                    let pixel = full_image.get_pixel(src_x, src_y);
                                    cropped_data.extend_from_slice(&pixel.0);
                                }
                            }
                        }
                        
                        // Create cropped image
                        let cropped_image: RgbaImage = ImageBuffer::from_raw(safe_width, safe_height, cropped_data)
                            .ok_or("Failed to create cropped image buffer")?;
                        
                        // Convert to PNG
                        let mut png_data = Vec::new();
                        {
                            use std::io::Cursor;
                            let mut cursor = Cursor::new(&mut png_data);
                            cropped_image.write_to(&mut cursor, image::ImageFormat::Png)
                                .map_err(|e| format!("Failed to encode cropped PNG: {}", e))?;
                        }
                        
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        
                        let context = ScreenshotContext {
                            screenshot_data: base64_data,
                            timestamp: current_time,
                            video_id,
                            video_title,
                            capture_method: "cropped-system-screenshot".to_string(),
                            image_format: "png".to_string(),
                            resolution: format!("{}x{}", safe_width, safe_height),
                        };
                        
                        debug_log!("✅ Cropped screenshot: {}x{} (from {}x{})", safe_width, safe_height, full_width, full_height);
                        Ok(serde_json::to_string(&context).map_err(|e| e.to_string())?)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}

// Replace the debug_save_cropped_screenshot function with this version

#[tauri::command]
pub async fn debug_save_cropped_screenshot(
    video_id: String,
    current_time: f64,
    video_title: String,
    crop_x: i32,
    crop_y: i32,
    crop_width: u32,
    crop_height: u32
) -> Result<String, String> {
    debug_log!("📸 Debug: Saving cropped screenshot to file...");
    
    // First capture the cropped screenshot
    let context_json = capture_cropped_youtube_screenshot(
        video_id.clone(), current_time, video_title.clone(),
        crop_x, crop_y, crop_width, crop_height
    ).await?;
    
    let context: ScreenshotContext = serde_json::from_str(&context_json)
        .map_err(|e| format!("Failed to parse context: {}", e))?;
    
    // Decode base64 and save to file
    let png_data = general_purpose::STANDARD.decode(&context.screenshot_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Use a more predictable path - Desktop folder
    let desktop_path = match std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
        Ok(home) => {
            if cfg!(target_os = "windows") {
                format!("{}/Desktop/lyra_screenshots", home)
            } else {
                format!("{}/Desktop/lyra_screenshots", home)
            }
        },
        Err(_) => {
            // Fallback to current directory
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            format!("{}/debug_screenshots", current_dir.display())
        }
    };
    
    // Create directory
    if !Path::new(&desktop_path).exists() {
        fs::create_dir_all(&desktop_path).map_err(|e| format!("Failed to create directory {}: {}", desktop_path, e))?;
    }
    
    // Generate filename
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let filename = format!("{}/cropped_{}_{:.1}s_{}x{}_{}.png", 
                         desktop_path, video_id, current_time, crop_width, crop_height, timestamp);
    
    // Save file
    fs::write(&filename, &png_data)
        .map_err(|e| format!("Failed to save file: {}", e))?;
    
    debug_log!("✅ Cropped screenshot saved to: {}", filename);
    
    // Also print current working directory for debugging
    let cwd = std::env::current_dir().unwrap_or_default();
    debug_log!("📁 Current working directory: {:?}", cwd);
    
    Ok(format!("Cropped screenshot saved to: {}\nSize: {}x{}\nCurrent working dir: {}", 
             filename, crop_width, crop_height, cwd.display()))
}

// Capture cropped screenshot using specific coordinates
#[tauri::command]
pub async fn capture_cropped_screenshot(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Capturing cropped screenshot at ({}, {}) {}x{}", x, y, width, height);
    
    use screenshots::Screen;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(full_image) => {
                        let full_width = full_image.width();
                        let full_height = full_image.height();
                        
                        debug_log!("📐 Full screen: {}x{}, cropping to {}x{} at ({}, {})", 
                               full_width, full_height, width, height, x, y);
                        
                        // Validate crop bounds
                        if x + width > full_width || y + height > full_height {
                            return Err(format!("Crop bounds exceed screen size: {}x{} vs {}x{}", 
                                             x + width, y + height, full_width, full_height));
                        }
                        
                        // Crop the image to the specified bounds
                        let cropped = crop_image(&full_image, x, y, width, height)?;
                        
                        // Balanced size for context - readable but not expensive (max 320px width)
// Better balance - 480px width for character recognition (was 320px)
let (target_width, target_height) = if width > 480 {
    let scale = 480.0 / width as f32;
    (480, (height as f32 * scale) as u32)
} else if width < 300 {
    // Ensure minimum readable size
    let scale = 300.0 / width as f32;
    (300, (height as f32 * scale) as u32)
} else {
    (width, height)
};

let resized_image = image::imageops::resize(&cropped, target_width, target_height, image::imageops::FilterType::Lanczos3);

// Convert to JPEG with better quality for character recognition
let mut jpeg_data = Vec::new();
{
    use std::io::Cursor;
    let mut cursor = Cursor::new(&mut jpeg_data);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 80); // Higher quality (was 65)
    encoder.encode_image(&resized_image).map_err(|e| format!("Failed to encode JPEG: {}", e))?;
}

let base64_data = general_purpose::STANDARD.encode(&jpeg_data);

let context = ScreenshotContext {
    screenshot_data: base64_data,
    timestamp: current_time,
    video_id,
    video_title,
    capture_method: "cropped-system-resized".to_string(),
    image_format: "jpeg".to_string(),
    resolution: format!("{}x{}", target_width, target_height),
};
                        
                        debug_log!("✅ Cropped screenshot captured: {}x{}", width, height);
                        Ok(serde_json::to_string(&context).map_err(|e| e.to_string())?)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}

// Helper function to crop image
fn crop_image(
    image: &RgbaImage, 
    x: u32, 
    y: u32, 
    width: u32, 
    height: u32
) -> Result<RgbaImage, String> {
    // Create a new image buffer for the cropped area
    let mut cropped = ImageBuffer::new(width, height);
    
    // Copy pixels from source to cropped image
    for crop_y in 0..height {
        for crop_x in 0..width {
            let source_x = x + crop_x;
            let source_y = y + crop_y;
            
            // Bounds check
            if source_x < image.width() && source_y < image.height() {
                let pixel = image.get_pixel(source_x, source_y);
                cropped.put_pixel(crop_x, crop_y, *pixel);
            }
        }
    }
    
    Ok(cropped)
}

// Debug version that saves cropped screenshot to file
#[tauri::command]
pub async fn debug_capture_cropped_with_file(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Debug: Capturing cropped screenshot with file save...");
    
    use screenshots::Screen;
    use std::fs;
    use std::path::Path;
    
    match Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                match screen.capture() {
                    Ok(full_image) => {
                        // Crop the image
                        let cropped = crop_image(&full_image, x, y, width, height)?;
                        
                        // Create debug directory
                        let debug_dir = "debug_screenshots";
                        if !Path::new(debug_dir).exists() {
                            fs::create_dir_all(debug_dir).map_err(|e| format!("Failed to create debug directory: {}", e))?;
                        }
                        
                        // Generate filename
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        
                        let filename = format!("{}/cropped_{}_{:.1}s_{}x{}_{}.png", 
                                             debug_dir, video_id, current_time, width, height, timestamp);
                        
                        // Save to file
                        cropped.save_with_format(&filename, image::ImageFormat::Png)
                            .map_err(|e| format!("Failed to save cropped image: {}", e))?;
                        
                        // Resize image to reasonable size for Claude (max 512px width)
let (target_width, target_height) = if width > 512 {
    let scale = 512.0 / width as f32;
    (512, (height as f32 * scale) as u32)
} else {
    (width, height)
};

let resized_image = image::imageops::resize(&cropped, target_width, target_height, image::imageops::FilterType::Lanczos3);

// Convert to JPEG with compression instead of PNG
let mut jpeg_data = Vec::new();
{
    use std::io::Cursor;
    let mut cursor = Cursor::new(&mut jpeg_data);
    resized_image.write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
}

let base64_data = general_purpose::STANDARD.encode(&jpeg_data);

let context = ScreenshotContext {
    screenshot_data: base64_data,
    timestamp: current_time,
    video_id,
    video_title,
    capture_method: "cropped-system-resized".to_string(),
    image_format: "jpeg".to_string(),
    resolution: format!("{}x{}", target_width, target_height),
};
                        
                        let result = format!("Cropped screenshot saved to: {}\nSize: {}x{}\nData: {}", 
                                           "[path]", width, height, 
                                           serde_json::to_string(&context).map_err(|e| e.to_string())?);
                        
                        Ok(result)
                    },
                    Err(e) => Err(format!("Screen capture failed: {}", e))
                }
            } else {
                Err("No screens found".to_string())
            }
        },
        Err(e) => Err(format!("Failed to get screens: {}", e))
    }
}

// Enhanced capture that uses frontend-detected bounds
#[tauri::command]
pub async fn capture_youtube_player_area(
    bounds: String, // JSON string with {x, y, width, height}
    video_id: String,
    current_time: f64,
    video_title: String
) -> Result<String, String> {
    debug_log!("📸 Capturing YouTube player area with bounds: {}", bounds);
    
    // Parse bounds JSON
    let bounds_data: serde_json::Value = serde_json::from_str(&bounds)
        .map_err(|e| format!("Failed to parse bounds JSON: {}", e))?;
    
    let x = bounds_data["x"].as_u64().ok_or("Missing x coordinate")? as u32;
    let y = bounds_data["y"].as_u64().ok_or("Missing y coordinate")? as u32;
    let width = bounds_data["width"].as_u64().ok_or("Missing width")? as u32;
    let height = bounds_data["height"].as_u64().ok_or("Missing height")? as u32;
    
    // Use the cropped capture function
    capture_cropped_screenshot(x, y, width, height, video_id, current_time, video_title).await
}