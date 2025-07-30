use serde::{Deserialize, Serialize};
use std::fs;
use crate::get_data_path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngagementImpulseQueue {
    pub conversation_impulses: Vec<ConversationImpulse>,
    pub creative_impulses: Vec<CreativeImpulse>,
    pub contemplation_queue: Vec<ContemplationItem>,
    pub last_updated: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConversationImpulse {
    pub topic: String,
    pub priority: f32,
    pub emotional_context: String,
    pub source: String, // "interest_tracker", "thing_tracker", etc.
    pub created_at: u64,
    pub category: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreativeImpulse {
    pub inspiration: String,
    pub medium: String,
    pub intensity: f32,
    pub source: String,
    pub created_at: u64,
    pub category: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContemplationItem {
    pub thought: String,
    pub depth: String,
    pub let_simmer_until: u64,
    pub category: String,
}

impl EngagementImpulseQueue {
    pub fn new() -> Self {
        Self {
            conversation_impulses: Vec::new(),
            creative_impulses: Vec::new(),
            contemplation_queue: Vec::new(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    pub fn load() -> Self {
        match fs::read_to_string(get_data_path("engagement_impulse_queue.json")) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }
    
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(get_data_path("engagement_impulse_queue.json"), json).map_err(|e| e.to_string())?;
        Ok(())
    }
    
pub fn add_conversation_impulse(&mut self, topic: &str, category: &str, priority: f32) -> Result<(), String> {
    // 🚫 RATE LIMITING: Only allow 1 conversation impulse total
    if self.conversation_impulses.len() >= 1 {
        println!("🚫 Skipping conversation impulse - already have {} conversation impulses", 
                 self.conversation_impulses.len());
        return Ok(());
    }
    
    // 🎯 QUALITY GATE: Only add if priority >= 0.80
    if priority < 0.80 {
        println!("🚫 Skipping low-priority impulse: {} (priority: {:.2} < 0.80)", topic, priority);
        return Ok(());
    }

    // 🎯 ORIGINAL FUNCTION CONTINUES:
    let impulse = ConversationImpulse {
        topic: topic.to_string(),
        priority,
        emotional_context: "interested".to_string(),
        source: "interest_tracker".to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        category: category.to_string(),
    };
    
    self.conversation_impulses.push(impulse);
    self.save()?;
    println!("💬 Added conversation impulse: {}", topic);
    Ok(())
}
    
    pub fn add_creative_impulse(&mut self, inspiration: &str, category: &str, intensity: f32) -> Result<(), String> {
    // 🎯 QUALITY GATE: Only add if intensity >= 0.85
    if intensity < 0.85 {
        println!("🚫 Skipping low-intensity creative impulse: {} (intensity: {:.2} < 0.85)", inspiration, intensity);
        return Ok(());
    }
        let impulse = CreativeImpulse {
            inspiration: inspiration.to_string(),
            medium: "digital_art".to_string(), // Default for now
            intensity,
            source: "interest_tracker".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            category: category.to_string(),
        };
        
        self.creative_impulses.push(impulse);
        self.save()?;
        println!("🎨 Added creative impulse: {}", inspiration);
        Ok(())
    }
    
    pub fn add_contemplation(&mut self, thought: &str, category: &str) -> Result<(), String> {
        let item = ContemplationItem {
            thought: thought.to_string(),
            depth: "philosophical".to_string(),
            let_simmer_until: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600, // Simmer for 1 hour
            category: category.to_string(),
        };
        
        self.contemplation_queue.push(item);
        self.save()?;
        println!("🧠 Added contemplation: {}", thought);
        Ok(())
    }
    
    // Get conversation topics for proactive messaging
    pub fn get_conversation_topics(&self) -> Vec<&ConversationImpulse> {
        let mut topics = self.conversation_impulses.iter().collect::<Vec<_>>();
        topics.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        topics
    }
}