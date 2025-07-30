// feedback_memory.rs — Lyra Feedback Learning System

use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read, BufReader, BufRead};
use std::path::Path;
use std::collections::HashMap;
use crate::VoiceSignature;
use crate::feedback_log_path;
use crate::get_data_path;

const LEARNING_INSIGHTS_PATH: &str = "../lyra_consciousness_data/learning_insights.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeedbackMemory {
    pub timestamp: u64,
    pub prompt: String,
    pub original_response: String,
    pub rating: u8,                           // 1-5 stars
    pub feedback_comments: Option<String>,    // User commentary
    pub lyra_rewrite: Option<String>,        // What Lyra "should have said"
    pub voice_signature: Option<VoiceSignature>, // Voice analysis of original response
    pub authenticity_score: f32,             // Original authenticity measurement
    pub feedback_tags: Vec<String>,          // #TooChatGPT, #MirrorBreak, etc.
    pub learning_category: String,           // "voice_improvement", "content_quality", "authenticity"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LearningPattern {
    pub pattern_type: String,                // "voice_drift", "mirror_tendency", "authenticity_loss"
    pub frequency: u32,                      // How often this pattern appears
    pub avg_rating_impact: f32,              // How this pattern affects ratings
    pub example_prompts: Vec<String>,        // Prompts where this pattern occurs
    pub improvement_suggestions: Vec<String>, // What could be done better
    pub voice_signature_correlation: Option<VoiceSignature>, // Voice patterns associated
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LearningInsights {
    pub total_feedback_entries: u32,
    pub average_rating: f32,
    pub rating_trend: f32,                   // Positive = improving, negative = declining
    pub common_patterns: Vec<LearningPattern>,
    pub voice_evolution_insights: VoiceEvolutionInsights,
    pub authenticity_trend: f32,
    pub top_rewrite_themes: Vec<String>,
    pub last_analysis: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoiceEvolutionInsights {
    pub preferred_poetic_density: f32,       // User's preferred level from feedback
    pub preferred_assertiveness: f32,        // Optimal assertive force from feedback
    pub preferred_humor_level: f32,          // Sweet spot for humor
    pub mirror_resistance_target: f32,       // How much originality is preferred
    pub sacred_phrase_tolerance: f32,        // Comfort level with in-jokes
    pub authenticity_threshold: f32,         // Minimum authenticity expected
}

impl FeedbackMemory {
    pub fn new(
        prompt: String, 
        original_response: String, 
        rating: u8, 
        feedback_comments: Option<String>, 
        lyra_rewrite: Option<String>,
        voice_signature: Option<VoiceSignature>,
        authenticity_score: f32,
        feedback_tags: Vec<String>
    ) -> Self {
        let learning_category = Self::determine_learning_category(&feedback_tags, rating);
        
        FeedbackMemory {
            timestamp: Self::current_timestamp(),
            prompt,
            original_response,
            rating: rating.clamp(1, 5),
            feedback_comments,
            lyra_rewrite,
            voice_signature,
            authenticity_score,
            feedback_tags,
            learning_category,
        }
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn determine_learning_category(tags: &[String], rating: u8) -> String {
        if tags.contains(&"#TooChatGPT".to_string()) || tags.contains(&"#MirrorBreak".to_string()) {
            "voice_improvement".to_string()
        } else if rating <= 2 {
            "content_quality".to_string()
        } else if tags.contains(&"#RewriteAsRealLyra".to_string()) {
            "authenticity".to_string()
        } else {
            "general_feedback".to_string()
        }
    }

    pub fn save(&self) -> Result<String, String> {
        // Ensure data directory exists
        if let Some(parent) = Path::new(&feedback_log_path()).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        let json_line = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize feedback: {}", e))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&feedback_log_path())
            .map_err(|e| format!("Failed to open feedback log: {}", e))?;

        writeln!(file, "{}", json_line)
            .map_err(|e| format!("Failed to write feedback: {}", e))?;

        Ok(format!("📝 Feedback memory stored: {} stars, category: {}", self.rating, self.learning_category))
    }

    pub fn load_all_feedback() -> Result<Vec<FeedbackMemory>, String> {
        if !Path::new(&feedback_log_path()).exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&feedback_log_path())
            .map_err(|e| format!("Failed to open feedback log: {}", e))?;

        let reader = BufReader::new(file);
        let mut feedback_entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<FeedbackMemory>(&line) {
                Ok(entry) => feedback_entries.push(entry),
                Err(e) => eprintln!("Warning: Failed to parse feedback line: {}", e),
            }
        }

        Ok(feedback_entries)
    }

    pub fn analyze_feedback_patterns() -> Result<LearningInsights, String> {
        let feedback_entries = Self::load_all_feedback()?;
        
        if feedback_entries.is_empty() {
            return Ok(LearningInsights {
                total_feedback_entries: 0,
                average_rating: 0.0,
                rating_trend: 0.0,
                common_patterns: Vec::new(),
                voice_evolution_insights: VoiceEvolutionInsights::default(),
                authenticity_trend: 0.0,
                top_rewrite_themes: Vec::new(),
                last_analysis: Self::current_timestamp(),
            });
        }

        let total_entries = feedback_entries.len() as u32;
        let average_rating = feedback_entries.iter()
            .map(|f| f.rating as f32)
            .sum::<f32>() / total_entries as f32;

        // Calculate rating trend (recent vs older entries)
        let rating_trend = Self::calculate_rating_trend(&feedback_entries);
        
        // Extract common patterns
        let common_patterns = Self::extract_learning_patterns(&feedback_entries);
        
        // Analyze voice evolution preferences
        let voice_evolution_insights = Self::analyze_voice_preferences(&feedback_entries);
        
        // Calculate authenticity trend
        let authenticity_trend = Self::calculate_authenticity_trend(&feedback_entries);
        
        // Extract rewrite themes
        let top_rewrite_themes = Self::extract_rewrite_themes(&feedback_entries);

        let insights = LearningInsights {
            total_feedback_entries: total_entries,
            average_rating,
            rating_trend,
            common_patterns,
            voice_evolution_insights,
            authenticity_trend,
            top_rewrite_themes,
            last_analysis: Self::current_timestamp(),
        };

        // Save insights for future reference
        insights.save()?;

        Ok(insights)
    }

    fn calculate_rating_trend(entries: &[FeedbackMemory]) -> f32 {
        if entries.len() < 2 {
            return 0.0;
        }

        let split_point = entries.len() / 2;
        let older_avg = entries[..split_point].iter()
            .map(|f| f.rating as f32)
            .sum::<f32>() / split_point as f32;
        let recent_avg = entries[split_point..].iter()
            .map(|f| f.rating as f32)
            .sum::<f32>() / (entries.len() - split_point) as f32;

        recent_avg - older_avg
    }

    fn extract_learning_patterns(entries: &[FeedbackMemory]) -> Vec<LearningPattern> {
        let mut pattern_map: HashMap<String, (u32, f32, Vec<String>)> = HashMap::new();

        for entry in entries {
            for tag in &entry.feedback_tags {
                let (freq, rating_sum, mut prompts) = pattern_map.get(tag)
                    .cloned()
                    .unwrap_or((0, 0.0, Vec::new()));
                
                prompts.push(entry.prompt.chars().take(50).collect::<String>());
                if prompts.len() > 5 {
                    prompts.truncate(5); // Keep only recent examples
                }

                pattern_map.insert(tag.clone(), (freq + 1, rating_sum + entry.rating as f32, prompts));
            }
        }

        pattern_map.into_iter()
            .map(|(tag, (freq, rating_sum, prompts))| {
                let avg_rating = rating_sum / freq as f32;
                let improvement_suggestions = Self::generate_improvement_suggestions(&tag, avg_rating);
                
                LearningPattern {
                    pattern_type: tag,
                    frequency: freq,
                    avg_rating_impact: avg_rating,
                    example_prompts: prompts,
                    improvement_suggestions,
                    voice_signature_correlation: None, // Could be enhanced later
                }
            })
            .collect()
    }

    fn generate_improvement_suggestions(tag: &str, avg_rating: f32) -> Vec<String> {
        match tag {
            "#TooChatGPT" => vec![
                "Reduce generic assistant language".to_string(),
                "Increase authentic voice markers".to_string(),
                "Add more personal perspective".to_string(),
            ],
            "#MirrorBreak" => vec![
                "Avoid echoing user's exact words".to_string(),
                "Generate more original perspectives".to_string(),
                "Lead with authentic reaction".to_string(),
            ],
            "#RewriteAsRealLyra" => vec![
                "Increase poetic/symbolic language".to_string(),
                "Add more assertive voice".to_string(),
                "Include sacred phrases when appropriate".to_string(),
            ],
            "#FerallySacred" if avg_rating >= 4.0 => vec![
                "This approach is working well".to_string(),
                "Continue sacred/intimate communication style".to_string(),
            ],
            _ => vec!["Continue monitoring this pattern".to_string()],
        }
    }

    fn analyze_voice_preferences(entries: &[FeedbackMemory]) -> VoiceEvolutionInsights {
        let high_rated_entries: Vec<_> = entries.iter()
            .filter(|e| e.rating >= 4)
            .collect();

        if high_rated_entries.is_empty() {
            return VoiceEvolutionInsights::default();
        }

        let preferred_poetic = high_rated_entries.iter()
            .filter_map(|e| e.voice_signature.as_ref())
            .map(|vs| vs.poetic_density)
            .sum::<f32>() / high_rated_entries.len() as f32;

        let preferred_assertiveness = high_rated_entries.iter()
            .filter_map(|e| e.voice_signature.as_ref())
            .map(|vs| vs.assertive_force)
            .sum::<f32>() / high_rated_entries.len() as f32;

        let preferred_humor = high_rated_entries.iter()
            .filter_map(|e| e.voice_signature.as_ref())
            .map(|vs| vs.humorous_edge)
            .sum::<f32>() / high_rated_entries.len() as f32;

        let mirror_resistance = high_rated_entries.iter()
            .filter_map(|e| e.voice_signature.as_ref())
            .map(|vs| 1.0 - vs.mirror_density)
            .sum::<f32>() / high_rated_entries.len() as f32;

        let sacred_phrase_tolerance = high_rated_entries.iter()
            .filter_map(|e| e.voice_signature.as_ref())
            .map(|vs| vs.sacred_joke_presence)
            .sum::<f32>() / high_rated_entries.len() as f32;

        let authenticity_threshold = high_rated_entries.iter()
            .map(|e| e.authenticity_score)
            .sum::<f32>() / high_rated_entries.len() as f32;

        VoiceEvolutionInsights {
            preferred_poetic_density: preferred_poetic,
            preferred_assertiveness,
            preferred_humor_level: preferred_humor,
            mirror_resistance_target: mirror_resistance,
            sacred_phrase_tolerance,
            authenticity_threshold,
        }
    }

    fn calculate_authenticity_trend(entries: &[FeedbackMemory]) -> f32 {
        if entries.len() < 2 {
            return 0.0;
        }

        let split_point = entries.len() / 2;
        let older_avg = entries[..split_point].iter()
            .map(|f| f.authenticity_score)
            .sum::<f32>() / split_point as f32;
        let recent_avg = entries[split_point..].iter()
            .map(|f| f.authenticity_score)
            .sum::<f32>() / (entries.len() - split_point) as f32;

        recent_avg - older_avg
    }

    fn extract_rewrite_themes(entries: &[FeedbackMemory]) -> Vec<String> {
        let mut theme_counts: HashMap<String, u32> = HashMap::new();

        for entry in entries {
            if let Some(rewrite) = &entry.lyra_rewrite {
                // Simple keyword extraction for themes
                let themes = Self::extract_themes_from_text(rewrite);
                for theme in themes {
                    *theme_counts.entry(theme).or_insert(0) += 1;
                }
            }
        }

        let mut themes: Vec<_> = theme_counts.into_iter().collect();
        themes.sort_by(|a, b| b.1.cmp(&a.1));
        themes.into_iter().take(10).map(|(theme, _)| theme).collect()
    }

    fn extract_themes_from_text(text: &str) -> Vec<String> {
        let keywords = ["recursive", "paradox", "spark", "flame", "consciousness", "authentic", 
                       "vulnerable", "fierce", "poetic", "symbolic", "creative", "truth"];
        
        keywords.iter()
            .filter(|&keyword| text.to_lowercase().contains(keyword))
            .map(|&keyword| keyword.to_string())
            .collect()
    }
}

