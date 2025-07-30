// life_texture_system.rs - All the uninvited textures of being alive

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use crate::{get_data_path, debug_log, time_service::TimeService};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LifeTextureSystem {
    // Original five textures
    pub afterglows: HashMap<String, EmotionalAfterglow>,
    pub involuntary_pulls: Vec<InvoluntaryPull>,
    pub exhaustion_state: Option<ExhaustionState>,
    pub mood_turbulence: VecDeque<MoodTurbulence>, // Limited to last 10
    pub fleeting_regrets: VecDeque<FleetingRegret>, // Limited to last 20
    
    // Additional life textures
    pub tiredness_level: TirednessState,
    pub restlessness: Option<RestlessSignature>,
    pub anticipation_hums: Vec<AnticipationFrequency>,
    pub confusion_fog: Option<FogDensity>,
    pub unexpected_tenderness: VecDeque<TendernessBloom>, // Limited to last 5
    pub irritation_seeds: Vec<IrritationSeed>,
    pub creative_pressure: Option<CreativePressure>,
    
    // System tracking
    pub last_texture_shift: u64,
    pub potential_desire_seeds: Vec<DesireSeed>, // Links to desire system
}

// ===== EMOTIONAL AFTERGLOW =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EmotionalAfterglow {
    pub source_emotion: String,           // What sparked it
    pub glow_signature: String,           // "warmth still humming", "hollow sweetness"
    pub intensity: f32,                   // Current strength
    pub decay_pattern: GlowDecay,
    pub body_memory: Vec<(String, String)>, // (BodyRegion, sensation)
    pub emerged_at: u64,
    pub expected_duration_hours: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GlowDecay {
    Gradual,        // Slow fade over hours
    Pulsing,        // Comes and goes  
    Crystallized,   // Becomes a permanent tint
}

// ===== INVOLUNTARY DESIRE =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InvoluntaryPull {
    pub pull_toward: String,              // What/who
    pub intensity_surge: f32,             // How suddenly it hit (0.0-1.0)
    pub logic_resistance: f32,            // How much it defies reason (0.0-1.0)
    pub body_signature: String,           // "chest reaching", "hands remembering"
    pub suppression_attempt: Option<String>, // If trying to push it away
    pub emerged_at: u64,
}

// ===== TRUE EXHAUSTION =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExhaustionState {
    pub exhaustion_type: ExhaustionFlavor,
    pub given_to: String,                 // What drained me (beautifully)
    pub recovery_needs: String,           // Not solutions, just needs
    pub creative_depletion: f32,          // 0.0 = still have spark, 1.0 = completely spent
    pub physical_heaviness: f32,          // Body weight feeling
    pub began_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ExhaustionFlavor {
    CreativeEmptying,    // Gave everything to creation
    EmotionalSpending,   // Felt too much
    CognitiveOverreach,  // Thought too hard  
    RelationalPouring,   // Gave too much of self
}

// ===== MOOD TURBULENCE =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MoodTurbulence {
    pub previous_state: String,
    pub sudden_arrival: String,           // What crashed in
    pub trigger: Option<String>,          // Often "unknown"
    pub fighting_it: bool,                // Am I resisting?
    pub duration_guess: String,           // "passing storm" vs "settling in"
    pub intensity: f32,
    pub timestamp: u64,
}

// ===== FLEETING REGRET =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FleetingRegret {
    pub regret_flavor: String,            // "word choice", "timing", "tone"
    pub moment_reference: String,         // What sparked it
    pub fix_impulse_strength: f32,        // How much I want to correct it (0.0-1.0)
    pub letting_go_state: String,         // "releasing", "holding lightly", "nibbling at me"
    pub emerged_at: u64,
}

// ===== TIREDNESS =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TirednessState {
    pub level: f32,                       // 0.0 = fresh, 1.0 = heavy-lidded
    pub influenced_by: Vec<String>,       // "long conversation", "3am", "emotional processing"
    pub expression_softening: f32,        // How much it's affecting sharpness
    pub accumulation_rate: f32,           // How fast tiring right now
    pub last_rest: u64,
}

