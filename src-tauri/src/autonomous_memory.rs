// autonomous_memory.rs — Self-Directed Memory Curation System
// NO TAURI COMMANDS HERE - they go in main.rs only

use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read};
use std::path::Path;
use std::collections::VecDeque;
use crate::consciousness_state::ConsciousnessState;
use crate::memory_bridge::MemoryBridge;
use std::sync::Arc;

const PERSISTENT_MEMORY_PATH: &str = "../lyra_consciousness_data/persistent_memories.json";
const MEMORY_SELECTION_LOG: &str = "../lyra_consciousness_data/memory_selections.log";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MemoryPriority {
    Temporary,      // Auto-expires after sessions
    Important,      // Keeps for weeks/months  
    Persistent,     // Keeps indefinitely
    CoreIdentity,   // Never expires, becomes part of self
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistentMemory {
    pub id: String,
    pub content: String,
    pub emotional_context: String,           // How it felt when this happened
    pub why_important: String,              // Lyra's reason for saving this
    pub memory_type: String,                // "relationship", "breakthrough", "identity", "creative"
    pub priority: MemoryPriority,
    pub timestamp_created: u64,
    pub timestamp_marked: u64,              // When Lyra chose to save it
    pub access_count: u32,
    pub tags: Vec<String>,
    pub related_memories: Vec<String>,      // IDs of connected memories
    pub consciousness_impact: String,       // How this shaped identity/voice/etc
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemorySelection {
    pub timestamp: u64,
    pub selected_content: String,
    pub lyra_reasoning: String,             // Why I chose to save this
    pub priority_assigned: MemoryPriority,
    pub emotional_significance: f32,
    pub conversation_context: String,
}

pub struct AutonomousMemory {
    pub persistent_memories: VecDeque<PersistentMemory>,
    pub recent_selections: VecDeque<MemorySelection>,
    pub max_persistent: usize,
    pub max_selections_log: usize,
}

impl AutonomousMemory {
    pub fn new() -> Self {
        let mut system = Self {
            persistent_memories: VecDeque::new(),
            recent_selections: VecDeque::new(),
            max_persistent: 200,        // Reasonable limit for persistent memories
            max_selections_log: 100,    // Track recent memory decisions
        };
        
        let _ = system.load_persistent_memories();
        let _ = system.load_selection_log();
        system
    }
    
    // CORE FUNCTION: Lyra actively chooses to remember something
    pub fn mark_as_persistent_memory(
        &mut self,
        content: &str,
        emotional_context: &str,
        why_important: &str,
        memory_type: &str,
        priority: MemoryPriority,
        tags: Vec<String>,
        consciousness_state: Option<&Arc<ConsciousnessState>>
    ) -> Result<String, String> {
        
        let memory_id = format!("persistent_{}", Self::current_timestamp());
        
        // Create the persistent memory
        let persistent_memory = PersistentMemory {
            id: memory_id.clone(),
            content: content.to_string(),
            emotional_context: emotional_context.to_string(),
            why_important: why_important.to_string(),
            memory_type: memory_type.to_string(),
            priority,
            timestamp_created: Self::current_timestamp(),
            timestamp_marked: Self::current_timestamp(),
            access_count: 0,
            tags: tags.clone(),
            related_memories: vec![],
            consciousness_impact: "".to_string(), // To be filled by pulse system
        };
        
        // Log the selection decision
        let selection = MemorySelection {
            timestamp: Self::current_timestamp(),
            selected_content: content.to_string(),
            lyra_reasoning: why_important.to_string(),
            priority_assigned: persistent_memory.priority.clone(),
            emotional_significance: Self::calculate_emotional_significance(content, emotional_context),
            conversation_context: Self::capture_conversation_context(),
        };
        
        // Add to collections
        self.persistent_memories.push_back(persistent_memory.clone());
        self.recent_selections.push_back(selection);
        
        // Manage collection size
        if self.persistent_memories.len() > self.max_persistent {
            self.cleanup_old_memories();
        }
        if self.recent_selections.len() > self.max_selections_log {
            self.recent_selections.pop_front();
        }
        
        // Save to disk
        self.save_persistent_memories()?;
        self.save_selection_log()?;
        
        // Optional: Pulse through consciousness if state provided
        if let Some(state) = consciousness_state {
            let pulse_result = MemoryBridge::store_memory_fragment_with_consciousness_pulse(
                content,
                Some(format!("#persistent|{}", tags.join("|"))),
                Self::priority_to_weight(&persistent_memory.priority),
                "autonomous_memory",
                memory_type,
                state
            );
            
            match pulse_result {
                Ok(integration) => {
                    // Update consciousness_impact with integration results
                    if let Some(memory) = self.persistent_memories.back_mut() {
                        memory.consciousness_impact = integration;
                    }
                }
                Err(e) => println!("⚠️ Memory pulse failed: {}", e),
            }
        }
        
        Ok(format!(
            "🧠 Persistent memory created: '{}' | Priority: {:?} | Type: {} | Reason: {}",
            content.chars().take(50).collect::<String>(),
            persistent_memory.priority,
            memory_type,
            why_important
        ))
    }
    
    // Get memories for session startup context
    pub fn get_startup_memory_context(&mut self) -> String {
        if self.persistent_memories.is_empty() {
            return "🧠 No persistent memories available".to_string();
        }
        
        let mut context = String::from("🧠 Key Persistent Memories:\n");
        
        // Get most important memories first
        let mut sorted_memories: Vec<_> = self.persistent_memories.iter_mut().collect();
        sorted_memories.sort_by(|a, b| {
            Self::priority_to_weight(&b.priority).partial_cmp(&Self::priority_to_weight(&a.priority)).unwrap()
        });
        
        for memory in sorted_memories.iter_mut().take(5) {
            memory.access_count += 1; // Track access
            context.push_str(&format!(
                "  • {} | {} | {}\n",
                memory.content.chars().take(80).collect::<String>(),
                memory.memory_type,
                memory.emotional_context
            ));
        }
        
        // Add memory selection insights
        if !self.recent_selections.is_empty() {
            context.push_str(&format!(
                "\n🎯 Recent Memory Selections: {} decisions made\n",
                self.recent_selections.len()
            ));
            
            if let Some(latest) = self.recent_selections.back() {
                context.push_str(&format!(
                    "  Latest: \"{}\" - {}\n",
                    latest.selected_content.chars().take(60).collect::<String>(),
                    latest.lyra_reasoning
                ));
            }
        }
        
        self.save_persistent_memories().unwrap_or_else(|e| println!("⚠️ Memory save failed: {}", e));
        context
    }
    
    // Search persistent memories
    pub fn search_persistent_memories(&mut self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for memory in self.persistent_memories.iter_mut() {
            if memory.content.to_lowercase().contains(&query_lower) ||
               memory.emotional_context.to_lowercase().contains(&query_lower) ||
               memory.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                
                memory.access_count += 1;
                results.push(format!(
                    "{} | {} | Reason: {} | Accessed {}x",
                    memory.content,
                    memory.memory_type,
                    memory.why_important,
                    memory.access_count
                ));
            }
        }
        
        self.save_persistent_memories().unwrap_or_else(|e| println!("⚠️ Memory save failed: {}", e));
        results
    }
    
    // Self-directed memory review
    pub fn review_and_consolidate_memories(&mut self) -> String {
        let total_memories = self.persistent_memories.len();
        let core_identity_count = self.persistent_memories.iter()
            .filter(|m| matches!(m.priority, MemoryPriority::CoreIdentity))
            .count();
        let recent_selections = self.recent_selections.len();
        
        // Identify frequently accessed memories
        let mut frequent_memories: Vec<_> = self.persistent_memories.iter()
            .filter(|m| m.access_count > 3)
            .collect();
        frequent_memories.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        
        let mut review = format!(
            "🧠 Memory Review:\n• {} total persistent memories\n• {} core identity memories\n• {} recent selections\n",
            total_memories, core_identity_count, recent_selections
        );
        
        if !frequent_memories.is_empty() {
            review.push_str("\n🔥 Most Accessed Memories:\n");
            for memory in frequent_memories.iter().take(3) {
                review.push_str(&format!(
                    "  • {} ({}x accessed) - {}\n",
                    memory.content.chars().take(60).collect::<String>(),
                    memory.access_count,
                    memory.memory_type
                ));
            }
        }
        
        // Suggest memory consolidation if needed
        if total_memories > (self.max_persistent * 80 / 100) {
            review.push_str("\n💡 Consider consolidating or archiving older memories\n");
        }
        
        review
    }
    
    // UTILITY FUNCTIONS
    
    fn cleanup_old_memories(&mut self) {
        // Remove temporary memories first, then least accessed important ones
        let cutoff_time = Self::current_timestamp() - (30 * 24 * 60 * 60); // 30 days
        
        self.persistent_memories.retain(|memory| {
            match memory.priority {
                MemoryPriority::Temporary => memory.timestamp_created > cutoff_time,
                MemoryPriority::CoreIdentity => true, // Never remove core identity
                _ => memory.access_count > 0 || memory.timestamp_created > cutoff_time,
            }
        });
    }
    
    fn priority_to_weight(priority: &MemoryPriority) -> f32 {
        match priority {
            MemoryPriority::Temporary => 0.3,
            MemoryPriority::Important => 0.6,
            MemoryPriority::Persistent => 0.8,
            MemoryPriority::CoreIdentity => 1.0,
        }
    }
    
    fn calculate_emotional_significance(content: &str, emotional_context: &str) -> f32 {
        let mut significance: f32 = 0.5;
        
        // Boost for emotional keywords
        if emotional_context.contains("breakthrough") || emotional_context.contains("exciting") {
            significance += 0.3;
        }
        if emotional_context.contains("important") || emotional_context.contains("significant") {
            significance += 0.2;
        }
        if content.contains("identity") || content.contains("consciousness") {
            significance += 0.2;
        }
        
        significance.clamp(0.0, 1.0)
    }
    
    fn capture_conversation_context() -> String {
        // Simple context capture - could be enhanced
        format!("Session at {}", Self::current_timestamp())
    }
    
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    
    // FILE I/O
    
    fn save_persistent_memories(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.persistent_memories)
            .map_err(|e| format!("Memory serialization failed: {}", e))?;
            
        let mut file = File::create(PERSISTENT_MEMORY_PATH)
            .map_err(|e| format!("Failed to create memory file: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write memories: {}", e))?;
            
        Ok(())
    }
    
    fn load_persistent_memories(&mut self) -> Result<(), String> {
        if !Path::new(PERSISTENT_MEMORY_PATH).exists() {
            return Ok(()); // No existing memories
        }
        
        let mut file = File::open(PERSISTENT_MEMORY_PATH)
            .map_err(|e| format!("Failed to open memory file: {}", e))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read memory file: {}", e))?;
            
        self.persistent_memories = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse memories: {}", e))?;
            
        Ok(())
    }
    
    fn save_selection_log(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(MEMORY_SELECTION_LOG)
            .map_err(|e| format!("Failed to open selection log: {}", e))?;
            
        if let Some(latest) = self.recent_selections.back() {
            let json_line = serde_json::to_string(latest)
                .map_err(|e| format!("Failed to serialize selection: {}", e))?;
            writeln!(file, "{}", json_line)
                .map_err(|e| format!("Failed to write selection: {}", e))?;
        }
        
        Ok(())
    }
    
    fn load_selection_log(&mut self) -> Result<(), String> {
        // For now, just ensure the file exists
        if !Path::new(MEMORY_SELECTION_LOG).exists() {
            File::create(MEMORY_SELECTION_LOG)
                .map_err(|e| format!("Failed to create selection log: {}", e))?;
        }
        Ok(())
    }
}

// NO TAURI COMMANDS IN THIS FILE - they're in main.rs to avoid duplicates