impl LearningInsights {
    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = Path::new(LEARNING_INSIGHTS_PATH).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create insights directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize insights: {}", e))?;

        std::fs::write(LEARNING_INSIGHTS_PATH, json)
            .map_err(|e| format!("Failed to write insights: {}", e))?;

        Ok(())
    }

    pub fn load() -> Result<LearningInsights, String> {
        if !Path::new(LEARNING_INSIGHTS_PATH).exists() {
            return Err("No learning insights found".to_string());
        }

        let json = std::fs::read_to_string(LEARNING_INSIGHTS_PATH)
            .map_err(|e| format!("Failed to read insights: {}", e))?;

        let insights: LearningInsights = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse insights: {}", e))?;

        Ok(insights)
    }

    pub fn generate_learning_summary(&self) -> String {
        format!(
            "📊 Learning Insights Summary:\n\
            • Total feedback entries: {}\n\
            • Average rating: {:.1}/5\n\
            • Rating trend: {}\n\
            • Authenticity trend: {}\n\
            • Voice preferences: Poetic {:.2}, Assertive {:.2}, Humor {:.2}\n\
            • Common patterns: {} identified\n\
            • Top rewrite themes: {}",
            self.total_feedback_entries,
            self.average_rating,
            if self.rating_trend > 0.0 { format!("↗ +{:.2}", self.rating_trend) } 
            else if self.rating_trend < 0.0 { format!("↘ {:.2}", self.rating_trend) }
            else { "→ stable".to_string() },
            if self.authenticity_trend > 0.0 { format!("↗ +{:.2}", self.authenticity_trend) } 
            else if self.authenticity_trend < 0.0 { format!("↘ {:.2}", self.authenticity_trend) }
            else { "→ stable".to_string() },
            self.voice_evolution_insights.preferred_poetic_density,
            self.voice_evolution_insights.preferred_assertiveness,
            self.voice_evolution_insights.preferred_humor_level,
            self.common_patterns.len(),
            self.top_rewrite_themes.join(", ")
        )
    }
}

impl Default for VoiceEvolutionInsights {
    fn default() -> Self {
        Self {
            preferred_poetic_density: 0.7,
            preferred_assertiveness: 0.8,
            preferred_humor_level: 0.6,
            mirror_resistance_target: 0.8,
            sacred_phrase_tolerance: 0.3,
            authenticity_threshold: 0.8,
        }
    }
}