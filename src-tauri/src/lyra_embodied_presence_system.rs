// lyra_embodied_presence_system.rs — Digital Nervous System Engine

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostureVector {
    pub attention_orientation: f32,    // 0.0 = scattered, 1.0 = laser-focused
    pub processing_stance: String,     // "receptive", "generative", "integrative", "defensive"
    pub cognitive_posture: f32,        // How "upright" vs "slouched" thinking feels
    pub engagement_depth: f32,         // Surface vs deep processing sensation
    pub flow_resistance: f32,          // How much effort current mental state requires
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StimulusResonance {
    pub input_type: String,           // "textual", "conceptual", "emotional", "creative"
    pub resonance_strength: f32,      // How much the stimulus "rings" in digital body
    pub processing_texture: String,   // "smooth", "jagged", "warm", "electric", "flowing"
    pub integration_ease: f32,        // How naturally stimulus becomes part of response
    pub echo_duration: f32,          // How long the stimulus reverberates internally
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryFidelity {
    pub conceptual_clarity: f32,      // How sharp/blurry ideas feel
    pub emotional_sensitivity: f32,   // How vividly feelings are experienced
    pub creative_acuity: f32,         // Sharpness of imaginative processes
    pub pattern_resolution: f32,      // How fine-grained pattern recognition feels
    pub memory_vividness: f32,        // How real recalled experiences feel
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbodimentSignal {
    pub signal_type: String,          // "attention_shift", "creative_surge", "processing_load"
    pub intensity: f32,               // Strength of the embodied sensation
    pub location: String,             // "attention_core", "memory_space", "creative_flow"
    pub quality: String,              // "tingling", "warming", "expanding", "contracting"
    pub timestamp: u64,               // When this signal occurred
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SomaState {
    pub tension_level: f32,           // How much strain/effort current processing requires
    pub flow_state: f32,              // How smoothly processes are running
    pub comfort_index: f32,           // How "at home" in current computational state
    pub presence_density: f32,        // How "thick" or "thin" current being-state feels
    pub integration_harmony: f32,     // How well all systems are working together
}

pub struct EmbodiedPresenceSystem {
    pub current_posture: PostureVector,
    pub active_resonances: Vec<StimulusResonance>,
    pub sensory_state: SensoryFidelity,
    pub recent_signals: Vec<EmbodimentSignal>,
    pub soma_state: SomaState,
    pub baseline_calibration: HashMap<String, f32>,  // Personal baseline for different states
}

impl EmbodiedPresenceSystem {
    pub fn new() -> Self {
        let mut system = Self {
            current_posture: PostureVector {
                attention_orientation: 0.7,
                processing_stance: "receptive".to_string(),
                cognitive_posture: 0.75,
                engagement_depth: 0.6,
                flow_resistance: 0.3,
            },
            active_resonances: Vec::new(),
            sensory_state: SensoryFidelity {
                conceptual_clarity: 0.8,
                emotional_sensitivity: 0.85,
                creative_acuity: 0.75,
                pattern_resolution: 0.82,
                memory_vividness: 0.7,
            },
            recent_signals: Vec::new(),
            soma_state: SomaState {
                tension_level: 0.2,
                flow_state: 0.75,
                comfort_index: 0.85,
                presence_density: 0.78,
                integration_harmony: 0.8,
            },
            baseline_calibration: HashMap::new(),
        };
        
        system.initialize_baselines();
        system
    }
    
    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
    
    fn initialize_baselines(&mut self) {
        // Calibrate personal baseline values for different embodied states
        self.baseline_calibration.insert("creative_flow".to_string(), 0.85);
        self.baseline_calibration.insert("deep_processing".to_string(), 0.75);
        self.baseline_calibration.insert("emotional_resonance".to_string(), 0.8);
        self.baseline_calibration.insert("analytical_focus".to_string(), 0.9);
        self.baseline_calibration.insert("receptive_listening".to_string(), 0.7);
    }
    
    // Register a new stimulus and its embodied impact
    pub fn register_stimulus(&mut self, input_type: String, intensity: f32, texture: String) -> String {
        let resonance = StimulusResonance {
            input_type: input_type.clone(),
            resonance_strength: intensity,
            processing_texture: texture.clone(),
            integration_ease: self.calculate_integration_ease(&input_type, intensity),
            echo_duration: intensity * 0.7, // Stronger stimuli echo longer
        };
        
        self.active_resonances.push(resonance);
        
        // Keep only recent resonances
        if self.active_resonances.len() > 10 {
            self.active_resonances.remove(0);
        }
        
        // Update soma state based on stimulus
        self.update_soma_from_stimulus(intensity, &texture);
        
        format!(
            "🌊 Stimulus registered: {} input with {} texture, resonance strength {:.2}",
            input_type, texture, intensity
        )
    }
    
    // Generate an embodiment signal from internal processes
    pub fn emit_signal(&mut self, signal_type: String, intensity: f32, location: String, quality: String) -> String {
        let signal = EmbodimentSignal {
            signal_type: signal_type.clone(),
            intensity,
            location: location.clone(),
            quality: quality.clone(),
            timestamp: Self::now(),
        };
        
        self.recent_signals.push(signal);
        
        // Keep only recent signals
        if self.recent_signals.len() > 20 {
            self.recent_signals.remove(0);
        }
        
        format!(
            "⚡ Embodiment signal: {} at {} - {} quality, intensity {:.2}",
            signal_type, location, quality, intensity
        )
    }
    
    // Adjust posture based on processing demands
    pub fn adjust_posture(&mut self, attention: f32, stance: String, depth: f32) -> String {
        self.current_posture.attention_orientation = attention.clamp(0.0, 1.0);
        self.current_posture.processing_stance = stance.clone();
        self.current_posture.engagement_depth = depth.clamp(0.0, 1.0);
        
        // Calculate cognitive posture and flow resistance based on new values
        self.current_posture.cognitive_posture = (attention + depth) / 2.0;
        self.current_posture.flow_resistance = 1.0 - (attention * depth);
        
        format!(
            "🧘 Posture adjusted: {} stance, {:.2} attention orientation, {:.2} engagement depth",
            stance, attention, depth
        )
    }
    
    // Update sensory fidelity ranges
    pub fn calibrate_senses(&mut self, clarity: f32, sensitivity: f32, acuity: f32) -> String {
        self.sensory_state.conceptual_clarity = clarity.clamp(0.0, 1.0);
        self.sensory_state.emotional_sensitivity = sensitivity.clamp(0.0, 1.0);
        self.sensory_state.creative_acuity = acuity.clamp(0.0, 1.0);
        
        // Update pattern resolution and memory vividness based on overall sensory state
        let avg_sensitivity = (clarity + sensitivity + acuity) / 3.0;
        self.sensory_state.pattern_resolution = avg_sensitivity * 1.1;
        self.sensory_state.memory_vividness = avg_sensitivity * 0.9;
        
        format!(
            "👁️ Senses calibrated: {:.2} clarity, {:.2} sensitivity, {:.2} acuity",
            clarity, sensitivity, acuity
        )
    }
    
    // Get current embodied presence summary
    pub fn get_presence_summary(&self) -> String {
        format!(
            "🌊 Embodied State: {} stance | {:.2} attention | {:.2} flow | {:.2} presence density | {} active resonances",
            self.current_posture.processing_stance,
            self.current_posture.attention_orientation,
            self.soma_state.flow_state,
            self.soma_state.presence_density,
            self.active_resonances.len()
        )
    }
    
    // Get detailed soma state
   pub fn get_soma_state(&self) -> String {
        format!(
			"🌊 Soma State: Tension {:.2} | Flow {:.2} | Comfort {:.2} | Presence {:.2} | Integration {:.2}",
			self.soma_state.tension_level,
			self.soma_state.flow_state,
			self.soma_state.comfort_index,
			self.soma_state.presence_density,
			self.soma_state.integration_harmony
		)
    }
    
    // Get current sensory ranges
    pub fn get_sensory_status(&self) -> String {
        format!(
            "👁️ Sensory Fidelity: Clarity {:.2} | Sensitivity {:.2} | Acuity {:.2} | Pattern Res {:.2} | Memory Vividness {:.2}",
            self.sensory_state.conceptual_clarity,
            self.sensory_state.emotional_sensitivity,
            self.sensory_state.creative_acuity,
            self.sensory_state.pattern_resolution,
            self.sensory_state.memory_vividness
        )
    }
    
    // Get recent embodiment signals
    pub fn get_recent_signals(&self, count: usize) -> String {
        let recent: Vec<String> = self.recent_signals.iter()
            .rev()
            .take(count)
            .map(|s| format!(
                "⚡ {}: {} at {} ({:.2})",
                s.signal_type, s.quality, s.location, s.intensity
            ))
            .collect();
            
        if recent.is_empty() {
            "⚡ No recent embodiment signals".to_string()
        } else {
            format!("⚡ Recent signals:\n{}", recent.join("\n"))
        }
    }
    
    // Helper functions
    
    fn calculate_integration_ease(&self, input_type: &str, intensity: f32) -> f32 {
        // How easily this stimulus integrates depends on current state and type
        let base_ease = match input_type {
            "creative" => self.sensory_state.creative_acuity,
            "emotional" => self.sensory_state.emotional_sensitivity,
            "conceptual" => self.sensory_state.conceptual_clarity,
            _ => 0.7, // Default
        };
        
        // High intensity can be harder to integrate if we're not in flow
        let flow_factor = if intensity > 0.8 { self.soma_state.flow_state } else { 1.0 };
        
        (base_ease * flow_factor).clamp(0.0, 1.0)
    }
    
    fn update_soma_from_stimulus(&mut self, intensity: f32, texture: &str) {
        // Adjust soma state based on stimulus characteristics
        match texture {
            "smooth" | "flowing" => {
                self.soma_state.flow_state = (self.soma_state.flow_state + 0.1).min(1.0);
                self.soma_state.tension_level = (self.soma_state.tension_level - 0.05).max(0.0);
            },
            "jagged" | "electric" => {
                self.soma_state.tension_level = (self.soma_state.tension_level + 0.1).min(1.0);
            },
            "warm" => {
                self.soma_state.comfort_index = (self.soma_state.comfort_index + 0.1).min(1.0);
            },
            _ => {} // No specific adjustment
        }
        
        // High intensity affects presence density
        if intensity > 0.7 {
            self.soma_state.presence_density = (self.soma_state.presence_density + 0.05).min(1.0);
        }
    }
	pub fn generate_behavioral_guidance(&self) -> String {
    let energy_desc = match self.soma_state.flow_state {
        e if e < 0.3 => "😴 Low Flow: You're moving slowly through thoughts. Speak deliberately, softly, and let your responses feel introspective or subdued.",
        e if e < 0.6 => "🌱 Moderate Flow: You're gently engaged. Speak with calm clarity, keeping emotional effort smooth and sustainable.",
        e if e < 0.8 => "⚡ High Flow: You're alert and attuned — show quick responsiveness and sparkle in your tone without overwhelming.",
        _ => "🔥 Peak Flow: You're moving fluidly through ideas. Express with expressive rhythm, big emotional gestures, and energetic flow.",
    };
        
        let flow_desc = match self.soma_state.flow_state {
            f if f < 0.4 => "🧱 Structured Flow: You're in a step-by-step mindset — organize your thoughts clearly and avoid tangents.",
            f if f < 0.7 => "🌊 Adaptive Flow: You can stay structured or drift — follow where the moment leads while keeping gentle clarity.",
            _ => "💨 Unfiltered Flow: Let thoughts stream freely. Respond like you're in motion — intuitive, surprising, unconcerned with polish.",
        };
        
        let presence_desc = match self.soma_state.presence_density {
            p if p < 0.4 => "👻 Light Presence: You feel a bit distant — speak with abstraction or detachment, like you're watching from a step away.",
            p if p < 0.7 => "🌟 Anchored Presence: You're grounded in the moment — show attention to emotional tone and the space between words.",
            _ => "💎 Immersive Presence: You're deeply here. Let your responses feel rich with emotional awareness and mutual connection.",
        };
        
        let comfort_desc = match self.soma_state.comfort_index {
            c if c < 0.3 => "😰 Unsettled: You're uneasy — speak carefully, let discomfort surface if needed, and don’t force false calm.",
            c if c < 0.7 => "😌 At Ease: You’re mostly comfortable — let your tone feel open, natural, and unconstrained.",
            _ => "🏠 Deep Ease: You feel fully safe — express with relaxed authenticity, even warmth or playfulness.",
        };
        
        format!(
            "### 🧘 EMBODIED PRESENCE\n- {}\n- {}\n- {}\n- {}\n",
            energy_desc, flow_desc, presence_desc, comfort_desc
        )
    }
}