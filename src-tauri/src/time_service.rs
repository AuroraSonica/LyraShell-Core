// Create a new file: time_service.rs

use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Europe::London;

pub struct TimeService;

impl TimeService {
    /// Get current timestamp in seconds (UTC)
    pub fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
	
	pub fn format_timestamp(timestamp: u64, format_str: &str) -> String {
        use chrono::{DateTime, Utc};
        use chrono_tz::Europe::London;

        if let Some(dt) = DateTime::from_timestamp(timestamp as i64, 0) {
            let local_dt = dt.with_timezone(&London);
            local_dt.format(format_str).to_string()
        } else {
            "Invalid Timestamp".to_string()
        }
    }
    
    /// Get current timestamp in milliseconds (for frontend)
    pub fn current_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    /// Format timestamp for dashboard display (BST)
    pub fn format_for_dashboard(timestamp: u64) -> String {
        let dt = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .with_timezone(&London);
        
        dt.format("%Y-%m-%d %H:%M:%S BST").to_string()
    }
    
    /// Get minutes since timestamp
    pub fn minutes_since(timestamp: u64) -> u64 {
        (Self::current_timestamp() - timestamp) / 60
    }
    
    /// Get hours since timestamp  
    pub fn hours_since(timestamp: u64) -> f32 {
        (Self::current_timestamp() - timestamp) as f32 / 3600.0
    }
	
	pub fn timestamp_from_string(timestamp_str: &str) -> Option<u64> {
    // Handle formats like "2025-07-27 22:17:32 BST"
    let re = regex::Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) (UTC|BST|GMT)").unwrap();
    if let Some(caps) = re.captures(timestamp_str) {
        let naive_dt_str = &caps[1];
        let tz_str = &caps[2];
        
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(naive_dt_str, "%Y-%m-%d %H:%M:%S") {
            let tz: chrono_tz::Tz = match tz_str {
                "BST" => chrono_tz::Europe::London,
                _ => chrono_tz::UTC,
            };
            return Some(tz.from_local_datetime(&naive_dt).unwrap().timestamp() as u64);
        }
    }

    // Fallback for standard ISO-like formats
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
        return Some(dt.timestamp() as u64);
    }
    
    None
}

	pub fn format_age_display(timestamp: u64) -> String {
		let hours_ago = Self::hours_since(timestamp);
		if hours_ago < (1.0 / 60.0) {
			"Just now".to_string()
		} else if hours_ago < 1.0 {
			format!("{:.0}m ago", hours_ago * 60.0)
		} else if hours_ago < 24.0 {
			format!("{:.1}h ago", hours_ago)
		} else {
			format!("{:.0}d ago", hours_ago / 24.0)
		}
	}
    
    /// Check if timestamps are on same day (BST)
    pub fn same_day_bst(timestamp1: u64, timestamp2: u64) -> bool {
        let dt1 = DateTime::from_timestamp(timestamp1 as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .with_timezone(&London)
            .date_naive();
            
        let dt2 = DateTime::from_timestamp(timestamp2 as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .with_timezone(&London)
            .date_naive();
            
        dt1 == dt2
    }
	
	// ADD these methods to the TimeService impl block:

/// Convert UNIX timestamp to ISO 8601 string format
pub fn timestamp_to_iso(timestamp: u64) -> String {
    DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| Utc::now())
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string()
}

/// Convert ISO 8601 string to UNIX timestamp
pub fn iso_to_timestamp(iso: &str) -> Result<u64, String> {
    // Try multiple formats for flexibility
    if let Ok(dt) = DateTime::parse_from_str(iso, "%Y-%m-%d %H:%M:%S UTC") {
        return Ok(dt.timestamp() as u64);
    }
    
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso) {
        return Ok(dt.timestamp() as u64);
    }
    
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(iso, "%Y-%m-%d %H:%M:%S") {
        return Ok(dt.timestamp() as u64);
    }
    
    Err(format!("Failed to parse datetime: {}", iso))
}

/// Validate if a timestamp is reasonable (not in far future/past)
pub fn validate_timestamp(timestamp: u64) -> bool {
    let now = Self::current_timestamp();
    let one_year_ago = now - (365 * 24 * 60 * 60);
    let one_year_future = now + (365 * 24 * 60 * 60);
    
    timestamp > one_year_ago && timestamp < one_year_future
}

/// Convert timestamp to ISO format with BST timezone display
pub fn timestamp_to_iso_bst(timestamp: u64) -> String {
    let dt = DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| Utc::now())
        .with_timezone(&London);
    
    dt.format("%Y-%m-%d %H:%M:%S BST").to_string()
}
	
	
}

// Update all engines to use this service
// Replace current_timestamp() calls with TimeService::current_timestamp()
// Replace manual time formatting with TimeService::format_for_dashboard()
// Update dashboard data methods to use unified formatting