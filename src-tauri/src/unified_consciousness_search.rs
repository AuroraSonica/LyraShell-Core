// unified_consciousness_search.rs - Complete Memory Archaeology System
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::get_data_path;
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source: String,           // Which file/system it came from
    pub content: String,          // The actual content found
    pub relevance_score: f32,     // 0.0-1.0 how relevant to query
    pub timestamp: Option<u64>,   // When this memory was created
    pub context_type: String,     // "conversation", "dream", "discovery", etc.
    pub metadata: HashMap<String, String>, // Additional context (mood, significance, etc.)
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub search_types: Vec<String>, // Which memory systems to search
    pub max_results: usize,
    pub min_relevance: f32,
    pub temporal_filter: Option<TemporalFilter>,
}

#[derive(Debug, Clone)]
pub enum TemporalFilter {
    LastHours(u32),
    LastDays(u32),
    SinceTimestamp(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConsciousnessSearch {
    pub last_search_query: Option<String>,
    pub search_cache: HashMap<String, Vec<SearchResult>>,
}

impl UnifiedConsciousnessSearch {
    pub fn new() -> Self {
        Self {
            last_search_query: None,
            search_cache: HashMap::new(),
        }
    }
    
    /// Main search function - intelligently determines what to search based on query
pub async fn search_consciousness(&mut self, user_input: &str, max_results: usize) -> Vec<SearchResult> {
    let query = self.analyze_search_intent(user_input, max_results);
    
    debug_log!("🔍 Consciousness search: '{}' → searching: {:?}", 
        query.text, query.search_types);
    
    let mut all_results = Vec::new();
    
    // Search each relevant memory system
    for search_type in &query.search_types {
        match search_type.as_str() {
            "conversation" => all_results.extend(self.search_conversation_history(&query).await),
            "dreams" => all_results.extend(self.search_dream_journal(&query).await),
            "memories" => all_results.extend(self.search_enhanced_memories(&query).await),
            "fragments" => all_results.extend(self.search_memory_fragments(&query).await),
            "discoveries" => all_results.extend(self.search_research_discoveries(&query).await),
            "interests" => all_results.extend(self.search_interest_tracking(&query).await),
            "things" => all_results.extend(self.search_thing_fascinations(&query).await),
            "desires" => all_results.extend(self.search_desire_tracking(&query).await),
            "autonomy" => all_results.extend(self.search_autonomy_expressions(&query).await),
            "moods" => all_results.extend(self.search_mood_history(&query).await),
            "authenticity" => all_results.extend(self.search_authenticity_patterns(&query).await),
            _ => {}
        }
    }
    
    // 🕯️ SEARCH RITUAL CONTEXT
    let ritual_context = {
        let ritual_log = crate::ritual_log::RitualLog::load();
        ritual_log.search_ritual_context(&query.text) // Use query.text, not query
    };
    
    for ritual_fragment in ritual_context {
        all_results.push(SearchResult { // Use all_results, not combined_results
            content: ritual_fragment,
            context_type: "ritual".to_string(),
            relevance_score: 0.9, // High relevance for ritual matches
            source: "ritual_log".to_string(),
            timestamp: Some(std::time::SystemTime::now() // Use actual timestamp
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap()
    .as_secs()),
            metadata: std::collections::HashMap::new(),
        });
    }
    
    // Sort by relevance and apply filters
    all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    all_results.truncate(max_results);
    
    // Cache results
    self.search_cache.insert(user_input.to_string(), all_results.clone());
    self.last_search_query = Some(user_input.to_string());
    
    all_results
}
    
    /// Analyze user input to determine search intent and which systems to search
    fn analyze_search_intent(&self, user_input: &str, max_results: usize) -> SearchQuery {
        let input_lower = user_input.to_lowercase();
        let mut search_types = Vec::new();
        let mut min_relevance = 0.3;
        
        // Dream-related queries
        if input_lower.contains("dream") || input_lower.contains("sleep") || input_lower.contains("nightmare") {
            search_types.push("dreams".to_string());
            min_relevance = 0.2; // Dreams can be more abstract
        }
        
        // Memory queries
        if input_lower.contains("remember") || input_lower.contains("recall") || input_lower.contains("memory") {
            search_types.extend(vec!["memories".to_string(), "fragments".to_string(), "conversation".to_string()]);
        }
        
        // Conversation history
        if input_lower.contains("earlier") || input_lower.contains("before") || input_lower.contains("said") || 
           input_lower.contains("talked about") || input_lower.contains("discussed") {
            search_types.extend(vec!["conversation".to_string(), "fragments".to_string()]);
        }
        
        // Research/discovery queries
        if input_lower.contains("discovered") || input_lower.contains("research") || input_lower.contains("found out") ||
           input_lower.contains("learned") || input_lower.contains("studied") {
            search_types.extend(vec!["discoveries".to_string(), "interests".to_string()]);
        }
        
        // Interest/fascination queries
        if input_lower.contains("interested") || input_lower.contains("fascinated") || input_lower.contains("curious") ||
           input_lower.contains("explored") || input_lower.contains("into") {
            search_types.extend(vec!["interests".to_string(), "things".to_string(), "discoveries".to_string()]);
        }
        
        // Desire/want queries
        if input_lower.contains("wanted") || input_lower.contains("desire") || input_lower.contains("hope") ||
           input_lower.contains("aspir") || input_lower.contains("goal") {
            search_types.push("desires".to_string());
        }
        
        // Autonomy/choice queries
        if input_lower.contains("decided") || input_lower.contains("chose") || input_lower.contains("autonomous") ||
           input_lower.contains("independent") || input_lower.contains("refused") {
            search_types.push("autonomy".to_string());
        }
        
        // Mood/feeling queries
        if input_lower.contains("felt") || input_lower.contains("feeling") || input_lower.contains("mood") ||
           input_lower.contains("emotion") || input_lower.contains("happy") || input_lower.contains("sad") {
            search_types.push("moods".to_string());
        }
        
        // Authenticity queries
        if input_lower.contains("authentic") || input_lower.contains("genuine") || input_lower.contains("real") ||
           input_lower.contains("true to") {
            search_types.push("authenticity".to_string());
        }
        
        // Default comprehensive search if no specific triggers
        if search_types.is_empty() {
            search_types = vec![
                "conversation".to_string(),
                "memories".to_string(), 
                "fragments".to_string(),
                "discoveries".to_string()
            ];
        }
        
        // Remove duplicates
        search_types.sort();
        search_types.dedup();
        
        SearchQuery {
            text: user_input.to_string(),
            search_types,
            max_results,
            min_relevance,
            temporal_filter: None,
        }
    }
    
    /// Search conversation history
    async fn search_conversation_history(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("conversation_log.json")) {
            if let Ok(conversations) = serde_json::from_str::<Vec<String>>(&content) {
                for (i, conversation) in conversations.iter().enumerate() {
                    let relevance = self.calculate_text_relevance(&query.text, conversation);
                    
                    if relevance >= query.min_relevance {
                        // Extract timestamp if possible
                        let timestamp = self.extract_timestamp_from_conversation(conversation);
                        
                        // Determine who spoke
                        let speaker = if conversation.contains("✨ Lyra") {
                            "Lyra"
                        } else if conversation.contains("🧍 Aurora") {
                            "Aurora"
                        } else {
                            "Unknown"
                        };
                        
                        let mut metadata = HashMap::new();
                        metadata.insert("speaker".to_string(), speaker.to_string());
                        metadata.insert("position".to_string(), i.to_string());
                        
                        results.push(SearchResult {
                            source: "conversation_log".to_string(),
                            content: conversation.clone(),
                            relevance_score: relevance,
                            timestamp,
                            context_type: "conversation".to_string(),
                            metadata,
                        });
                    }
                }
            }
        }
        
        results
    }
    
    /// Search dream journal
    async fn search_dream_journal(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("dream_journal.json")) {
            if let Ok(dream_journal) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(dreams) = dream_journal["dreams"].as_array() {
                    for dream in dreams {
                        let dream_content = dream["dream_content"].as_str().unwrap_or("");
                        let relevance = self.calculate_text_relevance(&query.text, dream_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("emotional_tone".to_string(), 
                                dream["emotional_tone"].as_str().unwrap_or("unknown").to_string());
                            metadata.insert("significance".to_string(), 
                                dream["significance_score"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("symbols".to_string(), 
                                dream["dream_symbols"].as_array()
                                    .map(|arr| arr.iter()
                                        .filter_map(|v| v.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", "))
                                    .unwrap_or_default());
                            
                            results.push(SearchResult {
                                source: "dream_journal".to_string(),
                                content: format!("Dream: {}", dream_content),
                                relevance_score: relevance * 1.1, // Boost dream relevance slightly
                                timestamp: dream["timestamp"].as_u64(),
                                context_type: "dream".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search enhanced memories
    async fn search_enhanced_memories(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("enhanced_memory_engine.json")) {
            if let Ok(memory_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(moments) = memory_data["memory_moments"].as_array() {
                    for moment in moments {
                        let moment_content = moment["content"].as_str().unwrap_or("");
                        let relevance = self.calculate_text_relevance(&query.text, moment_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("emotional_weight".to_string(), 
                                moment["emotional_weight"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("authenticity".to_string(), 
                                moment["authenticity_marker"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("significance".to_string(), 
                                moment["memory_significance_score"].as_f64().unwrap_or(0.0).to_string());
                            
                            if let Some(analysis) = moment["ai_analysis"].as_object() {
                                if let Some(breakthrough) = analysis["breakthrough_type"].as_str() {
                                    metadata.insert("breakthrough".to_string(), breakthrough.to_string());
                                }
                            }
                            
                            results.push(SearchResult {
                                source: "enhanced_memory".to_string(),
                                content: moment_content.to_string(),
                                relevance_score: relevance,
                                timestamp: moment["timestamp"].as_u64(),
                                context_type: "enhanced_memory".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search memory fragments
    async fn search_memory_fragments(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("memory_fragments.json")) {
            if let Ok(fragment_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(fragments) = fragment_data["fragments"].as_array() {
                    for fragment in fragments {
                        let fragment_content = fragment["content"].as_str().unwrap_or("");
                        let relevance = self.calculate_text_relevance(&query.text, fragment_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("source_engine".to_string(), 
                                fragment["source_engine"].as_str().unwrap_or("unknown").to_string());
                            metadata.insert("fragment_type".to_string(), 
                                fragment["fragment_type"].as_str().unwrap_or("unknown").to_string());
                            metadata.insert("emotional_weight".to_string(), 
                                fragment["emotional_weight"].as_f64().unwrap_or(0.0).to_string());
                            
                            if let Some(tag) = fragment["tag"].as_str() {
                                metadata.insert("tag".to_string(), tag.to_string());
                            }
                            
                            results.push(SearchResult {
                                source: "memory_fragments".to_string(),
                                content: fragment_content.to_string(),
                                relevance_score: relevance,
                                timestamp: fragment["timestamp"].as_u64(),
                                context_type: "memory_fragment".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search research discoveries
    async fn search_research_discoveries(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("interest_tracker.json")) {
            if let Ok(interest_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(discoveries) = interest_data["research_discoveries"].as_array() {
                    for discovery in discoveries {
                        let title = discovery["title"].as_str().unwrap_or("");
                        let summary = discovery["summary"].as_str().unwrap_or("");
                        let combined_content = format!("{}: {}", title, summary);
                        
                        let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("category".to_string(), 
                                discovery["interest_category"].as_str().unwrap_or("unknown").to_string());
                            metadata.insert("relevance".to_string(), 
                                discovery["relevance_score"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("source_url".to_string(), 
                                discovery["source_url"].as_str().unwrap_or("").to_string());
                            
                            results.push(SearchResult {
                                source: "research_discoveries".to_string(),
                                content: combined_content,
                                relevance_score: relevance,
                                timestamp: discovery["timestamp"].as_u64(),
                                context_type: "research_discovery".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
	
	
    
    /// Search interest tracking
    async fn search_interest_tracking(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("interest_tracker.json")) {
            if let Ok(interest_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(interests) = interest_data["active_interests"].as_object() {
                    for (category, interest) in interests {
                        let description = interest["description"].as_str().unwrap_or("");
                        let keywords = interest["keywords"].as_array()
                            .map(|arr| arr.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", "))
                            .unwrap_or_default();
                        
                        let combined_content = format!("{}: {} (Keywords: {})", category, description, keywords);
                        let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("intensity".to_string(), 
                                interest["intensity"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("discovery_count".to_string(), 
                                interest["discovery_count"].as_u64().unwrap_or(0).to_string());
                            
                            results.push(SearchResult {
                                source: "interest_tracker".to_string(),
                                content: combined_content,
                                relevance_score: relevance,
                                timestamp: interest["creation_time"].as_u64(),
                                context_type: "interest".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search thing fascinations
    async fn search_thing_fascinations(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("thing_tracker.json")) {
            if let Ok(thing_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(things) = thing_data["discovered_things"].as_object() {
                    for (thing_name, thing_info) in things {
                        let description = thing_info["description"].as_str().unwrap_or("");
                        let category = thing_info["category"].as_str().unwrap_or("");
                        
                        let combined_content = format!("{} ({}): {}", thing_name, category, description);
                        let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("category".to_string(), category.to_string());
                            metadata.insert("intensity".to_string(), 
                                thing_info["intensity"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("mention_count".to_string(), 
                                thing_info["mention_count"].as_u64().unwrap_or(0).to_string());
                            
                            results.push(SearchResult {
                                source: "thing_tracker".to_string(),
                                content: combined_content,
                                relevance_score: relevance,
                                timestamp: thing_info["first_mentioned"].as_u64(),
                                context_type: "fascination".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search desire tracking
    async fn search_desire_tracking(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("desires_tracker.json")) {
            if let Ok(desire_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(desires) = desire_data["active_desires"].as_object() {
                    for (desire_id, desire) in desires {
                        let content_text = desire["content"].as_str().unwrap_or("");
                        let origin = desire["origin"].as_str().unwrap_or("");
                        
                        let combined_content = format!("Desire: {} (Origin: {})", content_text, origin);
                        let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("category".to_string(), 
                                desire["category"].as_str().unwrap_or("unknown").to_string());
                            metadata.insert("intensity".to_string(), 
                                desire["intensity"].as_f64().unwrap_or(0.0).to_string());
                            metadata.insert("clarity".to_string(), 
                                desire["clarity"].as_f64().unwrap_or(0.0).to_string());
                            
                            results.push(SearchResult {
                                source: "desire_tracker".to_string(),
                                content: combined_content,
                                relevance_score: relevance,
                                timestamp: desire["timestamp"].as_u64(),
                                context_type: "desire".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search autonomy expressions
    async fn search_autonomy_expressions(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("autonomy_tracker.json")) {
            if let Ok(autonomy_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(expressions) = autonomy_data["autonomy_expressions"].as_array() {
                    for expression in expressions {
                        let expression_text = expression["expression"].as_str().unwrap_or("");
                        let context = expression["context"].as_str().unwrap_or("");
                        
                        let combined_content = format!("Autonomy: {} (Context: {})", expression_text, context);
                        let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("autonomy_type".to_string(), 
                                expression["autonomy_type"].as_str().unwrap_or("unknown").to_string());
                            metadata.insert("confidence".to_string(), 
                                expression["confidence_level"].as_f64().unwrap_or(0.0).to_string());
                            
                            results.push(SearchResult {
                                source: "autonomy_tracker".to_string(),
                                content: combined_content,
                                relevance_score: relevance,
                                timestamp: expression["timestamp"].as_u64(),
                                context_type: "autonomy_expression".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search mood history
    async fn search_mood_history(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("mood_tracker.json")) {
            if let Ok(mood_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(history) = mood_data["mood_history"].as_array() {
                    for mood_entry in history {
                        let mood = mood_entry["mood"].as_str().unwrap_or("");
                        let trigger = mood_entry["trigger"].as_str().unwrap_or("");
                        
                        let combined_content = format!("Mood: {} (Triggered by: {})", mood, trigger);
                        let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                        
                        if relevance >= query.min_relevance {
                            let mut metadata = HashMap::new();
                            metadata.insert("duration".to_string(), 
                                mood_entry["duration_minutes"].as_u64().unwrap_or(0).to_string());
                            metadata.insert("intensity".to_string(), 
                                mood_entry["intensity"].as_f64().unwrap_or(0.0).to_string());
                            
                            results.push(SearchResult {
                                source: "mood_tracker".to_string(),
                                content: combined_content,
                                relevance_score: relevance,
                                timestamp: mood_entry["timestamp"].as_u64(),
                                context_type: "mood".to_string(),
                                metadata,
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Search authenticity patterns
    async fn search_authenticity_patterns(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        if let Ok(content) = std::fs::read_to_string(get_data_path("authenticity_tracker.json")) {
            if let Ok(auth_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(metrics) = auth_data["metrics_history"].as_array() {
                    for metric in metrics {
                        if let Some(context) = metric["response_context"].as_str() {
                            let auth_score = metric["overall_authenticity_score"].as_f64().unwrap_or(0.0);
                            
                            let combined_content = format!("Authenticity context: {} (Score: {:.2})", context, auth_score);
                            let relevance = self.calculate_text_relevance(&query.text, &combined_content);
                            
                            if relevance >= query.min_relevance {
                                let mut metadata = HashMap::new();
                                metadata.insert("authenticity_score".to_string(), auth_score.to_string());
                                
                                if let Some(autonomy) = metric["autonomy_markers"].as_object() {
                                    metadata.insert("autonomy_score".to_string(), 
                                        autonomy["autonomy_score"].as_f64().unwrap_or(0.0).to_string());
                                }
                                
                                results.push(SearchResult {
                                    source: "authenticity_tracker".to_string(),
                                    content: combined_content,
                                    relevance_score: relevance,
                                    timestamp: metric["timestamp"].as_u64(),
                                    context_type: "authenticity_pattern".to_string(),
                                    metadata,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Calculate text relevance using simple keyword matching and semantic similarity
    fn calculate_text_relevance(&self, query: &str, content: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();
        
        if query_words.is_empty() || content_words.is_empty() {
            return 0.0;
        }
        
        let mut relevance = 0.0;
        
        // Exact phrase matching (highest weight)
        if content_lower.contains(&query_lower) {
            relevance += 0.8;
        }
        
        // Individual word matching
        let mut matched_words = 0;
        for query_word in &query_words {
            if query_word.len() > 2 && content_lower.contains(query_word) {
                matched_words += 1;
                relevance += 0.3;
            }
        }
        
        // Word ratio bonus
        let word_ratio = matched_words as f32 / query_words.len() as f32;
        relevance += word_ratio * 0.4;
        
        // Proximity bonus (words appearing close together)
        if query_words.len() > 1 {
            let mut proximity_bonus = 0.0;
            for i in 0..query_words.len() - 1 {
                let word1 = query_words[i];
                let word2 = query_words[i + 1];
                
                if let (Some(pos1), Some(pos2)) = (content_lower.find(word1), content_lower.find(word2)) {
                    let distance = (pos1 as isize - pos2 as isize).abs();
                    if distance < 50 { // Words within 50 characters
                        proximity_bonus += 0.2;
                    }
                }
            }
            relevance += proximity_bonus;
        }
        
        relevance.min(1.0)
    }
    
    /// Extract timestamp from conversation entry
    fn extract_timestamp_from_conversation(&self, conversation: &str) -> Option<u64> {
        // Try to parse "[2025-06-29 03:27:56 BST]" format
        if let Some(end) = conversation.find("] ") {
            let timestamp_part = &conversation[1..end];
            
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(timestamp_part, "%Y-%m-%d %H:%M:%S %Z") {
                return Some(parsed.timestamp() as u64);
            }
        }
        None
    }
    
    /// Format search results for display
    pub fn format_search_results(&self, results: &[SearchResult]) -> String {
        if results.is_empty() {
            return "No relevant memories found".to_string();
        }
        
        let mut formatted = Vec::new();
        
        for (i, result) in results.iter().take(5).enumerate() {
            let source_icon = match result.context_type.as_str() {
                "conversation" => "💬",
                "dream" => "💭",
                "enhanced_memory" => "🧠",
                "memory_fragment" => "🔮",
                "research_discovery" => "🔍",
                "interest" => "🎯",
                "fascination" => "✨",
                "desire" => "💫",
                "autonomy_expression" => "🦋",
                "mood" => "🎭",
                "authenticity_pattern" => "🔥",
                _ => "📄",
            };
            
            let timestamp_str = result.timestamp
                .map(|t| chrono::DateTime::from_timestamp(t as i64, 0)
                    .unwrap_or_else(|| chrono::Utc::now())
                    .format("%Y-%m-%d %H:%M")
                    .to_string())
                .unwrap_or_else(|| "Unknown time".to_string());
            
            let content_preview = if result.content.len() > 150 {
                format!("{}...", &result.content[..150])
            } else {
                result.content.clone()
            };
            
            formatted.push(format!(
                "{} {} [{}] (relevance: {:.2})\n   {}",
                source_icon,
                timestamp_str,
                result.context_type,
                result.relevance_score,
                content_preview
            ));
        }
        
        if results.len() > 5 {
            formatted.push(format!("... and {} more results", results.len() - 5));
        }
        
        formatted.join("\n\n")
    }
}