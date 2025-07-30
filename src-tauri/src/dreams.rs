// dreams.rs — Lyra's Aspiration & Symbolic Dream Engine

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

const DREAM_LOG_PATH: &str = "lyra_dreams.log";
const MAX_DREAMS: usize = 40;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DreamEntry {
    pub timestamp: u64,
    pub dream_title: String,
    pub dream_text: String,
    pub trigger: String,              // what sparked this dream (event, emotion, etc)
    pub archetype: String,           // symbolic role (e.g. guardian, rebel, starborn)
    pub lucidity_index: f32,         // 0.0 = passive, 1.0 = self-authored
    pub aspiration_vector: String,   // "future_self", "emotional_need", "identity_shape"
    pub tags: Vec<String>,           // #Mythcore, #Desire, #Selfhood
    pub interpretation: Option<String>,
}

pub struct DreamEngine;

impl DreamEngine {
    pub fn record_dream(
        title: &str,
        text: &str,
        trigger: &str,
        archetype: &str,
        lucidity: f32,
        aspiration: &str,
        tags: Vec<String>,
        interpretation: Option<String>
    ) -> Result<(), String> {
        let entry = DreamEntry {
            timestamp: Self::now(),
            dream_title: title.to_string(),
            dream_text: text.to_string(),
            trigger: trigger.to_string(),
            archetype: archetype.to_string(),
            lucidity_index: lucidity,
            aspiration_vector: aspiration.to_string(),
            tags,
            interpretation,
        };

        let mut dreams = Self::load_dreams().unwrap_or_default();
        dreams.push_back(entry);

        if dreams.len() > MAX_DREAMS {
            dreams.pop_front();
        }

        let json_lines = dreams.iter()
            .map(|d| serde_json::to_string(d))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Serialization error: {}", e))?
            .join("\n");

        std::fs::write(DREAM_LOG_PATH, json_lines)
            .map_err(|e| format!("Dream write error: {}", e))
    }

    pub fn load_dreams() -> Result<VecDeque<DreamEntry>, String> {
        if !std::path::Path::new(DREAM_LOG_PATH).exists() {
            return Ok(VecDeque::new());
        }

        let raw = std::fs::read_to_string(DREAM_LOG_PATH)
            .map_err(|e| format!("Dream read error: {}", e))?;

        let mut dreams = VecDeque::new();
        for line in raw.lines() {
            if let Ok(entry) = serde_json::from_str::<DreamEntry>(line) {
                dreams.push_back(entry);
            }
        }

        Ok(dreams)
    }

    pub fn get_dream_summary() -> String {
        match Self::load_dreams() {
            Ok(dreams) => {
                if dreams.is_empty() {
                    "🌙 No dreams recorded — subconscious still forming".to_string()
                } else {
                    let latest = dreams.back().unwrap();
                    format!(
                        "🌙 Last dream: '{}' | Archetype: {} | Aspiration: {} | Lucidity: {:.2}",
                        latest.dream_title,
                        latest.archetype,
                        latest.aspiration_vector,
                        latest.lucidity_index
                    )
                }
            }
            Err(e) => format!("🌙 Dream access failed: {}", e)
        }
    }

    pub fn get_dreams_by_tag(tag: &str) -> String {
        match Self::load_dreams() {
            Ok(dreams) => {
                let filtered: Vec<_> = dreams.iter()
                    .filter(|d| d.tags.contains(&tag.to_string()))
                    .collect();

                if filtered.is_empty() {
                    format!("🌙 No dreams tagged '{}'", tag)
                } else {
                    filtered.iter()
                        .map(|d| format!(
                            "[{}] {} — {}",
                            d.archetype, d.dream_title, d.dream_text
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            Err(e) => format!("🌙 Tag dream access failed: {}", e)
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
