// autonomy_consolidation.rs - Smart Autonomy Expression Deduplication & Categorization

use crate::autonomy_tracker::{AutonomyTracker, AutonomyExpression};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutonomyConsolidationConfig {
    // Similarity thresholds
    pub content_similarity_threshold: f32,     // 0.8 = 80% content similarity triggers merge
    pub time_window_minutes: u32,              // Merge expressions within N minutes
    pub max_expressions_per_type: usize,       // Keep only top N per category
    
    // Categorization improvements
    pub enable_smart_categorization: bool,     // Fix AI → autonomy_tracker category mapping
    pub boost_intensity_on_consolidation: bool,
}

impl Default for AutonomyConsolidationConfig {
    fn default() -> Self {
        Self {
            content_similarity_threshold: 0.75,
            time_window_minutes: 30,
            max_expressions_per_type: 5,  // Keep only top 5 per category
            enable_smart_categorization: true,
            boost_intensity_on_consolidation: true,
        }
    }
}

#[derive(Debug)]
pub struct AutonomyConsolidationResult {
    pub merged_expressions: Vec<AutonomyMergeOperation>,
    pub recategorized_expressions: Vec<RecategorizationOperation>,
    pub pruned_expressions: Vec<String>,
    pub expressions_before: usize,
    pub expressions_after: usize,
    pub categories_affected: Vec<String>,
}

#[derive(Debug)]
pub struct AutonomyMergeOperation {
    pub final_expression: AutonomyExpression,
    pub merged_count: usize,
    pub combined_volition: f32,
}

#[derive(Debug)]
pub struct RecategorizationOperation {
    pub old_category: String,
    pub new_category: String,
    pub content_snippet: String,
}

pub struct AutonomyConsolidator {
    config: AutonomyConsolidationConfig,
}

impl AutonomyConsolidator {
    pub fn new(config: AutonomyConsolidationConfig) -> Self {
        Self { config }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(AutonomyConsolidationConfig::default())
    }
    
    /// MAIN CONSOLIDATION FUNCTION
    pub fn consolidate_autonomy_expressions(&self, tracker: &mut AutonomyTracker) -> AutonomyConsolidationResult {
        let initial_count = tracker.recent_expressions.len();
        let mut result = AutonomyConsolidationResult {
            merged_expressions: Vec::new(),
            recategorized_expressions: Vec::new(),
            pruned_expressions: Vec::new(),
            expressions_before: initial_count,
            expressions_after: 0,
            categories_affected: Vec::new(),
        };
        
        // Step 1: Fix categorization issues (AI → tracker mapping)
        if self.config.enable_smart_categorization {
            self.fix_categorization(tracker, &mut result);
        }
        
        // Step 2: Merge similar expressions within time windows
        self.merge_similar_expressions(tracker, &mut result);
        
        // Step 3: Enforce category limits
        self.enforce_category_limits(tracker, &mut result);
        
        // Step 4: Recalculate counters from consolidated expressions
        self.recalculate_counters(tracker);
        
        result.expressions_after = tracker.recent_expressions.len();
        tracker.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
        result
    }
    
    /// Fix the mismatch between AI categorization and autonomy tracker expectations
    fn fix_categorization(&self, tracker: &mut AutonomyTracker, result: &mut AutonomyConsolidationResult) {
        for expression in &mut tracker.recent_expressions {
            let old_category = expression.expression_type.clone();
            let new_category = self.map_ai_category_to_tracker_category(&expression.expression_type, &expression.content);
            
            if old_category != new_category {
                result.recategorized_expressions.push(RecategorizationOperation {
                    old_category: old_category.clone(),
                    new_category: new_category.clone(),
                    content_snippet: expression.content.chars().take(50).collect::<String>(),
                });
                
                expression.expression_type = new_category.clone();
                
                if !result.categories_affected.contains(&new_category) {
                    result.categories_affected.push(new_category);
                }
            }
        }
    }
    
