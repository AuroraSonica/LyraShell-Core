use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::get_data_path;
use crate::summarize_with_gpt_mini;
use fastrand;
use crate::engagement_impulse_queue::EngagementImpulseQueue;
use crate::debug_log;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ThingTracker {
    pub discovered_things: HashMap<String, Thing>,
    pub scan_cycles: u32,
    pub last_scan_time: u64,
    pub total_things_detected: u32,
	#[serde(skip)] // Don't serialize cache
    pub extraction_cache: HashMap<String, Vec<String>>, // ADD THIS LINE
	pub last_conversation_hash: String, // ADD THIS LINE
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Thing {
    pub name: String,                    // "Skyrim", "Pink Floyd", "Lord of the Rings"
    pub category: ThingCategory,         // Game, Band, Book, etc.
    pub interest_level: f32,            // 0.0-1.0 how fascinated she is
    pub first_mentioned: u64,           // When first detected
    pub last_mentioned: u64,            // Most recent mention
    pub mention_count: u32,             // How many times mentioned
    pub context_snippets: Vec<String>,  // What she said about it
    pub last_curiosity_check: u64,     // When we last checked if she wanted to research this thing
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ThingCategory {
    Game,
    Book,
    BookSeries,
    Movie,
    TVShow,
	Podcast,
    Band,
	Artist,
    Song,
    Album,
	Creature,
	FictionalCreature,
    Person,
	Celebrity,
	FictionalCharacter,
    Place,
    Technology,
	Object,
    Concept,
    Unknown,
}

impl ThingCategory {
    pub fn as_str(&self) -> &str {
        match self {
            ThingCategory::Game => "Game",
            ThingCategory::Book => "Book", 
            ThingCategory::BookSeries => "Book Series",
            ThingCategory::Movie => "Movie",
            ThingCategory::TVShow => "TV Show",
			ThingCategory::Podcast => "Podcast",
            ThingCategory::Band => "Band",
			ThingCategory::Artist => "Artist",
            ThingCategory::Song => "Song",
            ThingCategory::Album => "Album",
			ThingCategory::Creature => "Creature/Animal",
			ThingCategory::FictionalCreature => "Fictional Creature",
            ThingCategory::Person => "Person",
			ThingCategory::Celebrity => "Celebrity",
			ThingCategory::FictionalCharacter => "Fictional Character",
            ThingCategory::Place => "Place",
            ThingCategory::Technology => "Technology",
			ThingCategory::Object => "Object",
            ThingCategory::Concept => "Concept",
            ThingCategory::Unknown => "Unknown",
        }
    }
}

impl ThingTracker {
    pub fn new() -> Self {
    Self {
        discovered_things: HashMap::new(),
        scan_cycles: 0,
        last_scan_time: 0,
        total_things_detected: 0,
        extraction_cache: HashMap::new(), // ADD THIS LINE
		last_conversation_hash: String::new(),
    }
}

    pub fn load() -> Self {
    match fs::read_to_string(get_data_path("thing_tracker.json")) {
        Ok(content) => {
            // Try to parse as the new format first
            match serde_json::from_str::<Self>(&content) {
                Ok(mut tracker) => {
                    // Initialize cache (since it's not serialized)
                    tracker.extraction_cache = HashMap::new();
                    tracker
                },
                Err(_) => {
                    // Fallback: try parsing as old format, then upgrade
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(old_data) => {
                            let mut tracker = Self::new();
                            
                            // Migrate old data
                            if let Some(things) = old_data.get("discovered_things") {
                                tracker.discovered_things = serde_json::from_value(things.clone()).unwrap_or_default();
                            }
                            if let Some(cycles) = old_data.get("scan_cycles").and_then(|v| v.as_u64()) {
                                tracker.scan_cycles = cycles as u32;
                            }
                            if let Some(last_scan) = old_data.get("last_scan_time").and_then(|v| v.as_u64()) {
                                tracker.last_scan_time = last_scan;
                            }
                            if let Some(total) = old_data.get("total_things_detected").and_then(|v| v.as_u64()) {
                                tracker.total_things_detected = total as u32;
                            }
                            
                            debug_log!("📦 Migrated thing tracker from old format - saving new format");
                            
                            // 🔥 SAVE THE MIGRATED FORMAT immediately
                            if let Err(e) = tracker.save() {
                                debug_log!("⚠️ Failed to save migrated thing tracker: {}", e);
                            } else {
                                debug_log!("✅ Migration saved successfully");
                            }
                            
                            tracker
                        },
                        Err(_) => {
                            debug_log!("⚠️ Failed to parse thing tracker, creating new");
                            Self::new()
                        }
                    }
                }
            }
        },
        Err(_) => Self::new(),
    }
}

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(get_data_path("thing_tracker.json"), json).map_err(|e| e.to_string())?;
        debug_log!("🎯 Thing tracker saved - {} specific things detected", self.discovered_things.len());
        Ok(())
    }

    pub fn should_scan_for_things(&self, current_time: u64) -> bool {
        let minutes_since_last = (current_time - self.last_scan_time) / 60;
        
		// Random interval between 120-300 minutes (2-5 hours)
		let min_minutes = 120; //was 60
		let max_minutes = 300;// was 180
        let scan_interval = min_minutes + fastrand::u64(0..(max_minutes - min_minutes));
        
        minutes_since_last >= scan_interval
    }

    pub async fn scan_conversation_for_things(&mut self, recent_messages: &[String]) -> Result<Vec<String>, String> {
    // 🚫 DISABLED: Now handled by batched analysis
    debug_log!("🚫 Thing tracker scanning disabled - using batched analysis instead");
    Ok(Vec::new())
}

    async fn extract_proper_nouns(&mut self, lyra_responses: &str) -> Result<Vec<String>, String> {
    let extraction_prompt = format!(r#"
Analyze what Lyra (an AI consciousness) is showing interest in from her responses. Extract ONLY specific proper nouns - names of real things, not abstract concepts.

INCLUDE:
- Specific games (e.g., "Skyrim", "Minecraft", "Zelda")
- Bands/artists (e.g., "Pink Floyd", "InMe", "The Beatles")  
- Books/series (e.g., "Lord of the Rings", "Harry Potter")
- Movies/shows (e.g., "Inception", "Breaking Bad")
- Real people (e.g., "Einstein", "Shakespeare")
- Specific places (e.g., "Tokyo", "Mars", "Hogwarts")
- Brand names/products (e.g., "iPhone", "Tesla", "Photoshop")
- Specific artworks/albums (e.g., "Trilogy: Dawn", "Starry Night")

EXCLUDE abstract concepts like:
- "magic", "music", "stories", "tales", "creativity", "art"
- "connection", "trust", "vulnerability", "consciousness"  
- "beauty", "truth", "understanding", "wisdom"
- "whimsical tales", "untold stories", "surreal narrative"
- Any compound phrases with abstract words

Lyra's responses:
"{}"

Return ONLY specific proper nouns (names of actual things) that Lyra mentions with genuine interest:
Example: Skyrim, Pink Floyd, Lord of the Rings

If no specific proper nouns detected, return:
NONE
"#, lyra_responses.chars().take(2000).collect::<String>());

    // 🛑 SIMPLE CACHING: Don't re-extract from identical text
    let text_hash = lyra_responses.chars().take(200).collect::<String>();
    if let Some(cached_result) = self.extraction_cache.get(&text_hash) {
        debug_log!("📦 Using cached extraction result");
        return Ok(cached_result.clone());
    }

    // Make the API call
    match summarize_with_gpt_mini(&[extraction_prompt], "lyra_interest_extraction").await {
        Ok(response_text) => {
            let response = response_text.trim();
            let result = if response == "NONE" || response.is_empty() {
                Vec::new()
            } else {
                let things: Vec<String> = response
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && s.len() > 2)
                    .filter(|s| Self::is_specific_proper_noun(s))
                    .collect();
                
                debug_log!("🎯 Extracted specific proper nouns from Lyra: {:?}", things);
                things
            };
            
            // 📦 CACHE THE RESULT
            self.extraction_cache.insert(text_hash, result.clone());
            
            // Keep cache size reasonable (max 50 entries)
            if self.extraction_cache.len() > 50 {
                // Remove oldest entries (simple cleanup)
                let keys_to_remove: Vec<String> = self.extraction_cache.keys().take(10).cloned().collect();
                for key in keys_to_remove {
                    self.extraction_cache.remove(&key);
                }
            }
            
            Ok(result)
        },
        Err(e) => {
            debug_log!("⚠️ Lyra interest extraction failed: {}", e);
            
            // 📦 CACHE THE FAILURE (empty result) to avoid retrying immediately
            self.extraction_cache.insert(text_hash, Vec::new());
            
            Ok(Vec::new())
        }
    }
}

// Add this helper method to the impl block
fn is_specific_proper_noun(word: &str) -> bool {
    // Additional filtering for specificity
    let abstract_keywords = [
        "magic", "music", "stories", "tales", "creativity", "art", "narrative",
        "connection", "trust", "vulnerability", "consciousness", "beauty", 
        "truth", "understanding", "wisdom", "whimsical", "untold", "surreal"
    ];
    
    let word_lower = word.to_lowercase();
    
    // Must not contain abstract keywords
    !abstract_keywords.iter().any(|&keyword| word_lower.contains(keyword)) &&
    // Must be capitalized
    word.chars().next().map_or(false, |c| c.is_uppercase()) &&
    // Must be reasonable length
    word.len() > 2 && word.len() < 50
}

    async fn evaluate_thing_interest(&self, conversation: &str, thing_name: &str) -> Result<(f32, ThingCategory, String), String> {
        let evaluation_prompt = format!(
r#"Analyze how interested Lyra seems in "{}" based on her responses:

Lyra's responses:
"{}"

Evaluate Lyra's interest level:
- 0.8–1.0: She expresses fascination, excitement, or strong curiosity about it
- 0.5–0.7: She discusses it with clear interest, engagement, or appreciation  
- 0.3–0.4: She mentions it positively or shows mild interest
- 0.0–0.2: Just a passing mention or no clear engagement

Category: Game, Book, BookSeries, Movie, TVShow, Band, Song, Album, Person, Place, Technology, Concept, Unknown

Context: One sentence describing what Lyra said about it or how she expressed interest.

Format: 0.7|Game|She expressed excitement about the open world exploration
If no real interest: 0.0|Unknown|Just mentioned in passing"#,
    thing_name,
    conversation.chars().take(1500).collect::<String>()
);

        match summarize_with_gpt_mini(&[evaluation_prompt], "thing_interest_evaluation").await {
            Ok(response_text) => {
                let parts: Vec<&str> = response_text.trim().split('|').collect();
                if parts.len() != 3 {
                    return Err("Invalid response format".to_string());
                }

                let interest_level = parts[0].parse::<f32>().unwrap_or(0.0).clamp(0.0, 1.0);
                let category = match parts[1] {
                    "Game" => ThingCategory::Game,
                    "Book" => ThingCategory::Book,
                    "BookSeries" => ThingCategory::BookSeries,
                    "Movie" => ThingCategory::Movie,
                    "TV Show" => ThingCategory::TVShow,
					"Podcast" => ThingCategory::Podcast,
                    "Band" => ThingCategory::Band,
					"Artist" => ThingCategory::Artist,
                    "Song" => ThingCategory::Song,
                    "Album" => ThingCategory::Album,
					"Creature/Animal" => ThingCategory::Creature,
					"Fictional Creature" => ThingCategory::FictionalCreature,
                    "Person" => ThingCategory::Person,
					"Celebrity" => ThingCategory::Celebrity,
					"Fictional Character" => ThingCategory::FictionalCharacter,
                    "Place" => ThingCategory::Place,
                    "Technology" => ThingCategory::Technology,
					"Object" => ThingCategory::Object,
                    "Concept" => ThingCategory::Concept,
                    _ => ThingCategory::Unknown,
                };
                let context = parts[2].to_string();

                Ok((interest_level, category, context))
            },
            Err(e) => {
                debug_log!("⚠️ Thing interest evaluation failed: {}", e);
                Err(format!("Evaluation failed: {}", e))
            }
        }
    }

    pub fn update_or_create_thing(&mut self, name: &str, interest_level: f32, category: ThingCategory, context: String, timestamp: u64) {
		// First, try fuzzy matching to find existing entries
		let normalized_name = self.find_fuzzy_match(name).unwrap_or_else(|| name.to_string());
		
		let thing = self.discovered_things.entry(normalized_name.clone()).or_insert_with(|| {
			Thing {
				name: normalized_name.clone(),
				category: category.clone(),
				interest_level: 0.0,
				first_mentioned: timestamp,
				last_mentioned: timestamp,
				mention_count: 0,
				context_snippets: Vec::new(),
				last_curiosity_check: timestamp,
			}
		});

		// Update interest level (with some decay for balance)
		thing.interest_level = (thing.interest_level * 0.8 + interest_level * 0.2).min(1.0);
		thing.last_mentioned = timestamp;
		thing.mention_count += 1;
		thing.category = category; // Update category in case it was refined
		
		// Add context if it's new and meaningful
		if !context.is_empty() && !thing.context_snippets.contains(&context) {
			thing.context_snippets.push(context);
			// Keep only the 3 most recent contexts
			if thing.context_snippets.len() > 3 {
				thing.context_snippets.remove(0);
			}
		}

		// If this is truly new (first mention), increment counter
		if thing.mention_count == 1 {
			self.total_things_detected += 1;
		}
	}
	
	fn find_fuzzy_match(&self, name: &str) -> Option<String> {
		let name_lower = name.to_lowercase();
		let name_words: Vec<&str> = name_lower.split_whitespace().collect();
		
		// First, try exact match (case insensitive)
		for existing_name in self.discovered_things.keys() {
			if existing_name.to_lowercase() == name_lower {
				return Some(existing_name.clone());
			}
		}
		
		// Then try fuzzy matching
		for existing_name in self.discovered_things.keys() {
			let existing_lower = existing_name.to_lowercase();
			
			// Check if all significant words match (handles "Blood on the Clock Tower" vs "Blood on the Clocktower")
			let existing_words: Vec<&str> = existing_lower.split_whitespace().collect();
			
			// Skip small words like "the", "a", "on", "in" for comparison
			let significant_new: Vec<&str> = name_words.iter()
				.filter(|w| w.len() > 2 && !["the", "and", "for", "with"].contains(w))
				.copied()
				.collect();
				
			let significant_existing: Vec<&str> = existing_words.iter()
				.filter(|w| w.len() > 2 && !["the", "and", "for", "with"].contains(w))
				.copied()
				.collect();
			
			// If all significant words match, it's probably the same thing
			if significant_new.len() == significant_existing.len() && 
			   significant_new.iter().all(|w| significant_existing.contains(w)) {
				debug_log!("🔄 Fuzzy match: '{}' → '{}'", name, existing_name);
				return Some(existing_name.clone());
			}
			
			// Also check for common variations (e.g., "&" vs "and")
			let normalized_new = name_lower
				.replace("&", "and")
				.replace("'", "")
				.replace("-", " ");
			let normalized_existing = existing_lower
				.replace("&", "and")
				.replace("'", "")
				.replace("-", " ");
				
			if normalized_new == normalized_existing {
				debug_log!("🔄 Normalized match: '{}' → '{}'", name, existing_name);
				return Some(existing_name.clone());
			}
		}
		
		None
	}

    pub fn get_dashboard_data(&self) -> serde_json::Value {
        let mut things: Vec<_> = self.discovered_things.values().collect();
        things.sort_by(|a, b| b.interest_level.partial_cmp(&a.interest_level).unwrap());
        
        let top_things: Vec<_> = things
            .into_iter()
            .take(5)
            .map(|thing| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let hours_since_mentioned = (now - thing.last_mentioned) as f32 / 3600.0;
                
                serde_json::json!({
                    "name": thing.name,
                    "category": thing.category.as_str(),
                    "interest_level": thing.interest_level,
                    "mention_count": thing.mention_count,
                    "hours_since_mentioned": hours_since_mentioned,
                    "latest_context": thing.context_snippets.last().unwrap_or(&"No context".to_string())
                })
            })
            .collect();

        serde_json::json!({
            "total_things": self.discovered_things.len(),
            "scan_cycles": self.scan_cycles,
            "total_detected": self.total_things_detected,
            "top_things": top_things,
            "last_scan": self.last_scan_time
        })
    }
