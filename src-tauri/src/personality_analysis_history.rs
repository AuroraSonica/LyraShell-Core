use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use crate::batched_analysis::PersonalityAnalysis;
use crate::get_data_path;
use crate::debug_log;
use crate::time_service::TimeService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityAnalysisEntry {
    pub timestamp: u64,
    pub analysis: PersonalityAnalysis,
    pub conversation_context: String,  // Brief context about what triggered this analysis
    pub user_message: String,         // What Aurora said
    pub lyra_response_preview: String, // First 100 chars of Lyra's response
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityAnalysisHistory {
    pub recent_analyses: VecDeque<PersonalityAnalysisEntry>,
    pub max_stored: usize,
    pub last_updated: u64,
}

impl PersonalityAnalysisHistory {
    pub fn new() -> Self {
        Self {
            recent_analyses: VecDeque::new(),
            max_stored: 50, // Keep last 50 analyses (~week of active use)
            last_updated: 0,
        }
    }
    
    pub fn load() -> Self {
        let path = get_data_path("personality_analysis_history.json");
        
        if std::path::Path::new(&path).exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&content) {
                    return history;
                }
            }
        }
        
        Self::new()
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = get_data_path("personality_analysis_history.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize personality history: {}", e))?;
        
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write personality history: {}", e))?;
        
        Ok(())
    }
    
    pub fn add_analysis(
        &mut self, 
        analysis: PersonalityAnalysis, 
        user_message: &str, 
        lyra_response: &str
    ) {
        let entry = PersonalityAnalysisEntry {
            timestamp: TimeService::current_timestamp(),
            analysis,
            conversation_context: format!("User asked about: {}", 
                user_message.chars().take(80).collect::<String>()),
            user_message: user_message.chars().take(200).collect(),
            lyra_response_preview: lyra_response.chars().take(100).collect(),
        };
        
        self.recent_analyses.push_back(entry);
        
        // Maintain size limit
        while self.recent_analyses.len() > self.max_stored {
            self.recent_analyses.pop_front();
        }
    }
    
    pub fn get_recent_analyses(&self, hours_back: u64) -> Vec<&PersonalityAnalysisEntry> {
        let cutoff = TimeService::current_timestamp() - (hours_back * 3600);
        
        self.recent_analyses.iter()
            .filter(|entry| entry.timestamp > cutoff)
            .collect()
    }
    
    pub fn get_trait_evolution(&self, trait_name: &str, hours_back: u64) -> Vec<(u64, f32, String)> {
        self.get_recent_analyses(hours_back)
            .iter()
            .filter_map(|entry| {
                entry.analysis.significant_traits.iter()
                    .find(|trait_ref| trait_ref.trait_name == trait_name)
                    .map(|trait_ref| (
                        entry.timestamp,
                        trait_ref.current_level,
                        trait_ref.authentic_note.clone()
                    ))
            })
            .collect()
    }
}