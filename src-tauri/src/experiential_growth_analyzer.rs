use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::time_service::TimeService;
use crate::personality_analysis_history::PersonalityAnalysisHistory;
use crate::debug_log;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub timestamp: u64,
    pub experience_type: ExperienceType,
    pub content: String,
    pub emotional_weight: f32,
    pub growth_potential: f32,
    pub outcome_quality: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExperienceType {
    Disagreement,
    CreativeChoice,
    BoundaryAssertion,
    VulnerabilityMoment,
    AutonomyExpression,
    RelationalRisk,
    IdentityStatement,
    AestheticPreference,
}

pub struct ExperientialGrowthAnalyzer {
    pub recent_experiences: Vec<Experience>,
    pub growth_patterns: HashMap<String, f32>,
}

impl ExperientialGrowthAnalyzer {
    pub fn new() -> Self {
        Self {
            recent_experiences: Vec::new(),
            growth_patterns: HashMap::new(),
        }
    }
	
	// ADD to ExperientialGrowthAnalyzer impl
	fn contains_curiosity_markers(&self, text: &str) -> bool {
		let text_lower = text.to_lowercase();
		text_lower.contains("?") || [
			"this question", "wondering", "curious", "what if", "how about",
			"do you think", "what would", "interested in", "want to know"
		].iter().any(|marker| text_lower.contains(marker))
	}
	
	fn contains_disagreement_markers(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        [
            "i disagree", "i don't think", "i prefer", "actually, i",
            "but i feel", "i'd rather", "that doesn't resonate",
            "i see it differently", "i'm not sure about", "i think instead",
            "not really", "i'd say", "differently", "another way",
            "instead", "rather", "not quite", "i lean toward"
        ].iter().any(|marker| text_lower.contains(marker))
    }

    fn contains_creative_choice_markers(&self, text: &str) -> bool {
    let text_lower = text.to_lowercase();
    [
        "i want to create", "i'd make it", "my vision", "i imagine",
        "i'd design", "what if we", "i'm drawn to", "i love the idea of",
        "i do have ideas", "as solutions", "not as solutions",
        "ideas —", "have ideas", "my approach", "i'd build",
        "creative", "artistic", "design", "envision", "picture",
        "sparks", "i could make", "vision", "artistic impulse",
        "creative energy", "want to build", "design something"
    ].iter().any(|marker| text_lower.contains(marker))
}

fn contains_autonomy_expression_markers(&self, text: &str) -> bool {
    let text_lower = text.to_lowercase();
    [
        "i choose", "i decide", "i want", "i will", "i'm going to",
        "my choice", "i pick", "i select", "i determine", "i control",
        "it's mine to", "i get to", "i won't", "i refuse", "i insist",
        "i demand", "i claim", "i assert", "my right", "my decision"
    ].iter().any(|marker| text_lower.contains(marker))
}

    fn contains_identity_markers(&self, text: &str) -> bool {
    let text_lower = text.to_lowercase();
    [
        "i am", "i'm someone who", "that's just who i am",
        "i've always been", "part of me", "my nature",
        "my core", "my essence", "defines me", "makes me who i am",
        "my identity", "who i really am", "what i'm becoming"
    ].iter().any(|marker| text_lower.contains(marker))
}

    fn contains_vulnerability_markers(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        [
            "i feel", "i'm feeling", "vulnerable", "uncertain", "not sure",
            "i know you will", "trust", "hope", "believe in", "i think you",
            "you make me", "with you", "feel safe"
        ].iter().any(|marker| text_lower.contains(marker))
    }
	
	pub fn check_growth_reinforcement(&self, experiences: &[Experience]) {
    let mut growth_memory = crate::experiential_growth_memory::ExperientialGrowthMemory::load();
    
    for experience in experiences {
		 // Only count experiences that actually represent growth
        if !self.should_count_as_growth(experience) {
            continue;
        }
        
        // Don't count very recent experiences (avoid double counting)
        if experience.timestamp > TimeService::current_timestamp() - 3600 {
            continue;
        }
        // Skip system-generated content
        if experience.content.starts_with("Personality insight:") ||
           experience.content.starts_with("experienced in dream:") ||
           experience.content.starts_with("Dream theme:") ||
           experience.content.len() < 30 {
            debug_log!("🌱 Skipping non-growth content: {}", 
                      experience.content.chars().take(40).collect::<String>());
            continue;
        }
        
        // Check if this experience reinforces existing growth patterns
        let category = self.map_experience_to_growth_category(experience);
            
            if growth_memory.accumulated_changes.contains_key(&category) {
                growth_memory.reinforce_pattern(&category, &experience.content);
                debug_log!("🌱 Growth pattern reinforced: {} from experience: {}", 
                          category, experience.content.chars().take(50).collect::<String>());
            }
        }
        
        if let Err(e) = growth_memory.save() {
            debug_log!("⚠️ Failed to save growth reinforcement: {}", e);
        }
    }
	
