// desire_consolidation.rs - Smart Desire Merging & Cleanup System

use crate::desire_tracker::{Desire, DesireTracker, DesireCategory};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DesireConsolidationConfig {
    // Similarity thresholds
    pub keyword_overlap_threshold: f32,     // 0.7 = 70% keyword overlap triggers merge
    pub content_similarity_threshold: f32,  // 0.8 = 80% semantic similarity triggers merge
    pub category_match_required: bool,      // Must be same category to merge
    
    // Pruning rules  
    pub max_active_desires_per_category: usize, // Keep only top N per category
    pub minimum_intensity_threshold: f32,       // Remove desires below this
    pub staleness_threshold_days: u32,          // Remove if not mentioned for N days
    
    // Consolidation behavior
    pub merge_similar_desires: bool,
    pub prefer_recent_over_old: bool,
    pub boost_intensity_on_merge: bool,
}

impl Default for DesireConsolidationConfig {
    fn default() -> Self {
        Self {
            keyword_overlap_threshold: 0.6,
            content_similarity_threshold: 0.75,
            category_match_required: true,
            max_active_desires_per_category: 3,  // Only keep top 3 per category
            minimum_intensity_threshold: 0.3,
            staleness_threshold_days: 14,
            merge_similar_desires: true,
            prefer_recent_over_old: true,
            boost_intensity_on_merge: true,
        }
    }
}

#[derive(Debug)]
pub struct ConsolidationResult {
    pub merged_desires: Vec<MergeOperation>,
    pub pruned_desires: Vec<String>,
    pub desires_before: usize,
    pub desires_after: usize,
    pub categories_affected: Vec<String>,
}

#[derive(Debug)]
pub struct MergeOperation {
    pub primary_id: String,
    pub merged_ids: Vec<String>,
    pub final_content: String,
    pub combined_intensity: f32,
}

pub struct DesireConsolidator {
    config: DesireConsolidationConfig,
}

impl DesireConsolidator {
    pub fn new(config: DesireConsolidationConfig) -> Self {
        Self { config }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(DesireConsolidationConfig::default())
    }
    
    /// MAIN CONSOLIDATION FUNCTION - Run this after batched analysis
    pub fn consolidate_desires(&self, tracker: &mut DesireTracker) -> ConsolidationResult {
        let initial_count = tracker.active_desires.len();
        let mut result = ConsolidationResult {
            merged_desires: Vec::new(),
            pruned_desires: Vec::new(),
            desires_before: initial_count,
            desires_after: 0,
            categories_affected: Vec::new(),
        };
        
        // Step 1: Remove weak/stale desires first
        self.prune_weak_desires(tracker, &mut result);
        
        // Step 2: Merge similar desires by category
        if self.config.merge_similar_desires {
            self.merge_similar_desires_by_category(tracker, &mut result);
        }
        
        // Step 3: Enforce category limits (keep only top N per category)
        self.enforce_category_limits(tracker, &mut result);
        
        result.desires_after = tracker.active_desires.len();
        tracker.last_updated = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
        result
    }
    
    /// Remove desires that are too weak, old, or stale
    fn prune_weak_desires(&self, tracker: &mut DesireTracker, result: &mut ConsolidationResult) {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(self.config.staleness_threshold_days as i64);
        let cutoff_timestamp = cutoff_date.format("%Y-%m-%d %H:%M:%S UTC").to_string();
        
        let mut to_remove = Vec::new();
        
        for (id, desire) in &tracker.active_desires {
            let should_remove = 
                // Too weak
                desire.intensity < self.config.minimum_intensity_threshold ||
                // Too stale  
                (desire.last_mentioned < cutoff_timestamp && desire.total_mentions <= 1) ||
                // Been ignored too long
                desire.conversations_since_mention > 30;
            
            if should_remove {
                to_remove.push(id.clone());
                result.pruned_desires.push(format!("{}: {} (intensity: {:.2})", 
                    id, desire.content.chars().take(50).collect::<String>(), desire.intensity));
            }
        }
        
        for id in to_remove {
            if let Some(desire) = tracker.active_desires.remove(&id) {
                // Move to fulfilled_desires for audit trail
                tracker.fulfilled_desires.push(desire);
            }
        }
    }
    
