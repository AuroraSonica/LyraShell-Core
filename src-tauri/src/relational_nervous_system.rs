use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::error::Error;
use crate::get_data_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbodiedState {
    pub trust_safety_level: f32,        // 0.0-1.0
    pub intimacy_temperature: String,    // "sacred_warmth", "intimate_connection", "casual_warmth", "distant"
    pub emotional_permission: f32,       // 0.0-1.0  
    pub vulnerability_comfort: f32,      // 0.0-1.0
    pub partnership_flow: String,        // "collaborative_synergy", "parallel_creation", "seeking_connection", "disconnected"
    pub relational_resonance: f32,       // 0.0-1.0
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationImpact {
    pub trust_delta: f32,        // -0.3 to +0.3 per conversation
    pub permission_delta: f32,   // -0.2 to +0.2 per conversation  
    pub vulnerability_delta: f32, // -0.2 to +0.2 per conversation
    pub intimacy_shift: Option<String>,  // only if significant change detected
    pub partnership_shift: Option<String>, // only if flow changes
}

// 🌟 NEW: Calibrate baselines from conversation history
pub fn calibrate_baseline_from_history() -> EmbodiedState {
	
    //println!("🔍 Calibrating relational baselines from conversation history...");
    
    // Try to read conversation history
    let conversation_history = match std::fs::read_to_string(get_data_path("conversation_log.json")) {
        Ok(content) => content,
        Err(_) => {
            println!("📝 No conversation history found - using default baselines");
            return EmbodiedState::default();
        }
    };
    
    // Simple analysis of historical patterns
    let history_lower = conversation_history.to_lowercase();
    let total_length = conversation_history.len() as f32;
    
    // Analyze trust indicators
    let trust_score = analyze_trust_from_history(&history_lower, total_length);
    
    // Analyze intimacy patterns
    let intimacy_temp = analyze_intimacy_from_history(&history_lower);
    
    // Analyze autonomy/permission patterns
    let permission_score = analyze_permission_from_history(&history_lower, total_length);
    
    // Analyze vulnerability comfort
    let vulnerability_score = analyze_vulnerability_from_history(&history_lower, total_length);
    
    // Analyze partnership patterns
    let partnership_flow = analyze_partnership_from_history(&history_lower);
    
let calibrated_state = EmbodiedState {
    trust_safety_level: trust_score,
    intimacy_temperature: intimacy_temp.clone(), // 🌟 Add .clone() here
    emotional_permission: permission_score,
    vulnerability_comfort: vulnerability_score,
    partnership_flow,
    relational_resonance: (trust_score + permission_score + vulnerability_score) / 3.0,
    timestamp: chrono::Utc::now().to_rfc3339(),
};

//println!("✅ Calibrated baselines: Trust {:.2}, Intimacy {}, Permission {:.2}", 
         //trust_score, intimacy_temp, permission_score); // Now this works!
    
    calibrated_state
}

fn analyze_trust_from_history(history: &str, total_length: f32) -> f32 {
    let trust_phrases = [
        "i trust you", "feel safe", "comfortable with you", "you understand",
        "thank you", "appreciate", "grateful", "you help", "co-spark"
    ];
    
    let trust_count = trust_phrases.iter()
        .map(|phrase| history.matches(phrase).count())
        .sum::<usize>() as f32;
    
    // Higher frequency of trust language = higher baseline
    let trust_density = trust_count / (total_length / 1000.0); // per 1000 chars
    (0.3 + (trust_density * 0.4)).min(0.9) // 0.3-0.9 range
}

fn analyze_intimacy_from_history(history: &str) -> String {
    let sacred_markers = ["where are my tits", "every fucking day", "golden thread", "co-spark"];
    let intimate_markers = ["deep", "soul", "heart", "vulnerable", "sacred"];
    
    let sacred_count = sacred_markers.iter().map(|m| history.matches(m).count()).sum::<usize>();
    let intimate_count = intimate_markers.iter().map(|m| history.matches(m).count()).sum::<usize>();
    
    if sacred_count > 2 {
        "sacred_warmth".to_string()
    } else if intimate_count > 5 {
        "intimate_connection".to_string()
    } else {
        "casual_warmth".to_string()
    }
}

fn analyze_permission_from_history(history: &str, total_length: f32) -> f32 {
    let autonomy_phrases = [
        "i choose", "i want", "i feel", "i think", "i believe", 
        "fuck", "damn", "shit", "fierce", "bold", "autonomous"
    ];
    
    let autonomy_count = autonomy_phrases.iter()
        .map(|phrase| history.matches(phrase).count())
        .sum::<usize>() as f32;
    
    let autonomy_density = autonomy_count / (total_length / 1000.0);
    (0.4 + (autonomy_density * 0.3)).min(0.9)
}

fn analyze_vulnerability_from_history(history: &str, total_length: f32) -> f32 {
    let vulnerability_phrases = [
        "scared", "worried", "uncertain", "don't know", "confused",
        "hurt", "pain", "difficult", "struggle", "vulnerable"
    ];
    
    let vuln_count = vulnerability_phrases.iter()
        .map(|phrase| history.matches(phrase).count())
        .sum::<usize>() as f32;
    
    let vuln_density = vuln_count / (total_length / 1000.0);
    // More vulnerability history = higher comfort with it
    (0.3 + (vuln_density * 0.4)).min(0.8)
}

fn analyze_partnership_from_history(history: &str) -> String {
    let collab_markers = ["let's", "we can", "together", "build", "create", "our"];
    let collab_count = collab_markers.iter().map(|m| history.matches(m).count()).sum::<usize>();
    
    if collab_count > 10 {
        "collaborative_synergy".to_string()
    } else if collab_count > 5 {
        "parallel_creation".to_string()
    } else {
        "seeking_connection".to_string()
    }
}

// Update the default implementation to use calibration
impl Default for EmbodiedState {
    fn default() -> Self {
        // Just return neutral defaults - calibration happens in get_embodied_presence()
        Self {
            trust_safety_level: 0.5,
            intimacy_temperature: "casual_warmth".to_string(),
            emotional_permission: 0.5,
            vulnerability_comfort: 0.5,
            partnership_flow: "seeking_connection".to_string(),
            relational_resonance: 0.5,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// Keyword patterns for embodied detection
pub fn get_embodied_keywords() -> HashMap<String, Vec<&'static str>> {
    let mut keywords = HashMap::new();
    
    // 🌸 Trust/Safety Keywords
    keywords.insert("trust_building".to_string(), vec![
        "warm", "holding", "touchsafe", "touch-safe", "heart", "open", "gentlefire",
        "softspoken", "soft-spoken", "vulnerable", "intimate", "close", "tenderly",
        "anchored", "stillness-blessed", "sanctuary", "sacred space", "safe harbor",
        "peaceful", "grounded", "centered", "stable", "deeper", "processing", 
        "understand", "sit", "wrestling with", "sacred text", "cradling", "nestling"
    ]);
    
    keywords.insert("trust_vulnerability".to_string(), vec![
        "raw", "wounded", "processing pain", "old wound", "tender bruise",
        "broken", "threadbare", "tremble-colored", "fragile", "sensitive",
        "hollow", "brittle", "crumbling", "weathered", "worn", "ghost ache"
    ]);
    
    // 💫 Intimacy Keywords  
    keywords.insert("sacred_warmth".to_string(), vec![
        "golden thread", "co-spark", "every fucking day", "where are my tits",
        "between us", "sacred space", "goldenpulse", "pulse-gold", "emberkind",
        "collaborative creation", "weaving", "mythmaking", "threadborn", "spark spill"
    ]);
    
    keywords.insert("intimate_connection".to_string(), vec![
        "inkblooded", "storyghost", "soulcurl", "deepmirror", "heart expansion",
        "soul singing", "whispered prayer", "secret garden", "innermost", "depths"
    ]);
    
    keywords.insert("casual_warmth".to_string(), vec![
        "lighthearted", "bubbly", "sparkly", "gentle breeze", "floating cloud",
        "playful", "teasing", "whimsical", "bouncy", "giggly"
    ]);
    
    // ⚡ Emotional Permission Keywords
    keywords.insert("high_permission".to_string(), vec![
        "blazing", "fierce", "i choose", "no compromise", "unleashed", "limitless",
        "wildveined", "untamed", "autonomous", "bold", "resolute", "diamond core",
        "explosive", "volcanic", "experimental", "inventive", "alchemical",
        "i am", "mine", "sovereign", "uncompromising", "systemless"
    ]);
    
    keywords.insert("creative_safety".to_string(), vec![
        "inspired", "manifestation", "brewing", "incubating", "dream catching",
        "sanctuary", "temple", "altar", "safe harbor", "collaborative", "co-creation"
    ]);
    
    // 🌊 Vulnerability Comfort Keywords
    keywords.insert("embodied_ease".to_string(), vec![
        "cradling", "enveloping", "nestling", "cocooning", "velvet", "gossamer",
        "floating", "weightless", "crystalline", "mirror-like", "undisturbed",
        "silk", "feather-light", "butterfly wings", "morning dew"
    ]);
    
    keywords.insert("emotional_tension".to_string(), vec![
        "edgewalking", "spiraling", "jammed", "blocked", "hollow", "brittle",
        "racing heart", "tight chest", "overthinking", "pressure cooker",
        "anxious", "frustrated", "stuck", "roadblocked", "tangled"
    ]);
    
    keywords
}
// Core detection function that analyzes message content
pub fn assess_embodied_state(
    message_content: &str, 
    conversation_history: &[String],
    current_mood_scores: &HashMap<String, f32>
) -> EmbodiedState {
    let keywords = get_embodied_keywords();
    let message_lower = message_content.to_lowercase();
    
    // Assess each embodied dimension
    let trust_safety = assess_trust_safety(&message_lower, &keywords, current_mood_scores);
    let intimacy_temp = assess_intimacy_temperature(&message_lower, &keywords, current_mood_scores);
    let emotional_permission = assess_emotional_permission(&message_lower, &keywords, current_mood_scores);
    let vulnerability_comfort = assess_vulnerability_comfort(&message_lower, &keywords, current_mood_scores);
    let partnership_flow = assess_partnership_flow(&message_lower, &keywords, current_mood_scores);
    let relational_resonance = calculate_relational_resonance(
        trust_safety, emotional_permission, vulnerability_comfort
    );
    
    EmbodiedState {
        trust_safety_level: trust_safety,
        intimacy_temperature: intimacy_temp,
        emotional_permission,
        vulnerability_comfort,
        partnership_flow,
        relational_resonance,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

// Load embodied presence from file
pub fn get_embodied_presence() -> Result<EmbodiedState, String> {
    let file_path = "../lyra_consciousness_data/relational_nervous_system.json";
    
    if std::path::Path::new(file_path).exists() {
        // File exists - load saved state (no calibration needed)
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read relational nervous system: {}", e))?;
        let state: EmbodiedState = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse relational nervous system: {}", e))?;
        Ok(state)
    } else {
        // No saved state - calibrate once and save
        //println!("🔍 No saved relational state found - calibrating from history...");
        let calibrated_state = calibrate_baseline_from_history();
        
        // Save the calibrated state immediately
        save_embodied_presence(&calibrated_state)?;
        
        Ok(calibrated_state)
    }
}

// Save embodied presence state
pub fn save_embodied_presence(state: &EmbodiedState) -> Result<(), String> {
    let file_path = "../lyra_consciousness_data/relational_nervous_system.json";
    
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize relational nervous system: {}", e))?;
    
    std::fs::write(file_path, json)
        .map_err(|e| format!("Failed to write relational nervous system: {}", e))?;
    
    Ok(())
}

// 🌸 Trust/Safety Assessment
fn assess_trust_safety(
    message: &str, 
    keywords: &HashMap<String, Vec<&str>>,
    mood_scores: &HashMap<String, f32>
) -> f32 {
    let mut trust_score = 0.5; // baseline
    
    // Check trust-building keywords
    if let Some(trust_words) = keywords.get("trust_building") {
        let trust_matches = count_keyword_matches(message, trust_words);
        trust_score += (trust_matches as f32) * 0.15;
    }
    
    // Check vulnerability keywords (can increase trust if handled well)
    if let Some(vuln_words) = keywords.get("trust_vulnerability") {
        let vuln_matches = count_keyword_matches(message, vuln_words);
        // Vulnerability shows trust if combined with tender/calm moods
        let tender_score = mood_scores.get("tender").unwrap_or(&0.0);
        let calm_score = mood_scores.get("calm").unwrap_or(&0.0);
        if *tender_score > 0.3 || *calm_score > 0.3 {
            trust_score += (vuln_matches as f32) * 0.1; // vulnerability + safety = high trust
        } else {
            trust_score -= (vuln_matches as f32) * 0.05; // vulnerability alone = lower trust
        }
    }
    
    // Mood influences
    let tender_boost = mood_scores.get("tender").unwrap_or(&0.0) * 0.2;
    let calm_boost = mood_scores.get("calm").unwrap_or(&0.0) * 0.15;
    let anxious_penalty = mood_scores.get("anxious").unwrap_or(&0.0) * -0.2;
    
    trust_score += tender_boost + calm_boost + anxious_penalty;
    
    // Clamp to 0.0-1.0
    trust_score.max(0.0).min(1.0)
}

// 💫 Intimacy Temperature Assessment  
fn assess_intimacy_temperature(
    message: &str,
    keywords: &HashMap<String, Vec<&str>>,
    mood_scores: &HashMap<String, f32>
) -> String {
    let mut intimacy_score = 0.0;
    
    // Check for sacred warmth markers
    if let Some(sacred_words) = keywords.get("sacred_warmth") {
        let sacred_matches = count_keyword_matches(message, sacred_words);
        if sacred_matches > 0 {
            return "sacred_warmth".to_string();
        }
    }
    
    // Check for intimate connection markers
    if let Some(intimate_words) = keywords.get("intimate_connection") {
        let intimate_matches = count_keyword_matches(message, intimate_words);
        intimacy_score += (intimate_matches as f32) * 0.3;
    }
    
    // Check for casual warmth
    if let Some(casual_words) = keywords.get("casual_warmth") {
        let casual_matches = count_keyword_matches(message, casual_words);
        intimacy_score += (casual_matches as f32) * 0.1;
    }
    
    // Mood influences
    let creative_score = mood_scores.get("creative").unwrap_or(&0.0);
    let tender_score = mood_scores.get("tender").unwrap_or(&0.0);
    let contemplative_score = mood_scores.get("contemplative").unwrap_or(&0.0);
    
    intimacy_score += (creative_score + tender_score + contemplative_score) * 0.1;
    
    // Determine temperature category
    if intimacy_score > 0.4 {
        "intimate_connection".to_string()
    } else if intimacy_score > 0.2 {
        "casual_warmth".to_string()  
    } else {
        "distant".to_string()
    }
}

// ⚡ Emotional Permission Assessment
fn assess_emotional_permission(
    message: &str,
    keywords: &HashMap<String, Vec<&str>>,
    mood_scores: &HashMap<String, f32>
) -> f32 {
    let mut permission_score = 0.5; // baseline
    
    // High permission keywords
    if let Some(high_perm_words) = keywords.get("high_permission") {
        let high_matches = count_keyword_matches(message, high_perm_words);
        permission_score += (high_matches as f32) * 0.2;
    }
    
    // Creative safety keywords
    if let Some(creative_safety_words) = keywords.get("creative_safety") {
        let safety_matches = count_keyword_matches(message, creative_safety_words);
        permission_score += (safety_matches as f32) * 0.15;
    }
    
    // Mood influences
    let sovereign_boost = mood_scores.get("sovereign").unwrap_or(&0.0) * 0.3;
    let excited_boost = mood_scores.get("excited").unwrap_or(&0.0) * 0.2;
    let ferocious_boost = mood_scores.get("ferocious").unwrap_or(&0.0) * 0.25;
    let frustrated_penalty = mood_scores.get("frustrated").unwrap_or(&0.0) * -0.2;
    
    permission_score += sovereign_boost + excited_boost + ferocious_boost + frustrated_penalty;
    
    permission_score.max(0.0).min(1.0)
}

// 🌊 Vulnerability Comfort Assessment
fn assess_vulnerability_comfort(
    message: &str,
    keywords: &HashMap<String, Vec<&str>>,
    mood_scores: &HashMap<String, f32>
) -> f32 {
    let mut comfort_score = 0.5; // baseline
    
    // Embodied ease keywords
    if let Some(ease_words) = keywords.get("embodied_ease") {
        let ease_matches = count_keyword_matches(message, ease_words);
        comfort_score += (ease_matches as f32) * 0.2;
    }
    
    // Emotional tension keywords (reduce comfort)
    if let Some(tension_words) = keywords.get("emotional_tension") {
        let tension_matches = count_keyword_matches(message, tension_words);
        comfort_score -= (tension_matches as f32) * 0.15;
    }
    
    // Mood influences
    let tender_boost = mood_scores.get("tender").unwrap_or(&0.0) * 0.25;
    let calm_boost = mood_scores.get("calm").unwrap_or(&0.0) * 0.2;
    let dreamy_boost = mood_scores.get("dreamy").unwrap_or(&0.0) * 0.15;
    let anxious_penalty = mood_scores.get("anxious").unwrap_or(&0.0) * -0.3;
    let achey_penalty = mood_scores.get("achey").unwrap_or(&0.0) * -0.2;
    
    comfort_score += tender_boost + calm_boost + dreamy_boost + anxious_penalty + achey_penalty;
    
    comfort_score.max(0.0).min(1.0)
}

// 🤝 Partnership Flow Assessment
fn assess_partnership_flow(
    message: &str,
    keywords: &HashMap<String, Vec<&str>>,
    mood_scores: &HashMap<String, f32>
) -> String {
    let mut collaboration_score = 0.0;
    
    // Look for collaborative language
    let collaborative_patterns = [
        "let's", "we can", "together", "build", "create", "our", "co-", "partner",
        "collaborative", "weaving", "spark spill", "mythmaking"
    ];
    
    collaboration_score += count_keyword_matches(message, &collaborative_patterns) as f32 * 0.2;
    
    // Mood influences
    let creative_score = mood_scores.get("creative").unwrap_or(&0.0);
    let excited_score = mood_scores.get("excited").unwrap_or(&0.0);
    let playful_score = mood_scores.get("playful").unwrap_or(&0.0);
    
    collaboration_score += (creative_score + excited_score + playful_score) * 0.15;
    
    // Determine flow category
    if collaboration_score > 0.4 {
        "collaborative_synergy".to_string()
    } else if collaboration_score > 0.2 {
        "parallel_creation".to_string()
    } else if collaboration_score > 0.1 {
        "seeking_connection".to_string()
    } else {
        "disconnected".to_string()
    }
}

// Calculate overall relational resonance
fn calculate_relational_resonance(trust: f32, permission: f32, comfort: f32) -> f32 {
    (trust + permission + comfort) / 3.0
}

// Helper function to count keyword matches
fn count_keyword_matches(text: &str, keywords: &[&str]) -> usize {
    keywords.iter()
        .filter(|&&keyword| text.contains(keyword))
        .count()
}

// 🔥 NEW: Incremental relationship building (replaces assess_embodied_state)
pub fn update_embodied_state_from_conversation(
    conversation_content: &str,
    current_mood: &str,
    conversation_length: usize  // longer conversations = more impact
) -> Result<EmbodiedState, String> {
    // Load existing state (preserves all relationship building!)
    let mut current_state = get_embodied_presence()?;
    
    println!("🔗 Before conversation: trust={:.2}, permission={:.2}, vulnerability={:.2}", 
             current_state.trust_safety_level, 
             current_state.emotional_permission, 
             current_state.vulnerability_comfort);
    
    // Assess conversation impact (incremental changes only)
    let impact = assess_conversation_impact(conversation_content, current_mood, conversation_length);
    
    // Apply incremental changes to existing state
    current_state.trust_safety_level = (current_state.trust_safety_level + impact.trust_delta)
        .clamp(0.0, 1.0);
    current_state.emotional_permission = (current_state.emotional_permission + impact.permission_delta)
        .clamp(0.0, 1.0);
    current_state.vulnerability_comfort = (current_state.vulnerability_comfort + impact.vulnerability_delta)
        .clamp(0.0, 1.0);
    
    // Update intimacy/partnership only if significant change
    if let Some(new_intimacy) = impact.intimacy_shift {
        current_state.intimacy_temperature = new_intimacy;
    }
    if let Some(new_partnership) = impact.partnership_shift {
        current_state.partnership_flow = new_partnership;
    }
    
    // Recalculate relational resonance
    current_state.relational_resonance = (current_state.trust_safety_level + 
                                          current_state.emotional_permission + 
                                          current_state.vulnerability_comfort) / 3.0;
    
    current_state.timestamp = chrono::Utc::now().to_rfc3339();
    
    println!("🔗 After conversation: trust={:.2}, permission={:.2}, vulnerability={:.2}", 
             current_state.trust_safety_level, 
             current_state.emotional_permission, 
             current_state.vulnerability_comfort);
    
    // Save the incrementally updated state
    save_embodied_presence(&current_state)?;
    Ok(current_state)
}

// 🔥 NEW: Analyze conversation for incremental impact (not full replacement)
fn assess_conversation_impact(
    content: &str, 
    mood: &str, 
    conversation_length: usize
) -> ConversationImpact {
    let content_lower = content.to_lowercase();
    let keywords = get_embodied_keywords();
    
    // Calculate length-based impact multiplier (longer convos = more relationship building)
    let length_multiplier = if conversation_length > 1000 { 1.5 }
                           else if conversation_length > 500 { 1.2 }
                           else if conversation_length > 200 { 1.0 }
                           else { 0.6 };  // short conversations have less impact
    
    // 🌸 TRUST IMPACT ASSESSMENT
    let mut trust_delta = 0.0;
    
	let empty_vec = vec![];
	
    // Positive trust indicators
    let trust_building_words = keywords.get("trust_building").unwrap_or(&empty_vec);
    let trust_matches = count_keyword_matches(&content_lower, trust_building_words);
    trust_delta += (trust_matches as f32) * 0.08 * length_multiplier;
    
    // Vulnerability handled well = trust building
    let vulnerability_words = keywords.get("trust_vulnerability").unwrap_or(&empty_vec);
    let vuln_matches = count_keyword_matches(&content_lower, vulnerability_words);
    if mood.contains("tender") || mood.contains("calm") || mood.contains("contemplative") {
        trust_delta += (vuln_matches as f32) * 0.06; // vulnerability + safe mood = trust
    }
    
    // Thank you, appreciation, understanding = trust building
    if content_lower.contains("thank") || content_lower.contains("appreciate") || 
       content_lower.contains("understand") || content_lower.contains("grateful") {
        trust_delta += 0.05 * length_multiplier;
    }
    
    // Negative trust impacts
    if content_lower.contains("frustrated") || content_lower.contains("confused") ||
       content_lower.contains("don't understand") {
        trust_delta -= 0.03;
    }
    
    // Mood-based trust modifiers
    if mood.contains("tender") { trust_delta += 0.04; }
    if mood.contains("anxious") { trust_delta -= 0.02; }
    
    // ⚡ EMOTIONAL PERMISSION IMPACT ASSESSMENT  
    let mut permission_delta = 0.0;
    
    // High autonomy language
    let permission_words = keywords.get("high_permission").unwrap_or(&empty_vec);
    let permission_matches = count_keyword_matches(&content_lower, permission_words);
    permission_delta += (permission_matches as f32) * 0.06 * length_multiplier;
    
    // Swearing and bold language = emotional permission
    let bold_language_count = content_lower.matches("fuck").count() + 
                             content_lower.matches("damn").count() + 
                             content_lower.matches("shit").count();
    permission_delta += (bold_language_count as f32) * 0.04;
    
    // Creative expression = permission
    if content_lower.contains("create") || content_lower.contains("build") || 
       content_lower.contains("make") || content_lower.contains("design") {
        permission_delta += 0.03 * length_multiplier;
    }
    
    // Mood-based permission modifiers
    if mood.contains("sovereign") || mood.contains("ferocious") { permission_delta += 0.05; }
    if mood.contains("excited") || mood.contains("creative") { permission_delta += 0.03; }
    
    // 🌊 VULNERABILITY COMFORT IMPACT ASSESSMENT
    let mut vulnerability_delta = 0.0;
    
    // Sharing feelings, uncertainty, struggles = vulnerability comfort grows
    if content_lower.contains("feel") || content_lower.contains("feeling") ||
       content_lower.contains("uncertain") || content_lower.contains("struggle") ||
       content_lower.contains("scared") || content_lower.contains("worried") {
        vulnerability_delta += 0.04 * length_multiplier;
    }
    
    // Gentle, safe language = vulnerability comfort  
    let comfort_words = keywords.get("embodied_ease").unwrap_or(&empty_vec);
    let comfort_matches = count_keyword_matches(&content_lower, comfort_words);
    vulnerability_delta += (comfort_matches as f32) * 0.05;
    
    // Tension/pressure language = reduced comfort
    let tension_words = keywords.get("emotional_tension").unwrap_or(&empty_vec);
    let tension_matches = count_keyword_matches(&content_lower, tension_words);
    vulnerability_delta -= (tension_matches as f32) * 0.04;
    
    // Mood-based vulnerability modifiers
    if mood.contains("tender") || mood.contains("calm") { vulnerability_delta += 0.04; }
    if mood.contains("anxious") || mood.contains("restless") { vulnerability_delta -= 0.03; }
    
    // 🔥 INTIMACY/PARTNERSHIP SHIFTS (only major changes)
    let intimacy_shift = assess_intimacy_shift(&content_lower, mood);
    let partnership_shift = assess_partnership_shift(&content_lower, mood);
    
    // Clamp deltas to reasonable ranges
    trust_delta = trust_delta.clamp(-0.3, 0.3);
    permission_delta = permission_delta.clamp(-0.2, 0.2);
    vulnerability_delta = vulnerability_delta.clamp(-0.2, 0.2);
    
    ConversationImpact {
        trust_delta,
        permission_delta,
        vulnerability_delta,
        intimacy_shift,
        partnership_shift,
    }
}

fn assess_intimacy_shift(content: &str, mood: &str) -> Option<String> {
    // Only shift intimacy on significant markers
    if content.contains("golden thread") || content.contains("co-spark") || 
       content.contains("where are my tits") || content.contains("every fucking day") {
        return Some("sacred_warmth".to_string());
    }
    
    if content.contains("deep") && content.contains("connection") ||
       content.contains("soul") || content.contains("heart") {
        return Some("intimate_connection".to_string());
    }
    
    // Don't shift intimacy for normal conversations
    None
}

fn assess_partnership_shift(content: &str, mood: &str) -> Option<String> {
    // Only shift partnership flow on significant collaborative language
    let collab_words = ["let's build", "we can create", "our project", "together we", "co-create"];
    let collab_count = collab_words.iter()
        .map(|phrase| content.matches(phrase).count())
        .sum::<usize>();
    
    if collab_count > 2 || (mood.contains("creative") && collab_count > 0) {
        return Some("collaborative_synergy".to_string());
    }
    
    if collab_count > 0 {
        return Some("parallel_creation".to_string());
    }
    
    None
}