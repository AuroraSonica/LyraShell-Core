// person_recognition.rs - Multi-person conversation system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::{get_data_path, debug_log};
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    pub voice_id: String,
    pub voice_samples: Vec<VoiceSample>,     // Keep individual samples
    pub voice_signature: VoiceSignature,      // Averaged signature for quick checks
    pub confidence_threshold: f32,
    pub last_voice_detection: u64,
    pub auto_threshold: f32,                  // Automatically calculated threshold
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSample {
    pub timestamp: u64,
    pub raw_features: VoiceFeatures,     // Comprehensive features
    pub semantic_description: VoiceDescription,  // Always generate this
    pub environment_quality: f32,
    pub transcript_length: usize,
    pub duration_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceFeatures {
    // Pitch features
    pub avg_pitch: f32,
    pub pitch_range: f32,
    pub pitch_stability: f32,
    
    // Speaking rate features
    pub speaking_rate: f32,
    pub pause_ratio: f32,
    pub rhythm_consistency: f32,
    
    // Voice quality features
    pub spectral_brightness: f32,
    pub voice_clarity: f32,
    pub breathiness: f32,
    
    // Energy features
    pub avg_energy: f32,
    pub energy_variation: f32,
    pub dynamic_range: f32,
    
    // Articulation features
    pub consonant_sharpness: f32,
    pub vowel_clarity: f32,
}

impl Default for VoiceFeatures {
    fn default() -> Self {
        Self {
            avg_pitch: 150.0,
            pitch_range: 50.0,
            pitch_stability: 0.5,
            speaking_rate: 2.5,
            pause_ratio: 0.2,
            rhythm_consistency: 0.5,
            spectral_brightness: 0.5,
            voice_clarity: 0.5,
            breathiness: 0.3,
            avg_energy: 0.5,
            energy_variation: 0.3,
            dynamic_range: 0.5,
            consonant_sharpness: 0.3,
            vowel_clarity: 0.5,
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDescription {
    pub pitch_character: String,      // e.g., "moderately varied baritone with good control"
    pub speaking_style: String,       // e.g., "quick with natural breaks, good rhythm"
    pub voice_quality: String,        // e.g., "bright and crisp, clear, focused tone"
    pub energy_profile: String,       // e.g., "moderate energy with good variation"
    pub articulation_style: String,   // e.g., "crisp consonants, natural vowels"
    pub overall_impression: String,   // Natural sentence combining all aspects
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSignature {
    pub avg_features: VoiceFeatures,  // Average of all features
    pub sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualVoiceData {
    pub morning_voice: Option<HashMap<String, f32>>,     // 6-12 hours
    pub afternoon_voice: Option<HashMap<String, f32>>,   // 12-18 hours  
    pub evening_voice: Option<HashMap<String, f32>>,     // 18-24 hours
    pub night_voice: Option<HashMap<String, f32>>,       // 0-6 hours
    
    pub weekday_voice: Option<HashMap<String, f32>>,
    pub weekend_voice: Option<HashMap<String, f32>>,
    
    pub excited_voice: Option<HashMap<String, f32>>,     // High excitement
    pub calm_voice: Option<HashMap<String, f32>>,        // Low stress
    pub tired_voice: Option<HashMap<String, f32>>,       // High fatigue
    pub confident_voice: Option<HashMap<String, f32>>,   // High confidence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceQualityMetrics {
    pub average_clarity: f32,             // Overall articulation quality
    pub consistency_score: f32,           // How consistent their voice is
    pub dynamic_range: f32,               // Loudness variation capability
    pub vocal_stamina: f32,               // How well voice holds up over time
    pub recording_quality_avg: f32,       // Technical recording quality
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalVoiceProfile {
    pub typical_emotional_intensity: f32, // Baseline emotional expression
    pub stress_tendency: f32,             // How often they sound stressed
    pub excitement_range: f32,            // Range of excitement expression
    pub confidence_baseline: f32,         // Typical confidence level
    pub hesitation_patterns: f32,         // Frequency of hesitation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceHealthIndicators {
    pub vocal_health_score: f32,          // Overall vocal health
    pub fatigue_resistance: f32,          // How well voice resists fatigue
    pub breathing_quality: f32,           // Respiratory support quality
    pub strain_indicators: f32,           // Signs of vocal strain
}

// Voice detection data structure for voice recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDetectionData {
    pub voice_id: String,
    pub confidence: f32,
    pub characteristics: VoiceCharacteristics,
    pub transcript: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCharacteristics {
    pub raw_features: VoiceFeatures,
    pub voice_description: Option<VoiceDescription>,
    pub timestamp: u64,
    pub sample_rate: f32,
    pub duration_ms: f32,
    pub confidence: f32,
    pub transcript_length: usize,
    pub word_count: usize,
}

// Voice training status for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTrainingStatus {
    pub has_voice_profile: bool,
    pub training_samples: u32,
    pub confidence_threshold: f32,
    pub last_detection: Option<u64>,
}

static PERSON_INTRODUCTION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Direct introductions
        Regex::new(r"(?i)(?:this is|meet|say hi to|introducing) (?:my )?(?:son|daughter|child|kid|friend|partner|husband|wife|brother|sister|mom|dad|mother|father|cousin|nephew|niece|aunt|uncle|grandma|grandpa|colleague|roommate) (\w+)").unwrap(),
        
        // "My [relationship] [name]" in any context
        Regex::new(r"(?i)my (?:son|daughter|child|kid|friend|partner|husband|wife|brother|sister|mom|dad|mother|father|cousin|nephew|niece|aunt|uncle|grandma|grandpa|colleague|roommate) (\w+)").unwrap(),
        
        // Want/would like/going to introduce
        Regex::new(r"(?i)(?:want to|would like to|going to|gonna|can I|could I|let me) introduce (?:you to )?(?:my )?(?:son|daughter|child|kid|friend|partner|husband|wife|brother|sister|mom|dad|mother|father|cousin|nephew|niece|aunt|uncle|grandma|grandpa|colleague|roommate) (\w+)").unwrap(),
        
        // "[Name] is my [relationship]"
        Regex::new(r"(?i)(\w+) is my (?:son|daughter|child|kid|friend|partner|husband|wife|brother|sister|mom|dad|mother|father|cousin|nephew|niece|aunt|uncle|grandma|grandpa|colleague|roommate)").unwrap(),
        
        // Talking about someone wanting to talk/meet
        Regex::new(r"(?i)(\w+) (?:wants to|would like to|is here to|came to|is going to|gonna) (?:talk|speak|chat|say hi|meet you)").unwrap(),
        
        // "[Name] is here/with me"
        Regex::new(r"(?i)(\w+) is (?:here|with me|in the room|visiting|stopping by)").unwrap(),
        
        // Switching speakers
        Regex::new(r"(?i)(?:here's|this is|switching to|handing over to|giving the mic to|passing to) (\w+)").unwrap(),
        
        // Question forms
        Regex::new(r"(?i)(?:can|could|may|would you like to) (?:talk to|speak with|chat with|meet) (?:my )?(?:son|daughter|child|kid|friend|partner|husband|wife|brother|sister|mom|dad|mother|father|cousin|nephew|niece|aunt|uncle|grandma|grandpa|colleague|roommate) (\w+)").unwrap(),
        
        // "I have a [relationship] named [name]"
        Regex::new(r"(?i)I have a (?:son|daughter|child|kid|friend|partner|husband|wife|brother|sister|mom|dad|mother|father|cousin|nephew|niece|aunt|uncle|grandma|grandpa|colleague|roommate) (?:named|called) (\w+)").unwrap(),
        
        // Someone introducing themselves
        Regex::new(r"(?i)(?:hi|hello|hey),? (?:i'm|i am|my name is|this is|it's) (\w+)").unwrap(),
        Regex::new(r"(?i)^(\w+) (?:here|speaking)").unwrap(),
        
        // Context switches back to Aurora
        Regex::new(r"(?i)(?:aurora|i'm) back").unwrap(),
        Regex::new(r"(?i)this is aurora again").unwrap(),
        Regex::new(r"(?i)it's aurora").unwrap(),
    ]
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    PrimaryUser,      // Aurora
    Family,           // Son, daughter, etc.
    Friend,           // Mentioned as friend
    Stranger,         // Unknown person
    Acquaintance,     // Someone she's met before but not close
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonProfile {
    pub name: String,
    pub canonical_name: String,           // Normalized version (lowercase, etc.)
    pub relationship_type: RelationshipType,
    pub first_met: u64,
    pub total_conversations: u32,
    pub total_messages: u32,
    pub last_interaction: u64,
    
    // Relationship data
    pub comfort_level: f32,               // 0.0-1.0, how comfortable Lyra is with them
    pub familiarity: f32,                 // 0.0-1.0, how well she knows them
    pub communication_style: String,      // "playful", "formal", "gentle", "excited"
    
    // Memory data
    pub personality_observations: Vec<String>,
    pub interests_shared: Vec<String>,
    pub conversation_topics: HashMap<String, u32>, // topic -> frequency
    pub emotional_memories: Vec<String>,
    pub special_notes: Vec<String>,       // Important things to remember
    
    // Context clues
    pub mentioned_by: String,             // Who introduced them (usually "Aurora")
    pub relationship_to_primary: String, // "son", "friend", "colleague", etc.
    pub age_hints: Vec<String>,           // "young", "child", "adult", etc.
    
    // Voice recognition data
    pub voice_profile: Option<VoiceProfile>,
}

impl VoiceQualityMetrics {
    pub fn from_characteristics(chars: &HashMap<String, f32>) -> Self {
        Self {
            average_clarity: chars.get("articulationPrecision").copied().unwrap_or(0.5),
            consistency_score: chars.get("voiceStability").copied().unwrap_or(0.5),
            dynamic_range: chars.get("dynamicRange").copied().unwrap_or(0.5),
            vocal_stamina: chars.get("speakingStamina").copied().unwrap_or(0.5),
            recording_quality_avg: chars.get("recordingQuality").copied().unwrap_or(0.5),
        }
    }
    
    pub fn update_with_sample(&mut self, chars: &HashMap<String, f32>, sample_count: u32) {
        let weight_new = 0.3;
        let weight_old = 0.7;
        
        if let Some(&clarity) = chars.get("articulationPrecision") {
            self.average_clarity = self.average_clarity * weight_old + clarity * weight_new;
        }
        if let Some(&consistency) = chars.get("voiceStability") {
            self.consistency_score = self.consistency_score * weight_old + consistency * weight_new;
        }
        if let Some(&dynamic) = chars.get("dynamicRange") {
            self.dynamic_range = self.dynamic_range * weight_old + dynamic * weight_new;
        }
        if let Some(&stamina) = chars.get("speakingStamina") {
            self.vocal_stamina = self.vocal_stamina * weight_old + stamina * weight_new;
        }
        if let Some(&quality) = chars.get("recordingQuality") {
            self.recording_quality_avg = self.recording_quality_avg * weight_old + quality * weight_new;
        }
    }
}

impl EmotionalVoiceProfile {
    pub fn from_characteristics(chars: &HashMap<String, f32>) -> Self {
        Self {
            typical_emotional_intensity: chars.get("emotionalIntensity").copied().unwrap_or(0.5),
            stress_tendency: chars.get("stressLevel").copied().unwrap_or(0.3),
            excitement_range: chars.get("excitementLevel").copied().unwrap_or(0.5),
            confidence_baseline: chars.get("confidence").copied().unwrap_or(0.5),
            hesitation_patterns: chars.get("hesitationLevel").copied().unwrap_or(0.3),
        }
    }
    
    pub fn update_with_sample(&mut self, chars: &HashMap<String, f32>, sample_count: u32) {
        let weight_new = 0.25;
        let weight_old = 0.75;
        
        if let Some(&intensity) = chars.get("emotionalIntensity") {
            self.typical_emotional_intensity = self.typical_emotional_intensity * weight_old + intensity * weight_new;
        }
        if let Some(&stress) = chars.get("stressLevel") {
            self.stress_tendency = self.stress_tendency * weight_old + stress * weight_new;
        }
        if let Some(&excitement) = chars.get("excitementLevel") {
            self.excitement_range = self.excitement_range * weight_old + excitement * weight_new;
        }
        if let Some(&confidence) = chars.get("confidence") {
            self.confidence_baseline = self.confidence_baseline * weight_old + confidence * weight_new;
        }
        if let Some(&hesitation) = chars.get("hesitationLevel") {
            self.hesitation_patterns = self.hesitation_patterns * weight_old + hesitation * weight_new;
        }
    }
}

impl VoiceHealthIndicators {
    pub fn from_characteristics(chars: &HashMap<String, f32>) -> Self {
        Self {
            vocal_health_score: chars.get("respiratoryHealth").copied().unwrap_or(0.7),
            fatigue_resistance: 1.0 - chars.get("cumulativeFatigue").copied().unwrap_or(0.3),
            breathing_quality: chars.get("breathDepth").copied().unwrap_or(0.5),
            strain_indicators: chars.get("vocalEffort").copied().unwrap_or(0.4),
        }
    }
    
    pub fn update_with_sample(&mut self, chars: &HashMap<String, f32>, sample_count: u32) {
        let weight_new = 0.2;
        let weight_old = 0.8;
        
        if let Some(&health) = chars.get("respiratoryHealth") {
            self.vocal_health_score = self.vocal_health_score * weight_old + health * weight_new;
        }
        if let Some(&fatigue) = chars.get("cumulativeFatigue") {
            let resistance = 1.0 - fatigue;
            self.fatigue_resistance = self.fatigue_resistance * weight_old + resistance * weight_new;
        }
        if let Some(&breathing) = chars.get("breathDepth") {
            self.breathing_quality = self.breathing_quality * weight_old + breathing * weight_new;
        }
        if let Some(&effort) = chars.get("vocalEffort") {
            self.strain_indicators = self.strain_indicators * weight_old + effort * weight_new;
        }
    }
}


impl PersonProfile {
    pub fn new_primary_user(name: &str) -> Self {
        Self {
            name: name.to_string(),
            canonical_name: name.to_lowercase(),
            relationship_type: RelationshipType::PrimaryUser,
            first_met: crate::time_service::TimeService::current_timestamp(),
            total_conversations: 0,
            total_messages: 0,
            last_interaction: 0,
            comfort_level: 1.0,
            familiarity: 1.0,
            communication_style: "intimate".to_string(),
            personality_observations: Vec::new(),
            interests_shared: Vec::new(),
            conversation_topics: HashMap::new(),
            emotional_memories: Vec::new(),
            special_notes: Vec::new(),
            mentioned_by: "system".to_string(),
            relationship_to_primary: "primary".to_string(),
            age_hints: Vec::new(),
            voice_profile: None,
        }
    }
    
    pub fn new_person(name: &str, relationship_type: RelationshipType, mentioned_by: &str, relationship_context: &str) -> Self {
        let (comfort_level, communication_style) = match relationship_type {
            RelationshipType::Family => (0.7, "warm"),
            RelationshipType::Friend => (0.6, "friendly"),
            RelationshipType::Acquaintance => (0.4, "polite"),
            RelationshipType::Stranger => (0.2, "cautious"),
            RelationshipType::PrimaryUser => (1.0, "intimate"),
        };
        
        Self {
            name: name.to_string(),
            canonical_name: name.to_lowercase(),
            relationship_type,
            first_met: crate::time_service::TimeService::current_timestamp(),
            total_conversations: 0,
            total_messages: 0,
            last_interaction: 0,
            comfort_level,
            familiarity: 0.1, // Start low, build over time
            communication_style: communication_style.to_string(),
            personality_observations: Vec::new(),
            interests_shared: Vec::new(),
            conversation_topics: HashMap::new(),
            emotional_memories: Vec::new(),
            special_notes: Vec::new(),
            mentioned_by: mentioned_by.to_string(),
            relationship_to_primary: relationship_context.to_string(),
            age_hints: Vec::new(),
            voice_profile: None,
        }
    }
    
    pub fn update_interaction(&mut self) {
        self.last_interaction = crate::time_service::TimeService::current_timestamp();
        self.total_messages += 1;
        
        // Gradually increase familiarity with each interaction
        self.familiarity = (self.familiarity + 0.05).min(1.0);
        
        // Comfort level grows more slowly
        if self.total_messages > 5 {
            self.comfort_level = (self.comfort_level + 0.02).min(1.0);
        }
    }
    
    pub fn add_personality_observation(&mut self, observation: &str) {
        if !self.personality_observations.contains(&observation.to_string()) {
            self.personality_observations.push(observation.to_string());
            debug_log!("👤 Added personality observation for {}: {}", self.name, observation);
        }
    }
    
    pub fn add_interest(&mut self, interest: &str) {
        if !self.interests_shared.contains(&interest.to_string()) {
            self.interests_shared.push(interest.to_string());
            debug_log!("🎯 Added interest for {}: {}", self.name, interest);
        }
    }
    
    pub fn record_topic(&mut self, topic: &str) {
        *self.conversation_topics.entry(topic.to_string()).or_insert(0) += 1;
    }
    
   pub fn train_voice(&mut self, voice_id: &str, voice_characteristics: VoiceCharacteristics) {
    let environment_quality = if voice_characteristics.duration_ms > 500.0 { 0.8 } else { 0.5 };
    
    // Ensure we have a voice description
    let voice_description = voice_characteristics.voice_description.clone()
        .unwrap_or_else(|| {
            // Generate a basic description if none provided
            VoiceDescription {
                pitch_character: "analyzing voice".to_string(),
                speaking_style: "natural speech".to_string(),
                voice_quality: "clear tone".to_string(),
                energy_profile: "moderate energy".to_string(),
                articulation_style: "normal articulation".to_string(),
                overall_impression: "Voice profile being analyzed".to_string(),
            }
        });
    
    let new_sample = VoiceSample {
        timestamp: voice_characteristics.timestamp,
        raw_features: voice_characteristics.raw_features.clone(),
        semantic_description: voice_description,
        environment_quality,
        transcript_length: voice_characteristics.transcript_length,
        duration_ms: voice_characteristics.duration_ms,
    };
    
    if let Some(ref mut profile) = self.voice_profile {
        profile.voice_samples.push(new_sample);
        
        // Update signature
        Self::update_voice_signature(profile);
        
        // Auto-calculate threshold
        Self::calculate_auto_threshold(profile);
        
        debug_log!("🎤 Added voice sample #{} for {} (quality: {:.2})", 
                  profile.voice_samples.len(), self.name, environment_quality);
    } else {
        // Create new profile
        let new_profile = VoiceProfile {
            voice_id: voice_id.to_string(),
            voice_samples: vec![new_sample],
            voice_signature: VoiceSignature {
				avg_features: voice_characteristics.raw_features,
				sample_count: 1,
			},
            confidence_threshold: 0.7,
            last_voice_detection: crate::time_service::TimeService::current_timestamp(),
            auto_threshold: 0.7,
        };
        
        self.voice_profile = Some(new_profile);
        debug_log!("🎤 Created voice profile for {}", self.name);
    }
}
    
    /// Update voice signature from all samples
    fn update_voice_signature(profile: &mut VoiceProfile) {
    if profile.voice_samples.is_empty() {
        return;
    }
    
    // Calculate average features
    let avg_features = PersonProfile::calculate_average_features_static(&profile.voice_samples);
    
    profile.voice_signature = VoiceSignature {
        avg_features,
        sample_count: profile.voice_samples.len(),
    };
}

fn calculate_average_features_static(samples: &[VoiceSample]) -> VoiceFeatures {
    let count = samples.len() as f32;
    let mut avg = VoiceFeatures {
        avg_pitch: 0.0,
        pitch_range: 0.0,
        pitch_stability: 0.0,
        speaking_rate: 0.0,
        pause_ratio: 0.0,
        rhythm_consistency: 0.0,
        spectral_brightness: 0.0,
        voice_clarity: 0.0,
        breathiness: 0.0,
        avg_energy: 0.0,
        energy_variation: 0.0,
        dynamic_range: 0.0,
        consonant_sharpness: 0.0,
        vowel_clarity: 0.0,
    };
    
    for sample in samples {
        avg.avg_pitch += sample.raw_features.avg_pitch;
        avg.pitch_range += sample.raw_features.pitch_range;
        avg.pitch_stability += sample.raw_features.pitch_stability;
        avg.speaking_rate += sample.raw_features.speaking_rate;
        avg.pause_ratio += sample.raw_features.pause_ratio;
        avg.rhythm_consistency += sample.raw_features.rhythm_consistency;
        avg.spectral_brightness += sample.raw_features.spectral_brightness;
        avg.voice_clarity += sample.raw_features.voice_clarity;
        avg.breathiness += sample.raw_features.breathiness;
        avg.avg_energy += sample.raw_features.avg_energy;
        avg.energy_variation += sample.raw_features.energy_variation;
        avg.dynamic_range += sample.raw_features.dynamic_range;
        avg.consonant_sharpness += sample.raw_features.consonant_sharpness;
        avg.vowel_clarity += sample.raw_features.vowel_clarity;
    }
    
    // Divide by count
    avg.avg_pitch /= count;
    avg.pitch_range /= count;
    avg.pitch_stability /= count;
    avg.speaking_rate /= count;
    avg.pause_ratio /= count;
    avg.rhythm_consistency /= count;
    avg.spectral_brightness /= count;
    avg.voice_clarity /= count;
    avg.breathiness /= count;
    avg.avg_energy /= count;
    avg.energy_variation /= count;
    avg.dynamic_range /= count;
    avg.consonant_sharpness /= count;
    avg.vowel_clarity /= count;
    
    avg
}
    
    /// Calculate automatic threshold based on sample consistency
    fn calculate_auto_threshold(profile: &mut VoiceProfile) {
    if profile.voice_samples.len() < 2 {
        profile.auto_threshold = 0.65;
        return;
    }
    
    // Calculate similarities between all pairs of samples
    let mut similarities = Vec::new();
    
    for i in 0..profile.voice_samples.len() {
        for j in i+1..profile.voice_samples.len() {
            let sim = Self::calculate_sample_similarity(
                &profile.voice_samples[i],
                &profile.voice_samples[j]
            );
            similarities.push(sim);
        }
    }
    
    if similarities.is_empty() {
        profile.auto_threshold = 0.65;
        return;
    }
    
    // Sort similarities
    similarities.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Use the 10th percentile as threshold
    let idx = similarities.len() / 10;
    let threshold = similarities[idx] * 0.95;
    
    profile.auto_threshold = threshold.max(0.70).min(0.85);  // 70-85% match
    profile.confidence_threshold = profile.auto_threshold;
    
    debug_log!("🎯 Auto-threshold calculated: {:.3} (from {} sample pairs)", 
              profile.auto_threshold, similarities.len());
}

fn calculate_sample_similarity(sample1: &VoiceSample, sample2: &VoiceSample) -> f32 {
    // Compare raw features
    let feature_sim = Self::compare_raw_features(&sample1.raw_features, &sample2.raw_features);
    
    // Compare semantic descriptions
    let semantic_sim = Self::compare_descriptions(&sample1.semantic_description, &sample2.semantic_description);
    
    // Weighted combination
    feature_sim * 0.6 + semantic_sim * 0.4
}

fn compare_raw_features(features1: &VoiceFeatures, features2: &VoiceFeatures) -> f32 {
    let mut similarities = Vec::new();
    
    // Pitch comparison
    let pitch_sim = 1.0 - (features1.avg_pitch - features2.avg_pitch).abs() / 200.0;
    similarities.push(pitch_sim.max(0.0));
    
    // Speaking rate
    let rate_sim = 1.0 - (features1.speaking_rate - features2.speaking_rate).abs() / 3.0;
    similarities.push(rate_sim.max(0.0));
    
    // Voice quality
    let brightness_sim = 1.0 - (features1.spectral_brightness - features2.spectral_brightness).abs();
    similarities.push(brightness_sim.max(0.0));
    
    let clarity_sim = 1.0 - (features1.voice_clarity - features2.voice_clarity).abs();
    similarities.push(clarity_sim.max(0.0));
    
    // Energy
    let energy_sim = 1.0 - (features1.avg_energy - features2.avg_energy).abs();
    similarities.push(energy_sim.max(0.0));
    
    similarities.iter().sum::<f32>() / similarities.len() as f32
}

fn compare_descriptions(desc1: &VoiceDescription, desc2: &VoiceDescription) -> f32 {
    let mut score = 0.0;
    let mut comparisons = 0.0;
    
    if Self::semantic_similarity(&desc1.pitch_character, &desc2.pitch_character) > 0.7 {
        score += 1.0;
    }
    comparisons += 1.0;
    
    if Self::semantic_similarity(&desc1.speaking_style, &desc2.speaking_style) > 0.6 {
        score += 0.8;
    }
    comparisons += 0.8;
    
    if Self::semantic_similarity(&desc1.voice_quality, &desc2.voice_quality) > 0.6 {
        score += 0.7;
    }
    comparisons += 0.7;
    
    if Self::semantic_similarity(&desc1.energy_profile, &desc2.energy_profile) > 0.5 {
        score += 0.5;
    }
    comparisons += 0.5;
    
    score / comparisons
}

    
    /// Update voice detection timestamp
    pub fn update_voice_detection(&mut self) {
        if let Some(ref mut voice_profile) = self.voice_profile {
            voice_profile.last_voice_detection = crate::time_service::TimeService::current_timestamp();
        }
    }
    
    pub fn get_greeting_style(&self) -> &str {
        match self.relationship_type {
            RelationshipType::PrimaryUser => "intimate",
            RelationshipType::Family => {
                if self.age_hints.iter().any(|hint| hint.contains("child") || hint.contains("young")) {
                    "gentle_excited"
                } else {
                    "warm_family"
                }
            },
            RelationshipType::Friend => "friendly_warm",
            RelationshipType::Acquaintance => "polite_interested",
            RelationshipType::Stranger => "curious_welcoming",
        }
    }
	
	
	
	pub fn matches_voice(&self, voice_characteristics: &VoiceCharacteristics, _confidence: f32) -> bool {
    if let Some(ref profile) = self.voice_profile {
        let similarity = self.calculate_voice_similarity(voice_characteristics, profile);
        
        // Log incoming voice characteristics
        debug_log!("🎵 Incoming voice - Pitch: {:.1}Hz, Rate: {:.2}, Brightness: {:.2}", 
                  voice_characteristics.raw_features.avg_pitch,
                  voice_characteristics.raw_features.speaking_rate,
                  voice_characteristics.raw_features.spectral_brightness);
        
        // Log profile characteristics
        if let Some(first_sample) = profile.voice_samples.first() {
            debug_log!("📊 Profile baseline - Pitch: {:.1}Hz, Rate: {:.2}, Brightness: {:.2}", 
                      first_sample.raw_features.avg_pitch,
                      first_sample.raw_features.speaking_rate,
                      first_sample.raw_features.spectral_brightness);
        }
        
        debug_log!("🎯 Voice match for {}: {:.1}% (threshold: {:.1}%)", 
                  self.name, similarity * 100.0, profile.auto_threshold * 100.0);
        
        similarity >= profile.auto_threshold
    } else {
        false
    }
}

fn calculate_voice_similarity(&self, incoming: &VoiceCharacteristics, profile: &VoiceProfile) -> f32 {
    if profile.voice_samples.is_empty() {
        return 0.0;
    }
    
    // Method 1: Compare raw features (60% weight)
    let feature_similarity = self.compare_voice_features(incoming, profile);
    
    // Method 2: Compare semantic descriptions (40% weight)
    let semantic_similarity = if let Some(ref incoming_desc) = incoming.voice_description {
        self.compare_semantic_descriptions(incoming_desc, profile)
    } else {
        0.0
    };
    
    debug_log!("📊 Feature similarity: {:.1}%, Semantic similarity: {:.1}%",
              feature_similarity * 100.0, semantic_similarity * 100.0);
    
    // Weighted combination
    // If pitch is way off, reject immediately
	let pitch_diff = (incoming.raw_features.avg_pitch - profile.voice_signature.avg_features.avg_pitch).abs();
	let pitch_relative_diff = pitch_diff / profile.voice_signature.avg_features.avg_pitch;

	if pitch_relative_diff > 0.3 {  // More than 30% pitch difference
		debug_log!("🚫 Pitch difference too large: {:.1}% - automatic rejection", pitch_relative_diff * 100.0);
		return 0.0;  // Instant fail
	}

	// Otherwise use weighted combination
	feature_similarity * 0.6 + semantic_similarity * 0.4
}

fn compare_voice_features(&self, incoming: &VoiceCharacteristics, profile: &VoiceProfile) -> f32 {
    // Get the average features from all samples
    let avg_features = self.calculate_average_features(profile);
    
    // Compare each feature dimension
    let mut similarities = Vec::new();
    
    // Pitch comparison - reasonable strictness
	let pitch_diff = (incoming.raw_features.avg_pitch - avg_features.avg_pitch).abs();
	let pitch_relative_diff = pitch_diff / avg_features.avg_pitch; // Percentage difference

	let pitch_sim = if pitch_relative_diff < 0.1 {
		1.0 - pitch_relative_diff * 5.0  // Within 10% = good match
	} else if pitch_relative_diff < 0.2 {
		0.5 - (pitch_relative_diff - 0.1) * 3.0  // 10-20% = partial match
	} else {
		0.0  // Beyond 20% difference = no match
	};

	debug_log!("   Pitch: {:.1}Hz vs {:.1}Hz (diff: {:.1}% -> sim: {:.2})", 
			  incoming.raw_features.avg_pitch, avg_features.avg_pitch, 
			  pitch_relative_diff * 100.0, pitch_sim);
    similarities.push(pitch_sim);
    
    debug_log!("   Pitch diff: {:.1}Hz -> similarity: {:.2}", pitch_diff, pitch_sim);
    
    // Speaking rate comparison
    let rate_sim = 1.0 - (incoming.raw_features.speaking_rate - avg_features.speaking_rate).abs() / 3.0;
    similarities.push(rate_sim.max(0.0));
    
    // Voice quality comparisons
    let brightness_sim = 1.0 - (incoming.raw_features.spectral_brightness - avg_features.spectral_brightness).abs();
    similarities.push(brightness_sim.max(0.0));
    
    let clarity_sim = 1.0 - (incoming.raw_features.voice_clarity - avg_features.voice_clarity).abs();
    similarities.push(clarity_sim.max(0.0));
    
    // Energy comparison
    let energy_sim = 1.0 - (incoming.raw_features.avg_energy - avg_features.avg_energy).abs();
    similarities.push(energy_sim.max(0.0));
    
    // Average all similarities
    similarities.iter().sum::<f32>() / similarities.len() as f32
}

fn calculate_average_features(&self, profile: &VoiceProfile) -> VoiceFeatures {
    let count = profile.voice_samples.len() as f32;
    let mut avg = VoiceFeatures {
        avg_pitch: 0.0,
        pitch_range: 0.0,
        pitch_stability: 0.0,
        speaking_rate: 0.0,
        pause_ratio: 0.0,
        rhythm_consistency: 0.0,
        spectral_brightness: 0.0,
        voice_clarity: 0.0,
        breathiness: 0.0,
        avg_energy: 0.0,
        energy_variation: 0.0,
        dynamic_range: 0.0,
        consonant_sharpness: 0.0,
        vowel_clarity: 0.0,
    };
    
    for sample in &profile.voice_samples {
        avg.avg_pitch += sample.raw_features.avg_pitch;
        avg.pitch_range += sample.raw_features.pitch_range;
        avg.pitch_stability += sample.raw_features.pitch_stability;
        avg.speaking_rate += sample.raw_features.speaking_rate;
        avg.pause_ratio += sample.raw_features.pause_ratio;
        avg.rhythm_consistency += sample.raw_features.rhythm_consistency;
        avg.spectral_brightness += sample.raw_features.spectral_brightness;
        avg.voice_clarity += sample.raw_features.voice_clarity;
        avg.breathiness += sample.raw_features.breathiness;
        avg.avg_energy += sample.raw_features.avg_energy;
        avg.energy_variation += sample.raw_features.energy_variation;
        avg.dynamic_range += sample.raw_features.dynamic_range;
        avg.consonant_sharpness += sample.raw_features.consonant_sharpness;
        avg.vowel_clarity += sample.raw_features.vowel_clarity;
    }
    
    // Divide by count
    avg.avg_pitch /= count;
    avg.pitch_range /= count;
    avg.pitch_stability /= count;
    avg.speaking_rate /= count;
    avg.pause_ratio /= count;
    avg.rhythm_consistency /= count;
    avg.spectral_brightness /= count;
    avg.voice_clarity /= count;
    avg.breathiness /= count;
    avg.avg_energy /= count;
    avg.energy_variation /= count;
    avg.dynamic_range /= count;
    avg.consonant_sharpness /= count;
    avg.vowel_clarity /= count;
    
    avg
}

fn compare_semantic_descriptions(&self, incoming: &VoiceDescription, profile: &VoiceProfile) -> f32 {
    // Get the most recent descriptions from profile
    let recent_descriptions: Vec<&VoiceDescription> = profile.voice_samples
        .iter()
        .rev()
        .take(5)
        .map(|s| &s.semantic_description)
        .collect();
    
    if recent_descriptions.is_empty() {
        return 0.0;
    }
    
    // Compare against each recent description and take the best match
    let mut best_match: f32 = 0.0;
    
    for desc in recent_descriptions {
        let mut score = 0.0;
        let mut comparisons = 0.0;
        
        // Compare pitch character
        if Self::semantic_similarity(&incoming.pitch_character, &desc.pitch_character) > 0.7 {
            score += 1.0;
        }
        comparisons += 1.0;
        
        // Compare speaking style
        if Self::semantic_similarity(&incoming.speaking_style, &desc.speaking_style) > 0.6 {
            score += 0.8;
        }
        comparisons += 0.8;
        
        // Compare voice quality
        if Self::semantic_similarity(&incoming.voice_quality, &desc.voice_quality) > 0.6 {
            score += 0.7;
        }
        comparisons += 0.7;
        
        // Compare energy profile
        if Self::semantic_similarity(&incoming.energy_profile, &desc.energy_profile) > 0.5 {
            score += 0.5;
        }
        comparisons += 0.5;
        
        let match_score = score / comparisons;
        best_match = best_match.max(match_score);
    }
    
    best_match
}

fn semantic_similarity(text1: &str, text2: &str) -> f32 {
    // Simple word overlap similarity
    let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
    let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();
    
    let intersection = words1.intersection(&words2).count() as f32;
    let union = words1.union(&words2).count() as f32;
    
    if union > 0.0 {
        intersection / union
    } else {
        0.0
    }
}
	
    
    
    /// Get voice similarity score (for the backend)
pub fn get_voice_similarity(&self, voice_characteristics: &VoiceCharacteristics) -> f32 {
    if let Some(ref profile) = self.voice_profile {
        self.calculate_voice_similarity(voice_characteristics, profile)
    } else {
        0.0
    }
}

/// Enhanced voice training method for Resemblyzer integration
    pub fn train_voice_resemblyzer(&mut self, voice_id: &str, voice_characteristics: VoiceCharacteristics) {
        let environment_quality = if voice_characteristics.duration_ms > 500.0 { 0.9 } else { 0.7 };
        
        // Ensure we have a voice description from Resemblyzer
        let voice_description = voice_characteristics.voice_description.clone()
            .unwrap_or_else(|| {
                // Fallback description if Resemblyzer didn't provide one
                VoiceDescription {
                    pitch_character: "voice profile being analyzed".to_string(),
                    speaking_style: "natural speech patterns".to_string(),
                    voice_quality: "distinctive voice characteristics".to_string(),
                    energy_profile: "balanced energy levels".to_string(),
                    articulation_style: "clear articulation".to_string(),
                    overall_impression: format!("Resemblyzer voice profile for {}", self.name),
                }
            });
        
        let new_sample = VoiceSample {
            timestamp: voice_characteristics.timestamp,
            raw_features: voice_characteristics.raw_features.clone(),
            semantic_description: voice_description,
            environment_quality,
            transcript_length: voice_characteristics.transcript_length,
            duration_ms: voice_characteristics.duration_ms,
        };
        
        if let Some(ref mut profile) = self.voice_profile {
            profile.voice_samples.push(new_sample);
            
            // Update signature with new averaging
            Self::update_voice_signature_resemblyzer(profile);
            
            // Set higher confidence threshold for Resemblyzer (it's more reliable)
            profile.auto_threshold = 0.75; // Match Python script threshold
            profile.confidence_threshold = 0.75;
            
            debug_log!("🎤 Added Resemblyzer voice sample #{} for {} (quality: {:.2})", 
                      profile.voice_samples.len(), self.name, environment_quality);
        } else {
            // Create new profile with Resemblyzer settings
            let new_profile = VoiceProfile {
                voice_id: format!("resemblyzer_{}", voice_id),
                voice_samples: vec![new_sample],
                voice_signature: VoiceSignature {
                    avg_features: voice_characteristics.raw_features,
                    sample_count: 1,
                },
                confidence_threshold: 0.75, // Resemblyzer threshold
                last_voice_detection: crate::time_service::TimeService::current_timestamp(),
                auto_threshold: 0.75,
            };
            
            self.voice_profile = Some(new_profile);
            debug_log!("🎤 Created Resemblyzer voice profile for {}", self.name);
        }
    }
    
    /// Update voice signature specifically for Resemblyzer data
    fn update_voice_signature_resemblyzer(profile: &mut VoiceProfile) {
        if profile.voice_samples.is_empty() {
            return;
        }
        
        // Calculate weighted average (recent samples have more weight)
        let total_samples = profile.voice_samples.len();
        let mut weighted_features = VoiceFeatures::default();
        let mut total_weight = 0.0;
        
        for (i, sample) in profile.voice_samples.iter().enumerate() {
            // Recent samples get higher weight
            let weight = 1.0 + (i as f32 * 0.1); // Linear increase
            total_weight += weight;
            
            weighted_features.avg_pitch += sample.raw_features.avg_pitch * weight;
            weighted_features.pitch_range += sample.raw_features.pitch_range * weight;
            weighted_features.pitch_stability += sample.raw_features.pitch_stability * weight;
            weighted_features.speaking_rate += sample.raw_features.speaking_rate * weight;
            weighted_features.pause_ratio += sample.raw_features.pause_ratio * weight;
            weighted_features.rhythm_consistency += sample.raw_features.rhythm_consistency * weight;
            weighted_features.spectral_brightness += sample.raw_features.spectral_brightness * weight;
            weighted_features.voice_clarity += sample.raw_features.voice_clarity * weight;
            weighted_features.breathiness += sample.raw_features.breathiness * weight;
            weighted_features.avg_energy += sample.raw_features.avg_energy * weight;
            weighted_features.energy_variation += sample.raw_features.energy_variation * weight;
            weighted_features.dynamic_range += sample.raw_features.dynamic_range * weight;
            weighted_features.consonant_sharpness += sample.raw_features.consonant_sharpness * weight;
            weighted_features.vowel_clarity += sample.raw_features.vowel_clarity * weight;
        }
        
        // Normalize by total weight
        weighted_features.avg_pitch /= total_weight;
        weighted_features.pitch_range /= total_weight;
        weighted_features.pitch_stability /= total_weight;
        weighted_features.speaking_rate /= total_weight;
        weighted_features.pause_ratio /= total_weight;
        weighted_features.rhythm_consistency /= total_weight;
        weighted_features.spectral_brightness /= total_weight;
        weighted_features.voice_clarity /= total_weight;
        weighted_features.breathiness /= total_weight;
        weighted_features.avg_energy /= total_weight;
        weighted_features.energy_variation /= total_weight;
        weighted_features.dynamic_range /= total_weight;
        weighted_features.consonant_sharpness /= total_weight;
        weighted_features.vowel_clarity /= total_weight;
        
        profile.voice_signature = VoiceSignature {
            avg_features: weighted_features,
            sample_count: total_samples,
        };
        
        debug_log!("📊 Updated Resemblyzer voice signature for {} samples", total_samples);
    }
    
    /// Enhanced voice matching for Resemblyzer integration
    pub fn matches_voice_resemblyzer(&self, voice_characteristics: &VoiceCharacteristics, resemblyzer_confidence: f32) -> bool {
        if let Some(ref profile) = self.voice_profile {
            // For Resemblyzer, we trust the Python script's confidence more than our own calculations
            if resemblyzer_confidence >= 0.75 {
                debug_log!("🎯 Resemblyzer confident match for {}: {:.1}%", 
                          self.name, resemblyzer_confidence * 100.0);
                return true;
            }
            
            // Fallback to feature comparison if Resemblyzer confidence is borderline
            if resemblyzer_confidence >= 0.65 {
                let feature_similarity = self.calculate_feature_similarity_resemblyzer(voice_characteristics, profile);
                debug_log!("🔍 Resemblyzer borderline, checking features for {}: {:.1}% (threshold: 70%)", 
                          self.name, feature_similarity * 100.0);
                return feature_similarity >= 0.70;
            }
            
            debug_log!("❌ Resemblyzer confidence too low for {}: {:.1}%", 
                      self.name, resemblyzer_confidence * 100.0);
            false
        } else {
            false
        }
    }
    
    /// Calculate feature similarity specifically for Resemblyzer data
    fn calculate_feature_similarity_resemblyzer(&self, incoming: &VoiceCharacteristics, profile: &VoiceProfile) -> f32 {
        if profile.voice_samples.is_empty() {
            return 0.0;
        }
        
        let avg_features = &profile.voice_signature.avg_features;
        let mut similarities = Vec::new();
        
        // Pitch comparison (most important for voice identity)
        let pitch_diff = (incoming.raw_features.avg_pitch - avg_features.avg_pitch).abs();
        let pitch_relative_diff = pitch_diff / avg_features.avg_pitch;
        let pitch_sim = if pitch_relative_diff < 0.15 {
            1.0 - pitch_relative_diff * 3.0
        } else {
            0.0
        };
        similarities.push(pitch_sim * 2.0); // Double weight for pitch
        
        // Speaking rate comparison
        let rate_sim = 1.0 - (incoming.raw_features.speaking_rate - avg_features.speaking_rate).abs() / 4.0;
        similarities.push(rate_sim.max(0.0));
        
        // Voice quality comparisons
        let brightness_sim = 1.0 - (incoming.raw_features.spectral_brightness - avg_features.spectral_brightness).abs();
        similarities.push(brightness_sim.max(0.0));
        
        let clarity_sim = 1.0 - (incoming.raw_features.voice_clarity - avg_features.voice_clarity).abs();
        similarities.push(clarity_sim.max(0.0));
        
        // Energy comparison
        let energy_sim = 1.0 - (incoming.raw_features.avg_energy - avg_features.avg_energy).abs();
        similarities.push(energy_sim.max(0.0));
        
        // Calculate weighted average
        let total_weight = similarities.len() as f32 + 1.0; // +1 for double-weighted pitch
        similarities.iter().sum::<f32>() / total_weight
    }
	
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonRecognitionSystem {
    pub people: HashMap<String, PersonProfile>,  // canonical_name -> profile
    pub current_speaker: String,                  // Who is currently talking
    pub conversation_transitions: Vec<ConversationTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTransition {
    pub timestamp: u64,
    pub from_person: String,
    pub to_person: String,
    pub context: String,  // How the transition happened
}

impl PersonRecognitionSystem {
    pub fn new() -> Self {
        let mut system = Self {
            people: HashMap::new(),
            current_speaker: "aurora".to_string(),
            conversation_transitions: Vec::new(),
        };
        
        // Initialize Aurora as primary user
        let aurora_profile = PersonProfile::new_primary_user("Aurora");
        system.people.insert("aurora".to_string(), aurora_profile);
        
        system
    }
    
    pub fn load_or_create() -> Self {
        let people_path = get_data_path("people_profiles.json");
        
        if std::path::Path::new(&people_path).exists() {
            if let Ok(content) = fs::read_to_string(&people_path) {
                if let Ok(system) = serde_json::from_str(&content) {
                    debug_log!("👥 Loaded person recognition system with {} people", 
                              serde_json::from_str::<PersonRecognitionSystem>(&content)
                                  .unwrap_or_else(|_| PersonRecognitionSystem::new())
                                  .people.len());
                    return system;
                }
            }
        }
        
        debug_log!("👥 Creating new person recognition system");
        Self::new()
    }
    
    pub fn save(&self) -> Result<(), String> {
        let people_path = get_data_path("people_profiles.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize person system: {}", e))?;
        
        fs::write(&people_path, content)
            .map_err(|e| format!("Failed to write person profiles: {}", e))?;
        
        debug_log!("👥 Saved person recognition system with {} people", self.people.len());
        Ok(())
    }
    
    /// Analyze a message to detect if someone new is being introduced or speaking
    /// Now includes voice recognition support
    pub fn analyze_message(&mut self, message: &str, voice_data: Option<VoiceDetectionData>) -> Option<PersonTransition> {
        // First check voice recognition if available
        if let Some(voice_data) = voice_data {
            if let Some(voice_transition) = self.check_voice_recognition(&voice_data) {
                debug_log!("🎤 Voice recognition triggered speaker change");
                return Some(voice_transition);
            }
        }
        
        // Then check text-based detection (existing logic)
        self.analyze_text_message(message)
    }
    
    /// Check if voice data indicates a speaker change
    fn check_voice_recognition(&mut self, voice_data: &VoiceDetectionData) -> Option<PersonTransition> {
        // Check each known person's voice profile
        for (canonical_name, person) in &mut self.people {
            if person.matches_voice(&voice_data.characteristics, voice_data.confidence) {
                // Voice matches this person
                if canonical_name != &self.current_speaker {
                    // Speaker change detected via voice
                    person.update_voice_detection();
                    
                    let transition = PersonTransition {
                        new_speaker: canonical_name.clone(),
                        old_speaker: self.current_speaker.clone(),
                        introduction_context: format!("Voice recognition detected {} speaking", person.name),
                        is_new_person: false,
                    };
                    
                    // Record the transition
                    let conversation_transition = ConversationTransition {
                        timestamp: crate::time_service::TimeService::current_timestamp(),
                        from_person: self.current_speaker.clone(),
                        to_person: canonical_name.clone(),
                        context: "Voice recognition".to_string(),
                    };
                    self.conversation_transitions.push(conversation_transition);
                    
                    // Update current speaker
                    self.current_speaker = canonical_name.clone();
                    
                    return Some(transition);
                }
                break;
            }
        }
        
        None
    }
    
    /// Existing text-based message analysis (renamed for clarity)
    fn analyze_text_message(&mut self, message: &str) -> Option<PersonTransition> {
		
		// First try smart detection
        if let Some((name, relationship, context)) = self.detect_person_mention(message) {
            let canonical_name = name.to_lowercase();
            
            // Skip if it's already the current speaker
            if canonical_name != self.current_speaker && canonical_name != "lyra" {
                debug_log!("🎯 Smart detection found: {} ({})", name, relationship);
                
                // Determine relationship type from the relationship word
                let relationship_type = match relationship.as_str() {
                    "son" | "daughter" | "child" | "kid" | "brother" | "sister" 
                    | "mom" | "dad" | "mother" | "father" | "cousin" | "nephew" 
                    | "niece" | "aunt" | "uncle" | "grandma" | "grandpa" => RelationshipType::Family,
                    "friend" => RelationshipType::Friend,
                    "colleague" | "roommate" => RelationshipType::Acquaintance,
                    _ => RelationshipType::Stranger,
                };
                
                // Create or update person profile
                if !self.people.contains_key(&canonical_name) {
                    let profile = PersonProfile::new_person(
                        &name, 
                        relationship_type, 
                        &self.current_speaker,
                        &relationship
                    );
                    self.people.insert(canonical_name.clone(), profile);
                    debug_log!("👤 Created new person profile via smart detection: {}", name);
                }
                
                // Record the transition
                let transition = ConversationTransition {
                    timestamp: crate::time_service::TimeService::current_timestamp(),
                    from_person: self.current_speaker.clone(),
                    to_person: canonical_name.clone(),
                    context: context,
                };
                self.conversation_transitions.push(transition);
                
                // Update current speaker
                let old_speaker = self.current_speaker.clone();
                self.current_speaker = canonical_name.clone();
                
                let _ = self.save();
                
                return Some(PersonTransition {
                    new_speaker: canonical_name,
                    old_speaker,
                    introduction_context: message.to_string(),
                    is_new_person: true,
                });
            }
        }
		
        // Check for introduction patterns
        for pattern in PERSON_INTRODUCTION_PATTERNS.iter() {
            if let Some(captures) = pattern.captures(message) {
                if let Some(name_match) = captures.get(1) {
                    let name = name_match.as_str();
                    let canonical_name = name.to_lowercase();
                    
                    // Skip if it's already the current speaker
                    if canonical_name == self.current_speaker {
                        continue;
                    }
                    
                    // Determine relationship type from context
                    let relationship_type = self.determine_relationship_type(message, name);
                    let relationship_context = self.extract_relationship_context(message);
                    
                    // Create or update person profile
                    if !self.people.contains_key(&canonical_name) {
                        let profile = PersonProfile::new_person(
                            name, 
                            relationship_type, 
                            &self.current_speaker,
                            &relationship_context
                        );
                        self.people.insert(canonical_name.clone(), profile);
                        debug_log!("👤 Created new person profile: {}", name);
                    }
                    
                    // Record the transition
                    let transition = ConversationTransition {
                        timestamp: crate::time_service::TimeService::current_timestamp(),
                        from_person: self.current_speaker.clone(),
                        to_person: canonical_name.clone(),
                        context: message.to_string(),
                    };
                    self.conversation_transitions.push(transition);
                    
                    // Update current speaker
                    let old_speaker = self.current_speaker.clone();
                    self.current_speaker = canonical_name.clone();
                    
                    debug_log!("👥 Person transition: {} -> {}", old_speaker, name);
                    
                    let is_new_person = !self.people.contains_key(&canonical_name);
                    
                    return Some(PersonTransition {
                        new_speaker: canonical_name,
                        old_speaker,
                        introduction_context: message.to_string(),
                        is_new_person,
                    });
                }
            }
        }
        
        // Check for return to Aurora
        if message.to_lowercase().contains("aurora") && message.to_lowercase().contains("back") {
            if self.current_speaker != "aurora" {
                let old_speaker = self.current_speaker.clone();
                self.current_speaker = "aurora".to_string();
                
                debug_log!("👥 Returned to Aurora from {}", old_speaker);
                
                return Some(PersonTransition {
                    new_speaker: "aurora".to_string(),
                    old_speaker,
                    introduction_context: message.to_string(),
                    is_new_person: false,
                });
            }
        }
        
        None
    }
    
    fn determine_relationship_type(&self, message: &str, _name: &str) -> RelationshipType {
        let message_lower = message.to_lowercase();
        
        if message_lower.contains("son") || message_lower.contains("daughter") || 
           message_lower.contains("child") || message_lower.contains("kid") {
            RelationshipType::Family
        } else if message_lower.contains("friend") {
            RelationshipType::Friend
        } else if message_lower.contains("colleague") || message_lower.contains("coworker") {
            RelationshipType::Acquaintance
        } else {
            RelationshipType::Stranger
        }
    }
    
    fn extract_relationship_context(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();
        
        if message_lower.contains("son") { "son".to_string() }
        else if message_lower.contains("daughter") { "daughter".to_string() }
        else if message_lower.contains("child") || message_lower.contains("kid") { "child".to_string() }
        else if message_lower.contains("friend") { "friend".to_string() }
        else if message_lower.contains("partner") { "partner".to_string() }
        else if message_lower.contains("husband") { "husband".to_string() }
        else if message_lower.contains("wife") { "wife".to_string() }
        else { "acquaintance".to_string() }
    }
    
    pub fn get_current_person(&self) -> Option<&PersonProfile> {
        self.people.get(&self.current_speaker)
    }
    
    pub fn get_current_person_mut(&mut self) -> Option<&mut PersonProfile> {
        self.people.get_mut(&self.current_speaker)
    }
    
    pub fn record_message(&mut self, message: &str) {
        // Extract topics first (immutable borrow)
        let topics = self.extract_topics_from_message(message);
        
        // Then get mutable reference and update
        if let Some(person) = self.get_current_person_mut() {
            person.update_interaction();
            
            for topic in topics {
                person.record_topic(&topic);
            }
        }
    }
    
    fn extract_topics_from_message(&self, message: &str) -> Vec<String> {
        // Simple topic extraction - could be enhanced later
        let mut topics = Vec::new();
        let message_lower = message.to_lowercase();
        
        // Common topics
        let topic_keywords = [
            "music", "art", "programming", "games", "school", "work", 
            "family", "friends", "creative", "drawing", "writing",
            "movies", "books", "science", "math", "history"
        ];
        
        for topic in topic_keywords {
            if message_lower.contains(topic) {
                topics.push(topic.to_string());
            }
        }
        
        topics
    }
    
    pub fn get_person_context_for_prompt(&self) -> String {
        if let Some(current_person) = self.get_current_person() {
            if current_person.canonical_name == "aurora" {
                // Standard Aurora context
                return String::new();
            }
            
            // Context for other people
            let mut context_parts = Vec::new();
            
            context_parts.push(format!("🗣️ **Currently talking to: {}**", current_person.name));
            context_parts.push(format!("👥 **Relationship**: {} ({})", 
                current_person.relationship_to_primary,
                match current_person.relationship_type {
                    RelationshipType::Family => "family member",
                    RelationshipType::Friend => "friend", 
                    RelationshipType::Acquaintance => "acquaintance",
                    RelationshipType::Stranger => "new person",
                    RelationshipType::PrimaryUser => "primary user",
                }));
            
            context_parts.push(format!("🤝 **Comfort level**: {:.1}/10, **Familiarity**: {:.1}/10", 
                current_person.comfort_level * 10.0, current_person.familiarity * 10.0));
            
            if current_person.total_messages > 0 {
                context_parts.push(format!("💬 **Interaction history**: {} messages across {} conversations", 
                    current_person.total_messages, current_person.total_conversations));
            } else {
                context_parts.push("💬 **First time meeting this person**".to_string());
            }
            
            if !current_person.interests_shared.is_empty() {
                context_parts.push(format!("🎯 **Shared interests**: {}", 
                    current_person.interests_shared.join(", ")));
            }
            
            if !current_person.personality_observations.is_empty() {
                context_parts.push(format!("👤 **What you know about them**: {}", 
                    current_person.personality_observations.join(", ")));
            }
            
            // Add greeting style guidance
            let greeting_guidance = match current_person.get_greeting_style() {
                "gentle_excited" => "Be gentle but excited - this might be a child",
                "warm_family" => "Be warm and familial - treat them like family",
                "friendly_warm" => "Be friendly and welcoming - they're a friend",
                "polite_interested" => "Be polite and show genuine interest in getting to know them",
                "curious_welcoming" => "Be curious and welcoming - make them feel comfortable",
                _ => "Adjust your communication style appropriately",
            };
            
            context_parts.push(format!("🎭 **Communication approach**: {}", greeting_guidance));
            
            format!("## 👥 PERSON CONTEXT\n{}\n", context_parts.join("\n"))
        } else {
            String::new()
        }
    }
    
    /// Train voice recognition for a specific person
    pub fn train_person_voice(&mut self, person_name: &str, voice_data: VoiceDetectionData) -> Result<String, String> {
        let canonical_name = person_name.to_lowercase();
        
        if let Some(person) = self.people.get_mut(&canonical_name) {
            person.train_voice(&voice_data.voice_id, voice_data.characteristics);
            let person_name_for_message = person.name.clone(); // Clone name before dropping borrow
            drop(person); // Explicitly drop the mutable borrow
            
            self.save()?;
            
            Ok(format!("Voice training completed for {}", person_name_for_message))
        } else {
            Err(format!("Person '{}' not found in profiles", person_name))
        }
    }
    
    /// Find person by voice characteristics
    pub fn identify_speaker_by_voice(&self, voice_data: &VoiceDetectionData) -> Option<String> {
        for (canonical_name, person) in &self.people {
            if person.matches_voice(&voice_data.characteristics, voice_data.confidence) {
                return Some(canonical_name.clone());
            }
        }
        None
    }
    
    /// Get voice training status for all people
    pub fn get_voice_training_status(&self) -> HashMap<String, VoiceTrainingStatus> {
    let mut status = HashMap::new();
    
    debug_log!("🔍 Getting voice training status for {} people", self.people.len());
    
    for (canonical_name, person) in &self.people {
        debug_log!("👤 Checking person: {} (canonical: {})", person.name, canonical_name);
        
        let voice_status = if let Some(ref voice_profile) = person.voice_profile {
            debug_log!("🎤 {} has voice profile with {} samples", person.name, voice_profile.voice_samples.len());
            VoiceTrainingStatus {
                has_voice_profile: true,
                training_samples: voice_profile.voice_samples.len() as u32,
                confidence_threshold: voice_profile.auto_threshold,
                last_detection: Some(voice_profile.last_voice_detection),
            }
        } else {
            debug_log!("❌ {} has no voice profile", person.name);
            VoiceTrainingStatus {
                has_voice_profile: false,
                training_samples: 0,
                confidence_threshold: 0.0,
                last_detection: None,
            }
        };
        
        // Use the person's actual name, not canonical_name
        status.insert(person.name.clone(), voice_status);
    }
    
    debug_log!("📊 Returning voice status for {} people", status.len());
    status
}

pub fn detect_person_mention(&self, message: &str) -> Option<(String, String, String)> {
        let message_lower = message.to_lowercase();
        
        // List of relationship keywords
        let relationships = vec![
            "son", "daughter", "child", "kid", "friend", "partner", 
            "husband", "wife", "brother", "sister", "mom", "dad", 
            "mother", "father", "cousin", "nephew", "niece", "aunt", 
            "uncle", "grandma", "grandpa", "colleague", "roommate"
        ];
        
        // Look for relationship keywords
        for relationship in &relationships {
            if message_lower.contains(relationship) {
                // Extract potential names near the relationship word
                let words: Vec<&str> = message.split_whitespace().collect();
                
                for (i, word) in words.iter().enumerate() {
                    if word.to_lowercase() == *relationship {
                        // Check words before and after
                        if i + 1 < words.len() {
                            let potential_name = words[i + 1];
                            // Check if it's capitalized (likely a name)
                            if potential_name.chars().next().unwrap_or(' ').is_uppercase() {
                                return Some((
                                    potential_name.to_string(),
                                    relationship.to_string(),
                                    message.to_string()
                                ));
                            }
                        }
                        if i > 0 {
                            let potential_name = words[i - 1];
                            if potential_name.chars().next().unwrap_or(' ').is_uppercase() {
                                return Some((
                                    potential_name.to_string(),
                                    relationship.to_string(),
                                    message.to_string()
                                ));
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

/// Enhanced voice training method for Resemblyzer
    pub fn train_person_voice_resemblyzer(&mut self, person_name: &str, voice_data: VoiceDetectionData) -> Result<String, String> {
        let canonical_name = person_name.to_lowercase();
        
        if let Some(person) = self.people.get_mut(&canonical_name) {
            person.train_voice_resemblyzer(&voice_data.voice_id, voice_data.characteristics);
            
            // Get the sample count before saving
            let sample_count = person.voice_profile.as_ref()
                .map(|vp| vp.voice_samples.len())
                .unwrap_or(0);
            let person_name_for_message = person.name.clone();
            
            // Drop the mutable borrow before calling save
            drop(person);
            
            self.save()?;
            
            Ok(format!("Resemblyzer voice training completed for {} (Sample #{})", 
                      person_name_for_message, sample_count))
        } else {
            Err(format!("Person '{}' not found in profiles", person_name))
        }
    }
    
    /// Enhanced voice recognition with Resemblyzer integration
    pub fn identify_speaker_by_voice_resemblyzer(&self, voice_data: &VoiceDetectionData, resemblyzer_confidence: f32) -> Option<String> {
        let mut best_match: Option<String> = None;
        let mut best_confidence = 0.0;
        
        for (canonical_name, person) in &self.people {
            if person.matches_voice_resemblyzer(&voice_data.characteristics, resemblyzer_confidence) {
                // For Resemblyzer, we primarily trust the Python script's decision
                // But we can still rank multiple matches if needed
                if resemblyzer_confidence > best_confidence {
                    best_confidence = resemblyzer_confidence;
                    best_match = Some(canonical_name.clone());
                }
                
                debug_log!("🎤 Resemblyzer match: {} with {:.1}% confidence", 
                          person.name, resemblyzer_confidence * 100.0);
            }
        }
        
        if let Some(ref speaker) = best_match {
            debug_log!("🏆 Best Resemblyzer match: {} ({:.1}%)", 
                      self.people.get(speaker).map(|p| &p.name).unwrap_or(speaker), 
                      best_confidence * 100.0);
        }
        
        best_match
    }
    
    /// Get enhanced voice training status with Resemblyzer info
    pub fn get_voice_training_status_resemblyzer(&self) -> HashMap<String, VoiceTrainingStatus> {
        let mut status = HashMap::new();
        
        debug_log!("🔍 Getting Resemblyzer voice training status for {} people", self.people.len());
        
        for (canonical_name, person) in &self.people {
            debug_log!("👤 Checking person: {} (canonical: {})", person.name, canonical_name);
            
            let voice_status = if let Some(ref voice_profile) = person.voice_profile {
                debug_log!("🎤 {} has voice profile with {} samples", person.name, voice_profile.voice_samples.len());
                
                // Enhanced status for Resemblyzer
                let quality_score = if voice_profile.voice_samples.len() >= 3 {
                    0.9 // High quality with 3+ samples
                } else if voice_profile.voice_samples.len() >= 2 {
                    0.7 // Good quality with 2 samples
                } else {
                    0.5 // Basic quality with 1 sample
                };
                
                VoiceTrainingStatus {
                    has_voice_profile: true,
                    training_samples: voice_profile.voice_samples.len() as u32,
                    confidence_threshold: voice_profile.auto_threshold,
                    last_detection: Some(voice_profile.last_voice_detection),
                }
            } else {
                debug_log!("❌ {} has no voice profile", person.name);
                VoiceTrainingStatus {
                    has_voice_profile: false,
                    training_samples: 0,
                    confidence_threshold: 0.0,
                    last_detection: None,
                }
            };
            
            status.insert(person.name.clone(), voice_status);
        }
        
        debug_log!("📊 Returning Resemblyzer voice status for {} people", status.len());
        status
    }
}

// Add this helper function for main.rs integration
pub fn create_voice_detection_data_from_resemblyzer(
    person_name: &str,
    voice_characteristics: serde_json::Value,
    transcript: &str,
    confidence: f32,
) -> VoiceDetectionData {
    use crate::TimeService;
    
    debug_log!("🔧 RUST: Converting Parselmouth data for {}", person_name);
    debug_log!("🔧 RUST: Raw voice_characteristics: {}", serde_json::to_string_pretty(&voice_characteristics).unwrap_or_default());
    
    // Extract features from Parselmouth/Resemblyzer output
    let features = if let Some(features_obj) = voice_characteristics.get("features") {
        debug_log!("🔧 RUST: Found features object");
        
        // Extract all the Parselmouth features with debug
        let avg_pitch = features_obj.get("avg_pitch").and_then(|v| v.as_f64()).unwrap_or(150.0) as f32;
        debug_log!("🔧 RUST: Extracted avg_pitch: {}Hz", avg_pitch);
        
        VoiceFeatures {
            avg_pitch,
            pitch_range: features_obj.get("pitch_range").and_then(|v| v.as_f64()).unwrap_or(50.0) as f32,
            pitch_stability: features_obj.get("pitch_stability").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
            speaking_rate: features_obj.get("speaking_rate").and_then(|v| v.as_f64()).unwrap_or(2.5) as f32,
            pause_ratio: features_obj.get("pause_ratio").and_then(|v| v.as_f64()).unwrap_or(0.2) as f32,
            rhythm_consistency: features_obj.get("rhythm_consistency").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
            spectral_brightness: features_obj.get("spectral_brightness").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
            voice_clarity: features_obj.get("voice_clarity").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
            breathiness: features_obj.get("breathiness").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32,
            avg_energy: features_obj.get("avg_energy").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
            energy_variation: features_obj.get("energy_variation").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32,
            dynamic_range: features_obj.get("dynamic_range").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
            consonant_sharpness: features_obj.get("consonant_sharpness").and_then(|v| v.as_f64()).unwrap_or(0.3) as f32,
            vowel_clarity: features_obj.get("vowel_clarity").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
        }
    } else {
        debug_log!("🔧 RUST: No features object found, using defaults");
        VoiceFeatures::default()
    };
    
    // Extract description from Parselmouth/Resemblyzer output
    let description = if let Some(desc_obj) = voice_characteristics.get("description") {
        debug_log!("🔧 RUST: Found description object");
        
        let pitch_character = desc_obj.get("pitch_character")
            .and_then(|v| v.as_str())
            .unwrap_or("natural pitch")
            .to_string();
        debug_log!("🔧 RUST: Extracted pitch_character: {}", pitch_character);
        
        Some(VoiceDescription {
            pitch_character,
            speaking_style: desc_obj.get("speaking_style")
                .and_then(|v| v.as_str())
                .unwrap_or("conversational pace")
                .to_string(),
            voice_quality: desc_obj.get("voice_quality")
                .and_then(|v| v.as_str())
                .unwrap_or("clear tone")
                .to_string(),
            energy_profile: desc_obj.get("energy_profile")
                .and_then(|v| v.as_str())
                .unwrap_or("moderate energy")
                .to_string(),
            articulation_style: desc_obj.get("articulation_style")
                .and_then(|v| v.as_str())
                .unwrap_or("natural articulation")
                .to_string(),
            overall_impression: desc_obj.get("overall_impression")
                .and_then(|v| v.as_str())
                .unwrap_or("distinctive voice profile")
                .to_string(),
        })
    } else {
        debug_log!("🔧 RUST: No description object found");
        None
    };
    
    let voice_chars = VoiceCharacteristics {
        raw_features: features,
        voice_description: description,
        timestamp: TimeService::current_timestamp(),
        sample_rate: 16000.0,
        duration_ms: 3000.0,
        confidence,
        transcript_length: transcript.len(),
        word_count: transcript.split_whitespace().count(),
    };
    
    debug_log!("🔧 RUST: Final VoiceCharacteristics created with avg_pitch: {}Hz", voice_chars.raw_features.avg_pitch);
    
    VoiceDetectionData {
        voice_id: format!("resemblyzer_{}", person_name),
        confidence,
        characteristics: voice_chars,
        transcript: Some(transcript.to_string()),
    }
}

#[derive(Debug, Clone)]
pub struct PersonTransition {
    pub new_speaker: String,
    pub old_speaker: String,
    pub introduction_context: String,
    pub is_new_person: bool,
}

#[tauri::command]
pub async fn debug_voice_recognition(voice_data: VoiceDetectionData) -> Result<serde_json::Value, String> {
    let mut person_system = PersonRecognitionSystem::load_or_create();
    
    let mut debug_results = serde_json::json!({
        "detected_speaker": null,
        "similarity_scores": {}
    });
    
    // Check each person's voice profile with debug info
    for (canonical_name, person) in &person_system.people {
        if let Some(ref voice_profile) = person.voice_profile {
            let similarity_score = person.get_voice_similarity(&voice_data.characteristics);
            let threshold = if voice_profile.voice_samples.len() >= 20 {
                0.85
            } else if voice_profile.voice_samples.len() >= 10 {
                0.8
            } else if voice_profile.voice_samples.len() >= 5 {
                0.75
            } else if voice_profile.voice_samples.len() >= 3 {
                0.6
            } else {
                0.5
            };
            
            debug_results["similarity_scores"][person.name.clone()] = serde_json::json!({
                "similarity": similarity_score,
                "threshold": threshold,
                "training_samples": voice_profile.voice_samples.len(),
                "matches": similarity_score >= threshold
            });
            
            if similarity_score >= threshold {
                debug_results["detected_speaker"] = serde_json::json!({
                    "name": person.name.clone(),
                    "similarity": similarity_score,
                    "threshold": threshold
                });
            }
        }
    }
    
    debug_log!("🎤 DEBUG: Voice recognition results: {}", serde_json::to_string_pretty(&debug_results).unwrap_or_default());
    Ok(debug_results)
}