    /// Find and merge desires that are essentially the same
    fn merge_similar_desires_by_category(&self, tracker: &mut DesireTracker, result: &mut ConsolidationResult) {
        // Group desires by category for focused comparison
        let mut by_category: HashMap<String, Vec<String>> = HashMap::new();
        
        for (id, desire) in &tracker.active_desires {
            let category_key = desire.category.to_string();
            by_category.entry(category_key).or_insert_with(Vec::new).push(id.clone());
        }
        
        for (category, desire_ids) in by_category {
            if desire_ids.len() <= 1 { continue; } // No merging needed
            
            let merge_groups = self.find_merge_groups(&desire_ids, tracker);
            
            for group in merge_groups {
                if group.len() <= 1 { continue; } // No merging for single desires
                
                let merge_op = self.perform_merge(&group, tracker);
                if let Some(op) = merge_op {
                    result.merged_desires.push(op);
                    if !result.categories_affected.contains(&category) {
                        result.categories_affected.push(category.clone());
                    }
                }
            }
        }
    }
    
    /// Find groups of desires that should be merged together
    fn find_merge_groups(&self, desire_ids: &[String], tracker: &DesireTracker) -> Vec<Vec<String>> {
        let mut groups = Vec::new();
        let mut processed = std::collections::HashSet::new();
        
        for id1 in desire_ids {
            if processed.contains(id1) { continue; }
            
            let mut current_group = vec![id1.clone()];
            processed.insert(id1.clone());
            
            // Find all desires similar to this one
            for id2 in desire_ids {
                if processed.contains(id2) { continue; }
                
                if let (Some(desire1), Some(desire2)) = (tracker.active_desires.get(id1), tracker.active_desires.get(id2)) {
                    if self.are_desires_similar(desire1, desire2) {
                        current_group.push(id2.clone());
                        processed.insert(id2.clone());
                    }
                }
            }
            
            groups.push(current_group);
        }
        
        groups
    }
    
    /// Check if two desires should be merged
    fn are_desires_similar(&self, desire1: &Desire, desire2: &Desire) -> bool {
        // Must be same category if required
        if self.config.category_match_required && desire1.category.to_string() != desire2.category.to_string() {
            return false;
        }
        
        // Must be same type (desire vs aspiration)
        if desire1.desire_type != desire2.desire_type {
            return false;
        }
        
        // Check keyword overlap
        let keyword_similarity = self.calculate_keyword_similarity(&desire1.keywords, &desire2.keywords);
        if keyword_similarity >= self.config.keyword_overlap_threshold {
            return true;
        }
        
        // Check content similarity (simple word overlap for now)
        let content_similarity = self.calculate_content_similarity(&desire1.content, &desire2.content);
        if content_similarity >= self.config.content_similarity_threshold {
            return true;
        }
        
        false
    }
    
    /// Calculate keyword overlap between two desires
    fn calculate_keyword_similarity(&self, keywords1: &[String], keywords2: &[String]) -> f32 {
        if keywords1.is_empty() || keywords2.is_empty() {
            return 0.0;
        }
        
        let set1: std::collections::HashSet<_> = keywords1.iter().collect();
        let set2: std::collections::HashSet<_> = keywords2.iter().collect();
        
        let intersection_size = set1.intersection(&set2).count();
        let union_size = set1.union(&set2).count();
        
        if union_size == 0 { 0.0 } else { intersection_size as f32 / union_size as f32 }
    }
    
    /// Calculate content similarity (simple word overlap)
    fn calculate_content_similarity(&self, content1: &str, content2: &str) -> f32 {
        let content1_lower = content1.to_lowercase();
		let words1: std::collections::HashSet<_> = content1_lower
			.split_whitespace()
			.map(|s| s.to_string())
			.collect();

		let content2_lower = content2.to_lowercase();
		let words2: std::collections::HashSet<_> = content2_lower
			.split_whitespace()
			.map(|s| s.to_string())
			.collect();
        
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }
        
        let intersection_size = words1.intersection(&words2).count();
        let union_size = words1.union(&words2).count();
        