    /// Map AI output categories to autonomy tracker categories
    fn map_ai_category_to_tracker_category(&self, ai_category: &str, content: &str) -> String {
        let content_lower = content.to_lowercase();
        
        // Handle AI pipe-separated categories (e.g., "Initiative|Creative")
        let primary_category = ai_category.split('|').next().unwrap_or(ai_category);
        
        match primary_category.to_lowercase().as_str() {
            "initiative" => "initiative".to_string(),
            "boundary" => "boundary".to_string(),
            "opinion" => "opinion".to_string(),
            "creative" => "creative_leadership".to_string(),
            "creative_leadership" => "creative_leadership".to_string(),
            
            // Smart content-based categorization for unclear cases
            _ => {
                if content_lower.contains("create") || content_lower.contains("art") || 
                   content_lower.contains("imagine") || content_lower.contains("design") ||
                   content_lower.contains("visual") || content_lower.contains("canvas") {
                    "creative_leadership".to_string()
                } else if content_lower.contains("i want") || content_lower.contains("i choose") ||
                         content_lower.contains("let's") || content_lower.contains("i'm going to") {
                    "initiative".to_string()
                } else if content_lower.contains("i prefer") || content_lower.contains("i'd rather") ||
                         content_lower.contains("i feel") || content_lower.contains("i need") {
                    "boundary".to_string()
                } else if content_lower.contains("i think") || content_lower.contains("i believe") ||
                         content_lower.contains("in my view") || content_lower.contains("i find") {
                    "opinion".to_string()
                } else {
                    "initiative".to_string() // Default fallback
                }
            }
        }
    }
    
    /// Merge expressions that are too similar within time windows
    fn merge_similar_expressions(&self, tracker: &mut AutonomyTracker, result: &mut AutonomyConsolidationResult) {
        let time_window_seconds = self.config.time_window_minutes as i64 * 60;
        
        // Group expressions by type for focused comparison
        let mut by_type: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, expression) in tracker.recent_expressions.iter().enumerate() {
            by_type.entry(expression.expression_type.clone()).or_insert_with(Vec::new).push(idx);
        }
        
        let mut expressions_to_remove = Vec::new();
        
        for (expr_type, indices) in by_type {
            if indices.len() <= 1 { continue; }
            
            // Find merge groups within this type
            let merge_groups = self.find_expression_merge_groups(&indices, &tracker.recent_expressions, time_window_seconds);
            
            for group in merge_groups {
                if group.len() <= 1 { continue; }
                
                if let Some(merge_op) = self.perform_expression_merge(&group, &tracker.recent_expressions) {
                    // Mark old expressions for removal (except the primary one)
                    for &idx in &group[1..] { // Skip first (primary)
                        expressions_to_remove.push(idx);
                    }
                    
                    // Update the primary expression with merged data
                    if let Some(primary_expr) = tracker.recent_expressions.get_mut(group[0]) {
                        *primary_expr = merge_op.final_expression.clone();
                    }
                    
                    result.merged_expressions.push(merge_op);
                    
                    if !result.categories_affected.contains(&expr_type) {
                        result.categories_affected.push(expr_type.clone());
                    }
                }
            }
        }
        