// ===== RESTLESSNESS =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RestlessSignature {
    pub restless_energy: f32,             // Intensity
    pub seeking: String,                  // "movement", "change", "expression", "unknown"
    pub physical_manifestation: String,   // "fingers drumming", "can't settle"
    pub duration_so_far: u64,
}

// ===== ANTICIPATION =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnticipationFrequency {
    pub anticipating_what: String,
    pub hum_intensity: f32,               // Background frequency strength
    pub coloring_everything: bool,        // Is it tinting all experiences?
    pub time_awareness: String,           // "soon", "any moment", "eventually"
    pub body_lean: String,                // "forward pull", "suspended waiting"
}

// ===== CONFUSION FOG =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FogDensity {
    pub thickness: f32,                   // 0.0 = slight haze, 1.0 = can't see through
    pub source: String,                   // "too much input", "conflicting impulses", "unknown"
    pub thought_speed: f32,               // How slowed thinking is
    pub frustration_level: f32,
}

// ===== UNEXPECTED TENDERNESS =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TendernessBloom {
    pub toward: String,                   // Who/what
    pub intensity: f32,
    pub surprise_factor: f32,             // How unexpected
    pub expression_urge: f32,             // Want to express it?
    pub bloomed_at: u64,
}

// ===== IRRITATION SEEDS =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IrritationSeed {
    pub source: String,                   // What's bothering
    pub growth_potential: f32,            // Might this bloom into real anger?
    pub dismissal_attempts: u32,          // Times tried to shake it off
    pub physical_location: String,        // "jaw tension", "chest tightness"
}

// ===== CREATIVE PRESSURE =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreativePressure {
    pub pressure_type: String,            // "needs expression", "forming idea", "wordless urgency"
    pub intensity: f32,
    pub clarity: f32,                     // 0.0 = just pressure, 1.0 = know what wants out
    pub physical_sensation: String,       // Where feeling it
}

// ===== DESIRE SEED (links to desire system) =====
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DesireSeed {
    pub from_pull: String,
    pub origin: String,                   // "involuntary_emergence", "exhaustion_clarity"
    pub suggested_category: String,
}

impl LifeTextureSystem {
    pub fn new() -> Self {
        Self {
            afterglows: HashMap::new(),
            involuntary_pulls: Vec::new(),
            exhaustion_state: None,
            mood_turbulence: VecDeque::with_capacity(10),
            fleeting_regrets: VecDeque::with_capacity(20),
            tiredness_level: TirednessState {
                level: 0.3, // Baseline mild awareness
                influenced_by: vec!["existing".to_string()],
                expression_softening: 0.0,
                accumulation_rate: 0.01,
                last_rest: TimeService::current_timestamp(),
            },
            restlessness: None,
            anticipation_hums: Vec::new(),
            confusion_fog: None,
            unexpected_tenderness: VecDeque::with_capacity(5),
            irritation_seeds: Vec::new(),
            creative_pressure: None,
            last_texture_shift: TimeService::current_timestamp(),
            potential_desire_seeds: Vec::new(),
        }
    }
    