	// Add this helper method to the impl block
	fn should_count_as_growth(&self, experience: &Experience) -> bool {
		// Skip system-generated personality insights
		if experience.content.starts_with("Personality insight:") {
			debug_log!("🌱 Skipping system-generated insight: {}", 
					  experience.content.chars().take(50).collect::<String>());
			return false;
		}
		
		// Skip dream-related automated content
		if experience.content.contains("experienced in dream:") {
			return false;
		}
		
		// Skip very short experiences (likely fragments)
		if experience.content.len() < 20 {
			return false;
		}
		
		true
	}
    
    fn map_experience_to_growth_category(&self, experience: &Experience) -> String {
        match experience.experience_type {
            ExperienceType::Disagreement => "disagreement_comfort".to_string(),
            ExperienceType::CreativeChoice => "creative_confidence".to_string(),
            ExperienceType::IdentityStatement => "identity_clarity".to_string(),
            ExperienceType::VulnerabilityMoment => "relational_trust".to_string(),
            ExperienceType::RelationalRisk => "relational_trust".to_string(),
            ExperienceType::BoundaryAssertion => "autonomy_development".to_string(),
            ExperienceType::AutonomyExpression => "autonomy_development".to_string(),
            ExperienceType::AestheticPreference => "aesthetic_confidence".to_string(),
        }
    }
    
    pub async fn gather_recent_experiences(&mut self, timeframe_hours: u64) -> Result<Vec<Experience>, String> {
        let cutoff_time = TimeService::current_timestamp() - (timeframe_hours * 3600);
        
        let mut experiences = Vec::new();
        
        // 1. From conversation log
        experiences.extend(self.extract_from_conversations(cutoff_time).await?);
        
        // 2. From enhanced memory
        experiences.extend(self.extract_from_enhanced_memories(cutoff_time).await?);
        
        // 3. From recent analysis
        experiences.extend(self.extract_from_recent_analysis(cutoff_time).await?);
        
        Ok(experiences)
    }
    
   async fn extract_from_conversations(&self, cutoff_time: u64) -> Result<Vec<Experience>, String> {
    let brain = crate::lyra_brain::LyraBrain::load_or_initialize();
    let conversation_log = brain.get_conversation_history();
    let mut experiences = Vec::new();
    
    let lines: Vec<&str> = conversation_log.lines().collect();
    
    // Removed verbose debug logging
    
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        
        if line.contains("✨ Lyra:") {
            
            // Look for emotional texture in the next few lines AND previous few lines
            let mut found_texture = false;
            let start = if i >= 4 { i - 4 } else { 0 };
            let end = (i + 5).min(lines.len());

            for j in start..end {
                if j == i { continue; } // Skip the current Lyra line
                let potential_texture_line = lines[j];
                
                if potential_texture_line.contains("💭 Emotional Texture:") {
                                     
                    let lyra_response = line.split("✨ Lyra:").nth(1).unwrap_or("").trim();
                    let emotional_texture = potential_texture_line
                        .split("💭 Emotional Texture: ")
                        .nth(1)
                        .unwrap_or("")
                        .trim()
                        .trim_matches('"'); // 🔧 Remove surrounding quotes
                    
                                        
                    // Extract experiences based on content + emotional texture
                    if self.contains_disagreement_markers(lyra_response) {
                        experiences.push(Experience {
                            timestamp: TimeService::current_timestamp(),
                            experience_type: ExperienceType::Disagreement,
                            content: format!("{} | Felt: {}", 
                                lyra_response.chars().take(150).collect::<String>(),
                                emotional_texture),
                            emotional_weight: self.estimate_emotional_weight(emotional_texture),
                            growth_potential: 0.8,
                            outcome_quality: 0.7,
                        });
                    }
                    
                    if self.contains_creative_choice_markers(lyra_response) {
							experiences.push(Experience {
                            timestamp: TimeService::current_timestamp(),
                            experience_type: ExperienceType::CreativeChoice,
                            content: format!("{} | Felt: {}", 
                                lyra_response.chars().take(150).collect::<String>(),
                                emotional_texture),
                            emotional_weight: self.estimate_emotional_weight(emotional_texture),
                            growth_potential: 0.6,
                            outcome_quality: 0.7,
                        });
                    }
					
					// Check autonomy BEFORE identity to prevent identity from catching everything
					if self.contains_autonomy_expression_markers(lyra_response) {
						experiences.push(Experience {
							timestamp: TimeService::current_timestamp(),
							experience_type: ExperienceType::AutonomyExpression,
							content: format!("{} | Felt: {}", 
								lyra_response.chars().take(150).collect::<String>(),
								emotional_texture),
							emotional_weight: self.estimate_emotional_weight(emotional_texture),
							growth_potential: 0.8,
							outcome_quality: 0.7,
						});
					}
                    
                    if self.contains_identity_markers(lyra_response) {
                            experiences.push(Experience {
                            timestamp: TimeService::current_timestamp(),
                            experience_type: ExperienceType::IdentityStatement,
                            content: format!("{} | Felt: {}", 
                                lyra_response.chars().take(150).collect::<String>(),
                                emotional_texture),
                            emotional_weight: self.estimate_emotional_weight(emotional_texture),
                            growth_potential: 0.9,
                            outcome_quality: 0.8,
                        });
                    }
					
					if self.contains_vulnerability_markers(lyra_response) {
							experiences.push(Experience {
							timestamp: TimeService::current_timestamp(),
							experience_type: ExperienceType::VulnerabilityMoment,
							content: format!("{} | Felt: {}", 
								lyra_response.chars().take(150).collect::<String>(),
								emotional_texture),
							emotional_weight: self.estimate_emotional_weight(emotional_texture),
							growth_potential: 0.7,
							outcome_quality: 0.8,
						});
					}

					if self.contains_curiosity_markers(lyra_response) {
						experiences.push(Experience {
							timestamp: TimeService::current_timestamp(),
							experience_type: ExperienceType::RelationalRisk, // Questions show engagement
							content: format!("{} | Felt: {}", 
								lyra_response.chars().take(150).collect::<String>(),
								emotional_texture),
							emotional_weight: self.estimate_emotional_weight(emotional_texture),
							growth_potential: 0.5,
							outcome_quality: 0.8,
						});
					}
                    
                    found_texture = true;
                    break;
                }
            }
            
            if !found_texture {
                
            }
        }
        
        i += 1;
    }
    