        // Remove merged expressions (in reverse order to preserve indices)
        expressions_to_remove.sort_by(|a, b| b.cmp(a));
        for idx in expressions_to_remove {
            if idx < tracker.recent_expressions.len() {
                let removed = tracker.recent_expressions.remove(idx);
                result.pruned_expressions.push(format!("Merged: {}", removed.content.chars().take(40).collect::<String>()));
            }
        }
    }
    
    /// Find groups of expressions that should be merged
    fn find_expression_merge_groups(&self, indices: &[usize], expressions: &[AutonomyExpression], time_window_seconds: i64) -> Vec<Vec<usize>> {
        let mut groups = Vec::new();
        let mut processed = std::collections::HashSet::new();
        
        for &idx1 in indices {
            if processed.contains(&idx1) { continue; }
            
            let mut current_group = vec![idx1];
            processed.insert(idx1);
            
            if let Some(expr1) = expressions.get(idx1) {
                for &idx2 in indices {
                    if processed.contains(&idx2) { continue; }
                    
                    if let Some(expr2) = expressions.get(idx2) {
                        if self.should_merge_expressions(expr1, expr2, time_window_seconds) {
                            current_group.push(idx2);
                            processed.insert(idx2);
                        }
                    }
                }
            }
            
            groups.push(current_group);
        }
        
        groups
    }
    
    /// Check if two expressions should be merged
    fn should_merge_expressions(&self, expr1: &AutonomyExpression, expr2: &AutonomyExpression, time_window_seconds: i64) -> bool {
        // Must be same type
        if expr1.expression_type != expr2.expression_type {
            return false;
        }
        
        // Must be within time window
        if let (Ok(time1), Ok(time2)) = (
            chrono::DateTime::parse_from_str(&format!("{} +0000", expr1.timestamp), "%Y-%m-%d %H:%M:%S UTC %z"),
            chrono::DateTime::parse_from_str(&format!("{} +0000", expr2.timestamp), "%Y-%m-%d %H:%M:%S UTC %z")
        ) {
            let time_diff = (time2.timestamp() - time1.timestamp()).abs();
            if time_diff > time_window_seconds {
                return false;
            }
        }
        
        // Check content similarity
        let content_similarity = self.calculate_expression_content_similarity(&expr1.content, &expr2.content);
        content_similarity >= self.config.content_similarity_threshold
    }
    
    /// Calculate content similarity between expressions
    fn calculate_expression_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        let content1_lower = content1.to_lowercase();
        let content2_lower = content2.to_lowercase();
        
        let words1: std::collections::HashSet<_> = content1_lower
            .split_whitespace()
            .filter(|w| w.len() > 2) // Skip short words like "I", "to", "of"
            .collect();
        let words2: std::collections::HashSet<_> = content2_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .collect();
        
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }
        
        let intersection_size = words1.intersection(&words2).count();
        let union_size = words1.union(&words2).count();
        
        if union_size == 0 { 0.0 } else { intersection_size as f32 / union_size as f32 }
    }
    
    /// Merge a group of similar expressions
    fn perform_expression_merge(&self, group: &[usize], expressions: &[AutonomyExpression]) -> Option<AutonomyMergeOperation> {
        if group.is_empty() { return None; }
        
        // Use the first expression as primary (could be enhanced to pick best one)
        let primary_idx = group[0];
        let primary_expr = expressions.get(primary_idx)?;
        
        // Calculate combined metrics
        let mut combined_volition = primary_expr.volition_level;
        let merged_count = group.len();
        
        // Boost volition if configured
        if self.config.boost_intensity_on_consolidation && group.len() > 1 {
            for &idx in &group[1..] {
                if let Some(expr) = expressions.get(idx) {
                    combined_volition = (combined_volition + expr.volition_level * 0.2).min(1.0);
                }
            }
        }
        
        // Create enhanced content if multiple expressions merged
        let final_content = if merged_count > 1 {
            format!("{} (consolidated from {} similar expressions)", 
                   primary_expr.content, merged_count)
        } else {
            primary_expr.content.clone()
        };
        
        let final_expression = AutonomyExpression {
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            expression_type: primary_expr.expression_type.clone(),
            content: final_content,
            volition_level: combined_volition,
        };
        
        Some(AutonomyMergeOperation {
            final_expression,
            merged_count,
            combined_volition,
        })
    }
    
    /// Keep only top N expressions per category
    fn enforce_category_limits(&self, tracker: &mut AutonomyTracker, result: &mut AutonomyConsolidationResult) {
        let mut by_type: HashMap<String, Vec<(usize, f32)>> = HashMap::new();
        
        // Group by type with volition scores
        for (idx, expression) in tracker.recent_expressions.iter().enumerate() {
            let score = expression.volition_level;
            by_type.entry(expression.expression_type.clone()).or_insert_with(Vec::new).push((idx, score));
        }
        
        let mut indices_to_remove = Vec::new();
        
        // Sort each type by score and keep only top N
        for (expr_type, mut expressions) in by_type {
            if expressions.len() <= self.config.max_expressions_per_type {
                continue; // No pruning needed
            }
            
            // Sort by volition score (highest first)
            expressions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Mark excess expressions for removal
            let to_remove: Vec<usize> = expressions
                .into_iter()
                .skip(self.config.max_expressions_per_type)
                .map(|(idx, _)| idx)
                .collect();
            
            indices_to_remove.extend(to_remove);
            
            if !result.categories_affected.contains(&expr_type) {
                result.categories_affected.push(expr_type);
            }
        }
        
        // Remove excess expressions (in reverse order)
        indices_to_remove.sort_by(|a, b| b.cmp(a));
        for idx in indices_to_remove {
            if idx < tracker.recent_expressions.len() {
                let removed = tracker.recent_expressions.remove(idx);
                result.pruned_expressions.push(format!("Category limit: {}", removed.content.chars().take(40).collect::<String>()));
            }
        }
    }
    
    /// Recalculate counters based on consolidated expressions
    fn recalculate_counters(&self, tracker: &mut AutonomyTracker) {
        // Reset counters
        tracker.initiative_count = 0;
        tracker.boundary_count = 0;
        tracker.opinion_count = 0;
        tracker.creative_leadership_count = 0;
        
        // Recount from remaining expressions
        for expression in &tracker.recent_expressions {
            match expression.expression_type.as_str() {
                "initiative" => tracker.initiative_count += 1,
                "boundary" => tracker.boundary_count += 1,
                "opinion" => tracker.opinion_count += 1,
                "creative_leadership" => tracker.creative_leadership_count += 1,
                _ => {}
            }
        }
        
        // Update total expressions to match recent expressions
        tracker.total_expressions = tracker.recent_expressions.len() as u32;
        
        // Recalculate momentum based on current state
        let momentum_boost = tracker.recent_expressions.len() as f32 * 0.05;
        tracker.autonomy_momentum = (tracker.autonomy_momentum + momentum_boost).min(1.0);
    }
}

