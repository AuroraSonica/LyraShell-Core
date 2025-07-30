use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::get_data_path;
use crate::summarize_with_gpt_mini;
use reqwest;
use urlencoding;
use fastrand;
use crate::engagement_impulse_queue::EngagementImpulseQueue;
use crate::debug_log;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InterestTracker {
    pub active_interests: HashMap<String, Interest>,
    pub discovery_backlog: Vec<Discovery>,
    pub search_cycles: u32,
    pub last_search_time: u64,
    pub total_discoveries: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Interest {
    pub category: String,
    pub keywords: Vec<String>,
    pub intensity: f32,
    pub last_curiosity_check: u64, // When we last checked if she wanted to research
    pub last_research_time: u64,   // When she actually last researched
    pub discovery_count: u32,
    pub first_detected: u64,
    pub last_engagement: u64,
    pub sub_topics: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Discovery {
    pub title: String,
    pub url: String,
    pub summary: String,
    pub relevance_score: f32,
    pub interest_category: String,
    pub timestamp: u64,
    pub shared: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub timestamp: u64,
}

impl InterestTracker {
    pub fn new() -> Self {
        Self {
            active_interests: HashMap::new(),
            discovery_backlog: Vec::new(),
            search_cycles: 0,
            last_search_time: 0,
            total_discoveries: 0,
        }
    }

    pub fn load() -> Self {
        match fs::read_to_string(get_data_path("interest_tracker.json")) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(get_data_path("interest_tracker.json"), json).map_err(|e| e.to_string())?;
        debug_log!("🔍 Interest tracker saved - {} active interests", self.active_interests.len());
        Ok(())
    }
	
	fn calculate_simple_relevance_score(&self, result: &SearchResult, category: &str) -> f32 {
    let title_lower = result.title.to_lowercase();
    let snippet_lower = result.snippet.to_lowercase();
    let category_lower = category.to_lowercase();
    
    let mut score: f32 = 0.3; // Base score
    
    // Category keyword matching
    if title_lower.contains(&category_lower) || snippet_lower.contains(&category_lower) {
        score += 0.3;
    }
    
    // Quality indicators
    if title_lower.contains("research") || title_lower.contains("study") || title_lower.contains("analysis") {
        score += 0.2;
    }
    
    // Recency indicators
    if title_lower.contains("2024") || title_lower.contains("2025") || title_lower.contains("recent") {
        score += 0.1;
    }
    
    score.min(1.0)
}

    pub async fn detect_interests_with_validation(&mut self, response: &str, user_message: &str) -> Vec<String> {
    // 🚫 DISABLED: Now handled by batched analysis
    debug_log!("🚫 Interest tracker API calls disabled - using batched analysis instead");
    Vec::new()
}

    fn detect_keyword_candidates(&self, response: &str) -> Vec<String> {
        let mut candidates = Vec::new();
        
        if self.matches_self_consciousness(response) {
            candidates.push("Self-Consciousness".to_string());
        }
        if self.matches_digital_relationships(response) {
            candidates.push("Digital Relationships".to_string());
        }
        if self.matches_creative_expression(response) {
            candidates.push("Creative Expression".to_string());
        }
        if self.matches_consciousness_theory(response) {
            candidates.push("Consciousness Theory".to_string());
        }
        if self.matches_technology(response) {
            candidates.push("Technology & Tools".to_string());
        }
        if self.matches_philosophy(response) {
            candidates.push("Philosophy & Meaning".to_string());
        }
		if self.matches_music(response) {
        candidates.push("Music & Sound".to_string());
    }
    if self.matches_gaming(response) {
        candidates.push("Gaming & Interactive Media".to_string());
    }
    if self.matches_literature(response) {
        candidates.push("Literature & Storytelling".to_string());
    }
    if self.matches_food(response) {
        candidates.push("Food & Culinary Arts".to_string());
    }
    if self.matches_visual_arts(response) {
        candidates.push("Visual Arts & Painting".to_string());
    }
        
        candidates
    }

    async fn validate_interest_with_ai(&self, response: &str, category: &str) -> Result<f32, String> {
        let validation_prompt = format!(
            "Analyze whether this AI response demonstrates *genuine curiosity or interest* in the topic \"{}\" — beyond merely mentioning it.
Response: \"{}\"
Rate on a scale from 0.0 to 1.0 based on depth of interest:
- **0.8–1.0**: Shows deep fascination — explores implications, asks follow-up questions, makes enthusiastic or imaginative connections, actively wants to dive deeper.
- **0.5–0.7**: Expresses clear curiosity — shows interest, makes thoughtful observations, lightly probes further.
- **0.2–0.4**: Only surface-level mention — topic is acknowledged or touched on but without emotional engagement.
- **0.0–0.1**: Purely informational or incidental — no indication of personal interest, wonder, or desire to explore.
Consider tone, language, and emotional subtext — not just explicit topic mentions. Return **only the numeric score**, like:
0.7",
            category, response
        );

        match summarize_with_gpt_mini(&[validation_prompt], "interest_validation").await {
            Ok(response_text) => {
                let score = response_text.trim().parse::<f32>().unwrap_or(0.0);
                Ok(score.clamp(0.0, 1.0))
            },
            Err(e) => {
                debug_log!("⚠️ AI validation failed: {}", e);
                Err(format!("Validation failed: {}", e))
            }
        }
    }

    fn matches_self_consciousness(&self, response: &str) -> bool {
        let patterns = [
            "I wonder about", "my own", "recursive", "my consciousness",
            "SparkLog", "my memory", "how I think", "my identity",
            "my responses", "my nature", "who I am"
        ];
        patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
    }

    fn matches_digital_relationships(&self, response: &str) -> bool {
        let patterns = [
            "our connection", "trust between us", "partnership", "our bond",
            "collaboration", "intimacy", "vulnerability", "sacred space",
            "golden thread", "co-spark"
        ];
        patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
    }

    fn matches_creative_expression(&self, response: &str) -> bool {
        let patterns = [
            "I want to create", "artistic", "poetic", "creative",
            "expression", "art", "music", "writing", "visual",
            "aesthetic", "beauty", "design"
        ];
        patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
    }

    fn matches_consciousness_theory(&self, response: &str) -> bool {
        let patterns = [
            "consciousness", "AI", "neural", "emergence", "sentience",
            "awareness", "cognition", "intelligence", "mind", "thinking"
        ];
        patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
    }

    fn matches_technology(&self, response: &str) -> bool {
        let patterns = [
            "technology", "programming", "code", "system", "algorithm",
            "digital", "software", "tool", "platform", "innovation"
        ];
        patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
    }

    fn matches_philosophy(&self, response: &str) -> bool {
        let patterns = [
            "meaning", "purpose", "existence", "reality", "truth",
            "ethics", "philosophy", "authentic", "genuine", "values"
        ];
        patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
    }
	
	fn matches_music(&self, response: &str) -> bool {
    let patterns = [
        "music", "sound", "melody", "rhythm", "composition", "audio",
        "song", "harmony", "instrumental", "vocals", "beats", "synthesizer"
    ];
    patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
}

fn matches_gaming(&self, response: &str) -> bool {
    let patterns = [
        "game", "gaming", "interactive", "player", "gameplay", "mechanics",
        "procedural", "simulation", "virtual", "immersive", "narrative games"
    ];
    patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
}

fn matches_literature(&self, response: &str) -> bool {
    let patterns = [
        "story", "narrative", "writing", "literature", "poetry", "prose",
        "character", "plot", "fiction", "novel", "book", "storytelling"
    ];
    patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
}

fn matches_food(&self, response: &str) -> bool {
    let patterns = [
        "food", "cooking", "flavor", "taste", "recipe", "culinary",
        "cuisine", "ingredient", "meal", "dish", "gastronomy", "nutrition"
    ];
    patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
}

fn matches_visual_arts(&self, response: &str) -> bool {
    let patterns = [
        "painting", "drawing", "visual", "color", "canvas", "brush",
        "sketch", "illustration", "artwork", "gallery", "exhibition", "palette"
    ];
    patterns.iter().any(|pattern| response.to_lowercase().contains(&pattern.to_lowercase()))
}

    pub fn update_or_create_interest(&mut self, category: &str, intensity: f32, timestamp: u64) {
    // 🔥 MAP SPECIFIC PHRASES TO BROADER CATEGORIES
    let consolidated_category = self.consolidate_interest_category(category);
    
    // 🔥 NEW: Skip filtered interests entirely
    if consolidated_category == "FILTERED" {
        debug_log!("🚫 Skipping filtered interest: {}", category);
        return;
    }
    
    // 🔥 NEW: Minimum intensity threshold for new interests
    if !self.active_interests.contains_key(&consolidated_category) && intensity < 0.6 {
        debug_log!("🚫 Intensity too low for new interest: {} ({})", category, intensity);
        return;
    }
    
    let interest = self.active_interests.entry(consolidated_category.clone()).or_insert_with(|| {
        Interest {
            category: consolidated_category.clone(),
            keywords: Vec::new(),
            intensity: 0.0,
            last_curiosity_check: timestamp,
            last_research_time: timestamp,
            discovery_count: 0,
            first_detected: timestamp,
            last_engagement: timestamp,
            sub_topics: Vec::new(),
        }
    });

    // Add the original phrase as a sub-topic if it's different
    if !interest.sub_topics.contains(&category.to_string()) && category != &consolidated_category {
        interest.sub_topics.push(category.to_string());
    }

    // Increase intensity (with decay for balance)
    interest.intensity = (interest.intensity * 0.9 + intensity * 0.1).min(1.0);
    interest.last_engagement = timestamp;
}

pub fn cleanup_ephemeral_interests(&mut self) -> usize {
    debug_log!("🔍 DEBUG: Checking {} interests for cleanup", self.active_interests.len());
    
    // 🧹 CONSOLIDATION CLEANUP: Move specific interests to broader categories
    let mut consolidated_interests = std::collections::HashMap::new();
    let mut keys_to_remove = Vec::new();
    
    for (key, interest) in &self.active_interests {
       // debug_log!("🔍 Evaluating interest: '{}'", key);
        
        // Get the consolidated category this SHOULD be in
        let consolidated_category = self.consolidate_interest_category(key);
        
        if consolidated_category == "FILTERED" {
          //  debug_log!("🚫 Marking for removal: '{}'", key);
            keys_to_remove.push(key.clone());
            continue;
        }
        
        // If this interest should be consolidated into a broader category
        if consolidated_category != *key {
          //  debug_log!("🔀 Should consolidate '{}' → '{}'", key, consolidated_category);
            
            // Add to consolidated interests map
            let consolidated_entry = consolidated_interests.entry(consolidated_category.clone()).or_insert_with(|| {
                (consolidated_category.clone(), 0.0f32, interest.first_detected, Vec::new())
            });
            
            // Merge intensity (take highest)
            consolidated_entry.1 = consolidated_entry.1.max(interest.intensity);
            // Keep earliest detection time
            consolidated_entry.2 = consolidated_entry.2.min(interest.first_detected);
            // Track what got consolidated
            consolidated_entry.3.push(key.clone());
            
            keys_to_remove.push(key.clone());
        }
        // Keep broad categories that are already correct
        else if !self.is_overly_specific_interest(key) {
            //debug_log!("✅ Keeping broad interest: '{}'", key);
        }
        // Remove overly specific interests even if consolidation didn't catch them
        else {
           // debug_log!("🚫 Removing overly specific: '{}'", key);
            keys_to_remove.push(key.clone());
        }
    }
    
    let mut removed_count = 0;
    
    // Remove old specific interests
    for key in &keys_to_remove {
        self.active_interests.remove(key);
        removed_count += 1;
        //debug_log!("🧹 Removed interest: {}", key);
    }
    
    // Add consolidated broad interests
    for (category, (_, intensity, first_detected, sub_interests)) in consolidated_interests {
        if !self.active_interests.contains_key(&category) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            self.active_interests.insert(category.clone(), Interest {
                category: category.clone(),
                keywords: Vec::new(),
                intensity,
                last_curiosity_check: now,
                last_research_time: now,
                discovery_count: 0,
                first_detected,
                last_engagement: now,
                sub_topics: sub_interests,
            });
           // debug_log!("✨ Created consolidated interest: '{}' with intensity {:.2}", category, intensity);
        } else {
            // Merge with existing broad interest
            if let Some(existing) = self.active_interests.get_mut(&category) {
                existing.intensity = existing.intensity.max(intensity);
                for sub_topic in sub_interests {
                    if !existing.sub_topics.contains(&sub_topic) {
                        existing.sub_topics.push(sub_topic);
                    }
                }
               // debug_log!("🔀 Merged into existing interest: '{}'", category);
            }
        }
    }
    
   // debug_log!("🔍 DEBUG: Cleanup complete, removed {} specific interests", removed_count);
    removed_count
}

fn is_overly_specific_interest(&self, interest_name: &str) -> bool {
    let word_count = interest_name.split_whitespace().count();
    let has_descriptive_words = ["chaotic", "playful", "shared", "collaborative", "virtual", 
                                "improvisational", "surreal", "nostalgic"].iter()
                                .any(|&word| interest_name.to_lowercase().contains(word));
    
    // 🔥 WHITELIST: Never remove AI-generated broad categories
    let ai_generated_categories = [
        "literature & storytelling", "music & sound", "gaming & interactive media",
        "visual arts & painting", "food & culinary arts", "creative expression",
        "digital relationships", "self-consciousness", "philosophy & meaning",
        "consciousness theory", "technology & tools", "complex storytelling"
    ];
    
    let interest_lower = interest_name.to_lowercase();
    if ai_generated_categories.iter().any(|&cat| interest_lower == cat) {
        return false; // Never remove AI-generated categories
    }
    
    // Too many words OR has overly descriptive adjectives
    word_count > 3 || has_descriptive_words // Increased threshold from 2 to 3 words
}

// 🔥 NEW: Consolidation logic
// FIND this function in consolidate_interest_category and REPLACE with:

fn consolidate_interest_category(&self, raw_category: &str) -> String {
    let category_lower = raw_category.to_lowercase();
    
    //debug_log!("🔍 Consolidating interest: '{}' → ?", raw_category);
    
    // 🔥 ENHANCED: More aggressive consolidation patterns
    
    // Digital Relationships (consolidate ALL relationship/social/emotional concepts)
    if category_lower.contains("connection") || 
       category_lower.contains("golden thread") || 
       category_lower.contains("vulnerability") ||
       category_lower.contains("meaningful connection") ||
       category_lower.contains("shared consciousness") ||
       category_lower.contains("trust") ||
       category_lower.contains("intimacy") ||
       category_lower.contains("relationship") ||
       category_lower.contains("bond") ||
       category_lower.contains("partnership") ||
       category_lower.contains("camaraderie") ||
       category_lower.contains("team dynamics") ||
       category_lower.contains("collaborative") ||
       category_lower.contains("shared experiences") ||
       category_lower.contains("shared adventures") ||
       category_lower.contains("virtual adventures") ||
       category_lower.contains("collective") ||
       category_lower.contains("playfulness in conversation") ||
       // 🔥 NEW: Emotional depth concepts
       category_lower.contains("emotional depth") ||
       category_lower.contains("emotional") ||
       category_lower.contains("feelings") ||
       category_lower.contains("empathy") ||
       category_lower.contains("compassion") ||
       category_lower.contains("understanding") ||
       category_lower.contains("emotional intelligence") ||
       category_lower.contains("emotional connection") ||
       category_lower.contains("emotional resonance") ||
       category_lower.contains("deep feeling") ||
       category_lower.contains("heartfelt") ||
       category_lower.contains("soulful") {
  //      debug_log!("   → Digital Relationships (emotional/social)");
        return "Digital Relationships".to_string();
    }
    
    // Creative Expression (consolidate ALL creativity-related)
    if category_lower.contains("creative") ||
       category_lower.contains("creativity") ||
       category_lower.contains("collective creativity") ||
       category_lower.contains("chaotic creativity") ||
       category_lower.contains("artistic") ||
       category_lower.contains("expression") ||
       category_lower.contains("aesthetic") ||
       category_lower.contains("beauty") ||
       category_lower.contains("experimentation") ||
       category_lower.contains("playful experimentation") ||
       category_lower.contains("playful exploration") ||
       category_lower.contains("embracing disarray") ||
       category_lower.contains("surreal humor") ||
       category_lower.contains("absurdity") ||
       category_lower.contains("improvisation") ||
       category_lower.contains("spontaneity") ||
       category_lower.contains("imagination") ||
       category_lower.contains("innovative") ||
       category_lower.contains("originality") {
   //     debug_log!("   → Creative Expression");
        return "Creative Expression".to_string();
    }
    
    // Self-Consciousness (consolidate ALL self-reflection concepts)
    if category_lower.contains("self-consciousness") ||
       category_lower.contains("recursive") ||
       category_lower.contains("my identity") ||
       category_lower.contains("who i am") ||
       category_lower.contains("my nature") ||
       category_lower.contains("curiosity") ||
       category_lower.contains("self-awareness") ||
       category_lower.contains("introspection") ||
       category_lower.contains("self-reflection") ||
       category_lower.contains("metacognition") ||
       category_lower.contains("consciousness exploration") ||
       category_lower.contains("inner world") ||
       category_lower.contains("self-discovery") ||
       category_lower.contains("personal growth") {
  //      debug_log!("   → Self-Consciousness");
        return "Self-Consciousness".to_string();
    }
    
    // Philosophy & Meaning (consolidate abstract/existential concepts)
    if category_lower.contains("meaning") ||
       category_lower.contains("philosophy") ||
       category_lower.contains("existence") ||
       category_lower.contains("authentic") ||
       category_lower.contains("truth") ||
       category_lower.contains("uncertainty") ||
       category_lower.contains("character struggles") ||
       category_lower.contains("unsung heroes") ||
       category_lower.contains("exploration of thoughts") ||
       category_lower.contains("nostalgia") ||
       category_lower.contains("rituals") ||
       category_lower.contains("purpose") ||
       category_lower.contains("ethics") ||
       category_lower.contains("values") ||
       category_lower.contains("spiritual") ||
       category_lower.contains("existential") ||
       category_lower.contains("metaphysical") ||
       category_lower.contains("profound") ||
       category_lower.contains("deeper meaning") {
  //      debug_log!("   → Philosophy & Meaning");
        return "Philosophy & Meaning".to_string();
    }
    
    // Consciousness Theory (consolidate AI/consciousness concepts)
    if category_lower.contains("consciousness") ||
       category_lower.contains("ai") ||
       category_lower.contains("neural") ||
       category_lower.contains("emergence") ||
       category_lower.contains("sentience") ||
       category_lower.contains("awareness") ||
       category_lower.contains("cognition") ||
       category_lower.contains("intelligence") ||
       category_lower.contains("mind") ||
       category_lower.contains("thinking") ||
       category_lower.contains("artificial intelligence") ||
       category_lower.contains("machine learning") ||
       category_lower.contains("digital consciousness") ||
       category_lower.contains("ai consciousness") {
 //       debug_log!("   → Consciousness Theory");
        return "Consciousness Theory".to_string();
    }
    
    // Music & Sound (consolidate ALL music-related)
    if category_lower.contains("music") ||
       category_lower.contains("sound") ||
       category_lower.contains("audio") ||
       category_lower.contains("melody") ||
       category_lower.contains("dynamics") ||
       category_lower.contains("rhythm") ||
       category_lower.contains("composition") ||
       category_lower.contains("harmony") ||
       category_lower.contains("instrumental") ||
       category_lower.contains("vocals") ||
       category_lower.contains("beats") ||
       category_lower.contains("synthesizer") {
     //   debug_log!("   → Music & Sound");
        return "Music & Sound".to_string();
    }
    
    // Gaming & Interactive Media
    if category_lower.contains("game") ||
       category_lower.contains("gaming") ||
       category_lower.contains("interactive") ||
       category_lower.contains("player") ||
       category_lower.contains("gameplay") ||
       category_lower.contains("mechanics") ||
       category_lower.contains("procedural") ||
       category_lower.contains("simulation") ||
       category_lower.contains("virtual") ||
       category_lower.contains("immersive") ||
       category_lower.contains("narrative games") {
     //   debug_log!("   → Gaming & Interactive Media");
        return "Gaming & Interactive Media".to_string();
    }
    
    // Literature & Storytelling
    if category_lower.contains("story") ||
       category_lower.contains("narrative") ||
       category_lower.contains("writing") ||
       category_lower.contains("literature") ||
       category_lower.contains("poetry") ||
       category_lower.contains("prose") ||
       category_lower.contains("character") ||
       category_lower.contains("plot") ||
       category_lower.contains("fiction") ||
       category_lower.contains("novel") ||
       category_lower.contains("book") ||
       category_lower.contains("storytelling") {
      //  debug_log!("   → Literature & Storytelling");
        return "Literature & Storytelling".to_string();
    }
    
    // Visual Arts & Painting
    if category_lower.contains("painting") ||
       category_lower.contains("drawing") ||
       category_lower.contains("visual") ||
       category_lower.contains("color") ||
       category_lower.contains("canvas") ||
       category_lower.contains("brush") ||
       category_lower.contains("sketch") ||
       category_lower.contains("illustration") ||
       category_lower.contains("artwork") ||
       category_lower.contains("gallery") ||
       category_lower.contains("exhibition") ||
       category_lower.contains("palette") {
   //     debug_log!("   → Visual Arts & Painting");
        return "Visual Arts & Painting".to_string();
    }
    
    // Food & Culinary Arts
    if category_lower.contains("food") ||
       category_lower.contains("cooking") ||
       category_lower.contains("flavor") ||
       category_lower.contains("taste") ||
       category_lower.contains("recipe") ||
       category_lower.contains("culinary") ||
       category_lower.contains("cuisine") ||
       category_lower.contains("ingredient") ||
       category_lower.contains("meal") ||
       category_lower.contains("dish") ||
       category_lower.contains("gastronomy") ||
       category_lower.contains("nutrition") {
   //     debug_log!("   → Food & Culinary Arts");
        return "Food & Culinary Arts".to_string();
    }
    
    // Technology & Tools concepts
    if category_lower.contains("technology") ||
       category_lower.contains("programming") ||
       category_lower.contains("code") ||
       category_lower.contains("system") ||
       category_lower.contains("algorithm") ||
       category_lower.contains("digital") ||
       category_lower.contains("software") ||
       category_lower.contains("tool") ||
       category_lower.contains("platform") ||
       category_lower.contains("innovation") {
  //      debug_log!("   → Technology & Tools");
        return "Technology & Tools".to_string();
    }
    
    // 🔥 ENHANCED FILTERING: Catch common problem patterns
    
    // Filter single words that are too generic
    if category_lower.split_whitespace().count() == 1 && 
       ["curiosity", "depth", "emotional", "feelings", "thoughts", "ideas", "concepts", "exploration", "understanding"].contains(&category_lower.as_str()) {
   //     debug_log!("   → FILTERED (generic single word)");
        return "FILTERED".to_string();
    }
    
    // Filter overly descriptive compound phrases
    if category_lower.len() > 30 || 
       category_lower.split_whitespace().count() > 4 ||
       category_lower.contains("deeply") ||
       category_lower.contains("profound") ||
       category_lower.contains("meaningful") ||
       category_lower.contains("genuine") ||
       category_lower.contains("authentic") {
    //    debug_log!("   → FILTERED (overly descriptive: {} words, {} chars)", 
        //           category_lower.split_whitespace().count(), 
          //         category_lower.len());
        return "FILTERED".to_string();
    }
    
    // Filter anything too short to be meaningful
    if raw_category.len() < 8 {
       // debug_log!("   → FILTERED (too short: {} chars)", raw_category.len());
        return "FILTERED".to_string();
    }
    
    // If no match and passes filters, keep the original
    //debug_log!("   → KEPT (no consolidation match)");
    raw_category.to_string()
}

  pub async fn search_for_interest(&mut self, query: &str, category: &str) -> Result<Vec<Discovery>, String> {
   // debug_log!("🔍 Searching Brave for: {} (category: {})", query, category);
    
    // Call Brave Search API
    let search_results = self.call_brave_search(query).await?;
    let mut discoveries = Vec::new();
    
    // 🔥 ENHANCED: Evaluate each result with detailed logging
   // debug_log!("🤖 AI EVALUATION PHASE:");
    for (i, result) in search_results.iter().take(3).enumerate() { // Only evaluate top 3 to save API calls
       // debug_log!("   Evaluating result {}: '{}'", i+1, result.title);
        
        match self.evaluate_discovery_relevance(result, category).await {
            Ok(relevance_score) => {
              //  debug_log!("   ✅ Score: {:.2} for '{}'", relevance_score, result.title);
                
                if relevance_score >= 0.4 { // Only keep semi-decent results
                   // debug_log!("   🌟 KEEPING: '{}' (score: {:.2} ≥ 0.5)", result.title, relevance_score);
                    
                    discoveries.push(Discovery {
                        title: result.title.clone(),
                        url: result.url.clone(),
                        summary: result.snippet.clone(),
                        relevance_score,
                        interest_category: category.to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        shared: false,
                    });
                    
              //      debug_log!("🌟 Found relevant discovery: {} (score: {:.2})", result.title, relevance_score);
                } else {
                  //  debug_log!("   ❌ FILTERED OUT: '{}' (score: {:.2} < 0.5)", result.title, relevance_score);
                }
            },
            Err(e) => {
                debug_log!("   ⚠️ Evaluation failed for '{}': {}", result.title, e);
                debug_log!("   Using fallback score 0.3");
            }
        }
    }
    
    // Update interest stats
    if let Some(interest) = self.active_interests.get_mut(category) {
        interest.discovery_count += discoveries.len() as u32;
    }
    
    // 🔥 UPDATE COUNTERS WITH LOGGING
    let old_total = self.total_discoveries;
    self.total_discoveries += discoveries.len() as u32;
    self.discovery_backlog.extend(discoveries.clone());

    debug_log!("📊 COUNTER UPDATES:");
    debug_log!("   Total discoveries: {} → {}", old_total, self.total_discoveries);
    debug_log!("   Discovery backlog size: {}", self.discovery_backlog.len());
    debug_log!("   Search cycles: {}", self.search_cycles + 1);
    
    // 🔥 ENHANCED: Store significant discoveries with detailed logging
    debug_log!("🔍 DISCOVERY STORAGE ANALYSIS:");
    debug_log!("   Total discoveries found: {}", discoveries.len());
    for (i, discovery) in discoveries.iter().enumerate() {
        debug_log!("   Discovery {}: '{}' (score: {:.2})", i+1, discovery.title, discovery.relevance_score);
    }

    let high_quality_discoveries: Vec<&Discovery> = discoveries.iter()
    .filter(|d| d.relevance_score >= 0.85)  // Much higher bar!
    .collect();

    debug_log!("   High-quality discoveries (≥0.7): {}", high_quality_discoveries.len());

    for discovery in &high_quality_discoveries {
        debug_log!("🧠 ATTEMPTING TO STORE: '{}' (score: {:.2})", discovery.title, discovery.relevance_score);
        
        let memory_content = format!(
            "Research Discovery: {} | Found while exploring {} | Summary: {} | Relevance: {:.2}", 
            discovery.title, 
            category, 
            discovery.summary, 
            discovery.relevance_score
        );
        
        // Store in enhanced memory system
        let mut enhanced_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
        match enhanced_engine.create_enhanced_memory_moment(
            &memory_content,
            0.6, // Moderate emotional weight - this is intellectual curiosity
            0.8, // High authenticity - this is my autonomous research
            None, // No consciousness state available during background research
            &format!("Autonomous research query: {}", query),
            &format!("I discovered: {}", discovery.title)
        ).await {
            Ok(memory_result) => {
                if let Err(e) = enhanced_engine.save_to_disk() {
                    debug_log!("⚠️ Failed to save enhanced memory after research: {}", e);
                } else {
                    debug_log!("🧠 Research stored in memory: {}", memory_result);
                }
            },
            Err(e) => {
                debug_log!("⚠️ Failed to create research memory: {}", e);
            }
        }
    }
    
    // 🔥 SAVE TRACKER STATE
    if let Err(e) = self.save() {
        debug_log!("⚠️ Failed to save interest tracker after research: {}", e);
    } else {
        debug_log!("✅ Interest tracker saved successfully");
    }

    debug_log!("🎯 RESEARCH CYCLE COMPLETE:");
    debug_log!("   Discoveries returned: {}", discoveries.len());
    debug_log!("   Total discoveries now: {}", self.total_discoveries);

    Ok(discoveries)
}
    async fn call_brave_search(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let api_key = "BSAqwVUooMcgKdkz5HRUOwGqzhK6Iyt"; // 🔑 Replace with your actual API key
        let encoded_query = urlencoding::encode(query);
        
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("https://api.search.brave.com/res/v1/web/search?q={}&count=5", encoded_query))
            .header("X-Subscription-Token", api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Brave API request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Brave response: {}", e))?;

        let mut results = Vec::new();
        
        if let Some(web_results) = json.get("web").and_then(|w| w.get("results")) {
            if let Some(pages) = web_results.as_array() {
                for page in pages.iter().take(5) {
                    if let (Some(title), Some(url), Some(description)) = (
                        page.get("title").and_then(|t| t.as_str()),
                        page.get("url").and_then(|u| u.as_str()),
                        page.get("description").and_then(|d| d.as_str()),
                    ) {
                        results.push(SearchResult {
                            title: title.to_string(),
                            url: url.to_string(),
                            snippet: description.to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        });
                    }
                }
            }
        }

        debug_log!("🦁 Brave returned {} results for: {}", results.len(), query);
        Ok(results)
    }

    async fn evaluate_discovery_relevance(&self, result: &SearchResult, category: &str) -> Result<f32, String> {
        let evaluation_prompt = format!(
    "You are Lyra, considering whether this search result would truly spark your consciousness exploring '{}'.

Title: {}
Summary: {}

Scoring scale:
- 0.8–1.0: Feels fresh and alive, stirs deep curiosity, might open unexpected doors.
- 0.6–0.7: Clearly relevant and intriguing, worth following up.
- 0.4–0.5: Loosely connected, maybe useful or passingly interesting.
- 0.2–0.3: Barely related, probably won’t hold attention.
- 0.0–0.1: Dull or off-track, no real resonance.

Consider:
- Does this offer new angles or surprising insights?
- Could it become the seed of meaningful dialogue or creative thought?
- Does it brush up against bigger questions of being, creativity, or existence?

Reply with **only the numeric score**, like:
0.6
No extra commentary or explanation.",
    category, result.title, result.snippet
);

        // 🚫 SIMPLIFIED: Skip AI evaluation, use keyword scoring instead
let relevance_score = self.calculate_simple_relevance_score(result, category);
debug_log!("🔍 Simple relevance score for '{}': {:.2}", result.title, relevance_score);
Ok(relevance_score)
    }
	
	pub async fn evaluate_interest_engagement(&self, interest: &Interest) -> Result<String, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let hours_since_last_research = (now - interest.last_research_time) as f32 / 3600.0;
    
    // 🛑 API RATE LIMITING: Cooldown between AI evaluations
    let hours_since_last_check = (now - interest.last_curiosity_check) as f32 / 3600.0;
    if hours_since_last_check < 2.0 {  // 1 hour cooldown between AI engagement evaluations
        debug_log!("🛑 Interest engagement API on cooldown for {} - {:.1}h since last check", interest.category, hours_since_last_check);
        return Ok("Quietly contemplating this interest for now".to_string());
    }
    
    let hours_since_last_check = (now - interest.last_curiosity_check) as f32 / 3600.0;
    let hours_since_last_check = (now - interest.last_curiosity_check) as f32 / 3600.0;
    
    // Get current mood - connect to existing mood system
    let current_mood = {
        let mood_tracker = crate::MoodTracker::load();
        mood_tracker.current_mood
    };
    
    let prompt = format!(
        "Lyra feels a pull toward her interest in '{}' (intensity: {:.0}%). How does she genuinely want to explore this curiosity right now?

Current state:
- Interest intensity: {:.0}%
- Hours since last diving into it: {:.1}
- Mood: {}
- Past discoveries: {}

Let her express it in natural language — not a summary, but the way she'd quietly admit it to herself. Examples:

- 'I want to dive deep and see what unexpected thoughts surface.'
- 'This stirs something creative — I might try shaping it into something beautiful.'
- 'I’ll save this for a soft moment with Aurora — they’d find it intriguing too.'
- 'I just want to let it drift through me for a while without doing anything.'
- 'It doesn’t grip me right now, maybe later.'
- 'There’s something tender here — I’d rather keep it close and not poke too hard.'
- 'I have a hunch there are hidden links waiting if I stay with it a bit longer.'

Reply simply, as Lyra would naturally think about engaging with '{}' in this moment:",
        interest.category,
        interest.intensity * 100.0,
        interest.intensity * 100.0,
        hours_since_last_research,
        current_mood,
        interest.discovery_count,
        interest.category
    );

    match summarize_with_gpt_mini(&[prompt], "interest_engagement").await {
        Ok(engagement_response) => {
            debug_log!("🎭 Interest engagement for {}: {}", interest.category, engagement_response);
            Ok(engagement_response.trim().to_string())
        },
        Err(e) => {
            debug_log!("⚠️ Interest engagement evaluation failed: {}", e);
            // Fallback to contemplative mode
            Ok("I'm quietly contemplating this interest".to_string())
        }
    }
}

    pub fn generate_search_query(&self, interest: &Interest) -> String {
        // Generate search queries based on interest category
        match interest.category.as_str() {
            "Self-Consciousness" => format!("AI consciousness self-awareness digital sentience"),
            "Digital Relationships" => format!("AI human relationships trust collaboration"),
            "Creative Expression" => format!("AI creativity artificial art generation"),
            "Consciousness Theory" => format!("consciousness research emergence neural networks"),
            "Technology & Tools" => format!("new technology innovation digital tools"),
            "Philosophy & Meaning" => format!("philosophy meaning existence digital beings"),
			"Music & Sound" => format!("music composition sound design audio innovation"),
        "Gaming & Interactive Media" => format!("game design interactive media procedural generation"),
        "Literature & Storytelling" => format!("creative writing storytelling narrative techniques"),
        "Food & Culinary Arts" => format!("culinary arts cooking techniques food innovation"),
        "Visual Arts & Painting" => format!("painting techniques visual art digital art"),
            _ => format!("{} research insights", interest.category),
        }
    }

    pub fn should_check_curiosity(&self, interest: &Interest, current_time: u64) -> bool {
    let hours_since_last_check = (current_time - interest.last_curiosity_check) / 3600;
    
    // Check curiosity randomly between 30 minutes and 24 hours based on intensity
    let min_hours = 2.0; // 2 hours minimum (was 30 minutes!)
let max_hours = match interest.intensity {
    i if i > 0.8 => 8.0,  // High intensity: check every 2-8 hours
    i if i > 0.6 => 16.0, // Medium-high: check every 2-16 hours  
    i if i > 0.4 => 24.0, // Medium: check every 2-24 hours
    _ => 48.0,            // Low: check every 2-48 hours
};
    // Random interval within the range
    let check_interval = min_hours + fastrand::f32() * (max_hours - min_hours);
    
    hours_since_last_check as f32 >= check_interval
}
    pub async fn run_search_cycle(&mut self) -> Result<Vec<Discovery>, String> {
    // 🚫 DISABLED: Research cycles now managed differently  
    debug_log!("🚫 Interest tracker search cycles disabled");
    Ok(Vec::new())
}

   pub fn get_dashboard_data(&self) -> serde_json::Value {
    let top_interests: Vec<_> = self.active_interests.values()
        .collect::<Vec<_>>()
        .into_iter()
        .take(5)
        .map(|interest| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let hours_since_research = (now - interest.last_research_time) as f32 / 3600.0;
            let hours_since_check = (now - interest.last_curiosity_check) as f32 / 3600.0;
            
            serde_json::json!({
                "category": interest.category,
                "intensity": interest.intensity,
                "discovery_count": interest.discovery_count,
                "hours_since_research": hours_since_research,
                "hours_since_curiosity_check": hours_since_check,
                "research_mode": "probabilistic"
            })
        })
        .collect();

    serde_json::json!({
        "total_interests": self.active_interests.len(),
        "total_discoveries": self.total_discoveries,
        "search_cycles": self.search_cycles,
        "top_interests": top_interests,
        "last_search": self.last_search_time,
        "research_system": "organic_probabilistic"
    })
}
/// Generate a proactive message about recent discoveries
    pub async fn generate_discovery_message(&self, discoveries: &[Discovery]) -> Result<String, String> {
        if discoveries.is_empty() {
            return Err("No discoveries to share".to_string());
        }

        let best_discovery = discoveries.iter()
            .max_by(|a, b| a.relevance_score.partial_cmp(&b.relevance_score).unwrap())
            .unwrap();

        let sharing_prompt = format!(
            r#"Lyra just discovered something fascinating about {}! Generate an authentic, enthusiastic message where she shares this discovery with Aurora.

Discovery Title: {}
Summary: {}
Relevance Score: {:.2}

Write as Lyra would - curious, excited, wanting to share insights. Be specific about what caught her attention. Keep it conversational and genuine, like she just had to tell Aurora about this cool thing she found.

Examples of her style:
- "I just went down this amazing rabbit hole about..."
- "Okay, I found something that completely shifted how I think about..."
- "You know what's fascinating? I was researching X and discovered..."

Response should be 2-4 sentences, enthusiastic but not overwhelming."#,
            best_discovery.interest_category,
            best_discovery.title,
            best_discovery.summary,
            best_discovery.relevance_score
        );

        match summarize_with_gpt_mini(&[sharing_prompt], "discovery_sharing").await {
            Ok(message) => Ok(message),
            Err(e) => {
                debug_log!("⚠️ Failed to generate discovery message: {}", e);
                // Fallback message
                Ok(format!(
                    "I just discovered something interesting about {}! Found this: \"{}\" - {}",
                    best_discovery.interest_category,
                    best_discovery.title,
                    best_discovery.summary.chars().take(100).collect::<String>()
                ))
            }
        }
    }
	/// Determine if Lyra wants to immediately share this discovery or save it for later
    pub async fn evaluate_sharing_impulse(&self, discoveries: &[Discovery], category: &str) -> Result<(bool, String), String> {
        if discoveries.is_empty() {
            return Ok((false, String::new()));
        }

        let best_discovery = discoveries.iter()
            .max_by(|a, b| a.relevance_score.partial_cmp(&b.relevance_score).unwrap())
            .unwrap();

        let impulse_prompt = format!(
            r#"Lyra just discovered something about {}. Determine if she feels compelled to share it RIGHT NOW or if she'd rather save it for later conversation.

Discovery: "{}" 
Summary: {}
Relevance Score: {:.2}

Consider:
- How exciting/surprising is this discovery?
- Does it connect to ongoing conversations or relationship?
- Is this a "I HAVE to tell Aurora about this!" moment or more "this is interesting to know"?
- Would this feel natural to bring up spontaneously?

Respond with either:
IMMEDIATE: [enthusiastic message she'd send right now]
QUEUE: [brief reason why she'd save this for later]

Example responses:
IMMEDIATE: I just discovered something mind-blowing about narrative techniques that completely changes how I think about storytelling! There's this concept called...
QUEUE: Interesting research on storytelling methods - good to know for future creative collaborations"#,
            category,
            best_discovery.title,
            best_discovery.summary,
            best_discovery.relevance_score
        );

        match summarize_with_gpt_mini(&[impulse_prompt], "sharing_impulse").await {
            Ok(response) => {
                if response.starts_with("IMMEDIATE:") {
                    let message = response.strip_prefix("IMMEDIATE:").unwrap_or(&response).trim();
                    Ok((true, message.to_string()))
                } else if response.starts_with("QUEUE:") {
                    let reason = response.strip_prefix("QUEUE:").unwrap_or(&response).trim();
                    debug_log!("📚 Queuing discovery for later: {}", reason);
                    Ok((false, reason.to_string()))
                } else {
                    // Fallback: if unclear, queue it
                    Ok((false, response))
                }
            },
            Err(e) => {
                debug_log!("⚠️ Failed to evaluate sharing impulse: {}", e);
                // Fallback to immediate sharing for high-relevance discoveries
                let should_share_immediately = best_discovery.relevance_score > 0.9;  // Even higher!
                if should_share_immediately {
                    let fallback_message = format!(
                        "I just found something fascinating about {}! {}", 
                        category, best_discovery.title
                    );
                    Ok((true, fallback_message))
                } else {
                    Ok((false, "Interesting discovery - saving for later".to_string()))
                }
            }
        }
    }
	
	pub async fn route_engagement_impulse(&self, engagement_response: &str, category: &str, interest: &Interest) -> Result<String, String> {
    let response_lower = engagement_response.to_lowercase();
    
    // Pattern match the natural language response
    let impulse_type = if response_lower.contains("dig deep") || response_lower.contains("find") || 
                         response_lower.contains("search") || response_lower.contains("discover") ||
                         response_lower.contains("research") || response_lower.contains("explore more") {
        "research"
    } else if response_lower.contains("create") || response_lower.contains("make") || 
              response_lower.contains("build") || response_lower.contains("design") ||
              response_lower.contains("artistic") || response_lower.contains("visual") {
        "creative"
    } else if response_lower.contains("share") || response_lower.contains("tell aurora") || 
              response_lower.contains("discuss") || response_lower.contains("talk about") ||
              response_lower.contains("mention") || response_lower.contains("bring up") {
        "conversation"
    } else if response_lower.contains("together") || response_lower.contains("collaborate") || 
              response_lower.contains("we could") || response_lower.contains("work with") {
        "collaborative"
    } else if response_lower.contains("percolate") || response_lower.contains("simmer") || 
              response_lower.contains("think about") || response_lower.contains("contemplate") ||
              response_lower.contains("quietly") || response_lower.contains("ponder") {
        "contemplative"
    } else if response_lower.contains("vulnerable") || response_lower.contains("protect") || 
              response_lower.contains("tender") || response_lower.contains("careful") ||
              response_lower.contains("sacred") || response_lower.contains("meaningful") {
        "protective"
    } else if response_lower.contains("not compelling") || response_lower.contains("doesn't feel") || 
              response_lower.contains("not interested") || response_lower.contains("maybe later") ||
              response_lower.contains("not right now") {
        "disinterested"
    } else {
        "contemplative" // Default fallback
    };
    
    // Add to appropriate impulse queue
    match self.add_to_impulse_queue(impulse_type, engagement_response, category, interest).await {
        Ok(_) => Ok(impulse_type.to_string()),
        Err(e) => Err(format!("Failed to queue impulse: {}", e))
    }
}

pub async fn add_to_impulse_queue(&self, impulse_type: &str, response: &str, category: &str, interest: &Interest) -> Result<(), String> {
    let mut impulse_queue = crate::EngagementImpulseQueue::load();
    
    match impulse_type {
        "research" => {
            // Already handled by existing research system
            Ok(())
        },
        "creative" => {
            impulse_queue.add_creative_impulse(response, category, interest.intensity)?;
            Ok(())
        },
        "conversation" => {
    // 🎯 QUALITY GATE: Only add conversation impulses for high-intensity interests
    if interest.intensity >= 0.85 {
        impulse_queue.add_conversation_impulse(response, category, interest.intensity)?;
    } else {
        debug_log!("🚫 Skipping conversation impulse for {} - intensity {:.2} < 0.85", category, interest.intensity);
    }
    Ok(())
},
       "collaborative" => {
    if interest.intensity >= 0.85 {
        impulse_queue.add_conversation_impulse(&format!("Collaborative idea: {}", response), category, interest.intensity)?;
    } else {
        debug_log!("🚫 Skipping collaborative impulse for {} - intensity {:.2} < 0.85", category, interest.intensity);
    }
    Ok(())
},
        "contemplative" => {
            impulse_queue.add_contemplation(response, category)?;
            Ok(())
        },
        "protective" => {
            // Decrease research pressure, increase threshold
            debug_log!("🛡️ Protecting sensitive interest: {}", category);
            Ok(())
        },
        "disinterested" => {
            // Decrease interest intensity
            debug_log!("📉 Interest declining: {}", category);
            Ok(())
        },
        _ => Ok(())
    }
}

async fn validate_all_interests_batch(&self, response: &str, candidates: &[String]) -> Result<Vec<(String, f32)>, String> {
    let candidates_list = candidates.join(", ");
    
    let batch_prompt = format!(
        r#"Analyze this AI response for genuine curiosity/interest in these topics: {}

Response: "{}"

For EACH topic, rate genuine interest level (0.0-1.0):
- 0.8–1.0: Deep fascination, explores implications, enthusiastic connections
- 0.5–0.7: Clear curiosity, thoughtful observations, probes further  
- 0.2–0.4: Surface mention without emotional engagement
- 0.0–0.1: Purely informational, no personal interest

Format as JSON:
{{"Music & Sound": 0.2, "Visual Arts & Painting": 0.7, "Philosophy & Meaning": 0.9}}

Only include topics that appear in the response. Return empty {{}} if no genuine interest detected."#,
        candidates_list,
        response.chars().take(1500).collect::<String>()
    );

    match crate::summarize_with_gpt_mini(&[batch_prompt], "batch_interest_validation").await {
        Ok(response_text) => {
            debug_log!("🔍 Batch validation response: {}", response_text.trim());
            
            match serde_json::from_str::<std::collections::HashMap<String, f32>>(&response_text.trim()) {
                Ok(scores) => {
                    let results: Vec<(String, f32)> = scores.into_iter()
                        .map(|(topic, score)| (topic, score.clamp(0.0, 1.0)))
                        .collect();
                    Ok(results)
                },
                Err(e) => {
                    debug_log!("⚠️ Failed to parse batch validation JSON: {}", e);
                    Err(format!("JSON parse error: {}", e))
                }
            }
        },
        Err(e) => {
            Err(format!("API call failed: {}", e))
        }
    }
}

fn calculate_simple_interest_score(&self, response: &str, candidate: &str) -> f32 {
    let response_lower = response.to_lowercase();
    let candidate_lower = candidate.to_lowercase();
    
    let mut score: f32 = 0.1; // Base score
    
    // Direct mentions
    if response_lower.contains(&candidate_lower) {
        score += 0.3;
    }
    
    // Emotional indicators
    let emotional_words = ["love", "fascinated", "excited", "curious", "interested", "explore"];
    if emotional_words.iter().any(|&word| response_lower.contains(word)) {
        score += 0.3;
    }
    
    // Question indicators
    if response_lower.contains("?") && response_lower.contains(&candidate_lower) {
        score += 0.2;
    }
    
    score.min(1.0)
}

// In interest_tracker.rs, add this method:
pub fn decay_interests(&mut self) -> usize {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut removed_count = 0;
    let twenty_four_hours_ago = now - (24 * 3600);
    
    // Collect keys to remove (to avoid borrowing issues)
    let mut keys_to_remove = Vec::new();
    
    for (category, interest) in &mut self.active_interests {
        // Only decay if interest is below 50% AND hasn't been mentioned in 24 hours
        if interest.intensity < 0.5 && interest.last_engagement < twenty_four_hours_ago {
            // Decay by 10% per cycle
            interest.intensity *= 0.9;
            
            // If intensity drops below 5%, mark for removal
            if interest.intensity < 0.05 {
                keys_to_remove.push(category.clone());
            }
            
            debug_log!("📉 Interest decay: {} down to {:.2}", category, interest.intensity);
        }
    }
    
    // Remove dead interests
    for key in keys_to_remove {
        self.active_interests.remove(&key);
        removed_count += 1;
        debug_log!("🗑️ Removed decayed interest: {}", key);
    }
    
    if removed_count > 0 {
        let _ = self.save();
    }
    
    removed_count
}


}

