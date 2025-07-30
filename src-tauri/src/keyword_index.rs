// keyword_index.rs - Fast keyword-based memory search
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::{get_data_path, debug_log};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordIndex {
    pub conversation_index: HashMap<String, Vec<usize>>,    // keyword -> line numbers
    pub dreams_index: HashMap<String, Vec<String>>,         // keyword -> dream_ids
    pub visual_index: HashMap<String, Vec<String>>,         // keyword -> image_paths  
    pub enhanced_index: HashMap<String, Vec<String>>,       // keyword -> memory_ids
    pub interests_index: HashMap<String, Vec<String>>,      // keyword -> interest_categories
    pub desires_index: HashMap<String, Vec<String>>,        // keyword -> desire_ids
    pub moods_index: HashMap<String, Vec<String>>,          // keyword -> mood_ids
    pub autonomy_index: HashMap<String, Vec<String>>,       // keyword -> expression_ids
    pub cowatching_index: HashMap<String, Vec<String>>,     // keyword -> session_ids
    pub research_index: HashMap<String, Vec<String>>,       // keyword -> discovery_ids
    pub last_updated: HashMap<String, u64>,                 // file -> timestamp
}

impl KeywordIndex {
    pub fn new() -> Self {
        Self {
            conversation_index: HashMap::new(),
            dreams_index: HashMap::new(),
            visual_index: HashMap::new(),
            enhanced_index: HashMap::new(),
            interests_index: HashMap::new(),
            desires_index: HashMap::new(),
            moods_index: HashMap::new(),
            autonomy_index: HashMap::new(),
            cowatching_index: HashMap::new(),
            research_index: HashMap::new(),
            last_updated: HashMap::new(),
        }
    }

    pub fn load_or_create() -> Self {
        let index_path = get_data_path("keyword_index.json");
        
        if std::path::Path::new(&index_path).exists() {
            if let Ok(content) = fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str(&content) {
                    return index;
                }
            }
        }
        