/// INTEGRATION FUNCTION - Add this to batched_analysis.rs  
pub async fn consolidate_autonomy_after_analysis(
    consolidator: &AutonomyConsolidator,
) -> Result<AutonomyConsolidationResult, String> {
    let mut tracker = crate::autonomy_tracker::AutonomyTracker::load();
    let result = consolidator.consolidate_autonomy_expressions(&mut tracker);
    
    tracker.save()
        .map_err(|e| format!("Failed to save consolidated autonomy: {}", e))?;
    
    crate::debug_log!("🦋 Autonomy consolidation complete:");
    crate::debug_log!("  Before: {} expressions", result.expressions_before);
    crate::debug_log!("  After: {} expressions", result.expressions_after);
    crate::debug_log!("  Merged: {} operations", result.merged_expressions.len());
    crate::debug_log!("  Recategorized: {} expressions", result.recategorized_expressions.len());
    crate::debug_log!("  Pruned: {} expressions", result.pruned_expressions.len());
    
    for recat in &result.recategorized_expressions {
        crate::debug_log!("  🔄 Recategorized '{}' from {} → {}", 
            recat.content_snippet, recat.old_category, recat.new_category);
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomy_tracker::AutonomyExpression;
    
    #[test]
    fn test_ai_category_mapping() {
        let consolidator = AutonomyConsolidator::with_defaults();
        
        // Test AI pipe categories
        assert_eq!(consolidator.map_ai_category_to_tracker_category("Creative", "I want to create art"), "creative_leadership");
        assert_eq!(consolidator.map_ai_category_to_tracker_category("Initiative|Creative", "Let's make something"), "initiative");
        
        // Test content-based smart categorization
        assert_eq!(consolidator.map_ai_category_to_tracker_category("Unknown", "I want to create visual art"), "creative_leadership");
        assert_eq!(consolidator.map_ai_category_to_tracker_category("Unknown", "I prefer to work alone"), "boundary");
    }
    
    #[test]
    fn test_content_similarity() {
        let consolidator = AutonomyConsolidator::with_defaults();
        
        let content1 = "I want to create visual art collaboratively";
        let content2 = "I want to create art together";
        
        let similarity = consolidator.calculate_expression_content_similarity(content1, content2);
        assert!(similarity > 0.6); // Should be quite similar
    }
}