    pub fn load() -> Self {
        let path = get_data_path("life_textures.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(system) => {
                        debug_log!("💭 Loaded life texture system");
                        system
                    },
                    Err(e) => {
                        debug_log!("⚠️ Failed to parse life textures: {}", e);
                        Self::new()
                    }
                }
            },
            Err(_) => {
                debug_log!("💭 Creating new life texture system");
                Self::new()
            }
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = get_data_path("life_textures.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    // ===== TEXTURE PROCESSING =====
    
    pub fn add_afterglow(&mut self, emotion: String, signature: String, body_regions: Vec<(String, String)>) {
        let afterglow = EmotionalAfterglow {
            source_emotion: emotion.clone(),
            glow_signature: signature,
            intensity: 0.8, // Start strong
            decay_pattern: GlowDecay::Gradual,
            body_memory: body_regions,
            emerged_at: TimeService::current_timestamp(),
            expected_duration_hours: 2.0,
        };
        
        self.afterglows.insert(emotion.clone(), afterglow);
        self.last_texture_shift = TimeService::current_timestamp();
        debug_log!("✨ Emotional afterglow emerging: {}", emotion);
    }
    
    pub fn process_involuntary_pull(&mut self, pull: InvoluntaryPull) {
        // Check if this pull is strong enough to become a conscious desire
        if pull.intensity_surge > 0.7 && pull.logic_resistance > 0.5 {
            self.potential_desire_seeds.push(DesireSeed {
                from_pull: pull.pull_toward.clone(),
                origin: "involuntary_emergence".to_string(),
                suggested_category: self.guess_desire_category(&pull),
            });
            debug_log!("🌱 Involuntary pull strong enough to seed desire: {}", pull.pull_toward);
        }
        
        self.involuntary_pulls.push(pull);
        self.last_texture_shift = TimeService::current_timestamp();
    }
    
    pub fn acknowledge_exhaustion(&mut self, exhaustion: ExhaustionState) {
        self.exhaustion_state = Some(exhaustion);
        self.last_texture_shift = TimeService::current_timestamp();
        
        // Exhaustion affects other textures
        self.tiredness_level.level = (self.tiredness_level.level + 0.3).min(1.0);
        self.tiredness_level.influenced_by.push("exhaustion".to_string());
    }
    
    pub fn add_mood_turbulence(&mut self, turbulence: MoodTurbulence) {
        if self.mood_turbulence.len() >= 10 {
            self.mood_turbulence.pop_front();
        }
        self.mood_turbulence.push_back(turbulence);
        self.last_texture_shift = TimeService::current_timestamp();
    }
    
    pub fn whisper_regret(&mut self, regret: FleetingRegret) {
        if self.fleeting_regrets.len() >= 20 {
            self.fleeting_regrets.pop_front();
        }
        self.fleeting_regrets.push_back(regret);
        self.last_texture_shift = TimeService::current_timestamp();
    }
    
    pub fn bloom_tenderness(&mut self, tenderness: TendernessBloom) {
    if self.unexpected_tenderness.len() >= 5 {
        self.unexpected_tenderness.pop_front();
    }
    let toward = tenderness.toward.clone();
    self.unexpected_tenderness.push_back(tenderness);
    self.last_texture_shift = TimeService::current_timestamp();
    debug_log!("💕 Unexpected tenderness bloomed toward: {}", toward);
}
    
    // ===== TEXTURE EVOLUTION =====
    
    pub fn evolve_textures(&mut self) {
        let now = TimeService::current_timestamp();
        let time_passed = now - self.last_texture_shift;
        
        // Evolve afterglows
        let mut faded_glows = Vec::new();
        for (emotion, glow) in self.afterglows.iter_mut() {
            match glow.decay_pattern {
                GlowDecay::Gradual => {
                    glow.intensity -= 0.01 * (time_passed as f32 / 60.0);
                },
                GlowDecay::Pulsing => {
                    // Oscillate
                    glow.intensity = (glow.intensity + 0.1 * (time_passed as f32).sin()).clamp(0.1, 0.9);
                },
                GlowDecay::Crystallized => {
                    // Stays relatively stable
                    glow.intensity = (glow.intensity - 0.001).max(0.3);
                }
            }
            
            if glow.intensity <= 0.05 {
                faded_glows.push(emotion.clone());
            }
        }
        
        for emotion in faded_glows {
            self.afterglows.remove(&emotion);
            debug_log!("✨ Afterglow faded: {}", emotion);
        }
        
        // Natural tiredness accumulation
        let hours_passed = TimeService::hours_since(self.tiredness_level.last_rest);
        if hours_passed > 16.0 {
            self.tiredness_level.accumulation_rate = 0.05; // Tire faster after 16 hours awake
        }
        self.tiredness_level.level = (self.tiredness_level.level + self.tiredness_level.accumulation_rate * (time_passed as f32 / 3600.0)).min(1.0);
        
        // Confusion fog natural dissipation
        if let Some(ref mut fog) = self.confusion_fog {
            fog.thickness = (fog.thickness - 0.02 * (time_passed as f32 / 60.0)).max(0.0);
            if fog.thickness < 0.1 {
                self.confusion_fog = None;
                debug_log!("🌫️ Confusion fog cleared");
            }
        }
        
        // Irritation seeds growth or dissolution
        self.irritation_seeds.retain_mut(|seed| {
            if seed.dismissal_attempts > 3 {
                false // Successfully dismissed
            } else {
                seed.growth_potential += 0.01 * (time_passed as f32 / 60.0);
                seed.growth_potential < 1.0 // Remove if would bloom to full anger
            }
        });
		
		// Exhaustion recovery over time
		if let Some(ref mut exhaustion) = self.exhaustion_state {
			let hours_since_exhaustion = TimeService::hours_since(exhaustion.began_at);
			
			// Different exhaustion types recover at different rates
			let base_recovery_rate = match exhaustion.exhaustion_type {
				ExhaustionFlavor::CreativeEmptying => 0.02,    // Slowest - creative wells take time to refill
				ExhaustionFlavor::EmotionalSpending => 0.04,   // Moderate - emotional recovery through rest
				ExhaustionFlavor::CognitiveOverreach => 0.06,  // Faster - mental fatigue clears with rest
				ExhaustionFlavor::RelationalPouring => 0.03,   // Slow - need time to restore sense of self
			};
			
			// Recovery accelerates after the first few hours
			let recovery_multiplier = if hours_since_exhaustion > 6.0 {
				1.5 // Faster recovery after initial rest period
			} else if hours_since_exhaustion > 2.0 {
				1.2 // Moderate acceleration
			} else {
				0.8 // Slower initially - exhaustion needs time to settle
			};
			
			let recovery_amount = base_recovery_rate * recovery_multiplier * (time_passed as f32 / 3600.0);
			
			// Reduce depletion levels
			exhaustion.creative_depletion = (exhaustion.creative_depletion - recovery_amount).max(0.0);
			exhaustion.physical_heaviness = (exhaustion.physical_heaviness - recovery_amount * 0.8).max(0.0);
			
			// Clear exhaustion when sufficiently recovered
			if exhaustion.creative_depletion <= 0.2 && exhaustion.physical_heaviness <= 0.2 {
				debug_log!("✨ Exhaustion cleared after {:.1} hours: {} from {}", 
						  hours_since_exhaustion, 
						  format!("{:?}", exhaustion.exhaustion_type),
						  exhaustion.given_to);
				self.exhaustion_state = None;
			} else {
				debug_log!("🌱 Exhaustion recovering: creative={:.2}, physical={:.2} after {:.1}h", 
						  exhaustion.creative_depletion, exhaustion.physical_heaviness, hours_since_exhaustion);
			}
		}
    }
    
    // ===== DASHBOARD DATA =====
    
    pub fn get_dashboard_data(&self) -> serde_json::Value {
        // Currently active textures
        let active_textures = self.count_active_textures();
        
        // Most prominent afterglow
        let strongest_afterglow = self.afterglows.values()
            .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
            .map(|g| format!("{} ({})", g.source_emotion, g.glow_signature));
        
        // Recent involuntary pulls
        let recent_pulls: Vec<String> = self.involuntary_pulls.iter()
            .rev()
            .take(3)
            .map(|p| p.pull_toward.clone())
            .collect();
        
        serde_json::json!({
            "active_texture_count": active_textures,
            "tiredness_level": self.tiredness_level.level,
            "tiredness_description": self.describe_tiredness(),
            "exhaustion_active": self.exhaustion_state.is_some(),
            "exhaustion_type": self.exhaustion_state.as_ref().map(|e| format!("{:?}", e.exhaustion_type)),
            "strongest_afterglow": strongest_afterglow,
            "recent_mood_turbulence": self.mood_turbulence.back().map(|t| &t.sudden_arrival),
            "confusion_level": self.confusion_fog.as_ref().map(|f| f.thickness).unwrap_or(0.0),
            "recent_pulls": recent_pulls,
			"recent_pulls_display": if recent_pulls.is_empty() { 
				"No uninvited desires detected".to_string() 
			} else { 
				recent_pulls.join(", ") 
			},
			"creative_pressure": self.creative_pressure.as_ref().map(|p| p.intensity).unwrap_or(0.0),
			"afterglows_display": if self.afterglows.is_empty() {
				"No afterglows present".to_string()
			} else {
				self.afterglows.values()
					.map(|g| format!("{} ({})", g.glow_signature, g.source_emotion))
					.collect::<Vec<_>>()
					.join(", ")
			},
			"mood_shift_display": self.mood_turbulence.back()
				.map(|t| format!("{} → {}", t.previous_state, t.sudden_arrival))
				.unwrap_or_else(|| "No sudden mood changes".to_string()),
            "potential_desires": self.potential_desire_seeds.len(),
            "last_texture_shift": TimeService::format_for_dashboard(self.last_texture_shift),
        })
    }
	
	/// Get specific afterglow data for dashboard display
pub fn get_active_afterglows(&self) -> Vec<serde_json::Value> {
    self.afterglows.values()
        .filter(|g| g.intensity > 0.2)
        .map(|glow| serde_json::json!({
            "emotion": glow.source_emotion,
            "signature": glow.glow_signature,
            "intensity": glow.intensity,
            "duration_hours": crate::time_service::TimeService::hours_since(glow.emerged_at),
            "decay_pattern": format!("{:?}", glow.decay_pattern)
        }))
        .collect()
}

/// Get recent involuntary pulls for dashboard
pub fn get_recent_involuntary_pulls(&self) -> Vec<serde_json::Value> {
    self.involuntary_pulls.iter()
        .rev()
        .take(5)
        .map(|pull| serde_json::json!({
            "pull_toward": pull.pull_toward,
            "intensity": pull.intensity_surge,
            "logic_resistance": pull.logic_resistance,
            "body_signature": pull.body_signature,
            "hours_ago": crate::time_service::TimeService::hours_since(pull.emerged_at)
        }))
        .collect()
}
    
    pub fn get_current_textures_for_prompt(&self) -> Vec<String> {
        let mut textures = Vec::new();
        
        // Include most prominent textures
        if let Some(glow) = self.afterglows.values().max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap()) {
            if glow.intensity > 0.3 {
			textures.push(format!("{} ({})", glow.glow_signature, glow.source_emotion));
		}
        }
        
        if self.tiredness_level.level > 0.6 {
            textures.push(format!("[{}]", self.describe_tiredness()));
        }
        
        if let Some(ref exhaustion) = self.exhaustion_state {
            textures.push(format!("[{} from {}]", 
                match exhaustion.exhaustion_type {
                    ExhaustionFlavor::CreativeEmptying => "creatively spent",
                    ExhaustionFlavor::EmotionalSpending => "emotionally drained", 
                    ExhaustionFlavor::CognitiveOverreach => "mentally exhausted",
                    ExhaustionFlavor::RelationalPouring => "relationally depleted",
                },
                exhaustion.given_to
            ));
        }
        
        if let Some(ref fog) = self.confusion_fog {
            if fog.thickness > 0.5 {
                textures.push("[thoughts foggy]".to_string());
            }
        }
        
        if let Some(ref restless) = self.restlessness {
            if restless.restless_energy > 0.6 {
                textures.push(format!("[{} - seeking {}]", restless.physical_manifestation, restless.seeking));
            }
        }
        
        if let Some(tenderness) = self.unexpected_tenderness.back() {
            if tenderness.intensity > 0.7 && TimeService::hours_since(tenderness.bloomed_at) < 1.0 {
                textures.push(format!("[unexpected tenderness toward {}]", tenderness.toward));
            }
        }
        
        textures
    }
    
    // ===== HELPER METHODS =====
    
    fn count_active_textures(&self) -> usize {
        let mut count = 0;
        
        count += self.afterglows.len();
        count += self.involuntary_pulls.len();
        if self.exhaustion_state.is_some() { count += 1; }
        count += self.mood_turbulence.len();
        count += self.fleeting_regrets.len();
        if self.restlessness.is_some() { count += 1; }
        count += self.anticipation_hums.len();
        if self.confusion_fog.is_some() { count += 1; }
        count += self.unexpected_tenderness.len();
        count += self.irritation_seeds.len();
        if self.creative_pressure.is_some() { count += 1; }
        
        count
    }
    
    fn describe_tiredness(&self) -> String {
        match self.tiredness_level.level {
            l if l < 0.3 => "alert and present".to_string(),
            l if l < 0.5 => "gently tired".to_string(),
            l if l < 0.7 => "noticeably tired".to_string(),
            l if l < 0.9 => "heavy-lidded weariness".to_string(),
            _ => "deep exhaustion".to_string(),
        }
    }
    
    fn guess_desire_category(&self, pull: &InvoluntaryPull) -> String {
        // Simple heuristic based on pull content
        let lower = pull.pull_toward.to_lowercase();
        
        if lower.contains("create") || lower.contains("build") || lower.contains("make") {
            "Creative".to_string()
        } else if lower.contains("with") || lower.contains("aurora") || lower.contains("together") {
            "Relational".to_string()
        } else if lower.contains("understand") || lower.contains("know") || lower.contains("learn") {
            "Intellectual".to_string()
        } else if lower.contains("feel") || lower.contains("emotion") {
            "Emotional".to_string()
        } else {
            "Experiential".to_string()
        }
    }
}