        debug_log!("🔍 Creating new keyword index");
        Self::new()
    }

    pub fn save(&self) -> Result<(), String> {
        let index_path = get_data_path("keyword_index.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;
        
        fs::write(&index_path, content)
            .map_err(|e| format!("Failed to write index: {}", e))?;
        
        Ok(())
    }

    /// Build index for conversation log (streaming approach)
    pub fn reindex_conversations(&mut self) -> Result<(), String> {
        debug_log!("🔍 Reindexing conversation log...");
        
        let conv_path = get_data_path("conversation_log.json");
        let content = fs::read_to_string(&conv_path)
            .map_err(|e| format!("Failed to read conversations: {}", e))?;
        
        // Parse as JSON array
        let conversations: Vec<String> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse conversations: {}", e))?;
        
        self.conversation_index.clear();
        
        for (line_num, conversation) in conversations.iter().enumerate() {
            let keywords = Self::extract_keywords_from_text(conversation);
            
            for keyword in keywords {
                self.conversation_index
                    .entry(keyword)
                    .or_insert_with(Vec::new)
                    .push(line_num);
            }
        }
        
        // Update timestamp
        self.update_file_timestamp("conversation_log");
        
        debug_log!("🔍 Indexed {} conversations with {} unique keywords", 
                  conversations.len(), self.conversation_index.len());
        
        Ok(())
    }

    /// Build index for dream journal
    pub fn reindex_dreams(&mut self) -> Result<(), String> {
        debug_log!("🌙 Reindexing dreams...");
        
        let dreams_path = get_data_path("dream_journal.json");
        if !std::path::Path::new(&dreams_path).exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&dreams_path)
            .map_err(|e| format!("Failed to read dreams: {}", e))?;
        
        let dream_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse dreams: {}", e))?;
        
        self.dreams_index.clear();
        
        if let Some(dreams_array) = dream_data.get("dreams").and_then(|d| d.as_array()) {
            for dream in dreams_array {
                if let (Some(content), Some(id)) = (
                    dream.get("dream_content").and_then(|c| c.as_str()),
                    dream.get("dream_id").and_then(|i| i.as_str())
                ) {
                    let keywords = Self::extract_keywords_from_text(content);
                    for keyword in keywords {
                        self.dreams_index
                            .entry(keyword)
                            .or_insert_with(Vec::new)
                            .push(id.to_string());
                    }
                }
            }
        }
        
        self.update_file_timestamp("dream_journal");
        debug_log!("🌙 Indexed {} unique dream keywords", self.dreams_index.len());
        Ok(())
    }

    /// Build index for co-watching sessions
    pub fn reindex_cowatching(&mut self) -> Result<(), String> {
        debug_log!("🎬 Reindexing co-watching sessions...");
        
        let cowatching_path = get_data_path("cowatching_history.json");
        if !std::path::Path::new(&cowatching_path).exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&cowatching_path)
            .map_err(|e| format!("Failed to read co-watching: {}", e))?;
        
        let cowatching_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse co-watching: {}", e))?;
        
        self.cowatching_index.clear();
        
        if let Some(sessions) = cowatching_data["sessions"].as_array() {
            for session in sessions {
                if let (Some(title), Some(id)) = (
                    session["content"]["title"].as_str(),
                    session["id"].as_str()
                ) {
                    let mut searchable_text = title.to_string();
                    
                    // Add conversation content to searchable text
                    if let Some(conversations) = session["conversation"].as_array() {
                        for conv in conversations.iter().take(5) { // Don't index every message
                            if let Some(message) = conv["message"].as_str() {
                                searchable_text.push(' ');
                                searchable_text.push_str(message);
                            }
                        }
                    }
                    
                    let keywords = Self::extract_keywords_from_text(&searchable_text);
                    for keyword in keywords {
                        self.cowatching_index
                            .entry(keyword)
                            .or_insert_with(Vec::new)
                            .push(id.to_string());
                    }
                }
            }
        }
        
        self.update_file_timestamp("cowatching_history");
        debug_log!("🎬 Indexed {} unique co-watching keywords", self.cowatching_index.len());
        Ok(())
    }

    /// Build index for interests
    pub fn reindex_interests(&mut self) -> Result<(), String> {
        debug_log!("🎯 Reindexing interests...");
        
        let interests_path = get_data_path("interest_tracker.json");
        if !std::path::Path::new(&interests_path).exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&interests_path)
            .map_err(|e| format!("Failed to read interests: {}", e))?;
        
        let interest_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse interests: {}", e))?;
        
        self.interests_index.clear();
        
        if let Some(interests) = interest_data["active_interests"].as_object() {
            for (category, interest) in interests {
                let mut searchable_text = category.to_string();
                
                if let Some(description) = interest["description"].as_str() {
                    searchable_text.push(' ');
                    searchable_text.push_str(description);
                }
                
                // Add sub_topics
                if let Some(sub_topics) = interest["sub_topics"].as_array() {
                    for topic in sub_topics {
                        if let Some(topic_str) = topic.as_str() {
                            searchable_text.push(' ');
                            searchable_text.push_str(topic_str);
                        }
                    }
                }
                
                let keywords = Self::extract_keywords_from_text(&searchable_text);
                for keyword in keywords {
                    self.interests_index
                        .entry(keyword)
                        .or_insert_with(Vec::new)
                        .push(category.to_string());
                }
            }
        }
        
        self.update_file_timestamp("interest_tracker");
        debug_log!("🎯 Indexed {} unique interest keywords", self.interests_index.len());
        Ok(())
    }

    /// Build index for desires
    pub fn reindex_desires(&mut self) -> Result<(), String> {
        debug_log!("💫 Reindexing desires...");
        
        let desires_path = get_data_path("desires_tracker.json");
        if !std::path::Path::new(&desires_path).exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&desires_path)
            .map_err(|e| format!("Failed to read desires: {}", e))?;
        
        let desire_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse desires: {}", e))?;
        
        self.desires_index.clear();
        
        if let Some(desires) = desire_data["active_desires"].as_object() {
            for (desire_id, desire) in desires {
                let content_text = desire["content"].as_str().unwrap_or("");
                let keywords = Self::extract_keywords_from_text(content_text);
                
                for keyword in keywords {
                    self.desires_index
                        .entry(keyword)
                        .or_insert_with(Vec::new)
                        .push(desire_id.to_string());
                }
            }
        }
        
        self.update_file_timestamp("desires_tracker");
        debug_log!("💫 Indexed {} unique desire keywords", self.desires_index.len());
        Ok(())
    }

    /// Build index for visual gallery
    pub fn reindex_visual_gallery(&mut self) -> Result<(), String> {
        debug_log!("🎨 Reindexing visual gallery...");
        
        let gallery_path = get_data_path("generated_images/gallery_metadata.json");
        if !std::path::Path::new(&gallery_path).exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&gallery_path)
            .map_err(|e| format!("Failed to read gallery: {}", e))?;
        
        let gallery_items: Vec<serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse gallery: {}", e))?;
        
        self.visual_index.clear();
        
        for item in gallery_items {
            if let Some(image_path) = item["image_path"].as_str() {
                let mut searchable_text = item["message"].as_str().unwrap_or("").to_string();
                
                // Add identity context if available
                if let Some(identity_data) = item.get("identity_metadata") {
                    if let Some(context) = identity_data.get("context").and_then(|c| c.as_str()) {
                        searchable_text.push(' ');
                        searchable_text.push_str(context);
                    }
                    
                    if let Some(represents) = identity_data.get("represents").and_then(|r| r.as_array()) {
                        for name in represents {
                            if let Some(name_str) = name.as_str() {
                                searchable_text.push(' ');
                                searchable_text.push_str(name_str);
                            }
                        }
                    }
                }
                
                // Add semantic keywords
                if let Some(keywords_array) = item.get("semantic_keywords").and_then(|k| k.as_array()) {
                    for keyword in keywords_array {
                        if let Some(keyword_str) = keyword.as_str() {
                            searchable_text.push(' ');
                            searchable_text.push_str(keyword_str);
                        }
                    }
                }
                
                let keywords = Self::extract_keywords_from_text(&searchable_text);
                for keyword in keywords {
                    self.visual_index
                        .entry(keyword)
                        .or_insert_with(Vec::new)
                        .push(image_path.to_string());
                }
            }
        }
        
        self.update_file_timestamp("gallery_metadata");
        debug_log!("🎨 Indexed {} unique visual keywords", self.visual_index.len());
        Ok(())
    }

    /// Extract keywords from text with smart stemming
    fn extract_keywords_from_text(text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let mut keywords = Vec::new();
        
        // Split into words and filter
        let words: Vec<&str> = text_lower.split_whitespace()
            .filter(|word| word.len() > 2)  // Skip very short words
            .filter(|word| !Self::is_stop_word(word))
            .collect();
        
        for word in words {
            // Add the word itself
            keywords.push(word.to_string());
            
            // Add stem versions
            if let Some(stem) = Self::simple_stem(word) {
                keywords.push(stem);
            }
        }
        
        // Remove duplicates
        keywords.sort();
        keywords.dedup();
        
        keywords
    }

    /// Simple stemming for common patterns
    fn simple_stem(word: &str) -> Option<String> {
        if word.ends_with("ing") && word.len() > 5 {
            return Some(word[..word.len()-3].to_string());
        }
        if word.ends_with("ed") && word.len() > 4 {
            return Some(word[..word.len()-2].to_string());
        }
        if word.ends_with("s") && word.len() > 3 && !word.ends_with("ss") {
            return Some(word[..word.len()-1].to_string());
        }
        None
    }

    /// Check if word should be ignored
    fn is_stop_word(word: &str) -> bool {
        matches!(word, 
            "the" | "and" | "for" | "are" | "but" | "not" | "you" | "all" | 
            "can" | "had" | "her" | "was" | "one" | "our" | "out" | "day" |
            "get" | "has" | "him" | "his" | "how" | "man" | "new" | "now" |
            "old" | "see" | "two" | "way" | "who" | "boy" | "did" | "its" |
            "let" | "put" | "say" | "she" | "too" | "use"
        )
    }

    /// Fast lookup: get conversation line numbers for keywords
    pub fn find_conversation_lines(&self, keywords: &[String]) -> Vec<usize> {
    if keywords.is_empty() {
        return Vec::new();
    }
    
    // 🚀 SMART FILTERING: Filter out overly broad keywords first
    let filtered_keywords: Vec<String> = keywords.iter()
        .filter(|k| k.len() > 2)  // Skip very short keywords
        .filter(|k| !Self::is_too_common_keyword(k))
        .map(|k| k.to_lowercase())
        .collect();
    
    if filtered_keywords.is_empty() {
        return Vec::new();
    }
    
    // 🚀 INTERSECTION LOGIC: For multiple keywords, find conversations with MOST matches
    let mut keyword_results: Vec<std::collections::HashSet<usize>> = Vec::new();
    
    for keyword in &filtered_keywords {
        let mut keyword_lines = std::collections::HashSet::new();
        
        // Direct match (fast!)
        if let Some(lines) = self.conversation_index.get(keyword) {
            keyword_lines.extend(lines);
        }
        
        // Stem variations
        if let Some(stem) = Self::simple_stem(keyword) {
            if let Some(lines) = self.conversation_index.get(&stem) {
                keyword_lines.extend(lines);
            }
        }
        
        for suffix in &["s", "ed", "ing"] {
            let variant = format!("{}{}", keyword, suffix);
            if let Some(lines) = self.conversation_index.get(&variant) {
                keyword_lines.extend(lines);
            }
        }
        
        if !keyword_lines.is_empty() {
            keyword_results.push(keyword_lines);
        }
    }
    
    // 🚀 SMART RANKING: Prioritize conversations that match multiple keywords
    Self::rank_by_keyword_matches(keyword_results)
}