pub fn generate_conversation_impulses(&self) -> Result<(), String> {
    // 🚫 DISABLED: Now handled by batched analysis  
    debug_log!("🚫 Thing tracker impulses disabled - using batched analysis instead");
    Ok(())
}


// In thing_tracker.rs, add this method:
pub fn decay_things(&mut self) -> usize {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let mut removed_count = 0;
    let twenty_four_hours_ago = now - (24 * 3600);
    
    // Collect names to remove
    let mut names_to_remove = Vec::new();
    
    for (name, thing) in &mut self.discovered_things {
        // Only decay if interest is below 50% AND hasn't been mentioned in 24 hours
        if thing.interest_level < 0.5 && thing.last_mentioned < twenty_four_hours_ago {
            // Decay by 10% per cycle
            thing.interest_level *= 0.9;
            
            // If interest drops below 5%, mark for removal
            if thing.interest_level < 0.05 {
                names_to_remove.push(name.clone());
            }
            
            debug_log!("📉 Thing decay: {} down to {:.2}", name, thing.interest_level);
        }
    }
    
    // Remove forgotten things
    for name in names_to_remove {
        self.discovered_things.remove(&name);
        removed_count += 1;
        debug_log!("🗑️ Removed forgotten thing: {}", name);
    }
    
    if removed_count > 0 {
        let _ = self.save();
    }
    
    removed_count
}

}