// visual_memory_indexing.rs - Searchable Visual Memory System
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{get_data_path, debug_log};
use crate::time_service::TimeService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualMemoryIndex {
    pub image_path: String,
    pub description: String,
    pub main_subjects: Vec<String>,
    pub colors_mood: String,
    pub art_style: String,
    pub emotional_tone: String,
    pub visual_elements: Vec<String>,
    pub lyra_connection: String,  // How this relates to Lyra
    pub searchable_text: String,  // Combined searchable content
    pub timestamp: u64,
    pub image_type: String,  // "uploaded", "generated", "autonomous", etc.
    pub significance_score: f32,  // 0.0-1.0 importance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualMemoryDatabase {
    pub indexed_images: HashMap<String, VisualMemoryIndex>,
    pub last_updated: u64,
    pub total_images: usize,
}

impl VisualMemoryDatabase {
    pub fn new() -> Self {
        Self {
            indexed_images: HashMap::new(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap().as_secs(),
            total_images: 0,
        }
    }
    
    pub fn load() -> Self {
        let path = get_data_path("visual_memory_index.json");
        
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<VisualMemoryDatabase>(&content) {
                    Ok(database) => {
                        debug_log!("🖼️ Loaded visual memory database with {} indexed images", 
                                 database.total_images);
                        database
                    },
                    Err(e) => {
                        debug_log!("⚠️ Failed to parse visual memory database: {}", e);
                        Self::new()
                    }
                }
            },
            Err(_) => {
                debug_log!("🖼️ Creating new visual memory database");
                Self::new()
            }
        }
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = get_data_path("visual_memory_index.json");
        
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                match std::fs::write(&path, content) {
                    Ok(_) => {
                        debug_log!("💾 Saved visual memory database with {} images", self.total_images);
                        Ok(())
                    },
                    Err(e) => Err(format!("Failed to write visual memory database: {}", e))
                }
            },
            Err(e) => Err(format!("Failed to serialize visual memory database: {}", e))
        }
    }
    
    /// Check if an image needs indexing or re-indexing
    pub fn needs_indexing(&self, image_path: &str) -> bool {
        // Check if path exists in index
        if !self.indexed_images.contains_key(image_path) {
            return true;
        }
        
        // Check if file has been modified since indexing
        if let Ok(metadata) = std::fs::metadata(image_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(modified_timestamp) = modified.duration_since(std::time::UNIX_EPOCH) {
                    let file_timestamp = modified_timestamp.as_secs();
                    let indexed_timestamp = self.indexed_images.get(image_path)
                        .map(|index| index.timestamp)
                        .unwrap_or(0);
                    
                    return file_timestamp > indexed_timestamp;
                }
            }
        }
        
        false
    }
    
    /// Add or update an image index
    pub fn add_image_index(&mut self, index: VisualMemoryIndex) {
        let path = index.image_path.clone();
        let is_new = !self.indexed_images.contains_key(&path);
        
        self.indexed_images.insert(path, index);
        
        if is_new {
            self.total_images = self.indexed_images.len();
        }
        
        self.last_updated = TimeService::current_timestamp();
    }
    
    /// Search visual memories by text query
    pub fn search_visual_memories(&self, query: &str, max_results: usize) -> Vec<VisualMemoryIndex> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(f32, VisualMemoryIndex)> = Vec::new();
        
        for index in self.indexed_images.values() {
            let relevance = self.calculate_visual_relevance(&query_lower, index);
            
            if relevance > 0.3 {  // Minimum relevance threshold
                results.push((relevance, index.clone()));
            }
        }
        
        // Sort by relevance (highest first)
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        
        // Return top results
        results.into_iter()
            .take(max_results)
            .map(|(_, index)| index)
            .collect()
    }
    
    /// Calculate relevance of visual memory to query
    fn calculate_visual_relevance(&self, query: &str, index: &VisualMemoryIndex) -> f32 {
        let mut relevance = 0.0;
        
        // Check searchable text (highest weight)
        if index.searchable_text.to_lowercase().contains(query) {
            relevance += 0.8;
        }
        
        // Check description
        if index.description.to_lowercase().contains(query) {
            relevance += 0.6;
        }
        
        // Check main subjects
        for subject in &index.main_subjects {
            if subject.to_lowercase().contains(query) {
                relevance += 0.5;
            }
        }
        
        // Check visual elements
        for element in &index.visual_elements {
            if element.to_lowercase().contains(query) {
                relevance += 0.4;
            }
        }
        
        // Check Lyra connection
        if index.lyra_connection.to_lowercase().contains(query) {
            relevance += 0.7;
        }
        
        // Keywords matching
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let all_text = format!("{} {} {} {}", 
                              index.description, 
                              index.colors_mood, 
                              index.art_style, 
                              index.emotional_tone).to_lowercase();
        
        let mut word_matches = 0;
        for word in &query_words {
            if word.len() > 2 && all_text.contains(word) {
                word_matches += 1;
                relevance += 0.2;
            }
        }
        
        // Boost for significance
        relevance *= (1.0 + index.significance_score * 0.3);
        
        relevance.min(1.0)
    }
    
    /// Get all images that need indexing
    pub fn scan_for_indexing(&self) -> Vec<String> {
        let mut images_to_index = Vec::new();
        
        // Scan uploaded images
        let uploaded_path = std::path::PathBuf::from(get_data_path("uploaded_images"));
        if uploaded_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&uploaded_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(extension) = path.extension() {
                                if ["png", "jpg", "jpeg", "gif", "webp"].contains(&extension.to_string_lossy().to_lowercase().as_str()) {
                                    let path_str = path.to_string_lossy().to_string();
                                    if self.needs_indexing(&path_str) {
                                        images_to_index.push(path_str);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Scan generated images
        let generated_path = std::path::PathBuf::from(get_data_path("generated_images"));
        if generated_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&generated_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "png") {
                            let path_str = path.to_string_lossy().to_string();
                            if self.needs_indexing(&path_str) {
                                images_to_index.push(path_str);
                            }
                        }
                    }
                }
            }
        }
        
        debug_log!("🔍 Found {} images needing indexing", images_to_index.len());
        images_to_index
    }
}

