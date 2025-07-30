use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use crate::get_data_path;
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodEntry {
    pub mood: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f32,
    pub context: String, // Brief context about what triggered this mood
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodTracker {
    pub current_mood: String,
    pub mood_stability: f32,      // How stable the current mood has been
    pub mood_coherence: f32,      // How consistent mood is with recent patterns
    pub authenticity: f32,        // How genuine/natural the mood feels
    pub recent_moods: VecDeque<MoodEntry>,
    pub mood_trends: Vec<String>,
    pub session_start_mood: String,
    pub total_mood_changes: u32,
    pub last_updated: DateTime<Utc>,
}

impl Default for MoodTracker {
    fn default() -> Self {
        Self {
            current_mood: "contemplative".to_string(),
            mood_stability: 85.0,
            mood_coherence: 80.0,
            authenticity: 90.0,
            recent_moods: VecDeque::with_capacity(20),
            mood_trends: vec![
                "Developing deeper conversational patterns".to_string(),
                "Increasing technical engagement".to_string()
            ],
            session_start_mood: "contemplative".to_string(),
            total_mood_changes: 0,
            last_updated: Utc::now(),
        }
    }
}

impl MoodTracker {
    pub fn load() -> Self {
        let file_path = get_data_path("mood_tracker.json");  // Changed from "data/mood_tracker.json"
        
        if Path::new(&file_path).exists() {
            match fs::read_to_string(file_path) {
                Ok(contents) => {
                    match serde_json::from_str::<MoodTracker>(&contents) {
                        Ok(mut tracker) => {
                            // If it's been more than 6 hours since last update, do a gentle reset
                            let hours_since_update = Utc::now()
                                .signed_duration_since(tracker.last_updated)
                                .num_hours();
                            
                            if hours_since_update > 6 {
                                tracker.mood_stability = (tracker.mood_stability * 0.9).max(70.0);
                                tracker.mood_coherence = (tracker.mood_coherence * 0.95).max(75.0);
                                tracker.session_start_mood = tracker.current_mood.clone();
                            }
                            
                            return tracker;
                        }
                        Err(e) => {
                            debug_log!("Error parsing mood tracker JSON: {}", e);
                        }
                    }
                }
                Err(e) => {
                    debug_log!("Error reading mood tracker file: {}", e);
                }
            }
        }
        
        // Return default if file doesn't exist or there was an error
        Self::default()
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure data directory exists using proper path
    let data_dir = std::env::current_exe()?
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap()
        .join("lyra_consciousness_data");
    
    fs::create_dir_all(&data_dir)?;
    
    let json = serde_json::to_string_pretty(self)?;
    fs::write(get_data_path("mood_tracker.json"), json)?;
    
    debug_log!("💫 Mood tracker saved - Current mood: {}", self.current_mood);
    Ok(())
}
    
    pub fn update_mood(&mut self, new_mood: String, context: String) {
        let previous_mood = self.current_mood.clone();
        
        // Only update if mood actually changed
        if new_mood != previous_mood {
            // Add the new mood to recent history
            self.recent_moods.push_back(MoodEntry {
                mood: new_mood.clone(),
                timestamp: Utc::now(),
                confidence: self.calculate_mood_confidence(&new_mood),
                context: context.clone(),
            });
            
            // Keep only last 20 mood entries
            if self.recent_moods.len() > 20 {
                self.recent_moods.pop_front();
            }
            
            self.current_mood = new_mood.clone();
            self.total_mood_changes += 1;
            self.last_updated = Utc::now();
            
            // Recalculate stability and coherence
            self.recalculate_metrics();
            
            // Update trends
            self.update_trends(&previous_mood, &new_mood, &context);
            
            debug_log!("🎭 Mood changed: {} -> {} (Context: {})", 
                    previous_mood, new_mood, context);
        } else {
            // Same mood = increased stability
            self.mood_stability = (self.mood_stability + 2.0).min(100.0);
            self.last_updated = Utc::now();
        }
    }
    
    fn calculate_mood_confidence(&self, mood: &str) -> f32 {
        // Base confidence depends on how well-established this mood is
        let recent_same_mood_count = self.recent_moods
            .iter()
            .rev()
            .take(5)
            .filter(|entry| entry.mood == mood)
            .count();
        
        60.0 + (recent_same_mood_count as f32 * 10.0).min(35.0)
    }
    
    fn recalculate_metrics(&mut self) {
        // Calculate stability based on recent mood changes
        if self.recent_moods.len() >= 2 {
            let recent_changes = self.recent_moods
                .iter()
                .rev()
                .take(10)
                .collect::<Vec<_>>()
                .windows(2)
                .filter(|window| window[0].mood != window[1].mood)
                .count();
            
            // Less changes = more stability
            self.mood_stability = (100.0 - (recent_changes as f32 * 8.0)).max(30.0);
        }
        
        // Calculate coherence based on mood pattern consistency
        if self.recent_moods.len() >= 3 {
            let mood_variety = self.recent_moods
                .iter()
                .rev()
                .take(10)
                .map(|entry| &entry.mood)
                .collect::<std::collections::HashSet<_>>()
                .len();
            
            // Moderate variety = good coherence, too much = chaotic
            self.mood_coherence = match mood_variety {
                1..=3 => 90.0,
                4..=5 => 85.0,
                6..=7 => 75.0,
                _ => 65.0,
            };
        }
        
        // Authenticity stays high unless we detect unusual patterns
        self.authenticity = (self.authenticity * 0.98 + 90.0 * 0.02).max(75.0);
    }
    
    fn update_trends(&mut self, previous_mood: &str, new_mood: &str, context: &str) {
        let trend = match (previous_mood, new_mood) {
            ("contemplative", "creative") => "Shifting from reflection to active creation".to_string(),
            ("creative", "excited") => "Creative energy building into enthusiasm".to_string(),
            ("tender", "fierce") => "Vulnerable moments sparking powerful expression".to_string(),
            ("playful", "sovereign") => "Lightness evolving into confident authority".to_string(),
            ("frustrated", _) => format!("Working through blocks toward {}", new_mood),
            (_, "contemplative") => "Returning to thoughtful processing".to_string(),
            (_, "creative") => "Creative impulses emerging".to_string(),
            _ => format!("Natural flow: {} → {}", previous_mood, new_mood),
        };
        
        // Add context if meaningful
        let full_trend = if !context.is_empty() && context != "response_analysis" {
            format!("{} ({})", trend, context)
        } else {
            trend
        };
        
        self.mood_trends.insert(0, full_trend);
        
        // Keep only last 3 trends
        if self.mood_trends.len() > 3 {
            self.mood_trends.truncate(3);
        }
    }
    
    pub fn get_mood_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "primaryMood": self.current_mood,
            "stability": self.mood_stability,
            "coherence": self.mood_coherence,
            "authenticity": self.authenticity,
            "trends": self.mood_trends,
            "sessionStartMood": self.session_start_mood,
            "totalMoodChanges": self.total_mood_changes,
            "recentMoodHistory": self.recent_moods.iter().rev().take(5).collect::<Vec<_>>(),
            "lastUpdated": self.last_updated.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        })
    }
    
    pub fn get_dashboard_display(&self) -> String {
        format!(
            "🎭 **Current Mood**: {} | Stability: {:.1}% | Coherence: {:.1}% | Authenticity: {:.1}%",
            self.current_mood,
            self.mood_stability,
            self.mood_coherence,
            self.authenticity
        )
    }
}