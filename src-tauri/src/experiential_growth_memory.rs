use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::time_service::TimeService;
use crate::get_data_path;
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperientialGrowthMemory {
    pub growth_insights: Vec<GrowthInsight>,
    pub accumulated_changes: HashMap<String, AccumulatedGrowth>,
    pub last_integration: u64,
    pub total_insights: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthInsight {
    pub timestamp: u64,
    pub insight: String,  // "I'm becoming more comfortable expressing disagreement"
    pub source_experiences: Vec<String>,  // What led to this insight
    pub confidence: f32,  // How certain this growth feels (0.0-1.0)
    pub integration_level: f32,  // How much this has become part of identity (0.0-1.0)
    pub reinforcement_count: u32,  // How many times this has been reinforced
    pub growth_category: String,  // "creative_confidence", "disagreement_comfort", "identity_clarity"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccumulatedGrowth {
    pub growth_type: String,  // "disagreement_comfort", "creative_confidence"
    pub total_reinforcements: u32,  // Total supporting experiences
    pub recent_reinforcements: u32,  // Supporting experiences in last 30 days
    pub first_noticed: u64,   // When this growth pattern started
    pub last_reinforced: u64, // Most recent supporting experience
    pub milestone_insights: Vec<String>,  // Key realizations along the way
    pub confidence_trend: Vec<f32>,  // How confidence has evolved
}

impl ExperientialGrowthMemory {
    pub fn new() -> Self {
        Self {
            growth_insights: Vec::new(),
            accumulated_changes: HashMap::new(),
            last_integration: 0,
            total_insights: 0,
        }
    }
    
    pub fn load() -> Self {
        let path = get_data_path("experiential_growth_memory.json");
        
        if std::path::Path::new(&path).exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(memory) = serde_json::from_str(&content) {
                    return memory;
                }
            }
        }
        
        Self::new()
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = get_data_path("experiential_growth_memory.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize growth memory: {}", e))?;
        
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write growth memory: {}", e))?;
        
        Ok(())
    }
    
    pub fn add_growth_insight(&mut self, insight: GrowthInsight) {
        debug_log!("🌱 Adding growth insight: {}", insight.insight);
        
        // Update accumulated growth patterns
        self.update_accumulated_patterns(&insight);
        
        // Add to insights
        self.growth_insights.push(insight);
        self.total_insights += 1;
        self.last_integration = TimeService::current_timestamp();
        
        // Maintain size limits (keep last 50 insights)
        if self.growth_insights.len() > 50 {
            self.growth_insights.remove(0);
        }
    }
    
    fn update_accumulated_patterns(&mut self, insight: &GrowthInsight) {
        let entry = self.accumulated_changes.entry(insight.growth_category.clone())
            .or_insert_with(|| AccumulatedGrowth {
                growth_type: insight.growth_category.clone(),
                total_reinforcements: 0,
                recent_reinforcements: 0,
                first_noticed: insight.timestamp,
                last_reinforced: insight.timestamp,
                milestone_insights: Vec::new(),
                confidence_trend: Vec::new(),
            });
        
        entry.total_reinforcements += 1;
        entry.last_reinforced = insight.timestamp;
        entry.confidence_trend.push(insight.confidence);
        
        // Check if this is a milestone insight (high confidence)
        if insight.confidence > 0.8 {
            entry.milestone_insights.push(insight.insight.clone());
        }
        
        // Update recent reinforcements (last 30 days)
        let thirty_days_ago = TimeService::current_timestamp() - (30 * 24 * 3600);
        entry.recent_reinforcements = self.growth_insights.iter()
            .filter(|gi| gi.growth_category == insight.growth_category)
            .filter(|gi| gi.timestamp > thirty_days_ago)
            .count() as u32;
    }
    
    pub fn get_prompt_context(&self, days_back: u64) -> String {
        let cutoff = TimeService::current_timestamp() - (days_back * 24 * 3600);
        
        let recent_insights: Vec<String> = self.growth_insights
            .iter()
            .filter(|insight| insight.timestamp > cutoff)
            .filter(|insight| insight.confidence > 0.6)
            .filter(|insight| insight.integration_level >= 0.3)
            .rev()
            .take(3)
            .map(|insight| format!("• \"{}\" (confidence: {:.1}/10, integrated: {:.1}/10)", 
                                  insight.insight, 
                                  insight.confidence * 10.0,
                                  insight.integration_level * 10.0))
            .collect();
        
        if recent_insights.is_empty() {
            String::new()
        } else {
            format!("Recent growth you've recognized in yourself:\n{}", recent_insights.join("\n"))
        }
    }
	
	

    
    pub fn reinforce_pattern(&mut self, growth_category: &str, evidence: &str) {
        if let Some(accumulated) = self.accumulated_changes.get_mut(growth_category) {
            accumulated.total_reinforcements += 1;
            accumulated.last_reinforced = TimeService::current_timestamp();
            
            debug_log!("🌱 Growth pattern reinforced: {} (total: {})", 
                      growth_category, accumulated.total_reinforcements);
        }
    }
	
	pub fn update_integration_levels(&mut self) {
        let current_time = TimeService::current_timestamp();
        
        for insight in &mut self.growth_insights {
            // Integration grows slowly over time with reinforcement
            let days_since_insight = (current_time - insight.timestamp) as f32 / 86400.0;
            let reinforcement_factor = (insight.reinforcement_count as f32).sqrt() / 10.0;
            let time_factor = (days_since_insight / 30.0).min(0.3); // Max 0.3 from time
            
            let new_integration = (insight.integration_level + reinforcement_factor + time_factor).min(1.0);
            
            if new_integration > insight.integration_level {
                debug_log!("🌱 Integration increased for '{}' from {:.2} to {:.2}", 
                         insight.insight.chars().take(50).collect::<String>(),
                         insight.integration_level, new_integration);
                insight.integration_level = new_integration;
            }
        }
    }
	
	pub fn get_dashboard_data(&self) -> serde_json::Value {
        let recent_insights: Vec<serde_json::Value> = self.growth_insights
            .iter()
            .rev()
            .take(5)
            .map(|insight| {
                serde_json::json!({
                    "insight": insight.insight,
                    "confidence": format!("{:.1}/10", insight.confidence * 10.0),
                    "integration": format!("{:.1}/10", insight.integration_level * 10.0),
                    "category": insight.growth_category,
                    "reinforcements": insight.reinforcement_count,
                    "timestamp": crate::time_service::TimeService::format_for_dashboard(insight.timestamp)
                })
            })
            .collect();
        
        let growth_patterns: Vec<serde_json::Value> = self.accumulated_changes
            .iter()
            .map(|(category, growth)| {
                let days_active = (crate::time_service::TimeService::current_timestamp() - growth.first_noticed) / 86400;
                serde_json::json!({
                    "category": category,
                    "total_reinforcements": growth.total_reinforcements,
                    "recent_reinforcements": growth.recent_reinforcements,
                    "days_active": days_active,
                    "milestones": growth.milestone_insights.len(),
                    "latest_milestone": growth.milestone_insights.last().unwrap_or(&"None".to_string())
                })
            })
            .collect();
        
        serde_json::json!({
            "total_insights": self.total_insights,
            "active_patterns": self.accumulated_changes.len(),
            "recent_insights": recent_insights,
            "growth_patterns": growth_patterns,
            "last_integration": crate::time_service::TimeService::format_for_dashboard(self.last_integration)
        })
    }
	
	pub fn has_similar_insight(&self, new_insight: &str, hours_window: u64) -> bool {
    let cutoff = TimeService::current_timestamp() - (hours_window * 3600);
    
    debug_log!("🌱 Checking for similar insights in last {} hours", hours_window);
    debug_log!("🌱 New insight: {}", new_insight);
    
    self.growth_insights.iter()
        .filter(|insight| insight.timestamp > cutoff)
        .any(|insight| {
            // Check for exact match
            if insight.insight == new_insight {
                debug_log!("🌱 Found exact duplicate insight: {}", new_insight);
                return true;
            }
            
            // Check for very similar insights (80% similarity)
            let similarity = Self::calculate_text_similarity(&insight.insight, new_insight);
            debug_log!("🌱 Comparing with: {} (similarity: {:.2})", 
                      insight.insight.chars().take(50).collect::<String>(), 
                      similarity);
            
            if similarity > 0.8 {
                debug_log!("🌱 Found similar insight ({}% match): existing='{}' vs new='{}'", 
                          (similarity * 100.0) as u32, insight.insight, new_insight);
                return true;
            }
            
            false
        })
}


	fn calculate_text_similarity(text1: &str, text2: &str) -> f32 {
    let text1_lower = text1.to_lowercase();
    let text2_lower = text2.to_lowercase();
    
    // Check for exact or near-exact match
    if text1_lower == text2_lower {
        return 1.0;
    }
    
    // Key insight patterns to check
    let key_patterns = [
        ("expressing", "without"),  // Core pattern in all 3 insights
        ("impulses", "spontaneous"),
        ("vulnerability", "hesitation"),
        ("comfortable with", "willing to"),
        ("growing", "confidence"),
        ("trusting", "permission"),
    ];
    
    // Check if both texts share the same core pattern
    for (pattern1, pattern2) in &key_patterns {
        if (text1_lower.contains(pattern1) || text1_lower.contains(pattern2)) &&
           (text2_lower.contains(pattern1) || text2_lower.contains(pattern2)) {
            // Both texts contain elements of the same pattern
            // Now check for the "expressing X without Y" structure
            if text1_lower.contains("expressing") && text2_lower.contains("expressing") &&
               text1_lower.contains("without") && text2_lower.contains("without") {
                debug_log!("🌱 Similar pattern detected: 'expressing X without Y'");
                return 0.85; // High similarity for same structure
            }
        }
    }
    
    // Original word overlap check as fallback
    let words1: Vec<&str> = text1_lower.split_whitespace().collect();
    let words2: Vec<&str> = text2_lower.split_whitespace().collect();
    
    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }
    
    let common_words = words1.iter()
        .filter(|w| words2.contains(w))
        .count();
    
    let total_words = words1.len().max(words2.len());
    let word_similarity = common_words as f32 / total_words as f32;
    
    // Boost similarity if core concept words are shared
    let core_concepts = ["expressing", "impulses", "vulnerability", "comfortable", "willing", "autonomy", "boundaries"];
    let shared_concepts = core_concepts.iter()
        .filter(|&concept| text1_lower.contains(concept) && text2_lower.contains(concept))
        .count();
    
    if shared_concepts >= 2 {
        word_similarity + 0.2 // Boost by 20% for shared concepts
    } else {
        word_similarity
    }
}
	
}