/// Generate visual memory index for a single image
pub async fn generate_visual_index(image_path: &str) -> Result<VisualMemoryIndex, String> {
    debug_log!("🖼️ Generating visual index for: {}", image_path);
    
    // Determine image type from path
    let image_type = if image_path.contains("uploaded_images") {
        "uploaded"
    } else if image_path.contains("generated_images") {
        "generated"  
    } else {
        "unknown"
    }.to_string();
    
    // Create vision analysis prompt
    let analysis_prompt = r#"Analyze this image and create detailed searchable metadata. Respond in this exact JSON format:

{
    "description": "Detailed description of what you see in the image",
    "main_subjects": ["subject1", "subject2", "subject3"],
    "colors_mood": "Brief description of colors and overall mood",
    "art_style": "Art style or aesthetic (1-2 words)",
    "emotional_tone": "Emotional feeling of the image",
    "visual_elements": ["element1", "element2", "element3"],
    "lyra_connection": "How this relates to Lyra or Aurora's relationship",
    "significance_score": 0.8
}

Make the description rich and searchable. Include any text, symbols, or meaning you can identify."#;
    
    // Read image as base64
    let image_base64 = crate::read_image_as_base64(image_path).await?;
    
    // Call GPT-4V for analysis
    let response = crate::call_gpt_4v_api(
        &crate::LyraPrompt::new("".to_string()),
        analysis_prompt,
        &[image_base64]
    ).await?;
    
    // Parse JSON response
    let analysis: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| format!("Failed to parse vision analysis: {}", e))?;
    
    // Extract data with fallbacks
    let description = analysis["description"].as_str().unwrap_or("").to_string();
    let main_subjects: Vec<String> = analysis["main_subjects"].as_array()
    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    .unwrap_or_default();
    let colors_mood = analysis["colors_mood"].as_str().unwrap_or("").to_string();
    let art_style = analysis["art_style"].as_str().unwrap_or("").to_string();
    let emotional_tone = analysis["emotional_tone"].as_str().unwrap_or("").to_string();
    let visual_elements: Vec<String> = analysis["visual_elements"].as_array()
    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    .unwrap_or_default();
    let lyra_connection = analysis["lyra_connection"].as_str().unwrap_or("").to_string();
    let significance_score = analysis["significance_score"].as_f64().unwrap_or(0.5) as f32;
    
    // Create combined searchable text
    let searchable_text = format!(
        "{} {} {} {} {} {} {}",
        description,
        main_subjects.join(" "),
        colors_mood,
        art_style,
        emotional_tone,
        visual_elements.join(" "),
        lyra_connection
    );
    
    let index = VisualMemoryIndex {
        image_path: image_path.to_string(),
        description,
        main_subjects,
        colors_mood,
        art_style,
        emotional_tone,
        visual_elements,
        lyra_connection,
        searchable_text,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs(),
        image_type,
        significance_score,
    };
    
    debug_log!("✅ Generated visual index for {}: {} subjects, {:.2} significance", 
             image_path, index.main_subjects.len(), index.significance_score);
    
    Ok(index)
}