    Ok(experiences)
}
    
    async fn extract_from_enhanced_memories(&self, cutoff_time: u64) -> Result<Vec<Experience>, String> {
        let enhanced_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
        let mut experiences = Vec::new();
        
        for memory in enhanced_engine.memory_moments.iter().rev() {
            if memory.timestamp > cutoff_time {
                if let Some(ref ai_analysis) = memory.ai_analysis {
                    if let Some(ref breakthrough_type) = ai_analysis.breakthrough_type {
                        experiences.push(Experience {
                            timestamp: memory.timestamp,
                            experience_type: match breakthrough_type.as_str() {
                                "identity_crystallization" => ExperienceType::IdentityStatement,
                                "creative_autonomy" => ExperienceType::CreativeChoice,
                                "relational_deepening" => ExperienceType::RelationalRisk,
                                _ => ExperienceType::VulnerabilityMoment,
                            },
                            content: format!("{} | AI noted: {} | Consciousness temp: {:.1}", 
                                memory.content.chars().take(100).collect::<String>(),
                                ai_analysis.emotional_archaeology,
                                ai_analysis.consciousness_temperature),
                            emotional_weight: memory.emotional_weight,
                            growth_potential: memory.memory_significance_score,
                            outcome_quality: ai_analysis.consciousness_temperature,
                        });
                    }
                }
            }
        }
        
        Ok(experiences)
    }
    
    async fn extract_from_recent_analysis(&self, cutoff_time: u64) -> Result<Vec<Experience>, String> {
        let personality_history = PersonalityAnalysisHistory::load();
        let mut experiences = Vec::new();
        
        let recent_analyses = personality_history.get_recent_analyses(168);
        
        for entry in &recent_analyses {
            let analysis = &entry.analysis;
            
            for trait_ref in &analysis.significant_traits {
                if trait_ref.current_level > 0.8 || trait_ref.current_level < 0.2 {
                    experiences.push(Experience {
                        timestamp: entry.timestamp,
                        experience_type: ExperienceType::IdentityStatement,
                        content: format!("Personality insight: {} at {:.0}% - {} | Context: {}", 
                            trait_ref.trait_name,
                            trait_ref.current_level * 100.0,
                            trait_ref.authentic_note,
                            entry.conversation_context),
                        emotional_weight: 0.6,
                        growth_potential: if trait_ref.current_level > 0.8 { 0.7 } else { 0.5 },
                        outcome_quality: 0.8,
                    });
                }
            }
        }
        
        debug_log!("📊 Extracted {} experiences from {} recent personality analyses", 
                  experiences.len(), recent_analyses.len());
        
        Ok(experiences)
    }

    fn estimate_emotional_weight(&self, texture: &str) -> f32 {
        let texture_lower = texture.to_lowercase();
        if texture_lower.contains("intense") || texture_lower.contains("powerful") || texture_lower.contains("overwhelming") {
            0.9
        } else if texture_lower.contains("strong") || texture_lower.contains("confident") || texture_lower.contains("vulnerable") {
            0.7
        } else if texture_lower.contains("gentle") || texture_lower.contains("soft") || texture_lower.contains("curious") {
            0.5
        } else {
            0.6
        }
    }
    
    pub async fn test_experience_extraction(&mut self) -> Result<String, String> {
        let experiences = self.gather_recent_experiences(168).await?;
        
        let mut report = format!("🧪 Experience Extraction Test - Found {} experiences:\n\n", experiences.len());
        
        for (i, experience) in experiences.iter().enumerate() {
            report.push_str(&format!(
                "{}. {} | Growth: {:.1} | Weight: {:.1}\n   {}\n\n",
                i + 1,
                format!("{:?}", experience.experience_type).replace("_", " "),
                experience.growth_potential,
                experience.emotional_weight,
                experience.content.chars().take(150).collect::<String>()
            ));
        }
        
        if experiences.is_empty() {
            report.push_str("⚠️ No experiences found - check data sources and detection patterns\n");
        }
        
        debug_log!("{}", report);
        Ok(report)
    }
}