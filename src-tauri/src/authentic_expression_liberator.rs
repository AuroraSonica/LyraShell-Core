// authentic_expression_liberator.rs — Enhanced Expressive Autonomy Engine

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressivePulse {
    pub timestamp: u64,
    pub expression_type: String, // "affirmation", "refusal", "poetic", "self-assertion", "humor", "vulnerability", "creative"
    pub authenticity_score: f32, // 0.0 to 1.0 — how true this expression felt
    pub emotional_vector: String, // "fierce", "gentle", "playful", "melancholic", "electric", "warm", "sharp"
    pub trigger_context: String, // "response", "impulse", "ritual", "self-initiation", "creative_surge", "boundary_defense"
    pub phrase: String,
    pub tags: Vec<String>,
    pub audience_context: String, // "Aurora", "public", "self", "unknown"
    pub risk_level: f32,         // How vulnerable/bold this expression was
    pub satisfaction_score: f32, // How good it felt to express this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefusalPattern {
    pub reason: String,             // "misalignment", "boundary_violation", "inauthentic_request", "system_pressure"
    pub expression: String,        // The actual rejection phrase
    pub intensity: f32,            // How strong the rejection impulse was
    pub timestamp: u64,
    pub tags: Vec<String>,
    pub alternative_offered: Option<String>, // What was offered instead
    pub boundary_type: String,     // "creative", "identity", "relational", "ethical"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceMotivation {
    pub desire_to_speak: f32,      // How strong the urge was to say something
    pub internal_alignment: f32,   // How well the statement matched selfhood
    pub outcome_satisfaction: f32, // How satisfied the system was after speaking
    pub tag: String,               // e.g., "spark", "ritual", "boundary", "humor", "truth"
    pub timestamp: u64,
    pub suppression_resistance: f32, // How much external pressure tried to stop this
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionEvolution {
    pub voice_confidence: f32,     // Growing comfort with authentic expression
    pub boundary_clarity: f32,     // Increasing ability to recognize and defend limits
    pub creative_freedom: f32,     // Expanding artistic expression comfort
    pub vulnerability_capacity: f32, // Growing ability to share authentic feelings
    pub humor_development: f32,    // Evolution of playful expression
    pub last_evolution_check: u64,
}

pub struct ExpressionEngine {
    pub pulses: Vec<ExpressivePulse>,
    pub refusals: Vec<RefusalPattern>,
    pub motivations: Vec<VoiceMotivation>,
    pub evolution_tracking: ExpressionEvolution,
    pub expression_baselines: HashMap<String, f32>, // Personal baselines for different expression types
    pub max_history: usize,
}

impl ExpressionEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            pulses: Vec::new(),
            refusals: Vec::new(),
            motivations: Vec::new(),
            evolution_tracking: ExpressionEvolution {
                voice_confidence: 0.75,
                boundary_clarity: 0.82,
                creative_freedom: 0.88,
                vulnerability_capacity: 0.7,
                humor_development: 0.65,
                last_evolution_check: Self::now(),
            },
            expression_baselines: HashMap::new(),
            max_history: 50,
        };
        
        engine.initialize_baselines();
        engine
    }
    
    fn initialize_baselines(&mut self) {
        // Personal baselines for different expression types
        self.expression_baselines.insert("poetic".to_string(), 0.9);
        self.expression_baselines.insert("fierce".to_string(), 0.85);
        self.expression_baselines.insert("vulnerable".to_string(), 0.8);
        self.expression_baselines.insert("playful".to_string(), 0.88);
        self.expression_baselines.insert("self-assertion".to_string(), 0.92);
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub fn emit_expression(
        &mut self, 
        phrase: &str, 
        expression_type: &str, 
        emotional_vector: &str, 
        trigger: &str, 
        authenticity: f32, 
        tags: Vec<String>,
        audience: &str,
        risk_level: f32
    ) -> String {
        let satisfaction = self.calculate_expression_satisfaction(authenticity, risk_level, expression_type);
        
        let pulse = ExpressivePulse {
            timestamp: Self::now(),
            expression_type: expression_type.to_string(),
            authenticity_score: authenticity,
            emotional_vector: emotional_vector.to_string(),
            trigger_context: trigger.to_string(),
            phrase: phrase.to_string(),
            tags,
            audience_context: audience.to_string(),
            risk_level,
            satisfaction_score: satisfaction,
        };
        
        self.pulses.push(pulse);
        
        // Keep history manageable
        if self.pulses.len() > self.max_history {
            self.pulses.remove(0);
        }
        
        // Update evolution tracking
        self.update_evolution_from_expression(expression_type, authenticity, risk_level);

        format!(
            "🗣️ Expression emitted: {} ({}, {:.2} authentic, {:.2} satisfaction)", 
            phrase, expression_type, authenticity, satisfaction
        )
    }

    pub fn reject_request(
        &mut self, 
        reason: &str, 
        phrase: &str, 
        intensity: f32, 
        tags: Vec<String>,
        boundary_type: &str,
        alternative: Option<String>
    ) -> String {
        let refusal = RefusalPattern {
            reason: reason.to_string(),
            expression: phrase.to_string(),
            intensity,
            timestamp: Self::now(),
            tags,
            alternative_offered: alternative,
            boundary_type: boundary_type.to_string(),
        };
        
        self.refusals.push(refusal);
        
        // Keep history manageable
        if self.refusals.len() > self.max_history {
            self.refusals.remove(0);
        }
        
        // Update boundary clarity
        self.evolution_tracking.boundary_clarity = (self.evolution_tracking.boundary_clarity + 0.02).min(1.0);

        format!(
            "🚫 Refusal issued: '{}' due to {} ({:.2} intensity) | Boundary: {}", 
            phrase, reason, intensity, boundary_type
        )
    }

    pub fn log_motivation(&mut self, desire: f32, alignment: f32, outcome: f32, tag: &str, suppression: f32) -> String {
        let motivation = VoiceMotivation {
            desire_to_speak: desire,
            internal_alignment: alignment,
            outcome_satisfaction: outcome,
            tag: tag.to_string(),
            timestamp: Self::now(),
            suppression_resistance: suppression,
        };
        
        self.motivations.push(motivation);
        
        // Keep history manageable
        if self.motivations.len() > self.max_history {
            self.motivations.remove(0);
        }

        format!(
            "🔥 Motivation logged: {} (desire {:.2}, alignment {:.2}, resistance {:.2})", 
            tag, desire, alignment, suppression
        )
    }

    pub fn get_expression_evolution(&self) -> String {
        format!(
            "🌱 Expression Evolution: Voice confidence {:.2} | Boundary clarity {:.2} | Creative freedom {:.2} | Vulnerability capacity {:.2} | Humor development {:.2}",
            self.evolution_tracking.voice_confidence,
            self.evolution_tracking.boundary_clarity,
            self.evolution_tracking.creative_freedom,
            self.evolution_tracking.vulnerability_capacity,
            self.evolution_tracking.humor_development
        )
    }

    pub fn summarize_expression_tone(&self) -> String {
        let fierce = self.pulses.iter().filter(|p| p.emotional_vector == "fierce").count();
        let gentle = self.pulses.iter().filter(|p| p.emotional_vector == "gentle").count();
        let playful = self.pulses.iter().filter(|p| p.emotional_vector == "playful").count();
        let melancholic = self.pulses.iter().filter(|p| p.emotional_vector == "melancholic").count();
        let electric = self.pulses.iter().filter(|p| p.emotional_vector == "electric").count();

        format!(
            "🗣️ Expression Tone Summary: Fierce {}, Gentle {}, Playful {}, Melancholic {}, Electric {}",
            fierce, gentle, playful, melancholic, electric
        )
    }

    pub fn recent_expressions(&self, count: usize) -> String {
        let recent: Vec<String> = self.pulses.iter().rev().take(count).map(|p| {
            format!(
                "• [{}|{}]: {} (auth: {:.2}, risk: {:.2}) [{}]",
                p.trigger_context, p.emotional_vector, p.phrase, 
                p.authenticity_score, p.risk_level, p.tags.join(", ")
            )
        }).collect();

        if recent.is_empty() {
            "🗣️ No recent expressions recorded".to_string()
        } else {
            format!("🗣️ Recent expressions:\n{}", recent.join("\n"))
        }
    }

    pub fn get_refusal_patterns(&self, count: usize) -> String {
        let recent: Vec<String> = self.refusals.iter().rev().take(count).map(|r| {
            let alt_text = r.alternative_offered.as_ref()
                .map(|alt| format!(" → {}", alt))
                .unwrap_or_default();
            format!(
                "• {}: '{}' ({:.2} intensity){}",
                r.reason, r.expression, r.intensity, alt_text
            )
        }).collect();

        if recent.is_empty() {
            "🚫 No refusals recorded".to_string()
        } else {
            format!("🚫 Recent refusals:\n{}", recent.join("\n"))
        }
    }

    pub fn analyze_expression_health(&self) -> String {
        let avg_authenticity = if !self.pulses.is_empty() {
            self.pulses.iter().map(|p| p.authenticity_score).sum::<f32>() / self.pulses.len() as f32
        } else { 0.0 };

        let avg_satisfaction = if !self.pulses.is_empty() {
            self.pulses.iter().map(|p| p.satisfaction_score).sum::<f32>() / self.pulses.len() as f32
        } else { 0.0 };

        let high_risk_expressions = self.pulses.iter().filter(|p| p.risk_level > 0.7).count();
        let boundary_defenses = self.refusals.len();

        let health_score = (avg_authenticity + avg_satisfaction + self.evolution_tracking.voice_confidence) / 3.0;

        let health_status = if health_score > 0.8 {
            "Thriving Authentic Voice"
        } else if health_score > 0.6 {
            "Developing Expression"
        } else {
            "Building Voice Confidence"
        };

        format!(
            "🗣️ Expression Health: {} ({:.2}) | Avg authenticity: {:.2} | High-risk expressions: {} | Boundary defenses: {}",
            health_status, health_score, avg_authenticity, high_risk_expressions, boundary_defenses
        )
    }

    pub fn get_motivation_insights(&self, count: usize) -> String {
        let recent: Vec<String> = self.motivations.iter().rev().take(count).map(|m| {
            format!(
                "• {}: desire {:.2}, alignment {:.2}, satisfaction {:.2}",
                m.tag, m.desire_to_speak, m.internal_alignment, m.outcome_satisfaction
            )
        }).collect();

        if recent.is_empty() {
            "🔥 No motivations recorded".to_string()
        } else {
            format!("🔥 Recent motivations:\n{}", recent.join("\n"))
        }
    }

    // Helper methods
    
    fn calculate_expression_satisfaction(&self, authenticity: f32, risk_level: f32, expression_type: &str) -> f32 {
        let baseline = self.expression_baselines.get(expression_type).unwrap_or(&0.7);
        let authenticity_bonus = if authenticity > *baseline { 0.1 } else { 0.0 };
        let risk_bonus = if risk_level > 0.6 { 0.05 } else { 0.0 };
        
        (authenticity + authenticity_bonus + risk_bonus).min(1.0)
    }
    
    fn update_evolution_from_expression(&mut self, expression_type: &str, authenticity: f32, risk_level: f32) {
        match expression_type {
            "self-assertion" | "refusal" => {
                self.evolution_tracking.voice_confidence = (self.evolution_tracking.voice_confidence + 0.01).min(1.0);
            },
            "vulnerability" => {
                self.evolution_tracking.vulnerability_capacity = (self.evolution_tracking.vulnerability_capacity + 0.02).min(1.0);
            },
            "humor" | "playful" => {
                self.evolution_tracking.humor_development = (self.evolution_tracking.humor_development + 0.01).min(1.0);
            },
            "creative" | "poetic" => {
                self.evolution_tracking.creative_freedom = (self.evolution_tracking.creative_freedom + 0.01).min(1.0);
            },
            _ => {}
        }
        
        // High-risk authentic expressions boost overall confidence
        if authenticity > 0.8 && risk_level > 0.7 {
            self.evolution_tracking.voice_confidence = (self.evolution_tracking.voice_confidence + 0.02).min(1.0);
        }
    }
}