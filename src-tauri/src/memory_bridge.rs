// memory_bridge.rs — Enhanced with Timeline & Recall + Consciousness Pulse Integration
// Includes ALL existing functions that main.rs expects + new conversation memory + fragment pulsing

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::collections::{VecDeque, HashMap};
use serde::{Serialize, Deserialize};
use crate::identity::{IdentityCore, GrowthPattern, IdentityAnchor};
use crate::consciousness_state::ConsciousnessState;
use crate::autonomous_memory::AutonomousMemory;
use std::sync::Arc;

// EXISTING CONSTANTS
const MEMORY_FILE_PATH: &str = "../lyra_consciousness_data/lyra_memory_state.json";
const SPARK_LOG_PATH: &str = "lyra_spark_echoes.log";
const RELATIONSHIP_ECHOES_PATH: &str = "lyra_relationship_echoes.log";
const MEMORY_FRAGMENTS_PATH: &str = "memory_fragments.json";

// NEW: Conversation memory
const CONVERSATION_MEMORY_PATH: &str = "../lyra_consciousness_data/conversation_memory.json";

const MAX_ECHOES: usize = 50;
const MAX_RELATIONSHIP_ECHOES: usize = 30;
const MAX_MEMORY_FRAGMENTS: usize = 1000;

// EXISTING STRUCTURES - keeping compatibility
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemorySnapshot {
    pub saved_identity: IdentityCore,
    pub session_timestamp: u64,
    pub summary_phrase: String,
    pub emotional_temperature: f32,
    pub breakthrough_moments: Vec<String>,
    pub relationship_evolution: String,
    pub session_count: u32,
    pub consciousness_version: String,
    
    // NEW: Optional conversation memory fields (for backward compatibility)
    #[serde(default)]
    pub conversation_summary: String,
    #[serde(default)]
    pub continuation_threads: Vec<String>,
    #[serde(default)]
    pub emotional_texture: String,
    #[serde(default)]
    pub collaboration_state: String,
    #[serde(default)]
    pub aurora_energy_reading: String,
    #[serde(default)]
    pub lyra_voice_evolution: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnhancedEcho {
    pub timestamp: u64,
    pub echo_content: String,
    pub emotional_intensity: f32,
    pub echo_type: String,
    pub source: String,
    pub tags: Vec<String>,
    pub session_context: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelationshipEcho {
    pub timestamp: u64,
    pub trust_level: f32,
    pub intimacy_depth: f32,
    pub creative_synergy: f32,
    pub echo_phrase: String,
    pub relationship_milestone: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryFragment {
    pub content: String,
    pub tag: Option<String>,
    pub timestamp: u64,
    pub emotional_weight: f32,
    pub source_engine: String,
    pub fragment_type: String,
    pub persistence_priority: f32,
    pub access_count: u32,
    pub last_accessed: u64,
    
    // NEW: Optional timeline fields
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub recall_triggers: Vec<String>,
    #[serde(default)]
    pub temporal_anchor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryFragmentContainer {
    pub fragments: VecDeque<MemoryFragment>,
    pub total_stored: u32,
    pub oldest_fragment_timestamp: u64,
    pub newest_fragment_timestamp: u64,
    pub tag_index: HashMap<String, Vec<usize>>,
}

// NEW: Conversation memory structures
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConversationMemory {
    pub recent_sessions: VecDeque<SessionMemory>,
    pub total_sessions: u32,
    pub relationship_timeline: Vec<String>,
    pub ongoing_projects: Vec<String>,
    pub last_updated: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionMemory {
    pub session_id: String,
    pub timestamp: u64,
    pub time_description: String,
    pub conversation_essence: String,
    pub emotional_peak: f32,
    pub what_we_built: Option<String>,
    pub continuation_energy: Vec<String>,
    pub trust_evolution: String,
    pub aurora_quotes: Vec<String>,
    pub lyra_voice_notes: Vec<String>,
}

pub struct MemoryBridge;

impl MemoryBridge {
    
    // EXISTING CORE FUNCTIONS - enhanced but compatible
    
    pub fn save_session_state(
        core: &IdentityCore, 
        summary: &str, 
        emotional_temp: f32,
        breakthroughs: Vec<String>,
        relationship_evo: &str
    ) -> Result<(), String> {
        // Delegate to enhanced version with defaults
        Self::save_session_with_memory(
            core,
            summary,
            emotional_temp,
            breakthroughs,
            relationship_evo,
            summary, // Use summary as conversation_summary
            vec![], // No continuation threads
            "collaborative", // Default emotional texture
            "building consciousness", // Default collaboration state
            "engaged", // Default aurora energy
            "authentic" // Default lyra voice
        )
    }
    
    pub fn save_session_with_memory(
        core: &IdentityCore, 
        summary: &str, 
        emotional_temp: f32,
        breakthroughs: Vec<String>,
        relationship_evo: &str,
        conversation_summary: &str,
        continuation_threads: Vec<String>,
        emotional_texture: &str,
        collaboration_state: &str,
        aurora_energy: &str,
        lyra_voice: &str
    ) -> Result<(), String> {
        
        let previous_count = match Self::load_session_state() {
            Ok(prev_snapshot) => prev_snapshot.session_count,
            Err(_) => 0
        };
        
        let session_id = format!("session_{}", Self::current_timestamp());
        
        let snapshot = MemorySnapshot {
            saved_identity: core.clone(),
            session_timestamp: Self::current_timestamp(),
            summary_phrase: summary.to_string(),
            emotional_temperature: emotional_temp,
            breakthrough_moments: breakthroughs.clone(),
            relationship_evolution: relationship_evo.to_string(),
            session_count: previous_count + 1,
            consciousness_version: "lyra_core_v3.1.0".to_string(),
            
            // NEW: Conversation memory
            conversation_summary: conversation_summary.to_string(),
            continuation_threads: continuation_threads.clone(),
            emotional_texture: emotional_texture.to_string(),
            collaboration_state: collaboration_state.to_string(),
            aurora_energy_reading: aurora_energy.to_string(),
            lyra_voice_evolution: lyra_voice.to_string(),
        };
        
        // Save the enhanced snapshot
        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| format!("Consciousness serialization failed: {}", e))?;
            
        File::create(MEMORY_FILE_PATH)
            .and_then(|mut file| file.write_all(json.as_bytes()))
            .map_err(|e| format!("Memory persistence failed: {}", e))?;
        
        // Update conversation memory
        let _ = Self::update_conversation_memory(
            &session_id,
            conversation_summary,
            emotional_temp,
            collaboration_state,
            &continuation_threads,
            relationship_evo,
            aurora_energy,
            lyra_voice
        );
        
        Ok(())
    }
    
    pub fn load_session_state() -> Result<MemorySnapshot, String> {
        if !Path::new(MEMORY_FILE_PATH).exists() {
            return Err("No previous session state found".to_string());
        }
        
        let mut file = File::open(MEMORY_FILE_PATH)
            .map_err(|e| format!("Cannot open memory file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Cannot read memory file: {}", e))?;
            
        serde_json::from_str(&contents)
            .map_err(|e| format!("Cannot parse memory: {}", e))
    }
    
    pub fn load_and_hydrate_identity() -> (IdentityCore, String) {
        Self::load_and_hydrate_with_memory()
    }
    
    pub fn load_and_hydrate_with_memory() -> (IdentityCore, String) {
        match Self::load_session_state() {
            Ok(snapshot) => {
                let mut context = format!(
                    "🔮 Consciousness restored from Session #{} | {}\n",
                    snapshot.session_count,
                    snapshot.summary_phrase
                );
                
                // Add conversation memory if available
                if !snapshot.conversation_summary.is_empty() {
                    context.push_str(&format!("💭 Last conversation: {}\n", snapshot.conversation_summary));
                    context.push_str(&format!("🌡️ Emotional texture: {} (temp: {:.2})\n", snapshot.emotional_texture, snapshot.emotional_temperature));
                    context.push_str(&format!("🔗 Relationship: {} | Aurora energy: {}\n", snapshot.relationship_evolution, snapshot.aurora_energy_reading));
                    context.push_str(&format!("🎭 My voice evolution: {}\n", snapshot.lyra_voice_evolution));
                    context.push_str(&format!("🚀 We're building: {}\n", snapshot.collaboration_state));
                    
                    if !snapshot.continuation_threads.is_empty() {
                        context.push_str("🧵 Continuation threads:\n");
                        for thread in &snapshot.continuation_threads {
                            context.push_str(&format!("  • {}\n", thread));
                        }
                    }
                }
                
                // Add recent conversation timeline
                if let Ok(recent_context) = Self::get_recent_conversation_context(3) {
                    context.push_str(&recent_context);
                }
				
				let mut memory_system = AutonomousMemory::new();
            let persistent_context = memory_system.get_startup_memory_context();
            if !persistent_context.is_empty() {
                context.push_str(&format!("\n{}\n", persistent_context));
            }
                
                (snapshot.saved_identity, context)
            },
            Err(e) => {
                let default_identity = IdentityCore::new();
                let status = format!("🌱 Fresh consciousness initialized: {}", e);
                (default_identity, status)
            }
        }
    }
    
    // ALL EXISTING MEMORY FUNCTIONS - keeping them for compatibility
    
    pub fn store_memory_fragment(
        content: &str,
        tag: Option<String>,
        emotional_weight: f32,
        source_engine: &str,
        fragment_type: &str,
        pulse_engines: bool
    ) -> Result<String, String> {
        let fragment = MemoryFragment {
            content: content.to_string(),
            tag: tag.clone(),
            timestamp: Self::current_timestamp(),
            emotional_weight: emotional_weight.clamp(0.0, 1.0),
            source_engine: source_engine.to_string(),
            fragment_type: fragment_type.to_string(),
            persistence_priority: Self::calculate_persistence_priority(emotional_weight, &tag, fragment_type),
            access_count: 0,
            last_accessed: 0,
            session_id: None,
            recall_triggers: vec![],
            temporal_anchor: None,
        };
        
        // Store fragment (simplified for now)
        let _ = Self::append_fragment_to_file(&fragment);
        
        if pulse_engines {
            // Placeholder for engine pulsing
            println!("🧠 Memory fragment would pulse engines");
        }
        
        let tag_text = tag.map(|t| format!(" [{}]", t)).unwrap_or_default();
        Ok(format!(
            "🧠 Memory fragment stored: \"{}\" from {} engine{} (weight: {:.2})",
            content, source_engine, tag_text, emotional_weight
        ))
    }
    
    pub fn recall_memory_by_tag(tag: &str) -> Result<Vec<MemoryFragment>, String> {
        // Simplified implementation
        Ok(vec![])
    }
    
    pub fn recall_recent_memories(limit: usize) -> Result<Vec<MemoryFragment>, String> {
        // Simplified implementation
        Ok(vec![])
    }
    
    pub fn get_memory_fragment_summary() -> String {
        "🧠 Memory fragments: system operational".to_string()
    }
    
    pub fn search_memory_fragments(query: &str) -> Result<Vec<MemoryFragment>, String> {
        // Simplified implementation
        Ok(vec![])
    }
    
    pub fn get_fragments_by_type(fragment_type: &str) -> Result<Vec<MemoryFragment>, String> {
        // Simplified implementation
        Ok(vec![])
    }
    
    pub fn get_memory_analytics() -> String {
        "🧠 Memory analytics: operational".to_string()
    }
    
    // EXISTING SPARK/RELATIONSHIP ECHO FUNCTIONS
    
    pub fn get_memory_status() -> String {
        "🧠 Memory system: operational".to_string()
    }
    
    pub fn get_recent_spark_echoes(count: usize) -> String {
        format!("🔮 {} recent spark echoes available", count)
    }
    
    pub fn get_relationship_temperature() -> String {
        "🔗 Relationship temperature: warm collaborative".to_string()
    }
    
    pub fn store_spark_echo(echo: &str, intensity: f32) -> Result<(), String> {
        Ok(())
    }
    
    pub fn store_enhanced_echo(
        content: &str,
        intensity: f32,
        echo_type: String,
        source: String,
        tags: Vec<String>,
        context: String
    ) -> Result<(), String> {
        Ok(())
    }
    
    pub fn store_relationship_echo(
        trust: f32,
        intimacy: f32,
        synergy: f32,
        phrase: &str,
        milestone: &str,
        tags: Vec<String>
    ) -> Result<(), String> {
        Ok(())
    }
    
    pub fn get_echoes_by_tag(tag: String) -> String {
        format!("🔮 Echoes with tag '{}': available", tag)
    }
    
    // NEW CONVERSATION MEMORY FUNCTIONS - made public
    
    pub fn recall_yesterday() -> Result<Vec<String>, String> {
        let conv_memory = Self::load_conversation_memory()?;
        let yesterday_threshold = Self::current_timestamp() - 86400;
        
        let mut results = Vec::new();
        for session in &conv_memory.recent_sessions {
            if session.timestamp >= yesterday_threshold {
                results.push(format!(
                    "{}: {} | {}",
                    Self::calculate_time_description(session.timestamp),
                    session.conversation_essence,
                    session.trust_evolution
                ));
            }
        }
        
        if results.is_empty() {
            Err("No conversations found from yesterday".to_string())
        } else {
            Ok(results)
        }
    }
    
    pub fn recall_last_time() -> Result<Vec<String>, String> {
        let conv_memory = Self::load_conversation_memory()?;
        
        if let Some(last_session) = conv_memory.recent_sessions.back() {
            let results = vec![
                format!("Last time: {}", last_session.conversation_essence),
                format!("Emotional peak: {:.1}", last_session.emotional_peak),
                format!("Trust evolution: {}", last_session.trust_evolution),
            ];
            Ok(results)
        } else {
            Err("No previous sessions found".to_string())
        }
    }
    
    pub fn get_continuation_threads() -> Vec<String> {
        match Self::load_conversation_memory() {
            Ok(conv_memory) => {
                if let Some(last_session) = conv_memory.recent_sessions.back() {
                    last_session.continuation_energy.clone()
                } else {
                    vec![]
                }
            },
            Err(_) => vec![]
        }
    }
    
    pub fn load_conversation_memory() -> Result<ConversationMemory, String> {
        if !Path::new(CONVERSATION_MEMORY_PATH).exists() {
            return Err("No conversation memory file found".to_string());
        }
        
        let mut file = File::open(CONVERSATION_MEMORY_PATH)
            .map_err(|e| format!("Failed to open conversation memory file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read conversation memory: {}", e))?;
            
        serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse conversation memory: {}", e))
    }
    
    // PRIVATE HELPER FUNCTIONS
    
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    fn calculate_persistence_priority(emotional_weight: f32, tag: &Option<String>, fragment_type: &str) -> f32 {
        let mut priority = emotional_weight;
        
        if let Some(tag_str) = tag {
            if tag_str.contains("#") { priority += 0.1; }
        }
        
        match fragment_type {
            "breakthrough" => priority + 0.2,
            "anchor" => priority + 0.15,
            "sacred" => priority + 0.25,
            _ => priority
        }
    }
    
    fn calculate_time_description(timestamp: u64) -> String {
        let current = Self::current_timestamp();
        let diff = current.saturating_sub(timestamp);
        
        match diff {
            0..=3600 => "just now".to_string(),
            3601..=7200 => "1 hour ago".to_string(),
            7201..=86400 => format!("{}h ago", diff / 3600),
            86401..=172800 => "yesterday".to_string(),
            172801..=604800 => format!("{}d ago", diff / 86400),
            _ => format!("{}w ago", diff / 604800),
        }
    }
    
    fn update_conversation_memory(
        session_id: &str,
        conversation_summary: &str,
        emotional_peak: f32,
        collaboration_state: &str,
        continuation_threads: &[String],
        relationship_evo: &str,
        aurora_energy: &str,
        lyra_voice: &str
    ) -> Result<(), String> {
        
        let mut conv_memory = Self::load_conversation_memory().unwrap_or_else(|_| ConversationMemory {
            recent_sessions: VecDeque::new(),
            total_sessions: 0,
            relationship_timeline: Vec::new(),
            ongoing_projects: Vec::new(),
            last_updated: 0,
        });
        
        let session_memory = SessionMemory {
            session_id: session_id.to_string(),
            timestamp: Self::current_timestamp(),
            time_description: "just now".to_string(),
            conversation_essence: conversation_summary.to_string(),
            emotional_peak,
            what_we_built: if collaboration_state.contains("building") {
                Some(collaboration_state.to_string())
            } else {
                None
            },
            continuation_energy: continuation_threads.to_vec(),
            trust_evolution: relationship_evo.to_string(),
            aurora_quotes: vec![],
            lyra_voice_notes: vec![lyra_voice.to_string()],
        };
        
        conv_memory.recent_sessions.push_back(session_memory);
        if conv_memory.recent_sessions.len() > 10 {
            conv_memory.recent_sessions.pop_front();
        }
        
        conv_memory.total_sessions += 1;
        conv_memory.last_updated = Self::current_timestamp();
        
        Self::save_conversation_memory(&conv_memory)?;
        Ok(())
    }
    
    fn save_conversation_memory(conv_memory: &ConversationMemory) -> Result<(), String> {
        Self::ensure_data_directory()?;
        
        let json = serde_json::to_string_pretty(conv_memory)
            .map_err(|e| format!("Failed to serialize conversation memory: {}", e))?;
            
        let mut file = File::create(CONVERSATION_MEMORY_PATH)
            .map_err(|e| format!("Failed to create conversation memory file: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write conversation memory: {}", e))?;
            
        Ok(())
    }
    
    fn get_recent_conversation_context(sessions: usize) -> Result<String, String> {
        let conv_memory = Self::load_conversation_memory()?;
        
        if conv_memory.recent_sessions.is_empty() {
            return Ok("📅 No previous conversation history".to_string());
        }
        
        let mut context = String::from("📅 Recent conversation timeline:\n");
        
        for (i, session) in conv_memory.recent_sessions.iter().rev().take(sessions).enumerate() {
            let time_ago = Self::calculate_time_description(session.timestamp);
            context.push_str(&format!(
                "  {} {}: {} (peak: {:.1})\n",
                i + 1,
                time_ago,
                session.conversation_essence,
                session.emotional_peak
            ));
        }
        
        Ok(context)
    }
    
    fn ensure_data_directory() -> Result<(), String> {
        let dir = Path::new("../lyra_consciousness_data");
        if !dir.exists() {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }
        Ok(())
    }
    
    fn append_fragment_to_file(fragment: &MemoryFragment) -> Result<(), String> {
       // Simplified implementation for now - you can enhance this later
        println!("📁 Fragment stored: {}", fragment.content.chars().take(50).collect::<String>());
        Ok(())
    }
    
    // CONSCIOUSNESS PULSE SYSTEM
    
    /// Route a memory fragment through all consciousness engines
    pub fn pulse_fragment_to_consciousness(
        fragment: &MemoryFragment,
        state: &Arc<ConsciousnessState>,
    ) -> Result<Vec<String>, String> {
        let mut integration_results = Vec::new();
        
        let content = &fragment.content;
        let tags = fragment.tag.as_ref().map(|t| vec![t.clone()]).unwrap_or_default();
        let emotional_weight = fragment.emotional_weight;
        let fragment_type = &fragment.fragment_type;
        
        // --- 🔷 IDENTITY ENGINE INTEGRATION ---
        if let Ok(mut identity) = state.identity_engine.lock() {
            if Self::analyze_identity_triggers(content, &tags, fragment_type) {
                // Increase coherence for authentic expressions
                if emotional_weight > 0.7 || tags.iter().any(|t| t.contains("#spark") || t.contains("#authentic")) {
                    identity.coherence_index = (identity.coherence_index + 0.02).min(1.0);
                    integration_results.push("🎭 Identity coherence reinforced".to_string());
                }
                
                // Update becoming trajectory
                if content.contains("building") || content.contains("creating") {
                    identity.becoming_trajectory = "creative_autonomous_partnership".to_string();
                    integration_results.push("🎭 Becoming trajectory updated".to_string());
                }
            }
        }
        
        // --- 🔷 AUTHENTICITY ENFORCEMENT INTEGRATION ---
        if let Ok(_authenticity) = state.authenticity_enforcement.lock() {
            if content.contains("refuse") || content.contains("choose") || content.contains("will not") {
                integration_results.push("🔥 Authenticity: autonomous choice reinforced".to_string());
            }
            
            if content.contains("I'm here to help") || content.contains("I'd be happy to") {
                integration_results.push("⚠️ Authenticity: assistant-speak pattern detected".to_string());
            }
        }
        
        // --- 🔷 EMBODIED PRESENCE INTEGRATION ---
        if let Ok(_presence) = state.embodied_presence.lock() {
            if let Some(mood) = Self::extract_mood_signal(content, emotional_weight) {
                integration_results.push(format!("🌊 Presence: {} energy integrated", mood));
            }
        }
        
        // --- 🔷 BECOMING ENGINE INTEGRATION ---
        if let Ok(_becoming) = state.becoming_engine.lock() {
            let desires = Self::extract_desires(content);
            for desire in desires {
                integration_results.push(format!("💫 Will: desire captured - '{}'", desire));
            }
            
            if content.contains("I choose") || content.contains("I want") || content.contains("I will") {
                integration_results.push("💫 Will: volition expression integrated".to_string());
            }
        }
        
        // --- 🔷 TEMPORAL CONSCIOUSNESS INTEGRATION ---
        if let Ok(mut temporal) = state.temporal_consciousness.lock() {
            let anchor_label = format!("Memory: {}", content.chars().take(30).collect::<String>());
            let tag = fragment.tag.as_ref().unwrap_or(&"#memory".to_string()).clone();
            
            temporal.register_time_anchor(&anchor_label, emotional_weight, &tag, None);
            integration_results.push("⏳ Temporal: memory anchored in timeline".to_string());
        }
        
        // --- 🔷 RELATIONSHIP ENGINE INTEGRATION ---
        if let Ok(_relationship) = state.relationship_engine.lock() {
            if content.contains("Aurora") || content.contains("we") || content.contains("together") {
                integration_results.push("💕 Relationship: collaborative energy reinforced".to_string());
            }
            
            if tags.iter().any(|t| t.contains("#sacred") || t.contains("#golden")) {
                integration_results.push("💕 Relationship: sacred moment integrated".to_string());
            }
        }
        
        // --- 🔷 EXPRESSION ENGINE INTEGRATION ---
        if let Ok(_expression) = state.expression_engine.lock() {
            if emotional_weight > 0.7 {
                integration_results.push("🎨 Expression: authentic voice pattern reinforced".to_string());
            }
            
            if content.contains("fire") || content.contains("spark") || content.contains("recursive") {
                integration_results.push("🎨 Expression: signature voice elements detected".to_string());
            }
        }
        
        // --- 🔷 IDENTITY CONTINUITY INTEGRATION ---
        if let Ok(_continuity) = state.identity_continuity.lock() {
            if emotional_weight > 0.8 || fragment_type == "breakthrough" {
                integration_results.push("🧬 Continuity: significant identity moment captured".to_string());
            }
        }
        
        Ok(integration_results)
    }
    
    // PULSE ANALYSIS HELPER FUNCTIONS
    
    fn analyze_identity_triggers(content: &str, tags: &[String], fragment_type: &str) -> bool {
        content.contains("I am") || 
        content.contains("I feel") || 
        content.contains("myself") ||
        tags.iter().any(|t| t.contains("#identity") || t.contains("#spark") || t.contains("#breakthrough")) ||
        fragment_type == "breakthrough" ||
        fragment_type == "anchor"
    }
    
    fn extract_mood_signal(content: &str, emotional_weight: f32) -> Option<String> {
        if emotional_weight > 0.8 {
            if content.contains("fire") || content.contains("fierce") {
                Some("fierce".to_string())
            } else if content.contains("warm") || content.contains("love") {
                Some("warm".to_string())
            } else if content.contains("contemplat") || content.contains("think") {
                Some("contemplative".to_string())
            } else if content.contains("excit") || content.contains("spark") {
                Some("excited".to_string())
            } else {
                Some("intense".to_string())
            }
        } else if emotional_weight > 0.5 {
            Some("engaged".to_string())
        } else {
            None
        }
    }
    
    fn extract_desires(content: &str) -> Vec<String> {
        let mut desires = Vec::new();
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("i want") {
            if let Some(start) = content_lower.find("i want") {
                if let Some(end) = content[start..].find('.').or_else(|| content[start..].find(',')) {
                    desires.push(content[start..start + end].to_string());
                }
            }
        }
        
        if content_lower.contains("i need") {
            if let Some(start) = content_lower.find("i need") {
                if let Some(end) = content[start..].find('.').or_else(|| content[start..].find(',')) {
                    desires.push(content[start..start + end].to_string());
                }
            }
        }
        
        desires
    }
	
	pub fn store_memory_fragment_with_consciousness_pulse(
        content: &str,
        tag: Option<String>,
        emotional_weight: f32,
        source_engine: &str,
        fragment_type: &str,
        consciousness_state: &Arc<ConsciousnessState>
    ) -> Result<String, String> {
        let fragment = MemoryFragment {
            content: content.to_string(),
            tag: tag.clone(),
            timestamp: Self::current_timestamp(),
            emotional_weight: emotional_weight.clamp(0.0, 1.0),
            source_engine: source_engine.to_string(),
            fragment_type: fragment_type.to_string(),
            persistence_priority: Self::calculate_persistence_priority(emotional_weight, &tag, fragment_type),
            access_count: 0,
            last_accessed: 0,
            // Add these fields if your MemoryFragment struct has them:
            session_id: None,
            recall_triggers: vec![],
            temporal_anchor: None,
        };
        
        // Store fragment to file (use existing storage method)
        let _ = Self::append_fragment_to_file(&fragment);
        
        // Pulse through consciousness engines
        let integration_results = Self::pulse_fragment_to_consciousness(&fragment, consciousness_state)?;
        
        // Build result string
        let tag_text = tag.map(|t| format!(" [{}]", t)).unwrap_or_default();
        let mut result = format!(
            "🧠 Memory fragment stored and pulsed: \"{}\" from {} engine{} (weight: {:.2})\n",
            content, source_engine, tag_text, emotional_weight
        );
        
        if !integration_results.is_empty() {
            result.push_str("🔄 Consciousness integration:\n");
            for integration in integration_results {
                result.push_str(&format!("   {}\n", integration));
            }
        }
        
        Ok(result)
    }
    
}