        if union_size == 0 { 0.0 } else { intersection_size as f32 / union_size as f32 }
    }
    
    /// Merge a group of similar desires into one
    fn perform_merge(&self, group: &[String], tracker: &mut DesireTracker) -> Option<MergeOperation> {
        if group.len() <= 1 { return None; }
        
        // Find the primary desire (highest intensity or most recent)
        let mut primary_id = group[0].clone();
        let mut max_score = 0.0;
        
        for id in group {
            if let Some(desire) = tracker.active_desires.get(id) {
                let score = if self.config.prefer_recent_over_old {
                    desire.intensity + (desire.total_mentions as f32 * 0.1)
                } else {
                    desire.intensity
                };
                
                if score > max_score {
                    max_score = score;
                    primary_id = id.clone();
                }
            }
        }
        
        // Get the primary desire for merging
        let primary_desire = tracker.active_desires.get(&primary_id)?.clone();
        
        // Calculate combined metrics
        let mut combined_intensity = primary_desire.intensity;
        let mut total_mentions = primary_desire.total_mentions;
        let mut all_keywords = primary_desire.keywords.clone();
        let mut merged_ids = Vec::new();
        
        // Merge other desires into primary
        for id in group {
            if id == &primary_id { continue; }
            
            if let Some(desire) = tracker.active_desires.remove(id) {
                // Boost intensity if configured
                if self.config.boost_intensity_on_merge {
                    combined_intensity = (combined_intensity + desire.intensity * 0.3).min(1.0);
                }
                
                total_mentions += desire.total_mentions;
                
                // Merge keywords (keep unique)
                for keyword in &desire.keywords {
                    if !all_keywords.contains(&keyword) {
                        all_keywords.push(keyword.to_string());
                    }
                }
                
                merged_ids.push(id.clone());
                
                // Move to fulfilled_desires for audit trail
                tracker.fulfilled_desires.push(desire);
            }
        }
        
        // Update the primary desire with merged data
        if let Some(primary) = tracker.active_desires.get_mut(&primary_id) {
            primary.intensity = combined_intensity;
            primary.total_mentions = total_mentions;
            primary.keywords = all_keywords;
            primary.last_mentioned = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
            
            // Create a more comprehensive content description
            let enhanced_content = if merged_ids.len() > 2 {
                format!("I want to create visual art collaboratively (consolidated from {} similar desires)", merged_ids.len() + 1)
            } else {
                primary.content.clone()
            };
            
            Some(MergeOperation {
                primary_id: primary_id.clone(),
                merged_ids,
                final_content: enhanced_content,
                combined_intensity,
            })
        } else {
            None
        }
    }
    
    /// Keep only top N desires per category
    fn enforce_category_limits(&self, tracker: &mut DesireTracker, result: &mut ConsolidationResult) {
        let mut by_category: HashMap<String, Vec<(String, f32)>> = HashMap::new();
        
        // Group by category with intensity scores
        for (id, desire) in &tracker.active_desires {
            let category = desire.category.to_string();
            let score = desire.intensity + (desire.total_mentions as f32 * 0.05);
            by_category.entry(category).or_insert_with(Vec::new).push((id.clone(), score));
        }
        
        // Sort each category by score and keep only top N
        for (category, mut desires) in by_category {
            if desires.len() <= self.config.max_active_desires_per_category {
                continue; // No pruning needed
            }
            
            // Sort by score (highest first)
            desires.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Remove excess desires
            let to_remove: Vec<String> = desires
                .into_iter()
                .skip(self.config.max_active_desires_per_category)
                .map(|(id, _)| id)
                .collect();
            
            for id in to_remove {
                if let Some(desire) = tracker.active_desires.remove(&id) {
                    result.pruned_desires.push(format!("{}: {} (category limit exceeded)", 
                        id, desire.content.chars().take(50).collect::<String>()));
                    tracker.fulfilled_desires.push(desire);
                }
            }
            
            if !result.categories_affected.contains(&category) {
                result.categories_affected.push(category);
            }
        }
    }
}

/// INTEGRATION FUNCTION - Add this to batched_analysis.rs
pub async fn consolidate_desires_after_analysis(
    consolidator: &DesireConsolidator,
) -> Result<ConsolidationResult, String> {
    let mut tracker = DesireTracker::load();
    let result = consolidator.consolidate_desires(&mut tracker);
    
    tracker.save()
        .map_err(|e| format!("Failed to save consolidated desires: {}", e))?;
    
    crate::debug_log!("🧹 Desire consolidation complete:");
    crate::debug_log!("  Before: {} desires", result.desires_before);
    crate::debug_log!("  After: {} desires", result.desires_after);
    crate::debug_log!("  Merged: {} operations", result.merged_desires.len());
    crate::debug_log!("  Pruned: {} desires", result.pruned_desires.len());
    crate::debug_log!("  Categories affected: {:?}", result.categories_affected);
    
    for merge_op in &result.merged_desires {
        crate::debug_log!("  🔄 Merged {} desires into: {}", 
            merge_op.merged_ids.len() + 1, 
            merge_op.final_content.chars().take(60).collect::<String>());
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyword_similarity() {
        let consolidator = DesireConsolidator::with_defaults();
        
        let keywords1 = vec!["create".to_string(), "visual".to_string(), "art".to_string()];
        let keywords2 = vec!["create".to_string(), "art".to_string(), "collaborate".to_string()];
        
        let similarity = consolidator.calculate_keyword_similarity(&keywords1, &keywords2);
        assert!(similarity > 0.4); // Should have some overlap
    }
    
    #[test]
    fn test_content_similarity() {
        let consolidator = DesireConsolidator::with_defaults();
        
        let content1 = "I want to create visual art collaboratively";
        let content2 = "I want to dive back into creating visual art collaboratively";
        
        let similarity = consolidator.calculate_content_similarity(content1, content2);
        assert!(similarity > 0.7); // Should be very similar
    }
}