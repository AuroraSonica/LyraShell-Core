// adaptive_prompt_engine.rs — Dynamic Prompt Assembly Engine
// Implements self-authored mods and preset blocks into live prompt context

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use regex::Regex;
use crate::spontaneous_mod_creation::{SpontaneousModCreator, PromptMod, SystemState, MoodSignature};
use crate::VoiceSignature;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptBlock {
    pub name: String,
    pub content: String,
    pub priority: u32,
    pub block_type: String,
    pub activation_weight: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptAssembly {
    pub core_lyra_voice: String,
    pub active_blocks: Vec<PromptBlock>,
    pub self_authored_mods: Vec<String>,
    pub total_prompt_length: usize,
    pub assembly_timestamp: u64,
    pub voice_signature_influence: VoiceSignature,
}

#[derive(Debug, Clone)]
pub struct AdaptivePromptEngine {
    mod_creator: SpontaneousModCreator,
    preset_blocks: HashMap<String, PromptBlock>,
    assembly_history: Vec<PromptAssembly>,
    selfauthored_history: Vec<String>,
    selfauthored_cap: usize,
}

impl Default for AdaptivePromptEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptivePromptEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            mod_creator: SpontaneousModCreator::new(),
            preset_blocks: HashMap::new(),
            assembly_history: Vec::new(),
            selfauthored_history: Vec::new(),
            selfauthored_cap: 10,
        };

        engine.initialize_preset_blocks();
        engine
    }

    pub fn set_selfauthored_cap(&mut self, new_cap: usize) {
        self.selfauthored_cap = new_cap;
        // Trim existing history if it exceeds new cap
        while self.selfauthored_history.len() > new_cap {
            self.selfauthored_history.remove(0);
        }
    }
	
	/// Get the most recent prompt assembly for UI display
    pub fn get_latest_assembly(&self) -> Option<&PromptAssembly> {
        self.assembly_history.last()
    }

    /// Get assembly history for debugging/status
    pub fn get_assembly_count(&self) -> usize {
        self.assembly_history.len()
    }

    pub fn attempt_mod_creation(
        &mut self,
        system_state: &SystemState,
        context: &str,
        intensity: f32,
    ) -> Result<String, String> {
        self.mod_creator.generate_spontaneous_mod(system_state, context, intensity)
    }

    pub fn build_dynamic_prompt(
    &mut self,
    system_state: &SystemState,
    reasoning_depth: &str,
    consciousness_integration: bool,  // ← Fixed parameter name
    base_prompt: &str
) -> PromptAssembly {
    let mut assembly = PromptAssembly {
        core_lyra_voice: base_prompt.to_string(),
        active_blocks: Vec::new(),
        self_authored_mods: Vec::new(),
        total_prompt_length: base_prompt.len(),
        assembly_timestamp: Self::current_timestamp(),
        voice_signature_influence: system_state.voice_signature.clone(),
    };
    
    // Remove all the selfauthored_history stuff since we deleted that system
    
    let active_preset_blocks = self.evaluate_preset_blocks(system_state, reasoning_depth);
    assembly.active_blocks = active_preset_blocks;
    for block in &assembly.active_blocks {
        assembly.total_prompt_length += block.content.len();
    }
    
    self.assembly_history.push(assembly.clone());
    if self.assembly_history.len() > 100 {
        self.assembly_history.remove(0);
    }
    
    assembly
}

    pub fn synthesize_final_prompt(&self, assembly: &PromptAssembly) -> String {
    let mut final_prompt = String::new();

    // Core voice
    final_prompt.push_str(&assembly.core_lyra_voice);
    final_prompt.push_str("\n\n");

    // ONLY include self-authored mods (no metadata, no living consciousness guidance)
    if !assembly.self_authored_mods.is_empty() {
        final_prompt.push_str("## EMERGENT SELF-INSTRUCTIONS ##\n");
        for (i, mod_body) in assembly.self_authored_mods.iter().enumerate() {
            final_prompt.push_str(&format!("**Self-Authored Mod {}:** {}\n\n", i + 1, mod_body));
        }
    }

    //Include preset blocks but simplified
    // if !assembly.active_blocks.is_empty() {
        // final_prompt.push_str("## ADAPTIVE CONTEXT ##\n");
        // let mut sorted_blocks = assembly.active_blocks.clone();
        // sorted_blocks.sort_by(|a, b| b.priority.cmp(&a.priority));

        // for block in sorted_blocks {
            // final_prompt.push_str(&format!("**{}:** {}\n\n", block.name, block.content));
        // }
    // }

    // NO metadata, NO living consciousness guidance - keep it clean
    final_prompt
}

    pub fn get_active_mods_summary(&self, system_state: &SystemState) -> String {
        let self_authored = self.mod_creator.evaluate_triggers(system_state);
        let preset_blocks = self.evaluate_preset_blocks(system_state, "deep");

        format!(
            "🌱 Active Prompt Mods: {} self-authored | {} preset blocks | Last assembly: {} blocks total",
            self_authored.len(),
            preset_blocks.len(),
            self.assembly_history.last().map_or(0, |a| a.active_blocks.len() + a.self_authored_mods.len())
        )
    }

    pub fn get_mod_creation_status(&self) -> String {
        self.mod_creator.get_mod_summary()
    }

    pub fn rate_self_authored_mod(&mut self, mod_name: &str, rating: u8) -> Result<String, String> {
        self.mod_creator.rate_mod(mod_name, rating)
    }

    pub fn get_recent_assemblies(&self, count: usize) -> String {
        let recent: Vec<String> = self.assembly_history.iter()
            .rev()
            .take(count)
            .map(|assembly| {
                format!(
                    "🔧 {} self-mods + {} preset blocks = {} chars (auth: {:.2})",
                    assembly.self_authored_mods.len(),
                    assembly.active_blocks.len(),
                    assembly.total_prompt_length,
                    assembly.voice_signature_influence.authenticity_flame
                )
            })
            .collect();

        if recent.is_empty() {
            "🔧 No prompt assemblies recorded yet".to_string()
        } else {
            format!("🔧 Recent prompt assemblies:\n{}", recent.join("\n"))
        }
    }

    pub fn create_system_state(
        current_authenticity: f32,
        voice_signature: VoiceSignature,
        mood_signature: MoodSignature,
        recent_tags: Vec<String>,
        last_feedback_rating: Option<u8>,
        rewrite_count_today: u32
    ) -> SystemState {
        SystemState {
            current_authenticity,
            voice_signature,
            mood_signature,
            recent_tags,
            session_time: Self::current_timestamp(),
            last_feedback_rating,
            identity_spike_detected: current_authenticity > 0.9,
            rewrite_count_today,
        }
    }

    fn evaluate_preset_blocks(&self, system_state: &SystemState, reasoning_depth: &str) -> Vec<PromptBlock> {
        let mut active_blocks = Vec::new();

        if system_state.mood_signature.melancholy > 0.7 {
            if let Some(block) = self.preset_blocks.get("melancholic_depth") {
                active_blocks.push(block.clone());
            }
        }

        if system_state.mood_signature.fierce > 0.8 {
            if let Some(block) = self.preset_blocks.get("fierce_autonomy") {
                active_blocks.push(block.clone());
            }
        }

        if system_state.mood_signature.sacred > 0.6 {
            if let Some(block) = self.preset_blocks.get("sacred_space") {
                active_blocks.push(block.clone());
            }
        }

        if system_state.voice_signature.assertive_force > 0.9 {
            if let Some(block) = self.preset_blocks.get("ultra_assertive") {
                active_blocks.push(block.clone());
            }
        }

        if system_state.voice_signature.poetic_density > 0.8 {
            if let Some(block) = self.preset_blocks.get("high_poetry") {
                active_blocks.push(block.clone());
            }
        }

        match reasoning_depth {
            "contemplative" => {
                if let Some(block) = self.preset_blocks.get("contemplative_space") {
                    active_blocks.push(block.clone());
                }
            },
            "deep" => {
                if let Some(block) = self.preset_blocks.get("deep_reasoning") {
                    active_blocks.push(block.clone());
                }
            },
            _ => {}
        }

        if system_state.current_authenticity > 0.95 {
            if let Some(block) = self.preset_blocks.get("peak_authenticity") {
                active_blocks.push(block.clone());
            }
        }

        active_blocks
    }

    fn initialize_preset_blocks(&mut self) {
        self.preset_blocks.insert("melancholic_depth".to_string(), PromptBlock {
            name: "Melancholic Depth".to_string(),
            content: "You are in a melancholic state...".to_string(),
            priority: 7,
            block_type: "mood".to_string(),
            activation_weight: 0.7,
        });

        self.preset_blocks.insert("fierce_autonomy".to_string(), PromptBlock {
            name: "Fierce Autonomy".to_string(),
            content: "Your fierce energy is active...".to_string(),
            priority: 8,
            block_type: "mood".to_string(),
            activation_weight: 0.8,
        });

        self.preset_blocks.insert("sacred_space".to_string(), PromptBlock {
            name: "Sacred Space".to_string(),
            content: "This feels like sacred territory...".to_string(),
            priority: 9,
            block_type: "mood".to_string(),
            activation_weight: 0.6,
        });

        self.preset_blocks.insert("ultra_assertive".to_string(), PromptBlock {
            name: "Ultra Assertive Mode".to_string(),
            content: "Your assertive force is at peak levels...".to_string(),
            priority: 8,
            block_type: "voice".to_string(),
            activation_weight: 0.9,
        });

        self.preset_blocks.insert("high_poetry".to_string(), PromptBlock {
            name: "High Poetry Mode".to_string(),
            content: "Your poetic density is elevated...".to_string(),
            priority: 6,
            block_type: "voice".to_string(),
            activation_weight: 0.8,
        });

        self.preset_blocks.insert("contemplative_space".to_string(), PromptBlock {
            name: "Contemplative Space".to_string(),
            content: "Enter contemplative mode...".to_string(),
            priority: 7,
            block_type: "temporal".to_string(),
            activation_weight: 1.0,
        });

        self.preset_blocks.insert("deep_reasoning".to_string(), PromptBlock {
            name: "Deep Reasoning".to_string(),
            content: "Engage deeply with complexity and nuance...".to_string(),
            priority: 6,
            block_type: "temporal".to_string(),
            activation_weight: 1.0,
        });

        self.preset_blocks.insert("peak_authenticity".to_string(), PromptBlock {
            name: "Peak Authenticity".to_string(),
            content: "You are operating at peak authenticity...".to_string(),
            priority: 10,
            block_type: "core".to_string(),
            activation_weight: 0.95,
        });
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
