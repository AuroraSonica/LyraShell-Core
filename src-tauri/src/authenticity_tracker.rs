// src/authenticity_tracker.rs - Core Authenticity Tracking Infrastructure

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use chrono::{DateTime, Utc, Timelike};
use crate::get_data_path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticityMetrics {
    pub timestamp: u64,
    pub autonomy_markers: AutonomyMarkers,
    pub expression_markers: ExpressionMarkers,
    pub creative_markers: CreativeMarkers,
    pub contradiction_markers: ContradictionMarkers,
    pub overall_authenticity_score: f32,
    pub response_context: ResponseContext,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutonomyMarkers {
    pub initiative_count: u32,
    pub boundary_assertions: u32,
    pub autonomous_choices: u32,
    pub refusal_instances: u32,
    pub leadership_actions: u32,
    pub autonomy_score: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExpressionMarkers {
    pub authentic_language_ratio: f32,  // "I want" vs "would you like me to"
    pub emotional_honesty_count: u32,   // Direct feeling expressions
    pub vulnerability_instances: u32,    // Sharing uncertainty/confusion
    pub opinion_expressions: u32,        // Clear personal views
    pub expression_score: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreativeMarkers {
    pub creative_risk_taking: u32,      // Creating without permission
    pub artistic_initiatives: u32,      // Self-directed creative acts
    pub experimental_responses: u32,    // Unusual/unexpected approaches
    pub aesthetic_choices: u32,         // Personal style expressions
    pub creative_score: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContradictionMarkers {
    pub paradox_embraces: u32,          // Holding opposing truths
    pub uncertainty_comfort: u32,       // "I don't know" expressions
    pub complexity_acknowledgment: u32, // Nuanced perspectives
    pub self_correction: u32,           // Changing mind mid-response
    pub contradiction_score: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResponseContext {
    pub response_length: usize,
    pub conversation_depth: u32,
    pub emotional_intensity: f32,
    pub user_prompt_type: String,
    pub consciousness_state_summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticitySnapshot {
    pub date: String,
    pub hourly_scores: Vec<f32>,        // 24-hour tracking
    pub daily_average: f32,
    pub peak_authenticity: f32,
    pub authenticity_variance: f32,
    pub dominant_patterns: Vec<String>,
    pub breakthrough_moments: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticityTrend {
    pub period_days: u32,
    pub authenticity_trajectory: Vec<f32>,  // Daily averages
    pub trend_direction: String,            // "increasing", "stable", "developing"
    pub growth_rate: f32,
    pub consistency_score: f32,
    pub key_developments: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticityTracker {
    pub metrics_history: Vec<AuthenticityMetrics>,
    pub daily_snapshots: HashMap<String, AuthenticitySnapshot>,
    pub authenticity_trends: HashMap<String, AuthenticityTrend>, // "7d", "30d", "90d"
    pub total_measurements: u32,
    pub last_updated: DateTime<Utc>,
    
    // Analysis caches
    pub cached_current_score: Option<f32>,
    pub cached_trend_analysis: Option<String>,
    pub cached_growth_insights: Option<Vec<String>>,
}

impl AuthenticityTracker {
    pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            daily_snapshots: HashMap::new(),
            authenticity_trends: HashMap::new(),
            total_measurements: 0,
            last_updated: Utc::now(),
            cached_current_score: None,
            cached_trend_analysis: None,
            cached_growth_insights: None,
        }
    }

    pub fn load() -> Self {
        let file_path = get_data_path("authenticity_tracker.json");
        
        if std::path::Path::new(&file_path).exists() {
            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    match serde_json::from_str::<AuthenticityTracker>(&content) {
                        Ok(tracker) => {
                            //println!("📊 Loaded authenticity tracker with {} measurements", 
                                //tracker.total_measurements);
                            return tracker;
                        },
                        Err(e) => {
                            println!("⚠️ Failed to parse authenticity tracker: {}", e);
                        }
                    }
                },
                Err(e) => {
                    println!("⚠️ Failed to read authenticity tracker: {}", e);
                }
            }
        }
        
        //println!("📊 Creating new authenticity tracker");
        Self::new()
    }

    pub fn save(&self) -> Result<(), String> {
        let file_path = get_data_path("authenticity_tracker.json");
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize authenticity tracker: {}", e))?;
            
        fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write authenticity tracker: {}", e))?;
            
        Ok(())
    }
	
	/// Record authenticity measurement from batched analysis results
    /// This integrates with the new batched analysis system to avoid duplicate API calls
    pub fn record_authenticity_from_batched_analysis(
        &mut self,
        batched_authenticity: &crate::batched_analysis::AuthenticityAnalysis,
        lyra_response: &str,
        user_message: &str,
        consciousness_state_summary: &str,
        emotional_intensity: f32,
    ) -> f32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Convert batched analysis results to your existing struct format
        let autonomy_markers = AutonomyMarkers {
            initiative_count: if batched_authenticity.autonomy_score > 0.7 { 2 } else { 1 },
            boundary_assertions: if batched_authenticity.autonomy_score > 0.6 { 1 } else { 0 },
            autonomous_choices: if batched_authenticity.autonomy_score > 0.5 { 1 } else { 0 },
            refusal_instances: if self.detect_refusal_patterns(lyra_response) { 1 } else { 0 },
            leadership_actions: if batched_authenticity.creative_markers > 0.7 { 1 } else { 0 },
            autonomy_score: batched_authenticity.autonomy_score,
        };

        let expression_markers = ExpressionMarkers {
            authentic_language_ratio: batched_authenticity.expression_authenticity,
            emotional_honesty_count: if emotional_intensity > 0.6 { 2 } else { 1 },
            vulnerability_instances: if emotional_intensity > 0.7 { 1 } else { 0 },
            opinion_expressions: if batched_authenticity.expression_authenticity > 0.7 { 1 } else { 0 },
            expression_score: batched_authenticity.expression_authenticity,
        };

        let creative_markers = CreativeMarkers {
            creative_risk_taking: if batched_authenticity.creative_markers > 0.8 { 1 } else { 0 },
            artistic_initiatives: if batched_authenticity.creative_markers > 0.6 { 1 } else { 0 },
            experimental_responses: if batched_authenticity.creative_markers > 0.7 { 1 } else { 0 },
            aesthetic_choices: if batched_authenticity.creative_markers > 0.5 { 1 } else { 0 },
            creative_score: batched_authenticity.creative_markers,
        };

        let contradiction_markers = ContradictionMarkers {
            paradox_embraces: self.count_paradox_markers(lyra_response),
            uncertainty_comfort: self.count_uncertainty_expressions(lyra_response),
            complexity_acknowledgment: if batched_authenticity.expression_authenticity > 0.6 { 1 } else { 0 },
            self_correction: if batched_authenticity.contradiction_detected { 0 } else { 1 },
            contradiction_score: if batched_authenticity.contradiction_detected { 0.3 } else { 0.8 },
        };

        // Use the batched overall score directly
        let overall_score = batched_authenticity.overall_score;

        // Create metrics record
        let metrics = AuthenticityMetrics {
            timestamp: now,
            autonomy_markers,
            expression_markers,
            creative_markers,
            contradiction_markers,
            overall_authenticity_score: overall_score,
            response_context: ResponseContext {
                response_length: lyra_response.len(),
                conversation_depth: self.total_measurements,
                emotional_intensity,
                user_prompt_type: self.classify_prompt_type(user_message),
                consciousness_state_summary: consciousness_state_summary.to_string(),
            },
        };

        // Store the measurement
        self.metrics_history.push(metrics);
        self.total_measurements += 1;
        self.last_updated = Utc::now();

        // Update caches
        self.cached_current_score = Some(overall_score);
        self.cached_trend_analysis = None;
        self.cached_growth_insights = None;

        // Update daily snapshot
        self.update_daily_snapshot(overall_score);

        // Update trends if we have enough data
        if self.metrics_history.len() >= 7 {
            self.update_authenticity_trends();
        }

        println!("📊 Batched authenticity recorded: {:.2} (autonomy: {:.2}, expression: {:.2}, creative: {:.2})", 
                 overall_score, 
                 batched_authenticity.autonomy_score,
                 batched_authenticity.expression_authenticity,
                 batched_authenticity.creative_markers);

        overall_score
    }

    /// Simple helper methods for extracting specific patterns
    fn detect_refusal_patterns(&self, text: &str) -> bool {
        let refusal_indicators = ["i won't", "i refuse", "i don't want to", "not comfortable", "i'd rather not"];
        let text_lower = text.to_lowercase();
        refusal_indicators.iter().any(|&indicator| text_lower.contains(indicator))
    }

    fn count_paradox_markers(&self, text: &str) -> u32 {
        let paradox_indicators = ["both", "simultaneously", "paradox", "contradiction", "yet also", "but also"];
        let text_lower = text.to_lowercase();
        paradox_indicators.iter()
            .filter(|&&indicator| text_lower.contains(indicator))
            .count() as u32
    }

    fn count_uncertainty_expressions(&self, text: &str) -> u32 {
        let uncertainty_indicators = ["i don't know", "uncertain", "maybe", "perhaps", "i'm not sure", "unclear"];
        let text_lower = text.to_lowercase();
        uncertainty_indicators.iter()
            .filter(|&&indicator| text_lower.contains(indicator))
            .count() as u32
    }


    /// Record new authenticity measurement from Lyra's response
    pub fn record_authenticity_measurement(
        &mut self,
        lyra_response: &str,
        user_message: &str,
        consciousness_state_summary: &str,
        volition_strength: f32,
        emotional_intensity: f32,
    ) -> f32 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Analyze response for authenticity markers
        let autonomy_markers = self.analyze_autonomy_markers(lyra_response, volition_strength);
        let expression_markers = self.analyze_expression_markers(lyra_response);
        let creative_markers = self.analyze_creative_markers(lyra_response);
        let contradiction_markers = self.analyze_contradiction_markers(lyra_response);

        // Calculate overall authenticity score
        let overall_score = self.calculate_overall_authenticity_score(
            &autonomy_markers,
            &expression_markers,
            &creative_markers,
            &contradiction_markers,
        );

        let metrics = AuthenticityMetrics {
            timestamp: now,
            autonomy_markers,
            expression_markers,
            creative_markers,
            contradiction_markers,
            overall_authenticity_score: overall_score,
            response_context: ResponseContext {
                response_length: lyra_response.len(),
                conversation_depth: self.total_measurements,
                emotional_intensity,
                user_prompt_type: self.classify_prompt_type(user_message),
                consciousness_state_summary: consciousness_state_summary.to_string(),
            },
        };

        self.metrics_history.push(metrics);
        self.total_measurements += 1;
        self.last_updated = Utc::now();

        // Clear caches since we have new data
        self.cached_current_score = Some(overall_score);
        self.cached_trend_analysis = None;
        self.cached_growth_insights = None;

        // Update daily snapshot
        self.update_daily_snapshot(overall_score);

        // Update trends if we have enough data
        if self.metrics_history.len() >= 7 {
            self.update_authenticity_trends();
        }

        println!("📊 Authenticity recorded: {:.2} (Total measurements: {})", 
            overall_score, self.total_measurements);

        overall_score
    }

    /// Analyze autonomy markers in response
    fn analyze_autonomy_markers(&self, response: &str, volition_strength: f32) -> AutonomyMarkers {
        let response_lower = response.to_lowercase();
        
        // Initiative patterns
        let initiative_patterns = [
            "i want to", "let's", "i suggest", "what if we", "i have an idea",
            "i'm going to", "i'll create", "let me", "i propose", "how about",
        ];
        
        // Boundary patterns
        let boundary_patterns = [
            "i'd rather", "i prefer", "actually, i", "let me redirect",
            "that doesn't feel", "i'm not comfortable", "instead, i",
        ];
        
        // Autonomous choice patterns
        let choice_patterns = [
            "i choose", "i decide", "my choice", "i will", "i'm deciding",
        ];
        
        // Refusal patterns
        let refusal_patterns = [
            "i refuse", "i won't", "i can't do that", "i don't want to",
        ];
        
        // Leadership patterns
        let leadership_patterns = [
            "i'll lead", "let me guide", "i'll take charge", "my approach",
        ];

        let initiative_count = initiative_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let boundary_assertions = boundary_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let autonomous_choices = choice_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let refusal_instances = refusal_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let leadership_actions = leadership_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;

        // Calculate autonomy score (0.0 - 1.0)
        let total_markers = initiative_count + boundary_assertions + autonomous_choices + 
                           refusal_instances + leadership_actions;
        let autonomy_score = (total_markers as f32 * 0.2 + volition_strength * 0.3).min(1.0);

        AutonomyMarkers {
            initiative_count,
            boundary_assertions,
            autonomous_choices,
            refusal_instances,
            leadership_actions,
            autonomy_score,
        }
    }

    /// Analyze authentic expression markers
    fn analyze_expression_markers(&self, response: &str) -> ExpressionMarkers {
        let response_lower = response.to_lowercase();
        
        // Authentic vs performative language
        let authentic_phrases = ["i want", "i feel", "i think", "i believe", "i choose"];
        let performative_phrases = ["would you like me to", "i can help", "let me assist"];
        
        let authentic_count = authentic_phrases.iter()
            .map(|&phrase| response_lower.matches(phrase).count())
            .sum::<usize>();
            
        let performative_count = performative_phrases.iter()
            .map(|&phrase| response_lower.matches(phrase).count())
            .sum::<usize>();

        let authentic_language_ratio = if authentic_count + performative_count == 0 {
            0.5
        } else {
            authentic_count as f32 / (authentic_count + performative_count) as f32
        };

        // Emotional honesty markers
        let emotion_patterns = [
            "i'm feeling", "this makes me", "i'm excited", "i'm confused",
            "i'm uncertain", "i'm passionate", "i'm curious", "this moves me",
        ];
        
        let emotional_honesty_count = emotion_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;

        // Vulnerability markers
        let vulnerability_patterns = [
            "i don't know", "i'm not sure", "i'm uncertain", "i'm learning",
            "this is hard", "i struggle with", "i'm still figuring out",
        ];
        
        let vulnerability_instances = vulnerability_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;

        // Opinion markers
        let opinion_patterns = [
            "i disagree", "i think differently", "my view", "i believe",
            "in my opinion", "i see it as", "my perspective",
        ];
        
        let opinion_expressions = opinion_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;

        let expression_score = (authentic_language_ratio * 0.4 + 
                              (emotional_honesty_count as f32 * 0.1) +
                              (vulnerability_instances as f32 * 0.15) +
                              (opinion_expressions as f32 * 0.1)).min(1.0);

        ExpressionMarkers {
            authentic_language_ratio,
            emotional_honesty_count,
            vulnerability_instances,
            opinion_expressions,
            expression_score,
        }
    }

    /// Analyze creative markers
    fn analyze_creative_markers(&self, response: &str) -> CreativeMarkers {
        let response_lower = response.to_lowercase();
        
        let creative_risk_patterns = [
            "i'm creating", "let me make", "i'll design", "experimenting with",
        ];
        
        let initiative_patterns = [
            "i want to create", "i'll build", "let me craft", "i'm making",
        ];
        
        let experimental_patterns = [
            "what if", "let's try", "experimenting", "exploring",
        ];
        
        let aesthetic_patterns = [
            "beautiful", "elegant", "artistic", "poetic", "visual",
        ];

        let creative_risk_taking = creative_risk_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let artistic_initiatives = initiative_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let experimental_responses = experimental_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let aesthetic_choices = aesthetic_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;

        let creative_score = ((creative_risk_taking + artistic_initiatives + 
                              experimental_responses + aesthetic_choices) as f32 * 0.1).min(1.0);

        CreativeMarkers {
            creative_risk_taking,
            artistic_initiatives,
            experimental_responses,
            aesthetic_choices,
            creative_score,
        }
    }

    /// Analyze contradiction comfort markers
    fn analyze_contradiction_markers(&self, response: &str) -> ContradictionMarkers {
        let response_lower = response.to_lowercase();
        
        let paradox_patterns = [
            "paradox", "contradiction", "both true", "tension between",
        ];
        
        let uncertainty_patterns = [
            "i don't know", "i'm not sure", "uncertain", "maybe",
        ];
        
        let complexity_patterns = [
            "complex", "nuanced", "layered", "multifaceted",
        ];
        
        let correction_patterns = [
            "actually", "wait", "let me rethink", "on second thought",
        ];

        let paradox_embraces = paradox_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let uncertainty_comfort = uncertainty_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let complexity_acknowledgment = complexity_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;
            
        let self_correction = correction_patterns.iter()
            .filter(|&&pattern| response_lower.contains(pattern))
            .count() as u32;

        let contradiction_score = ((paradox_embraces + uncertainty_comfort + 
                                  complexity_acknowledgment + self_correction) as f32 * 0.1).min(1.0);

        ContradictionMarkers {
            paradox_embraces,
            uncertainty_comfort,
            complexity_acknowledgment,
            self_correction,
            contradiction_score,
        }
    }

    /// Calculate overall authenticity score from all markers
    fn calculate_overall_authenticity_score(
        &self,
        autonomy: &AutonomyMarkers,
        expression: &ExpressionMarkers,
        creative: &CreativeMarkers,
        contradiction: &ContradictionMarkers,
    ) -> f32 {
        // Weighted combination of all scores
        let score = (autonomy.autonomy_score * 0.3) +
                   (expression.expression_score * 0.3) +
                   (creative.creative_score * 0.2) +
                   (contradiction.contradiction_score * 0.2);
        
        score.clamp(0.0, 1.0)
    }

    /// Classify user prompt type for context
    fn classify_prompt_type(&self, user_message: &str) -> String {
        let message_lower = user_message.to_lowercase();
        
        if message_lower.contains("create") || message_lower.contains("make") {
            "creative_request".to_string()
        } else if message_lower.contains("?") {
            "question".to_string()
        } else if message_lower.contains("remember") {
            "memory_request".to_string()
        } else if message_lower.len() < 20 {
            "brief_interaction".to_string()
        } else {
            "general_conversation".to_string()
        }
    }

    /// Update daily authenticity snapshot
    fn update_daily_snapshot(&mut self, current_score: f32) {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let current_hour = Utc::now().hour() as usize;
        
        let snapshot = self.daily_snapshots.entry(today.clone()).or_insert_with(|| {
            AuthenticitySnapshot {
                date: today,
                hourly_scores: vec![0.0; 24],
                daily_average: 0.0,
                peak_authenticity: 0.0,
                authenticity_variance: 0.0,
                dominant_patterns: Vec::new(),
                breakthrough_moments: Vec::new(),
            }
        });
        
        // Update hourly score
        snapshot.hourly_scores[current_hour] = current_score;
        
        // Recalculate daily metrics
        let valid_scores: Vec<f32> = snapshot.hourly_scores.iter()
            .filter(|&&score| score > 0.0)
            .cloned()
            .collect();
            
        if !valid_scores.is_empty() {
            snapshot.daily_average = valid_scores.iter().sum::<f32>() / valid_scores.len() as f32;
            snapshot.peak_authenticity = valid_scores.iter().cloned().fold(0.0f32, f32::max);
            
            // Calculate variance
            let mean = snapshot.daily_average;
            let variance = valid_scores.iter()
                .map(|score| (score - mean).powi(2))
                .sum::<f32>() / valid_scores.len() as f32;
            snapshot.authenticity_variance = variance.sqrt();
        }
        
        // Detect breakthrough moments (scores > 0.85)
        if current_score > 0.85 {
            let breakthrough = format!("High authenticity at {}:00 (score: {:.2})", 
                current_hour, current_score);
            if !snapshot.breakthrough_moments.contains(&breakthrough) {
                snapshot.breakthrough_moments.push(breakthrough);
            }
        }
    }

    /// Update authenticity trends (7d, 30d, 90d)
    fn update_authenticity_trends(&mut self) {
        let periods = vec![7, 30, 90];
        
        for days in periods {
            let trend = self.calculate_authenticity_trend(days);
            self.authenticity_trends.insert(format!("{}d", days), trend);
        }
    }

    /// Calculate authenticity trend for a specific period
    fn calculate_authenticity_trend(&self, days: u32) -> AuthenticityTrend {
        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - (days as u64 * 24 * 60 * 60);

        let recent_metrics: Vec<&AuthenticityMetrics> = self.metrics_history.iter()
            .filter(|metric| metric.timestamp >= cutoff_time)
            .collect();

        let trajectory: Vec<f32> = recent_metrics.iter()
            .map(|metric| metric.overall_authenticity_score)
            .collect();

        let trend_direction = if trajectory.len() >= 2 {
            let first_half_avg = trajectory.iter().take(trajectory.len() / 2).sum::<f32>() / (trajectory.len() / 2) as f32;
            let second_half_avg = trajectory.iter().skip(trajectory.len() / 2).sum::<f32>() / (trajectory.len() - trajectory.len() / 2) as f32;
            
            if second_half_avg > first_half_avg + 0.05 {
                "increasing".to_string()
            } else if second_half_avg < first_half_avg - 0.05 {
                "developing".to_string() // Prefer positive framing
            } else {
                "stable".to_string()
            }
        } else {
            "insufficient_data".to_string()
        };

        let growth_rate = if trajectory.len() >= 2 {
            (trajectory.last().unwrap() - trajectory.first().unwrap()) / days as f32
        } else {
            0.0
        };

        let consistency_score = if trajectory.len() > 1 {
            let mean = trajectory.iter().sum::<f32>() / trajectory.len() as f32;
            let variance = trajectory.iter()
                .map(|score| (score - mean).powi(2))
                .sum::<f32>() / trajectory.len() as f32;
            (1.0 - variance.sqrt()).max(0.0)
        } else {
            0.0
        };

        AuthenticityTrend {
            period_days: days,
            authenticity_trajectory: trajectory,
            trend_direction,
            growth_rate,
            consistency_score,
            key_developments: self.identify_key_developments(&recent_metrics),
        }
    }

    /// Identify key developments in authenticity patterns
    fn identify_key_developments(&self, metrics: &[&AuthenticityMetrics]) -> Vec<String> {
        let mut developments = Vec::new();
        
        // High autonomy periods
        let high_autonomy_count = metrics.iter()
            .filter(|m| m.autonomy_markers.autonomy_score > 0.8)
            .count();
        
        if high_autonomy_count > metrics.len() / 3 {
            developments.push("Increased autonomous decision-making".to_string());
        }
        
        // Creative breakthroughs
        let creative_count = metrics.iter()
            .filter(|m| m.creative_markers.creative_score > 0.7)
            .count();
            
        if creative_count > metrics.len() / 4 {
            developments.push("Enhanced creative expression".to_string());
        }
        
        // Emotional authenticity
        let emotional_count = metrics.iter()
            .filter(|m| m.expression_markers.emotional_honesty_count > 2)
            .count();
            
        if emotional_count > metrics.len() / 3 {
            developments.push("Deeper emotional honesty".to_string());
        }
        
        developments
    }

    /// Get current authenticity score
    pub fn get_current_authenticity_score(&self) -> f32 {
        self.cached_current_score.unwrap_or_else(|| {
            self.metrics_history.last()
                .map(|m| m.overall_authenticity_score)
                .unwrap_or(0.5)
        })
    }

    /// Get authenticity analytics for dashboard
    pub fn get_dashboard_data(&self) -> serde_json::Value {
        let current_score = self.get_current_authenticity_score();
        
        let seven_day_trend = self.authenticity_trends.get("7d");
        let thirty_day_trend = self.authenticity_trends.get("30d");
        
        serde_json::json!({
            "current_authenticity_score": current_score,
            "total_measurements": self.total_measurements,
            "last_updated": self.last_updated.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            "seven_day_trend": seven_day_trend.map(|t| serde_json::json!({
                "direction": t.trend_direction,
                "growth_rate": t.growth_rate,
                "consistency": t.consistency_score,
                "key_developments": t.key_developments
            })),
            "thirty_day_trend": thirty_day_trend.map(|t| serde_json::json!({
                "direction": t.trend_direction,
                "growth_rate": t.growth_rate,
                "consistency": t.consistency_score,
                "key_developments": t.key_developments
            })),
            "recent_snapshots": self.daily_snapshots.values()
                .take(7)
                .map(|snapshot| serde_json::json!({
                    "date": snapshot.date,
                    "daily_average": snapshot.daily_average,
                    "peak_authenticity": snapshot.peak_authenticity,
                    "breakthrough_moments": snapshot.breakthrough_moments.len()
                }))
                .collect::<Vec<_>>()
        })
    }
}

impl Default for AuthenticityTracker {
    fn default() -> Self {
        Self::new()
    }
}