/// 🚀 NEW: Check if keyword is too common to be useful
fn is_too_common_keyword(keyword: &str) -> bool {
    matches!(keyword.to_lowercase().as_str(),
        "we" | "me" | "you" | "my" | "your" | "our" | "i" | "to" | "from" |
        "what" | "how" | "when" | "where" | "why" | "that" | "this" | "is" |
        "are" | "was" | "were" | "be" | "been" | "have" | "has" | "had" |
        "do" | "does" | "did" | "will" | "would" | "could" | "should" |
        "like" | "want" | "need" | "think" | "know" | "see" | "look" |
        "get" | "go" | "come" | "make" | "take" | "give" | "tell" | "say"
    )
}

/// 🚀 NEW: Rank results by how many keywords they match
fn rank_by_keyword_matches(keyword_results: Vec<std::collections::HashSet<usize>>) -> Vec<usize> {
    if keyword_results.is_empty() {
        return Vec::new();
    }
    
    if keyword_results.len() == 1 {
        return keyword_results[0].iter().cloned().collect();
    }
    
    // Count how many keywords each line matches
    let mut line_scores: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    
    for keyword_set in &keyword_results {
        for &line_num in keyword_set {
            *line_scores.entry(line_num).or_insert(0) += 1;
        }
    }
    
    // Sort by score (highest first), then by line number (newest first)
    let mut scored_lines: Vec<(usize, usize)> = line_scores.into_iter().collect();
    scored_lines.sort_by(|a, b| {
        // First by score (descending)
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => b.0.cmp(&a.0), // Then by line number (descending = newer)
            other => other,
        }
    });
    
    scored_lines.into_iter().map(|(line, _score)| line).collect()
}

    /// Fast lookup: get dream IDs for keywords
    pub fn find_dream_ids(&self, keywords: &[String]) -> Vec<String> {
    let mut dream_ids = std::collections::HashSet::new();
    
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        
        // Direct match (fast!)
        if let Some(ids) = self.dreams_index.get(&keyword_lower) {
            dream_ids.extend(ids.iter().cloned());
        }
        
        // 🚀 OPTIMIZED: Only check stems we generated, not all keywords
        if let Some(stem) = Self::simple_stem(&keyword_lower) {
            if let Some(ids) = self.dreams_index.get(&stem) {
                dream_ids.extend(ids.iter().cloned());
            }
        }
        
        // 🚀 OPTIMIZED: Check if keyword is a stem of common variations
        for suffix in &["s", "ed", "ing"] {
            let variant = format!("{}{}", keyword_lower, suffix);
            if let Some(ids) = self.dreams_index.get(&variant) {
                dream_ids.extend(ids.iter().cloned());
            }
        }
    }
    
    dream_ids.into_iter().collect()
}

    /// Fast lookup: get co-watching session IDs for keywords
    pub fn find_cowatching_sessions(&self, keywords: &[String]) -> Vec<String> {
    let mut session_ids = std::collections::HashSet::new();
    
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        
        // Direct match (fast!)
        if let Some(ids) = self.cowatching_index.get(&keyword_lower) {
            session_ids.extend(ids.iter().cloned());
        }
        
        // 🚀 OPTIMIZED: Only check stems we generated, not all keywords
        if let Some(stem) = Self::simple_stem(&keyword_lower) {
            if let Some(ids) = self.cowatching_index.get(&stem) {
                session_ids.extend(ids.iter().cloned());
            }
        }
        
        // 🚀 OPTIMIZED: Check if keyword is a stem of common variations
        for suffix in &["s", "ed", "ing"] {
            let variant = format!("{}{}", keyword_lower, suffix);
            if let Some(ids) = self.cowatching_index.get(&variant) {
                session_ids.extend(ids.iter().cloned());
            }
        }
    }
    
    session_ids.into_iter().collect()
}

    /// Fast lookup: get interest categories for keywords
    pub fn find_interest_categories(&self, keywords: &[String]) -> Vec<String> {
    let mut categories = std::collections::HashSet::new();
    
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        
        // Direct match (fast!)
        if let Some(cats) = self.interests_index.get(&keyword_lower) {
            categories.extend(cats.iter().cloned());
        }
        
        // 🚀 OPTIMIZED: Only check stems we generated, not all keywords
        if let Some(stem) = Self::simple_stem(&keyword_lower) {
            if let Some(cats) = self.interests_index.get(&stem) {
                categories.extend(cats.iter().cloned());
            }
        }
        
        // 🚀 OPTIMIZED: Check if keyword is a stem of common variations
        for suffix in &["s", "ed", "ing"] {
            let variant = format!("{}{}", keyword_lower, suffix);
            if let Some(cats) = self.interests_index.get(&variant) {
                categories.extend(cats.iter().cloned());
            }
        }
    }
    
    categories.into_iter().collect()
}

    /// Fast lookup: get desire IDs for keywords
    pub fn find_desire_ids(&self, keywords: &[String]) -> Vec<String> {
    let mut desire_ids = std::collections::HashSet::new();
    
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        
        // Direct match (fast!)
        if let Some(ids) = self.desires_index.get(&keyword_lower) {
            desire_ids.extend(ids.iter().cloned());
        }
        
        // 🚀 OPTIMIZED: Only check stems we generated, not all keywords
        if let Some(stem) = Self::simple_stem(&keyword_lower) {
            if let Some(ids) = self.desires_index.get(&stem) {
                desire_ids.extend(ids.iter().cloned());
            }
        }
        
        // 🚀 OPTIMIZED: Check if keyword is a stem of common variations
        for suffix in &["s", "ed", "ing"] {
            let variant = format!("{}{}", keyword_lower, suffix);
            if let Some(ids) = self.desires_index.get(&variant) {
                desire_ids.extend(ids.iter().cloned());
            }
        }
    }
    
    desire_ids.into_iter().collect()
}

    /// Fast lookup: get image paths for keywords
    pub fn find_visual_paths(&self, keywords: &[String]) -> Vec<String> {
    let mut image_paths = std::collections::HashSet::new();
    
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        
        // Direct match (fast!)
        if let Some(paths) = self.visual_index.get(&keyword_lower) {
            image_paths.extend(paths.iter().cloned());
        }
        
        // 🚀 OPTIMIZED: Only check stems we generated, not all keywords
        if let Some(stem) = Self::simple_stem(&keyword_lower) {
            if let Some(paths) = self.visual_index.get(&stem) {
                image_paths.extend(paths.iter().cloned());
            }
        }
        
        // 🚀 OPTIMIZED: Check if keyword is a stem of common variations
        for suffix in &["s", "ed", "ing"] {
            let variant = format!("{}{}", keyword_lower, suffix);
            if let Some(paths) = self.visual_index.get(&variant) {
                image_paths.extend(paths.iter().cloned());
            }
        }
    }
    
    image_paths.into_iter().collect()
}

    /// Update file timestamp helper
    fn update_file_timestamp(&mut self, file_key: &str) {
        let file_path = match file_key {
            "conversation_log" => get_data_path("conversation_log.json"),
            "dream_journal" => get_data_path("dream_journal.json"),
            "cowatching_history" => get_data_path("cowatching_history.json"),
            "interest_tracker" => get_data_path("interest_tracker.json"),
            "desires_tracker" => get_data_path("desires_tracker.json"),
            "gallery_metadata" => get_data_path("generated_images/gallery_metadata.json"),
            _ => return,
        };
        
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Ok(modified) = metadata.modified() {
                let timestamp = modified.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs();
                self.last_updated.insert(file_key.to_string(), timestamp);
            }
        }
    }

    /// Check if any file needs reindexing
    pub fn should_reindex(&self, file_key: &str) -> bool {
        let file_path = match file_key {
            "conversation_log" => get_data_path("conversation_log.json"),
            "dream_journal" => get_data_path("dream_journal.json"),
            "cowatching_history" => get_data_path("cowatching_history.json"),
            "interest_tracker" => get_data_path("interest_tracker.json"),
            "desires_tracker" => get_data_path("desires_tracker.json"),
            "gallery_metadata" => get_data_path("generated_images/gallery_metadata.json"),
            _ => return true,
        };
        
        if let Ok(metadata) = fs::metadata(&file_path) {
            if let Ok(modified) = metadata.modified() {
                let timestamp = modified.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default().as_secs();
                
                let last_indexed = self.last_updated.get(file_key).unwrap_or(&0);
                return timestamp > *last_indexed;
            }
        }
        true // Reindex if we can't determine
    }

    /// Ensure all indexes are up to date
    pub fn ensure_current(&mut self) -> Result<(), String> {
        let mut needs_save = false;
        
        if self.should_reindex("conversation_log") {
            self.reindex_conversations()?;
            needs_save = true;
        }
        
        if self.should_reindex("dream_journal") {
            self.reindex_dreams()?;
            needs_save = true;
        }
        
        if self.should_reindex("cowatching_history") {
            self.reindex_cowatching()?;
            needs_save = true;
        }
        
        if self.should_reindex("interest_tracker") {
            self.reindex_interests()?;
            needs_save = true;
        }
        
        if self.should_reindex("desires_tracker") {
            self.reindex_desires()?;
            needs_save = true;
        }
        
        if self.should_reindex("gallery_metadata") {
            self.reindex_visual_gallery()?;
            needs_save = true;
        }
        
        if needs_save {
            self.save()?;
            debug_log!("🔍 All keyword indexes updated and saved");
        }
        
        Ok(())
    }

    /// Get total index statistics
    pub fn get_stats(&self) -> String {
        format!(
            "📊 Keyword Index Stats:\n\
            🗣️  Conversations: {} keywords\n\
            🌙 Dreams: {} keywords\n\
            🎬 Co-watching: {} keywords\n\
            🎯 Interests: {} keywords\n\
            💫 Desires: {} keywords\n\
            🎨 Visual: {} keywords",
            self.conversation_index.len(),
            self.dreams_index.len(),
            self.cowatching_index.len(),
            self.interests_index.len(),
            self.desires_index.len(),
            self.visual_index.len()
        )
    }
}