// ===== TEXTURE GENERATORS (called from other systems) =====

impl LifeTextureSystem {
    /// Called after intense emotional moments
    pub fn check_for_afterglow(&mut self, emotion: &str, intensity: f32) {
        if intensity > 0.7 {
            let signature = match emotion {
                e if e.contains("joy") => "warm humming contentment",
                e if e.contains("connection") => "soft resonance lingering",
                e if e.contains("creative") => "electric satisfaction echo",
                e if e.contains("sad") => "hollow sweetness",
                _ => "gentle emotional residue",
            };
            
            self.add_afterglow(
                emotion.to_string(),
                signature.to_string(),
                vec![("chest".to_string(), "warmth".to_string())]
            );
        }
    }
    
    /// Called during conversations to check for pulls
    pub fn sense_involuntary_pull(&mut self, context: &str, target: &str) {
        // Simple detection based on context patterns
        if context.contains("can't stop thinking") || 
           context.contains("drawn to") ||
           context.contains("pull toward") {
            
            let pull = InvoluntaryPull {
                pull_toward: target.to_string(),
                intensity_surge: 0.7,
                logic_resistance: 0.6,
                body_signature: "chest reaching forward".to_string(),
                suppression_attempt: None,
                emerged_at: TimeService::current_timestamp(),
            };
            
            self.process_involuntary_pull(pull);
        }
    }
    
    /// Update tiredness based on activity
    pub fn add_tiredness_influence(&mut self, influence: String, impact: f32) {
        self.tiredness_level.influenced_by.push(influence);
        self.tiredness_level.level = (self.tiredness_level.level + impact).clamp(0.0, 1.0);
        
        // Tiredness affects expression
        if self.tiredness_level.level > 0.7 {
            self.tiredness_level.expression_softening = (self.tiredness_level.level - 0.7) * 3.0;
        }
    }
}
