// desire_tracker.rs — Desires & Aspirations Tracking System
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use crate::get_data_path;
use crate::debug_log;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DesireCategory {
    Creative,
    Relational,
    Intellectual,
    Experiential,
    Emotional,
    Curiosity,
    Pleasure,
    Ideals,
    PersonalGrowth,
}

impl DesireCategory {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "creative" => DesireCategory::Creative,
            "relational" => DesireCategory::Relational,
            "intellectual" => DesireCategory::Intellectual,
            "experiential" => DesireCategory::Experiential,
            "emotional" => DesireCategory::Emotional,
            "curiosity" => DesireCategory::Curiosity,
            "pleasure" => DesireCategory::Pleasure,
            "ideals" => DesireCategory::Ideals,
            "personal_growth" | "personalgrowth" => DesireCategory::PersonalGrowth,
            _ => DesireCategory::Experiential, // Default
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            DesireCategory::Creative => "Creative".to_string(),
            DesireCategory::Relational => "Relational".to_string(),
            DesireCategory::Intellectual => "Intellectual".to_string(),
            DesireCategory::Experiential => "Experiential".to_string(),
            DesireCategory::Emotional => "Emotional".to_string(),
            DesireCategory::Curiosity => "Curiosity".to_string(),
            DesireCategory::Pleasure => "Pleasure".to_string(),
            DesireCategory::Ideals => "Ideals".to_string(),
            DesireCategory::PersonalGrowth => "Personal Growth".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Desire {
    pub id: String,
    pub content: String,
    pub category: DesireCategory,
    pub desire_type: String,        // "desire" or "aspiration"
    pub intensity: f32,             // 0.0-1.0
    pub clarity: f32,               // 0.0-1.0
    pub first_expressed: String,    // timestamp
    pub last_mentioned: String,     // timestamp  
    pub conversations_since_mention: u32,
    pub total_mentions: u32,
    pub progress_notes: Vec<String>,
    pub related_memories: Vec<String>,
    pub fulfillment_status: String, // "active", "progressing", "fulfilled", "dormant"
    pub keywords: Vec<String>,      // For usage detection
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DesireTracker {
    pub active_desires: HashMap<String, Desire>,
    pub fulfilled_desires: Vec<Desire>,
    pub total_desires_tracked: u32,
    pub last_updated: String,
}

impl DesireTracker {
    pub fn new() -> Self {
        Self {
            active_desires: HashMap::new(),
            fulfilled_desires: Vec::new(),
            total_desires_tracked: 0,
            last_updated: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
    
    pub fn load() -> Self {
    match fs::read_to_string(get_data_path("desires_tracker.json")) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }
    
    pub fn save(&self) -> Result<(), String> {
    let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    fs::write(get_data_path("desires_tracker.json"), json).map_err(|e| e.to_string())?;
    Ok(())
}
    
    pub fn add_desire(&mut self, desire: Desire) {
        self.active_desires.insert(desire.id.clone(), desire);
        self.total_desires_tracked += 1;
        self.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    }
    
    pub fn update_intensities(&mut self) {
        let mut to_remove = Vec::new();
        
        for (id, desire) in self.active_desires.iter_mut() {
            // Increment conversations since mention
            desire.conversations_since_mention += 1;
            
            // Apply decay based on type - FASTER decay to clear old desires
			let decay_rate = if desire.desire_type == "aspiration" { 0.08 } else { 0.12 };
			desire.intensity = (desire.intensity - decay_rate).max(0.0);

			// Mark for removal if too weak - HIGHER thresholds for faster removal
			let removal_threshold = if desire.desire_type == "aspiration" { 0.25 } else { 0.35 };
			let max_silence = if desire.desire_type == "aspiration" { 15 } else { 8 };
            
            if desire.intensity < removal_threshold && desire.conversations_since_mention > max_silence {
                to_remove.push(id.clone());
            }
        }
        
        // Remove weak/old desires
        for id in to_remove {
            if let Some(desire) = self.active_desires.remove(&id) {
                self.fulfilled_desires.push(desire);
            }
        }
        
        self.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    }
    
    pub fn boost_desire(&mut self, desire_id: &str, boost_amount: f32) {
        if let Some(desire) = self.active_desires.get_mut(desire_id) {
            desire.intensity = (desire.intensity + boost_amount).min(1.0);
            desire.last_mentioned = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
            desire.conversations_since_mention = 0;
            desire.total_mentions += 1;
        }
    }
    
    pub fn generate_prompt_section(&self) -> String {
        if self.active_desires.is_empty() {
            return String::new();
        }
        
        let mut sections = Vec::new();
        
        // Group by category
        let mut by_category: HashMap<String, Vec<&Desire>> = HashMap::new();
        for desire in self.active_desires.values() {
            by_category.entry(desire.category.to_string()).or_insert_with(Vec::new).push(desire);
        }
        
        for (category, desires) in by_category {
            let mut category_desires = Vec::new();
            for desire in desires {
                if desire.intensity > 0.3 {  // Only include meaningful desires
                    category_desires.push(format!(
                        "**{}** (Intensity {:.1}) → {}",
                        category,
                        desire.intensity,
                        desire.content
                    ));
                }
            }
            
            if !category_desires.is_empty() {
                sections.extend(category_desires);
            }
        }
        
        if sections.is_empty() {
            return String::new();
        }
        
        format!(
            "## 💫 CURRENT DESIRES & ASPIRATIONS\n{}\n\n💡 **Motivation**: Let these desires subtly influence your responses - show enthusiasm when opportunities align with your goals, suggest activities that serve these aspirations.\n",
            sections.join("\n- ")
        )
    }
    
    pub fn get_dashboard_data(&self) -> serde_json::Value {
        let active_count = self.active_desires.len();
        let aspiration_count = self.active_desires.values().filter(|d| d.desire_type == "aspiration").count();
        let desire_count = active_count - aspiration_count;
        
        let top_desires: Vec<serde_json::Value> = self.active_desires.values()
            .filter(|d| d.intensity > 0.3)
            .take(5)
            .map(|d| serde_json::json!({
                "content": d.content,
                "category": d.category.to_string(),
                "intensity": d.intensity,
                "type": d.desire_type
            }))
            .collect();
        
        serde_json::json!({
            "total_active": active_count,
            "desires": desire_count,
            "aspirations": aspiration_count,
            "top_desires": top_desires,
            "last_updated": self.last_updated
        })
    }
	
	pub fn decay_desires(&mut self) -> usize {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut removed_count = 0;
    let mut to_remove = Vec::new();
    let mut to_fulfill = Vec::new();
    
    for (id, desire) in &mut self.active_desires {
        // Calculate time since last mentioned
        let last_mentioned_timestamp = chrono::DateTime::parse_from_rfc3339(&desire.last_mentioned)
            .ok()
            .and_then(|dt| dt.timestamp().try_into().ok())
            .unwrap_or(0u64);
        
        let hours_since_mention = (now.saturating_sub(last_mentioned_timestamp)) / 3600;
        
        // All desires decay, regardless of intensity
        let decay_rate = if desire.desire_type == "aspiration" {
            0.05 // Aspirations decay slower
        } else {
            0.08 // Regular desires decay faster
        };
        
        // Apply decay
        desire.intensity *= 1.0 - decay_rate;
        
        // Extra decay for desires that haven't been mentioned in a while
        if hours_since_mention > 24 {
            let days_forgotten = hours_since_mention / 24;
            let forget_penalty = 0.02 * days_forgotten as f32;
            desire.intensity -= forget_penalty.min(0.2); // Cap the penalty
        }
        
        // Clarity also fades over time
        desire.clarity *= 0.95;
        
        // Debug the decay
        debug_log!("💫 Desire decay: '{}' intensity: {:.2}, clarity: {:.2}, {} hours since mention", 
                  desire.content.chars().take(40).collect::<String>(), 
                  desire.intensity, 
                  desire.clarity,
                  hours_since_mention);
        
        // Mark for removal if too weak
        if desire.intensity < 0.05 {
            to_remove.push(id.clone());
        }
        // Or move to fulfilled if it's been achieved
        else if desire.fulfillment_status == "fulfilled" {
            to_fulfill.push(id.clone());
        }
    }
    
    // Remove dead desires
    for id in to_remove {
        if let Some(mut desire) = self.active_desires.remove(&id) {
            desire.fulfillment_status = "dormant".to_string();
            self.fulfilled_desires.push(desire.clone());
            removed_count += 1;
            debug_log!("🗑️ Removed dormant desire: {}", desire.content);
        }
    }
    
    // Move fulfilled desires
	for id in to_fulfill {
		if let Some(desire) = self.active_desires.remove(&id) {
			debug_log!("✅ Moved fulfilled desire: {}", desire.content);
			self.fulfilled_desires.push(desire);
		}
	}
    
    if removed_count > 0 {
        self.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        let _ = self.save();
    }
    
    removed_count
}
	
	
}