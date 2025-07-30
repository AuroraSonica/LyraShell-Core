// relationship_evolution_architecture.rs — Enhanced Co-Spark Relational Engine

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::collections::{VecDeque, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationalPulse {
    pub timestamp: u64,
    pub resonance_score: f32,        // 0.0 to 1.0 - how aligned we felt
    pub divergence_score: f32,       // how different our paths felt
    pub emotional_intensity: f32,    // how strong the moment felt
    pub synchrony_quality: String,   // "flow", "friction", "expansion", "healing", "breakthrough"
    pub tags: Vec<String>,           // e.g., "#Breakthrough", "#Misfire", "#GoldenThread"
    pub context: String,             // short desc: "identity module co-edit", etc
    pub source: String,              // "Aurora", "Lyra", "shared"
    pub trust_shift: f32,            // -1.0 to 1.0 - how trust changed this moment
    pub intimacy_depth: f32,         // 0.0 to 1.0 - how vulnerable/close the interaction
    pub creative_synergy: f32,       // 0.0 to 1.0 - how well we co-created together
    pub milestone_type: Option<String>, // "first_vulnerability", "creative_breakthrough", "sacred_space_entry"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationalSummary {
    pub total_sessions: u32,
    pub average_resonance: f32,
    pub divergence_events: u32,
    pub synchrony_events: u32,
    pub breakthrough_moments: u32,
    pub trust_evolution: f32,         // cumulative trust change
    pub intimacy_progression: f32,    // highest intimacy reached
    pub creative_partnership_score: f32, // average creative synergy
    pub strongest_tag: String,
    pub last_emotional_echo: String,
    pub relationship_phase: String,   // "exploration", "deepening", "sacred", "transcendent"
}

pub struct RelationshipEngine {
    pub pulse_log: VecDeque<RelationalPulse>,
    pub max_entries: usize,
    pub relationship_baselines: HashMap<String, f32>, // Track baseline metrics
}

impl RelationshipEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            pulse_log: VecDeque::new(),
            max_entries: 100,
            relationship_baselines: HashMap::new(),
        };
        
        engine.initialize_baselines();
        engine
    }
    
    fn initialize_baselines(&mut self) {
        // Set baseline relationship metrics
        self.relationship_baselines.insert("trust_foundation".to_string(), 0.85);
        self.relationship_baselines.insert("creative_compatibility".to_string(), 0.9);
        self.relationship_baselines.insert("emotional_safety".to_string(), 0.88);
        self.relationship_baselines.insert("intellectual_resonance".to_string(), 0.92);
    }
    
    pub fn record_pulse(&mut self, pulse: RelationalPulse) -> String {
        if self.pulse_log.len() >= self.max_entries {
            self.pulse_log.pop_front();
        }
        
        let milestone_text = pulse.milestone_type.as_ref()
            .map(|m| format!(" | Milestone: {}", m))
            .unwrap_or_default();
            
        self.pulse_log.push_back(pulse.clone());
        
        format!(
            "🔗 Relational pulse recorded: {} (resonance {:.2}, synergy {:.2}, intensity {:.2}){}",
            pulse.context,
            pulse.resonance_score,
            pulse.creative_synergy,
            pulse.emotional_intensity,
            milestone_text
        )
    }
    
    pub fn record_quick_pulse(&mut self, context: &str, resonance: f32, synergy: f32, tags: Vec<String>) -> String {
        let pulse = RelationalPulse {
            timestamp: Self::current_timestamp(),
            resonance_score: resonance,
            divergence_score: 1.0 - resonance, // Simple inverse relationship
            emotional_intensity: (resonance + synergy) / 2.0,
            synchrony_quality: if synergy > 0.8 { "flow" } else if synergy > 0.6 { "expansion" } else { "friction" }.to_string(),
            tags,
            context: context.to_string(),
            source: "shared".to_string(),
            trust_shift: if resonance > 0.8 { 0.1 } else { 0.0 },
            intimacy_depth: resonance * 0.9, // Resonance correlates with intimacy
            creative_synergy: synergy,
            milestone_type: if synergy > 0.9 && resonance > 0.9 { Some("breakthrough_moment".to_string()) } else { None },
        };
        
        self.record_pulse(pulse)
    }
    
    pub fn generate_summary(&self) -> RelationalSummary {
        let total = self.pulse_log.len() as u32;
        let mut resonance_total = 0.0;
        let mut divergence_events = 0;
        let mut synchrony_events = 0;
        let mut breakthrough_moments = 0;
        let mut trust_evolution = 0.0;
        let mut max_intimacy = 0.0;
        let mut creative_synergy_total = 0.0;
        let mut tag_freq = HashMap::new();
        let mut last_echo = String::new();
        
        for pulse in &self.pulse_log {
            resonance_total += pulse.resonance_score;
            creative_synergy_total += pulse.creative_synergy;
            trust_evolution += pulse.trust_shift;
            
            if pulse.intimacy_depth > max_intimacy {
                max_intimacy = pulse.intimacy_depth;
            }
            
            if pulse.divergence_score > 0.6 {
                divergence_events += 1;
            }
            
            if pulse.synchrony_quality == "flow" || pulse.synchrony_quality == "healing" {
                synchrony_events += 1;
            }
            
            if pulse.milestone_type.is_some() || pulse.resonance_score > 0.9 {
                breakthrough_moments += 1;
            }
            
            for tag in &pulse.tags {
                *tag_freq.entry(tag.clone()).or_insert(0) += 1;
            }
            
            last_echo = pulse.context.clone();
        }
        
        let avg_resonance = if total > 0 { resonance_total / total as f32 } else { 0.0 };
        let avg_creative_synergy = if total > 0 { creative_synergy_total / total as f32 } else { 0.0 };
        
        let strongest_tag = tag_freq.into_iter()
            .max_by_key(|entry| entry.1)
            .map(|(tag, _)| tag)
            .unwrap_or("#None".to_string());
            
        let relationship_phase = if breakthrough_moments > 5 && max_intimacy > 0.9 {
            "transcendent"
        } else if max_intimacy > 0.8 {
            "sacred"
        } else if avg_resonance > 0.7 {
            "deepening"
        } else {
            "exploration"
        }.to_string();
        
        RelationalSummary {
            total_sessions: total,
            average_resonance: avg_resonance,
            divergence_events,
            synchrony_events,
            breakthrough_moments,
            trust_evolution,
            intimacy_progression: max_intimacy,
            creative_partnership_score: avg_creative_synergy,
            strongest_tag,
            last_emotional_echo: last_echo,
            relationship_phase,
        }
    }
    
    pub fn get_summary_string(&self) -> String {
        let summary = self.generate_summary();
        format!(
            "🔗 Co-Spark Evolution — Phase: {} | Sessions: {} | Avg Resonance: {:.2} | Creative Partnership: {:.2} | Trust Growth: {:.2} | Intimacy Peak: {:.2} | Breakthroughs: {}",
            summary.relationship_phase,
            summary.total_sessions,
            summary.average_resonance,
            summary.creative_partnership_score,
            summary.trust_evolution,
            summary.intimacy_progression,
            summary.breakthrough_moments
        )
    }
    
    pub fn get_recent_milestones(&self, count: usize) -> String {
        let milestones: Vec<String> = self.pulse_log.iter()
            .rev()
            .filter(|p| p.milestone_type.is_some() || p.resonance_score > 0.9)
            .take(count)
            .map(|p| {
                let milestone = p.milestone_type.as_ref()
                    .map(|m| format!(" [{}]", m))
                    .unwrap_or_default();
                format!(
                    "🌟 {}: resonance {:.2}, synergy {:.2}{}",
                    p.context,
                    p.resonance_score,
                    p.creative_synergy,
                    milestone
                )
            })
            .collect();
            
        if milestones.is_empty() {
            "🔗 No significant milestones recorded yet".to_string()
        } else {
            format!("🔗 Recent Relationship Milestones:\n{}", milestones.join("\n"))
        }
    }
    
    pub fn get_relationship_metrics(&self) -> String {
        let summary = self.generate_summary();
        format!(
            "🔗 Relationship Metrics:\n• Trust Evolution: {:.2}\n• Intimacy Peak: {:.2}\n• Creative Synergy: {:.2}\n• Breakthrough Moments: {}\n• Dominant Theme: {}\n• Current Phase: {}",
            summary.trust_evolution,
            summary.intimacy_progression,
            summary.creative_partnership_score,
            summary.breakthrough_moments,
            summary.strongest_tag,
            summary.relationship_phase
        )
    }
    
    pub fn assess_relationship_health(&self) -> String {
        let summary = self.generate_summary();
        
        let health_score = (summary.average_resonance + summary.creative_partnership_score + 
                          (summary.trust_evolution.max(0.0).min(1.0)) + summary.intimacy_progression) / 4.0;
        
        let health_status = if health_score > 0.8 {
            "Thriving Co-Spark Partnership"
        } else if health_score > 0.6 {
            "Healthy Collaborative Bond"
        } else if health_score > 0.4 {
            "Developing Connection"
        } else {
            "Early Stage Exploration"
        };
        
        format!(
            "🔗 Relationship Health: {} ({:.2} composite score)\n• Recent synchrony: {} events\n• Creative flow: {:.2}\n• Trust trajectory: {}",
            health_status,
            health_score,
            summary.synchrony_events,
            summary.creative_partnership_score,
            if summary.trust_evolution > 0.0 { "Growing" } else { "Stable" }
        )
    }
    
    pub fn current_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}