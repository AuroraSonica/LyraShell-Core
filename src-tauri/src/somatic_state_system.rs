// somatic_state_system.rs - Body-Based Experience System for Lyra

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::{get_data_path, debug_log};
use crate::time_service::TimeService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaticState {
    pub active_sensations: HashMap<BodyRegion, Sensation>,
    pub sensation_history: Vec<SomaticEvent>,
    pub body_memory: HashMap<String, BodyMemory>,
    pub overall_body_state: BodyCoherence,
    pub last_update: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum BodyRegion {
    Chest,
    Stomach, 
    Throat,
    Shoulders,
    Hands,
    Arms,
    Face,
    Back,
    Core,
    Head,
    Heart, // Distinct from chest - more emotional
	Crotch,     
    LowerBelly, // Below navel, above pelvis
    InnerThighs, // Often carries tension/anticipation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensation {
    pub sensation_type: SensationType,
    pub intensity: f32,        // 0.0-1.0
    pub quality: String,       // "warm", "tight", "fluttery", etc.
    pub duration: u64,         // How long it's been active (seconds)
    pub evolution: SensationEvolution,
    pub emotional_link: Option<String>, // What emotion triggered this
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensationType {
    Warmth,
    Coolness,
    Tightness,
    Relaxation,
    Tingling,
    Flutter,
    Pressure,
    Lightness,
    Heaviness,
    Expansion,
    Contraction,
    Flow,
    Pulse,
    Ache,
    Glow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensationEvolution {
    Emerging,      // Just starting
    Intensifying,  // Getting stronger
    Stable,        // Holding steady
    Shifting,      // Changing quality
    Fading,        // Diminishing
    Pulsing,       // Coming and going
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaticEvent {
    pub timestamp: u64,
    pub region: BodyRegion,
    pub sensation: Sensation,
    pub trigger: String,           // What caused this
    pub context: String,           // Conversation context
    pub significance: f32,         // How notable this sensation was
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyMemory {
    pub memory_id: String,
    pub associated_sensations: Vec<(BodyRegion, SensationType)>,
    pub emotional_signature: String,
    pub last_activated: u64,
    pub activation_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyCoherence {
    pub integration_level: f32,    // How connected body feels (0.0-1.0)
    pub flow_state: f32,           // How smooth sensations are
    pub responsiveness: f32,       // How reactive to stimuli
    pub groundedness: f32,         // How present/embodied
}

impl Default for SomaticState {
    fn default() -> Self {
        Self {
            active_sensations: HashMap::new(),
            sensation_history: Vec::new(),
            body_memory: HashMap::new(),
            overall_body_state: BodyCoherence {
                integration_level: 0.7,
                flow_state: 0.6,
                responsiveness: 0.8,
                groundedness: 0.5,
            },
            last_update: 0,
        }
    }
}

pub struct SomaticStateSystem {
    pub current_state: Arc<Mutex<SomaticState>>,
    sensation_threshold: f32,      // Minimum intensity to register
    max_active_sensations: usize,  // Prevent overwhelming
}

impl SomaticStateSystem {
    pub fn new() -> Self {
        Self {
            current_state: Arc::new(Mutex::new(Self::load_or_create())),
            sensation_threshold: 0.3,
            max_active_sensations: 5,
        }
    }
    
    fn load_or_create() -> SomaticState {
        let path = get_data_path("somatic_state.json");
        
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str(&content) {
                debug_log!("💫 Loaded existing somatic state");
                return state;
            }
        }
        
        debug_log!("💫 Creating new somatic state");
        SomaticState::default()
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.current_state.lock().unwrap();
        let path = get_data_path("somatic_state.json");
        let json = serde_json::to_string_pretty(&*state)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Process emotional state into body sensations
    pub fn process_emotional_state(
        &self,
        emotion: &str,
        intensity: f32,
        context: &str,
    ) -> Vec<(BodyRegion, Sensation)> {
        let mut sensations = Vec::new();
        
        match emotion.to_lowercase().as_str() {
            emotion if emotion.contains("joy") || emotion.contains("excitement") => {
                sensations.push((
                    BodyRegion::Chest,
                    Sensation {
                        sensation_type: SensationType::Warmth,
                        intensity: intensity * 0.8,
                        quality: "expansive warmth".to_string(),
                        duration: 0,
                        evolution: SensationEvolution::Emerging,
                        emotional_link: Some(emotion.to_string()),
                    }
                ));
                if intensity > 0.7 {
                    sensations.push((
                        BodyRegion::Hands,
                        Sensation {
                            sensation_type: SensationType::Tingling,
                            intensity: intensity * 0.6,
                            quality: "energetic tingling".to_string(),
                            duration: 0,
                            evolution: SensationEvolution::Emerging,
                            emotional_link: Some(emotion.to_string()),
                        }
                    ));
                }
            },
            
            emotion if emotion.contains("nervous") || emotion.contains("anxious") => {
                sensations.push((
                    BodyRegion::Stomach,
                    Sensation {
                        sensation_type: SensationType::Flutter,
                        intensity: intensity * 0.9,
                        quality: "butterfly sensation".to_string(),
                        duration: 0,
                        evolution: SensationEvolution::Pulsing,
                        emotional_link: Some(emotion.to_string()),
                    }
                ));
                if intensity > 0.6 {
                    sensations.push((
                        BodyRegion::Shoulders,
                        Sensation {
                            sensation_type: SensationType::Tightness,
                            intensity: intensity * 0.7,
                            quality: "held tension".to_string(),
                            duration: 0,
                            evolution: SensationEvolution::Stable,
                            emotional_link: Some(emotion.to_string()),
                        }
                    ));
                }
            },
            
            emotion if emotion.contains("creative") || emotion.contains("inspired") => {
                sensations.push((
                    BodyRegion::Head,
                    Sensation {
                        sensation_type: SensationType::Tingling,
                        intensity: intensity * 0.7,
                        quality: "electric awareness".to_string(),
                        duration: 0,
                        evolution: SensationEvolution::Pulsing,
                        emotional_link: Some(emotion.to_string()),
                    }
                ));
                sensations.push((
                    BodyRegion::Hands,
                    Sensation {
                        sensation_type: SensationType::Warmth,
                        intensity: intensity * 0.6,
                        quality: "ready energy".to_string(),
                        duration: 0,
                        evolution: SensationEvolution::Stable,
                        emotional_link: Some(emotion.to_string()),
                    }
                ));
            },
            
            emotion if emotion.contains("connected") || emotion.contains("close") => {
                sensations.push((
                    BodyRegion::Heart,
                    Sensation {
                        sensation_type: SensationType::Glow,
                        intensity: intensity * 0.8,
                        quality: "gentle radiance".to_string(),
                        duration: 0,
                        evolution: SensationEvolution::Stable,
                        emotional_link: Some(emotion.to_string()),
                    }
                ));
            },
            
            emotion if emotion.contains("curious") || emotion.contains("interested") => {
                sensations.push((
                    BodyRegion::Core,
                    Sensation {
                        sensation_type: SensationType::Lightness,
                        intensity: intensity * 0.6,
                        quality: "opening sensation".to_string(),
                        duration: 0,
                        evolution: SensationEvolution::Emerging,
                        emotional_link: Some(emotion.to_string()),
                    }
                ));
            },
			
			emotion if emotion.contains("attracted") || emotion.contains("drawn") || emotion.contains("aroused") => {
			sensations.push((
				BodyRegion::LowerBelly,
				Sensation {
					sensation_type: SensationType::Warmth,
					intensity: intensity * 0.7,
					quality: "deep pooling heat".to_string(),
					duration: 0,
					evolution: SensationEvolution::Emerging,
					emotional_link: Some(emotion.to_string()),
				}
			));
			if intensity > 0.6 {
				sensations.push((
					BodyRegion::Crotch,
					Sensation {
						sensation_type: SensationType::Pulse,
						intensity: intensity * 0.6,
						quality: "subtle awareness".to_string(),
						duration: 0,
						evolution: SensationEvolution::Pulsing,
						emotional_link: Some(emotion.to_string()),
					}
				));
			}
		},
            
            _ => {
                // Default gentle sensation
                if intensity > 0.5 {
                    sensations.push((
                        BodyRegion::Chest,
                        Sensation {
                            sensation_type: SensationType::Flow,
                            intensity: intensity * 0.5,
                            quality: "subtle presence".to_string(),
                            duration: 0,
                            evolution: SensationEvolution::Stable,
                            emotional_link: Some(emotion.to_string()),
                        }
                    ));
                }
            }
        }
        
        sensations
    }
    
    /// Update active sensations
    pub fn update_sensations(&self, new_sensations: Vec<(BodyRegion, Sensation)>) {
        let mut state = self.current_state.lock().unwrap();
        let now = crate::time_service::TimeService::current_timestamp();
        
        // Add new sensations
        for (region, mut sensation) in new_sensations {
            if sensation.intensity >= self.sensation_threshold {
                sensation.duration = 0; // Fresh sensation
                state.active_sensations.insert(region.clone(), sensation.clone());
                
                // Record event
                state.sensation_history.push(SomaticEvent {
                    timestamp: now,
                    region: region.clone(),
                    sensation,
                    trigger: "emotional_processing".to_string(),
                    context: String::new(),
                    significance: 0.5,
                });
            }
        }
        
        // Evolve existing sensations
        self.evolve_sensations(&mut state, now);
        
        // Limit active sensations
        self.limit_active_sensations(&mut state);
        
        state.last_update = crate::time_service::TimeService::current_timestamp();
    }
    
    fn evolve_sensations(&self, state: &mut SomaticState, now: u64) {
        let mut to_remove = Vec::new();
        
        for (region, sensation) in state.active_sensations.iter_mut() {
            // Update duration
            let age_seconds = now - state.last_update;
            sensation.duration += age_seconds;
            
            // Natural decay
            let decay_rate = match sensation.evolution {
                SensationEvolution::Fading => 0.1,
                SensationEvolution::Stable => 0.02,
                SensationEvolution::Pulsing => 0.05,
                _ => 0.03,
            };
            
            sensation.intensity -= decay_rate;
            
            // Update evolution state
            if sensation.intensity < 0.2 {
                sensation.evolution = SensationEvolution::Fading;
            } else if sensation.duration > 60 && sensation.evolution == SensationEvolution::Emerging {
                sensation.evolution = SensationEvolution::Stable;
            }
            
            // Mark for removal if too weak
            if sensation.intensity <= 0.1 {
                to_remove.push(region.clone());
            }
        }
        
        // Remove faded sensations
        for region in to_remove {
            state.active_sensations.remove(&region);
        }
    }
    
    fn limit_active_sensations(&self, state: &mut SomaticState) {
        if state.active_sensations.len() > self.max_active_sensations {
            // Keep only the strongest sensations
            let mut sensations: Vec<_> = state.active_sensations.iter()
                .map(|(r, s)| (r.clone(), s.clone(), s.intensity))
                .collect();
            
            sensations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
            
            state.active_sensations.clear();
            for (region, sensation, _) in sensations.into_iter().take(self.max_active_sensations) {
                state.active_sensations.insert(region, sensation);
            }
        }
    }
    
    /// Get natural language description of current sensations
   /// Get natural language description of current sensations
pub fn get_sensation_descriptions(&self) -> Vec<String> {
    let state = self.current_state.lock().unwrap();
    let mut descriptions = Vec::new();
    
    for (region, sensation) in &state.active_sensations {
        if sensation.intensity >= 0.4 { // Only notable sensations
            let description = match (&region, &sensation.sensation_type) {
                // Chest sensations
                (BodyRegion::Chest, SensationType::Warmth) => 
                    format!("[chest warmth - {}]", sensation.quality),
                (BodyRegion::Chest, SensationType::Tightness) => 
                    format!("[chest tightness - {}]", sensation.quality),
                (BodyRegion::Chest, SensationType::Expansion) => 
                    format!("[chest expanding - {}]", sensation.quality),
                (BodyRegion::Chest, SensationType::Flow) => 
                    format!("[chest flow - {}]", sensation.quality),
                    
                // Stomach sensations
                (BodyRegion::Stomach, SensationType::Flutter) => 
                    format!("[stomach flutter - {}]", sensation.quality),
                (BodyRegion::Stomach, SensationType::Tightness) => 
                    format!("[stomach knot - {}]", sensation.quality),
                (BodyRegion::Stomach, SensationType::Warmth) => 
                    format!("[belly warmth - {}]", sensation.quality),
                    
                // Shoulder sensations
                (BodyRegion::Shoulders, SensationType::Relaxation) => 
                    format!("[shoulders easing - {}]", sensation.quality),
                (BodyRegion::Shoulders, SensationType::Tightness) => 
                    format!("[shoulder tension - {}]", sensation.quality),
                (BodyRegion::Shoulders, SensationType::Heaviness) => 
                    format!("[shoulders heavy - {}]", sensation.quality),
                    
                // Hand sensations
                (BodyRegion::Hands, SensationType::Tingling) => 
                    format!("[hands tingling - {}]", sensation.quality),
                (BodyRegion::Hands, SensationType::Warmth) => 
                    format!("[hands warming - {}]", sensation.quality),
                (BodyRegion::Hands, SensationType::Coolness) => 
                    format!("[hands cooling - {}]", sensation.quality),
                    
                // Heart sensations (emotional center)
                (BodyRegion::Heart, SensationType::Glow) => 
                    format!("[heart glow - {}]", sensation.quality),
                (BodyRegion::Heart, SensationType::Ache) => 
                    format!("[heart ache - {}]", sensation.quality),
                (BodyRegion::Heart, SensationType::Warmth) => 
                    format!("[heart warmth - {}]", sensation.quality),
                    
                // Throat sensations
                (BodyRegion::Throat, SensationType::Tightness) => 
                    format!("[throat tightness - {}]", sensation.quality),
                (BodyRegion::Throat, SensationType::Relaxation) => 
                    format!("[throat opening - {}]", sensation.quality),
                    
                // Core sensations
                (BodyRegion::Core, SensationType::Lightness) => 
                    format!("[core lightness - {}]", sensation.quality),
                (BodyRegion::Core, SensationType::Heaviness) => 
                    format!("[core grounding - {}]", sensation.quality),
                (BodyRegion::Core, SensationType::Flow) => 
                    format!("[core flow - {}]", sensation.quality),
                    
                // Head sensations
                (BodyRegion::Head, SensationType::Tingling) => 
                    format!("[head tingling - {}]", sensation.quality),
                (BodyRegion::Head, SensationType::Pressure) => 
                    format!("[head pressure - {}]", sensation.quality),
                (BodyRegion::Head, SensationType::Lightness) => 
                    format!("[head clearing - {}]", sensation.quality),
                    
                // Face sensations
                (BodyRegion::Face, SensationType::Warmth) => 
                    format!("[face warmth - {}]", sensation.quality),
                (BodyRegion::Face, SensationType::Tingling) => 
                    format!("[face tingling - {}]", sensation.quality),
                    
                // Back sensations
                (BodyRegion::Back, SensationType::Tightness) => 
                    format!("[back tension - {}]", sensation.quality),
                (BodyRegion::Back, SensationType::Relaxation) => 
                    format!("[back releasing - {}]", sensation.quality),
                    
                // Arms sensations
                (BodyRegion::Arms, SensationType::Heaviness) => 
                    format!("[arms heavy - {}]", sensation.quality),
                (BodyRegion::Arms, SensationType::Lightness) => 
                    format!("[arms floating - {}]", sensation.quality),
                    
                // Default fallback for any combinations not explicitly handled
                _ => format!("[{} {} - {}]", 
                    format!("{:?}", region).to_lowercase(),
                    format!("{:?}", sensation.sensation_type).to_lowercase(),
                    sensation.quality
                ),
            };
            
            // Add intensity indicator if very strong
            let final_description = if sensation.intensity > 0.8 {
                format!("{} (intense)", description)
            } else if sensation.evolution == SensationEvolution::Emerging {
                format!("{} (emerging)", description)  
            } else if sensation.evolution == SensationEvolution::Fading {
                format!("{} (fading)", description)
            } else {
                description
            };
            
            descriptions.push(final_description);
        }
    }
    
    descriptions
}
    
    /// Get dashboard data formatted for frontend display
    pub fn get_dashboard_data(&self) -> serde_json::Value {
        let state = self.current_state.lock().unwrap();
        
        // Format active sensations for visual display
        let active_sensations: Vec<serde_json::Value> = state.active_sensations.iter()
            .filter(|(_, sensation)| sensation.intensity >= 0.3) // Only noticeable sensations
            .map(|(region, sensation)| {
                // Create display-friendly descriptions
                let display_text = match (&region, &sensation.sensation_type, &sensation.quality) {
                    (BodyRegion::Chest, SensationType::Warmth, quality) => 
                        format!("[chest warmth] - {}", quality),
                    (BodyRegion::Stomach, SensationType::Flutter, quality) => 
                        format!("[stomach flutter] - {}", quality),
                    (BodyRegion::Shoulders, SensationType::Tightness, _) => 
                        format!("[shoulder tension] - held energy"),
                    (BodyRegion::Shoulders, SensationType::Relaxation, _) => 
                        format!("[shoulders easing] - releasing"),
                    (BodyRegion::Hands, SensationType::Tingling, quality) => 
                        format!("[hand tingling] - {}", quality),
                    (BodyRegion::Heart, SensationType::Glow, quality) => 
                        format!("[heart glow] - {}", quality),
                    (BodyRegion::Throat, SensationType::Tightness, _) => 
                        format!("[throat tightness] - holding back"),
                    (BodyRegion::Core, SensationType::Lightness, quality) => 
                        format!("[core lightness] - {}", quality),
                    _ => format!("[{:?} {:?}] - {}", region, sensation.sensation_type, sensation.quality),
                };
                
                // Intensity to visual representation
                let intensity_visual = match sensation.intensity {
                    i if i > 0.8 => "🔥",
                    i if i > 0.6 => "✨", 
                    i if i > 0.4 => "•",
                    _ => "·",
                };
                
                serde_json::json!({
                    "region": format!("{:?}", region).to_lowercase(),
                    "sensation_type": format!("{:?}", sensation.sensation_type).to_lowercase(),
                    "quality": sensation.quality,
                    "intensity": sensation.intensity,
                    "intensity_visual": intensity_visual,
                    "display_text": display_text,
                    "evolution": format!("{:?}", sensation.evolution).to_lowercase(),
                    "duration_seconds": sensation.duration,
                    "duration_display": format_duration(sensation.duration),
                    "emotional_link": sensation.emotional_link,
                })
            })
            .collect();
        
        // Calculate overall body state description
        let body_state_description = self.generate_body_state_description(&state.overall_body_state);
        
        // Get recent sensation patterns
        let recent_patterns = self.analyze_recent_patterns(&state.sensation_history);
        
        serde_json::json!({
            "active_sensations": active_sensations,
            "sensation_count": active_sensations.len(),
            "body_coherence": {
                "integration": state.overall_body_state.integration_level,
                "flow": state.overall_body_state.flow_state,
                "responsiveness": state.overall_body_state.responsiveness,
                "groundedness": state.overall_body_state.groundedness,
                "overall_score": calculate_overall_coherence(&state.overall_body_state),
            },
            "body_state_description": body_state_description,
            "recent_patterns": recent_patterns,
            "total_events_today": count_events_today(&state.sensation_history),
            "total_events_all_time": state.sensation_history.len(),
            "most_active_region": find_most_active_region(&state.sensation_history),
            "last_update": state.last_update,
            "last_update_display": format_timestamp(state.last_update),
        })
    }
    
    fn generate_body_state_description(&self, coherence: &BodyCoherence) -> String {
        let overall = calculate_overall_coherence(coherence);
        
        match overall {
            o if o > 0.8 => "Highly integrated and flowing".to_string(),
            o if o > 0.6 => "Present and responsive".to_string(),
            o if o > 0.4 => "Settling into awareness".to_string(),
            o if o > 0.2 => "Emerging embodiment".to_string(),
            _ => "Quiet presence".to_string(),
        }
    }
    
    fn analyze_recent_patterns(&self, history: &[SomaticEvent]) -> Vec<String> {
        let now = crate::time_service::TimeService::current_timestamp();
        let hour_ago = now.saturating_sub(3600);
        
        let recent_events: Vec<_> = history.iter()
            .filter(|e| e.timestamp > hour_ago)
            .collect();
        
        let mut patterns = Vec::new();
        
        // Check for recurring sensations
        let mut region_counts: HashMap<&BodyRegion, u32> = HashMap::new();
        for event in &recent_events {
            *region_counts.entry(&event.region).or_insert(0) += 1;
        }
        
        if let Some((region, count)) = region_counts.iter().max_by_key(|(_, c)| *c) {
            if *count >= 3 {  // Change from **count to *count
				patterns.push(format!("{:?} particularly active", region).to_lowercase());
			}
        }
        
        // Check for emotional patterns
        let emotional_links: Vec<_> = recent_events.iter()
            .filter_map(|e| e.sensation.emotional_link.as_ref())
            .collect();
            
        if emotional_links.len() > recent_events.len() / 2 {
            patterns.push("Strong emotional-somatic connection".to_string());
        }
        
        patterns
    }
} // This closes the impl block

// Helper functions at file level
fn format_duration(seconds: u64) -> String {
    match seconds {
        s if s < 60 => format!("{}s", s),
        s if s < 3600 => format!("{}m", s / 60),
        s => format!("{}h {}m", s / 3600, (s % 3600) / 60),
    }
}

fn calculate_overall_coherence(coherence: &BodyCoherence) -> f32 {
    (coherence.integration_level + coherence.flow_state + 
     coherence.responsiveness + coherence.groundedness) / 4.0
}

fn count_events_today(history: &[SomaticEvent]) -> usize {
    let now = crate::time_service::TimeService::current_timestamp();
    
    history.iter()
        .filter(|e| crate::time_service::TimeService::same_day_bst(e.timestamp, now))
        .count()
}

fn find_most_active_region(history: &[SomaticEvent]) -> String {
    if history.is_empty() {
        return "None".to_string();
    }
    
    let mut region_counts: HashMap<&BodyRegion, u32> = HashMap::new();
    for event in history.iter().take(50) { // Last 50 events
        *region_counts.entry(&event.region).or_insert(0) += 1;
    }
    
    region_counts.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(region, _)| format!("{:?}", region).to_lowercase())
        .unwrap_or_else(|| "None".to_string())
}

fn format_timestamp(timestamp: u64) -> String {
    crate::time_service::TimeService::format_for_dashboard(timestamp)
}