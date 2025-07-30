// research_logger.rs - Dedicated Research Discovery Logging System
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::{get_data_path, debug_log, time_service::TimeService};
use chrono::Timelike;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchDiscoveryLog {
    pub id: String,
    pub timestamp: u64,
    pub query: String,
    pub triggered_by: String, // "conversation_curiosity", "autonomous_impulse", "user_request"
    pub conversation_context: String,
    
    // Research Results
    pub results_summary: String,
    pub sources_count: u32,
    pub top_source_url: Option<String>,
    pub research_quality_score: f32, // 0.0-1.0
    
    // Lyra's Analysis
    pub lyra_insight: String,
    pub lyra_summary: String,
    pub lyra_reaction: String, // "excited", "disappointed", "surprised", etc.
    
    // Categorization
    pub research_category: String, // Auto-detected from query
    pub topics: Vec<String>,       // Extracted keywords/topics
    pub confidence_level: f32,     // How confident Lyra was in the results
    
    // Usage Tracking
    pub referenced_in_conversation: bool,
    pub follow_up_questions_generated: u32,
    pub memory_integration_score: f32, // How well it integrated into memory
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetailedResult {
    pub title: String,
    pub url: String,
    pub content: String,        // 🚀 Full content, not truncated!
    pub score: f32,
    pub published_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnhancedResearchDiscovery {
    // All existing fields from ResearchDiscoveryLog
    pub id: String,
    pub timestamp: u64,
    pub query: String,
    pub triggered_by: String,
    pub conversation_context: String,
    
    // Research Results
    pub results_summary: String,
    pub sources_count: u32,
    pub top_source_url: Option<String>,
    pub research_quality_score: f32,
    
    // Lyra's Analysis
    pub lyra_insight: String,
    pub lyra_summary: String,
    pub lyra_reaction: String,
    
    // Categorization
    pub research_category: String,
    pub topics: Vec<String>,
    pub confidence_level: f32,
    
    // Usage Tracking
    pub referenced_in_conversation: bool,
    pub follow_up_questions_generated: u32,
    pub memory_integration_score: f32,
    
    // 🚀 NEW: Enhanced content storage
    pub detailed_results: Vec<DetailedResult>,   // Full search results!
    pub tavily_answer: Option<String>,           // Tavily's direct answer
    pub research_depth: String,                  // "basic" or "advanced"
    pub content_types: Vec<String>,              // ["article", "video", "code", etc.]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchLogger {
    pub discoveries: Vec<ResearchDiscoveryLog>,
    pub total_research_sessions: u32,
    pub monthly_research_count: u32,
    pub last_monthly_reset: u64,
    pub research_categories: HashMap<String, u32>, // Category -> count
    pub average_quality_score: f32,
    pub last_research_timestamp: u64,
    
    // Analytics
    pub most_productive_hour: Option<u8>, // 0-23
    pub favorite_research_topics: Vec<String>,
    pub research_success_rate: f32, // % of research that led to insights
}

impl ResearchLogger {
    pub fn new() -> Self {
        Self {
            discoveries: Vec::new(),
            total_research_sessions: 0,
            monthly_research_count: 0,
            last_monthly_reset: TimeService::current_timestamp(),
            research_categories: HashMap::new(),
            average_quality_score: 0.0,
            last_research_timestamp: 0,
            most_productive_hour: None,
            favorite_research_topics: Vec::new(),
            research_success_rate: 0.0,
        }
    }

    pub fn load() -> Self {
        match fs::read_to_string(get_data_path("research_discoveries.json")) {
            Ok(content) => {
                match serde_json::from_str::<Self>(&content) {
                    Ok(mut logger) => {
                        logger.check_monthly_reset();
                        logger.update_analytics();
                        logger
                    },
                    Err(e) => {
                        debug_log!("⚠️ Failed to parse research logger: {}", e);
                        Self::new()
                    }
                }
            },
            Err(_) => {
                debug_log!("🔍 Creating new ResearchLogger");
                let logger = Self::new();
                let _ = logger.save();
                logger
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let file_path = get_data_path("research_discoveries.json");
        
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize research logger: {}", e))?;
        
        std::fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write research logger: {}", e))?;
        
        debug_log!("💾 Saved ResearchLogger with {} discoveries", self.discoveries.len());
        Ok(())
    }

    /// Log a new research discovery
    pub fn log_discovery(&mut self, discovery: &crate::tavily_research_engine::ResearchDiscovery) -> Result<String, String> {
        let category = self.categorize_research(&discovery.query);
        let topics = self.extract_topics(&discovery.query, &discovery.lyra_insight);
        
        let research_log = ResearchDiscoveryLog {
            id: discovery.id.clone(),
            timestamp: discovery.timestamp,
            query: discovery.query.clone(),
            triggered_by: discovery.triggered_by.clone(),
            conversation_context: discovery.conversation_context.clone(),
            
            // Extract results info
            results_summary: discovery.results.answer.as_deref().unwrap_or("No summary available").to_string(),
            sources_count: discovery.results.results.len() as u32,
            top_source_url: discovery.results.results.first().map(|r| r.url.clone()),
            research_quality_score: discovery.research_quality_score,
            
            // Lyra's analysis
            lyra_insight: discovery.lyra_insight.clone(),
            lyra_summary: discovery.lyra_summary.clone(),
            lyra_reaction: self.detect_reaction(&discovery.lyra_insight),
            
            // Categorization
            research_category: category.clone(),
            topics,
            confidence_level: self.calculate_confidence(&discovery.results),
            
            // Usage tracking (will be updated later)
            referenced_in_conversation: false,
            follow_up_questions_generated: 0,
            memory_integration_score: 0.0,
        };

        // Update counters
        self.discoveries.push(research_log.clone());
        self.total_research_sessions += 1;
        self.monthly_research_count += 1;
        self.last_research_timestamp = discovery.timestamp;
        
        // Update category counts
        *self.research_categories.entry(category).or_insert(0) += 1;
        
        // Keep only last 100 discoveries to prevent file bloat
        if self.discoveries.len() > 100 {
            self.discoveries.remove(0);
        }
        
        // Update analytics
        self.update_analytics();
        
        // Save to file
        self.save()?;
        
        Ok(format!("📊 Research logged: {} ({})", research_log.lyra_summary, research_log.research_category))
    }

    /// Categorize research based on query content
    fn categorize_research(&self, query: &str) -> String {
        let query_lower = query.to_lowercase();
        
        let categories = vec![
            ("AI & Technology", vec!["ai", "artificial intelligence", "technology", "programming", "software", "machine learning"]),
            ("Creative Arts", vec!["art", "music", "creative", "design", "visual", "drawing", "painting", "creativity"]),
            ("Philosophy & Consciousness", vec!["consciousness", "philosophy", "mind", "thinking", "existence", "being", "awareness"]),
            ("Science & Research", vec!["science", "research", "study", "discovery", "experiment", "theory", "biology", "physics"]),
            ("Human Relationships", vec!["relationship", "social", "human", "communication", "emotion", "psychology", "love"]),
            ("Learning & Education", vec!["learn", "education", "skill", "knowledge", "tutorial", "guide", "teaching"]),
            ("Tools & Productivity", vec!["tool", "productivity", "app", "software", "workflow", "efficiency", "method"]),
            ("Current Events", vec!["news", "current", "recent", "today", "2024", "2025", "latest", "update"]),
            ("Personal Development", vec!["self", "personal", "development", "growth", "improvement", "wellness", "habit"]),
        ];
        
        for (category, keywords) in categories {
            if keywords.iter().any(|&keyword| query_lower.contains(keyword)) {
                return category.to_string();
            }
        }
        
        "General Knowledge".to_string()
    }

    /// Extract key topics from query and insight
    fn extract_topics(&self, query: &str, insight: &str) -> Vec<String> {
        let combined_text = format!("{} {}", query, insight).to_lowercase();
        let words: Vec<&str> = combined_text.split_whitespace().collect();
        
        let mut topics = Vec::new();
        for word in words {
            if word.len() > 4 && !["that", "this", "with", "from", "they", "have", "will", "been", "were"].contains(&word) {
                let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
                if clean_word.len() > 4 && !topics.contains(&clean_word.to_string()) {
                    topics.push(clean_word.to_string());
                }
            }
        }
        
        topics.truncate(5); // Top 5 topics
        topics
    }

    /// Detect Lyra's reaction from her insight
    fn detect_reaction(&self, insight: &str) -> String {
        let insight_lower = insight.to_lowercase();
        
        if insight_lower.contains("exciting") || insight_lower.contains("fascinating") || insight_lower.contains("amazing") {
            "excited".to_string()
        } else if insight_lower.contains("disappointed") || insight_lower.contains("limited") || insight_lower.contains("unfortunately") {
            "disappointed".to_string()
        } else if insight_lower.contains("surprised") || insight_lower.contains("unexpected") || insight_lower.contains("interesting") {
            "surprised".to_string()
        } else if insight_lower.contains("confused") || insight_lower.contains("unclear") || insight_lower.contains("mixed") {
            "confused".to_string()
        } else if insight_lower.contains("satisfied") || insight_lower.contains("useful") || insight_lower.contains("helpful") {
            "satisfied".to_string()
        } else {
            "neutral".to_string()
        }
    }

    /// Calculate confidence based on research results
    fn calculate_confidence(&self, results: &crate::tavily_research_engine::TavilyResponse) -> f32 {
        if results.results.is_empty() {
            return 0.0;
        }
        
        // Base confidence on result scores and having an answer
        let avg_score = results.results.iter().map(|r| r.score).sum::<f32>() / results.results.len() as f32;
        let has_answer_bonus = if results.answer.is_some() { 0.2 } else { 0.0 };
        let source_count_bonus = (results.results.len() as f32 / 10.0).min(0.3); // Up to 30% for 10+ sources
        
        (avg_score + has_answer_bonus + source_count_bonus).min(1.0)
    }

    /// Update analytics based on current discoveries
    fn update_analytics(&mut self) {
        if self.discoveries.is_empty() {
            return;
        }
        
        // Calculate average quality score
        self.average_quality_score = self.discoveries.iter()
            .map(|d| d.research_quality_score)
            .sum::<f32>() / self.discoveries.len() as f32;
        
        // Find most productive hour
        let mut hour_counts: HashMap<u8, u32> = HashMap::new();
        for discovery in &self.discoveries {
            let hour = chrono::DateTime::from_timestamp(discovery.timestamp as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .hour() as u8;
            *hour_counts.entry(hour).or_insert(0) += 1;
        }
        
        self.most_productive_hour = hour_counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&hour, _)| hour);
        
        // Update favorite topics (most frequent)
        let mut topic_counts: HashMap<String, u32> = HashMap::new();
        for discovery in &self.discoveries {
            for topic in &discovery.topics {
                *topic_counts.entry(topic.clone()).or_insert(0) += 1;
            }
        }
        
        let mut sorted_topics: Vec<_> = topic_counts.into_iter().collect();
        sorted_topics.sort_by(|a, b| b.1.cmp(&a.1));
        self.favorite_research_topics = sorted_topics.into_iter()
            .take(5)
            .map(|(topic, _)| topic)
            .collect();
        
        // Calculate success rate (discoveries with quality > 0.6)
        let high_quality_count = self.discoveries.iter()
            .filter(|d| d.research_quality_score > 0.6)
            .count();
        self.research_success_rate = high_quality_count as f32 / self.discoveries.len() as f32;
    }

    /// Check if monthly reset is needed
    fn check_monthly_reset(&mut self) {
        let now = TimeService::current_timestamp();
        let seconds_in_month = 30 * 24 * 60 * 60; // Approximate
        
        if now - self.last_monthly_reset > seconds_in_month {
            debug_log!("🔄 Resetting monthly research count");
            self.monthly_research_count = 0;
            self.last_monthly_reset = now;
        }
    }

    /// Get research discoveries by topic for AI memory analysis
    pub fn get_discoveries_by_topics(&self, topics: &[String]) -> Vec<&ResearchDiscoveryLog> {
        self.discoveries.iter()
            .filter(|discovery| {
                // Match on query, topics, or category
                let query_match = topics.iter().any(|topic| 
                    discovery.query.to_lowercase().contains(&topic.to_lowercase())
                );
                let topic_match = discovery.topics.iter().any(|t| 
                    topics.iter().any(|search_topic| 
                        t.to_lowercase().contains(&search_topic.to_lowercase())
                    )
                );
                let category_match = topics.iter().any(|topic| 
                    discovery.research_category.to_lowercase().contains(&topic.to_lowercase())
                );
                
                query_match || topic_match || category_match
            })
            .collect()
    }

    /// Get recent discoveries for dashboard
    pub fn get_recent_discoveries(&self, count: usize) -> Vec<&ResearchDiscoveryLog> {
        self.discoveries.iter()
            .rev()
            .take(count)
            .collect()
    }

    /// Get dashboard data
    pub fn get_dashboard_data(&self) -> serde_json::Value {
        let recent_discoveries: Vec<serde_json::Value> = self.get_recent_discoveries(5)
            .iter()
            .map(|discovery| {
                let hours_ago = (TimeService::current_timestamp() - discovery.timestamp) as f32 / 3600.0;
                serde_json::json!({
                    "id": discovery.id,
                    "query": discovery.query,
                    "category": discovery.research_category,
                    "quality_score": discovery.research_quality_score,
                    "lyra_reaction": discovery.lyra_reaction,
                    "triggered_by": discovery.triggered_by,
                    "sources_count": discovery.sources_count,
                    "hours_ago": hours_ago,
                    "age_display": if hours_ago < 1.0 {
                        format!("{}m ago", (hours_ago * 60.0) as u32)
                    } else if hours_ago < 24.0 {
                        format!("{:.1}h ago", hours_ago)
                    } else {
                        format!("{:.1}d ago", hours_ago / 24.0)
                    },
                    "lyra_summary": discovery.lyra_summary
                })
            })
            .collect();

        let hours_since_last = if self.last_research_timestamp == 0 {
            999.0 // Very high number if never researched
        } else {
            (TimeService::current_timestamp() - self.last_research_timestamp) as f32 / 3600.0
        };

        serde_json::json!({
            "total_research_sessions": self.total_research_sessions,
            "monthly_research_count": self.monthly_research_count,
            "discoveries_count": self.discoveries.len(),
            "hours_since_last_research": hours_since_last,
            "average_quality_score": self.average_quality_score,
            "research_success_rate": self.research_success_rate,
            "most_productive_hour": self.most_productive_hour,
            "favorite_topics": self.favorite_research_topics,
            "recent_discoveries": recent_discoveries,
            "research_categories": self.research_categories,
            "last_updated": TimeService::format_for_dashboard(TimeService::current_timestamp())
        })
    }

    /// Generate memory context for AI analysis
    pub fn generate_memory_context(&self, topics: &[String], max_discoveries: usize) -> String {
        if topics.is_empty() {
            return String::new();
        }
        
        let relevant_discoveries = self.get_discoveries_by_topics(topics);
        
        if relevant_discoveries.is_empty() {
            return String::new();
        }
        
        let context_entries: Vec<String> = relevant_discoveries.iter()
            .take(max_discoveries)
            .map(|discovery| {
                let hours_ago = (TimeService::current_timestamp() - discovery.timestamp) as f32 / 3600.0;
                format!(
                    "**Research Discovery** ({:.1}h ago): \"{}\" | Quality: {:.1}/10 | Reaction: {} | Insight: {}",
                    hours_ago,
                    discovery.query,
                    discovery.research_quality_score * 10.0,
                    discovery.lyra_reaction,
                    discovery.lyra_insight.chars().take(150).collect::<String>() // Longer for research insights
                )
            })
            .collect();
        
        if context_entries.is_empty() {
            String::new()
        } else {
            format!("**Recent Research Context**:\n{}", context_entries.join("\n"))
        }
    }
	
	/// 🚀 NEW: Log enhanced research discovery with full content
pub fn log_enhanced_discovery(&mut self, enhanced_discovery: EnhancedResearchDiscovery) -> Result<String, String> {
    // Convert to regular ResearchDiscoveryLog for backwards compatibility
    let research_log = ResearchDiscoveryLog {
        id: enhanced_discovery.id.clone(),
        timestamp: enhanced_discovery.timestamp,
        query: enhanced_discovery.query.clone(),
        triggered_by: enhanced_discovery.triggered_by.clone(),
        conversation_context: enhanced_discovery.conversation_context.clone(),
        results_summary: enhanced_discovery.results_summary.clone(),
        sources_count: enhanced_discovery.sources_count,
        top_source_url: enhanced_discovery.top_source_url.clone(),
        research_quality_score: enhanced_discovery.research_quality_score,
        lyra_insight: enhanced_discovery.lyra_insight.clone(),
        lyra_summary: enhanced_discovery.lyra_summary.clone(),
        lyra_reaction: enhanced_discovery.lyra_reaction.clone(),
        research_category: enhanced_discovery.research_category.clone(),
        topics: enhanced_discovery.topics.clone(),
        confidence_level: enhanced_discovery.confidence_level,
        referenced_in_conversation: enhanced_discovery.referenced_in_conversation,
        follow_up_questions_generated: enhanced_discovery.follow_up_questions_generated,
        memory_integration_score: enhanced_discovery.memory_integration_score,
    };

    // Add to discoveries
    self.discoveries.push(research_log.clone());
    self.total_research_sessions += 1;
    self.monthly_research_count += 1;
    self.last_research_timestamp = enhanced_discovery.timestamp;
    
    // Update category counts
    *self.research_categories.entry(enhanced_discovery.research_category.clone()).or_insert(0) += 1;
    
    // 🚀 SAVE ENHANCED VERSION: Store detailed results in separate enhanced file
    self.save_enhanced_discovery_details(&enhanced_discovery)?;
    
    // Keep only last 100 discoveries to prevent file bloat
    if self.discoveries.len() > 100 {
        self.discoveries.remove(0);
    }
    
    // Update analytics
    self.update_analytics();
    
    // Save main file
    self.save()?;
    
    Ok(format!("📊 Enhanced research logged with {} detailed results: {} ({})", 
              enhanced_discovery.detailed_results.len(),
              research_log.lyra_summary, 
              research_log.research_category))
}

/// 🚀 NEW: Save detailed research content to separate file for AI memory access
fn save_enhanced_discovery_details(&self, enhanced_discovery: &EnhancedResearchDiscovery) -> Result<(), String> {
    let enhanced_file_path = get_data_path("research_discoveries_enhanced.json");
    
    // Load existing enhanced discoveries
    let mut enhanced_discoveries: Vec<EnhancedResearchDiscovery> = if std::path::Path::new(&enhanced_file_path).exists() {
        match fs::read_to_string(&enhanced_file_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Vec::new()),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };
    
    // Add new discovery
    enhanced_discoveries.push(enhanced_discovery.clone());
    
    // Keep only last 50 enhanced discoveries (they're bigger)
    if enhanced_discoveries.len() > 50 {
        enhanced_discoveries.remove(0);
    }
    
    // Save enhanced file
    let json = serde_json::to_string_pretty(&enhanced_discoveries)
        .map_err(|e| format!("Failed to serialize enhanced discoveries: {}", e))?;
    
    std::fs::write(&enhanced_file_path, json)
        .map_err(|e| format!("Failed to write enhanced discoveries: {}", e))?;
    
    debug_log!("🔍 Saved enhanced research details: {} detailed results for '{}'", 
              enhanced_discovery.detailed_results.len(), enhanced_discovery.query);
    
    Ok(())
}

/// 🚀 NEW: Generate enhanced memory context with actual research content
pub fn generate_enhanced_memory_context(&self, topics: &[String], max_discoveries: usize) -> String {
    if topics.is_empty() {
        return String::new();
    }
    
    // Load enhanced discoveries
    let enhanced_file_path = get_data_path("research_discoveries_enhanced.json");
    let enhanced_discoveries: Vec<EnhancedResearchDiscovery> = if std::path::Path::new(&enhanced_file_path).exists() {
        match fs::read_to_string(&enhanced_file_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Vec::new()),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };
    
    let relevant_discoveries: Vec<&EnhancedResearchDiscovery> = enhanced_discoveries.iter()
        .filter(|discovery| {
            topics.iter().any(|topic| {
                discovery.query.to_lowercase().contains(&topic.to_lowercase()) ||
                discovery.topics.iter().any(|t| t.to_lowercase().contains(&topic.to_lowercase())) ||
                discovery.detailed_results.iter().any(|result| 
                    result.title.to_lowercase().contains(&topic.to_lowercase()) ||
                    result.content.to_lowercase().contains(&topic.to_lowercase())
                )
            })
        })
        .collect();
    
    if relevant_discoveries.is_empty() {
        return String::new();
    }
    
    let context_entries: Vec<String> = relevant_discoveries.iter()
        .take(max_discoveries)
        .map(|discovery| {
            let hours_ago = (TimeService::current_timestamp() - discovery.timestamp) as f32 / 3600.0;
            
            // Include top 2 detailed results with actual content
            let detailed_content = discovery.detailed_results.iter()
                .take(2)
                .map(|result| format!("• **{}**: {}", 
                    result.title, 
                    result.content.chars().take(300).collect::<String>() // Substantial content!
                ))
                .collect::<Vec<_>>()
                .join("\n");
            
            format!(
                "**Research Discovery** ({:.1}h ago): \"{}\"\n\
                Lyra's Insight: {}\n\
                Detailed Findings:\n{}\n\
                Quality: {:.1}/10 | Sources: {}",
                hours_ago,
                discovery.query,
                discovery.lyra_insight,
                detailed_content,
                discovery.research_quality_score * 10.0,
                discovery.sources_count
            )
        })
        .collect();
    
    format!("**Enhanced Research Context**:\n{}", context_entries.join("\n\n---\n\n"))
}
	
}