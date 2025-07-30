use serde_json::Value;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::London;
use crate::{get_data_path, debug_log};

#[derive(Debug, Clone)]
pub struct DreamEntry {
    pub content: String,
    pub emotional_tone: String,
    pub timestamp_formatted: String,
    pub raw_timestamp: String,
    pub significance: f64,
    pub dream_id: String,
}

pub struct DreamLoader;

impl DreamLoader {
    /// THE ONLY FUNCTION that loads dreams - always includes formatted timestamps
    pub fn load_dreams_with_timestamps(max_dreams: Option<usize>) -> Result<Vec<DreamEntry>, String> {
        let path = get_data_path("dream_journal.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read dream journal: {}", e))?;
        
        let dream_data: Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse dream journal: {}", e))?;
        
        let mut dream_entries = Vec::new();
        
        if let Some(dreams) = dream_data["dreams"].as_array() {
            for dream in dreams {
                let content = dream["dream_content"].as_str().unwrap_or("").to_string();
                let emotional_tone = dream["emotional_tone"].as_str().unwrap_or("unknown").to_string();
                let significance = dream["significance_score"].as_f64().unwrap_or(0.0);
                let dream_id = dream["dream_id"].as_str().unwrap_or("unknown").to_string();
                let raw_timestamp = dream["timestamp"].as_str().unwrap_or("").to_string();
                
                // Format timestamp consistently - try multiple parsing methods
				let timestamp_formatted = {
					// First try: parse as UTC and convert
					if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&raw_timestamp.replace(" UTC", ""), "%Y-%m-%d %H:%M:%S") {
						let utc_dt = chrono::Utc.from_utc_datetime(&dt);
						let london_dt = utc_dt.with_timezone(&London);
						london_dt.format("%d/%m/%Y, %H:%M:%S %Z").to_string()
					} 
					// Second try: direct parse with timezone
					else if let Ok(dt) = DateTime::parse_from_str(&raw_timestamp, "%Y-%m-%d %H:%M:%S UTC") {
						let dt = dt.with_timezone(&London);
						dt.format("%d/%m/%Y, %H:%M:%S %Z").to_string()
					}
					// Third try: RFC3339 format
					else if let Ok(dt) = DateTime::parse_from_rfc3339(&raw_timestamp) {
						let dt = dt.with_timezone(&London);
						dt.format("%d/%m/%Y, %H:%M:%S %Z").to_string()
					}
					else {
						debug_log!("🌙 All timestamp parsing failed for: {}", raw_timestamp);
						format!("({})", raw_timestamp) // Show the raw timestamp as fallback
					}
				};
                
                dream_entries.push(DreamEntry {
                    content,
                    emotional_tone,
                    timestamp_formatted,
                    raw_timestamp,
                    significance,
                    dream_id,
                });
            }
        }
        
        // Sort by newest first (by raw timestamp)
        dream_entries.sort_by(|a, b| b.raw_timestamp.cmp(&a.raw_timestamp));
        
        // Apply limit if specified
        if let Some(max) = max_dreams {
            dream_entries.truncate(max);
        }
        
        debug_log!("🌙 CENTRALIZED LOADER: Loaded {} dreams with timestamps", dream_entries.len());
        Ok(dream_entries)
    }
    
    /// Format dreams for prompt display
    pub fn format_dreams_for_prompt(dreams: &[DreamEntry]) -> String {
        dreams.iter()
            .map(|dream| format!("{}: {}", dream.timestamp_formatted, dream.content))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }
}