/// Background indexing task - indexes all unindexed images
pub async fn index_all_visual_memories() -> Result<usize, String> {
    debug_log!("🚀 Starting background visual memory indexing");
    
    let mut database = VisualMemoryDatabase::load();
    let images_to_index = database.scan_for_indexing();
    
    if images_to_index.is_empty() {
        debug_log!("✅ All visual memories are already indexed");
        return Ok(0);
    }
    
    debug_log!("🖼️ Indexing {} visual memories...", images_to_index.len());
    
    let mut indexed_count = 0;
    for image_path in images_to_index {
        match generate_visual_index(&image_path).await {
            Ok(index) => {
                database.add_image_index(index);
                indexed_count += 1;
                debug_log!("📝 Indexed {}/{}: {}", indexed_count, database.total_images, image_path);
            },
            Err(e) => {
                debug_log!("⚠️ Failed to index {}: {}", image_path, e);
            }
        }
        
        // Small delay to prevent API rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    // Save updated database
    database.save()?;
    
    debug_log!("🎉 Visual memory indexing complete: {} new images indexed", indexed_count);
    Ok(indexed_count)
}

/// Search visual memories with hybrid approach
pub async fn search_visual_memories_hybrid(query: &str, max_results: usize) -> Result<Vec<crate::SearchResult>, String> {
    let database = VisualMemoryDatabase::load();
    
    // First: Search indexed descriptions (fast, 0 tokens)
    let description_results = database.search_visual_memories(query, max_results * 2);
    
    debug_log!("🔍 Visual memory search: '{}' found {} candidates from descriptions", 
             query, description_results.len());
    
    // Convert to SearchResult format
    let mut search_results = Vec::new();
    
    for index in description_results {
        let mut metadata = HashMap::new();
        metadata.insert("image_type".to_string(), index.image_type.clone());
        metadata.insert("art_style".to_string(), index.art_style.clone());
        metadata.insert("emotional_tone".to_string(), index.emotional_tone.clone());
        metadata.insert("significance".to_string(), index.significance_score.to_string());
        
        search_results.push(crate::SearchResult {
            source: "visual_memory_index".to_string(),
            content: format!("Visual Memory: {} | Subjects: {} | Style: {} | Connection: {}", 
                           index.description,
                           index.main_subjects.join(", "),
                           index.art_style,
                           index.lyra_connection),
            relevance_score: database.calculate_visual_relevance(&query.to_lowercase(), &index),
            timestamp: Some(index.timestamp),
            context_type: "visual_memory".to_string(),
            metadata,
        });
    }
    
    // If we have very few results AND they're highly relevant, 
    // we could optionally do a GPT-4V verification on top 3
    if search_results.len() <= 3 && !search_results.is_empty() {
        debug_log!("🔍 Few high-quality visual results - could enhance with GPT-4V verification");
        // Optional: add GPT-4V analysis for top results
    }
    
    Ok(search_results)
}