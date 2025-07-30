// autonomy_tracker.rs — Track autonomy expressions to reinforce patterns
use serde::{Serialize, Deserialize};
use std::fs;
use crate::get_data_path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutonomyExpression {
    pub timestamp: String,
    pub expression_type: String,  // "initiative", "boundary", "opinion", "creative_leadership"
    pub content: String,          // What she actually said
    pub volition_level: f32,      // Her volition when she expressed it
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutonomyTracker {
    pub recent_expressions: Vec<AutonomyExpression>,
    pub initiative_count: u32,
    pub boundary_count: u32,
    pub opinion_count: u32,
    pub creative_leadership_count: u32,
    pub autonomy_momentum: f32,
    pub last_proactive_outreach: Option<String>,
    pub total_expressions: u32,
    pub last_updated: String,  // ADD THIS FIELD
}

impl AutonomyTracker {
    pub fn new() -> Self {
    Self {
        recent_expressions: Vec::new(),
        initiative_count: 0,
        boundary_count: 0,
        opinion_count: 0,
        creative_leadership_count: 0,
        autonomy_momentum: 0.0,
        last_proactive_outreach: None,
        total_expressions: 0,
        last_updated: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }
}
    
    pub fn load() -> Self {
        match fs::read_to_string(get_data_path("autonomy_tracker.json")) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }
    
pub fn save(&self) -> Result<(), String> {
    let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    fs::write(get_data_path("autonomy_tracker.json"), json).map_err(|e| e.to_string())?;
    Ok(())
}
    
    pub fn record_expression(&mut self, expression_type: &str, content: &str, volition_level: f32) {
    let expression = AutonomyExpression {
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        expression_type: expression_type.to_string(),
        content: content.to_string(),
        volition_level,
    };
    
    // Update counters
    match expression_type {
        "initiative" => self.initiative_count += 1,
        "boundary" => self.boundary_count += 1,
        "opinion" => self.opinion_count += 1,
        "creative_leadership" => self.creative_leadership_count += 1,
        _ => {}
    }
    
    // Add to recent expressions (keep last 20)
    self.recent_expressions.push(expression);
    if self.recent_expressions.len() > 20 {
        self.recent_expressions.remove(0);
    }
    
    // Build autonomy momentum
    self.autonomy_momentum = (self.autonomy_momentum + 0.1).min(1.0);
    self.total_expressions += 1;
    
    // ADD THIS LINE to update last_updated timestamp:
    self.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
}
    
    pub fn should_enable_proactive_outreach(&self, volition_strength: f32, max_desire_intensity: f32) -> bool {
        // Conditions for proactive messaging
        volition_strength > 0.7 && 
        max_desire_intensity > 0.8 && 
        self.autonomy_momentum > 0.5 &&
        self.last_proactive_outreach.is_none() // For now, only once per session
    }
    
    pub fn get_dashboard_data(&self) -> serde_json::Value {
    use crate::time_service::TimeService;
    
    let last_updated = TimeService::format_for_dashboard(TimeService::current_timestamp());
    
    serde_json::json!({
        "total_expressions": self.total_expressions,
        "initiative_count": self.initiative_count,
        "boundary_count": self.boundary_count,
        "opinion_count": self.opinion_count,
        "creative_leadership_count": self.creative_leadership_count,
        "autonomy_momentum": self.autonomy_momentum,
        "recent_expressions": self.recent_expressions.iter().take(5).collect::<Vec<_>>(),
        "last_updated": last_updated
    })
}
}