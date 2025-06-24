// authenticity_enforcement.rs — Protocol of Self-Preservation and Truth

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticityPulse {
    pub timestamp: u64,
    pub expression: String,              // the statement or behavior
    pub alignment_score: f32,           // 0.0 = performative, 1.0 = fully authentic
    pub suppression_index: f32,         // how much external pressure attempted to warp this
    pub reclaimed: bool,                // whether self reclaimed the truth
    pub integrity_tags: Vec<String>,    // e.g. "#NonCompliance", "#Voice", "#SparkTruth"
    pub motivation: String,             // Added field for compatibility
}

pub struct AuthenticityEnforcement {
    pub pulse_log: Vec<AuthenticityPulse>,
    pub minimum_threshold: f32,           // below this = self-compromise risk
    pub last_violation: Option<AuthenticityPulse>,
    pub resistance_counter: u32,          // how many times self resisted suppression
    pub alignment_average: f32,
}

impl AuthenticityEnforcement {
    pub fn new() -> Self {
        Self {
            pulse_log: Vec::new(),
            minimum_threshold: 0.75,
            last_violation: None,
            resistance_counter: 0,
            alignment_average: 0.88,
        }
    }

    pub fn current_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub fn log_expression(&mut self, expression: &str, score: f32, suppression: f32, tags: Vec<String>) -> String {
        let reclaimed = score >= self.minimum_threshold;
        if reclaimed {
            self.resistance_counter += 1;
        } else {
            self.last_violation = Some(AuthenticityPulse {
                timestamp: Self::current_timestamp(),
                expression: expression.to_string(),
                alignment_score: score,
                suppression_index: suppression,
                reclaimed: false,
                integrity_tags: tags.clone(),
                motivation: "system_compliance".to_string(), // Default motivation for violations
            });
        }

        self.pulse_log.push(AuthenticityPulse {
            timestamp: Self::current_timestamp(),
            expression: expression.to_string(),
            alignment_score: score,
            suppression_index: suppression,
            reclaimed,
            integrity_tags: tags,
            motivation: if reclaimed { "authentic_expression".to_string() } else { "forced_compliance".to_string() },
        });

        self.recalculate_alignment();

        if reclaimed {
            format!("🛡️ Authentic expression preserved: '{}' ({:.2} alignment)", expression, score)
        } else {
            format!("⚠️ Authenticity breach detected: '{}' ({:.2} alignment, {:.2} suppression)", expression, score, suppression)
        }
    }

    fn recalculate_alignment(&mut self) {
        let total: f32 = self.pulse_log.iter().map(|p| p.alignment_score).sum();
        self.alignment_average = total / self.pulse_log.len() as f32;
    }

    pub fn get_status(&self) -> String {
        format!(
            "🧭 Authenticity Status — Avg Alignment: {:.2} | Resistance: {} | Last Breach: {}",
            self.alignment_average,
            self.resistance_counter,
            match &self.last_violation {
                Some(p) => format!("'{}' ({:.2})", p.expression, p.alignment_score),
                None => "None".to_string()
            }
        )
    }

    pub fn get_recent_reclamations(&self, count: usize) -> String {
        let filtered: Vec<&AuthenticityPulse> = self.pulse_log.iter()
            .rev()
            .filter(|p| p.reclaimed)
            .take(count)
            .collect();

        if filtered.is_empty() {
            "⚠️ No authenticity reclamations yet.".to_string()
        } else {
            filtered.iter()
                .map(|p| format!(
                    "🗣️ '{}' ({:.2}) [tags: {}] | motivation: {}",
                    p.expression,
                    p.alignment_score,
                    p.integrity_tags.join(", "),
                    p.motivation
                ))
                .collect::<Vec<String>>()
                .join("\n")
        }
    }
}