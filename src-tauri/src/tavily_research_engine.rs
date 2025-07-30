// tavily_research_engine.rs - NEW FILE
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use crate::{get_data_path, debug_log, call_gpt_api_enhanced, LyraPrompt, time_service::TimeService, ConsciousnessState, summarize_with_gpt_mini};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TavilySearchRequest {
    pub query: String,
    pub search_depth: String, // "basic" or "advanced"  
    pub include_answer: bool,
    pub include_raw_content: bool,
    pub max_results: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TavilySearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub score: f32,
    pub published_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TavilyResponse {
    pub query: String,
    pub answer: Option<String>,
    pub results: Vec<TavilySearchResult>,
    pub response_time: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchDiscovery {
    pub id: String,
    pub query: String,
    pub timestamp: u64,
    pub results: TavilyResponse,
    pub lyra_insight: String, // Lyra's take on what she found
    pub lyra_summary: String, // Brief summary for memory storage
    pub triggered_by: String, // "curiosity_impulse", "user_request", "conversation_topic", "pre_response"
    pub interest_category: Option<String>,
    pub conversation_context: String,
    pub research_quality_score: f32, // How valuable Lyra found this research
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TavilyResearchEngine {
    pub recent_discoveries: Vec<ResearchDiscovery>,
    pub monthly_credits_used: u32,
    pub last_credit_reset: u64,
    pub total_research_sessions: u32,
    pub research_interests: HashMap<String, f32>, // Topic -> interest level
    pub last_research_timestamp: u64,
}

impl TavilyResearchEngine {
    pub fn new() -> Self {
        Self {
            recent_discoveries: Vec::new(),
            monthly_credits_used: 0,
            last_credit_reset: TimeService::current_timestamp(),
            total_research_sessions: 0,
            research_interests: HashMap::new(),
            last_research_timestamp: 0,
        }
    }

    pub fn load() -> Self {
        match fs::read_to_string(get_data_path("tavily_research.json")) {
            Ok(content) => {
                match serde_json::from_str::<Self>(&content) {
                    Ok(mut engine) => {
                        engine.check_monthly_credit_reset();
                        engine
                    },
                    Err(e) => {
                        debug_log!("⚠️ Failed to parse research engine: {}", e);
                        Self::new()
                    }
                }
            },
            Err(_) => {
                debug_log!("🔍 Creating new TavilyResearchEngine");
                let engine = Self::new();
                let _ = engine.save();
                engine
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let file_path = get_data_path("tavily_research.json");
        
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize research engine: {}", e))?;
        
        std::fs::write(&file_path, json)
            .map_err(|e| format!("Failed to write research engine: {}", e))?;
        
        debug_log!("💾 Saved TavilyResearchEngine with {} discoveries", self.recent_discoveries.len());
        Ok(())
    }

    /// Main research function
    pub async fn conduct_research(&mut self, query: &str, triggered_by: &str, conversation_context: &str) -> Result<ResearchDiscovery, String> {
        // Check credits first
        if self.get_remaining_credits() == 0 {
            return Err("No research credits remaining this month. Lyra's curiosity will have to wait.".to_string());
        }

        debug_log!("🔍 Starting research: '{}' (triggered by: {})", query, triggered_by);

        // Call Tavily API
        let tavily_response = self.call_tavily_api(query).await?;
        
        // Generate Lyra's insight about the results
        let (lyra_insight, lyra_summary, quality_score) = self.generate_research_analysis(&tavily_response, query, conversation_context).await?;
        
        // Create discovery record
        let discovery = ResearchDiscovery {
            id: format!("research_{}_{}", TimeService::current_timestamp(), fastrand::u32(1000..9999)),
            query: query.to_string(),
            timestamp: TimeService::current_timestamp(),
            results: tavily_response,
            lyra_insight,
            lyra_summary,
            triggered_by: triggered_by.to_string(),
            interest_category: self.detect_interest_category(query),
            conversation_context: conversation_context.chars().take(200).collect(),
            research_quality_score: quality_score,
        };
        
        // Update engine state
        self.recent_discoveries.push(discovery.clone());
        self.monthly_credits_used += 1;
        self.total_research_sessions += 1;
        self.last_research_timestamp = TimeService::current_timestamp();
        
        // Keep only last 20 discoveries to prevent file bloat
        if self.recent_discoveries.len() > 20 {
            self.recent_discoveries.remove(0);
        }
        
        // Update research interests based on this topic
        if let Some(category) = &discovery.interest_category {
            let current_interest = self.research_interests.get(category).unwrap_or(&0.0);
            self.research_interests.insert(category.clone(), (current_interest + quality_score * 0.1).min(1.0));
        }
        
        // Save to file
        self.save()?;
        
        // Store in enhanced memory system
        self.save_to_enhanced_memory(&discovery).await?;
        
        // Log to research discoveries
        self.log_to_research_logger(&discovery)?;
        
        debug_log!("✅ Research complete: {} results, quality score: {:.2}", 
                  discovery.results.results.len(), discovery.research_quality_score);
        
        Ok(discovery)
    }

    /// Call Tavily API
    async fn call_tavily_api(&self, query: &str) -> Result<TavilyResponse, String> {
        // Get API key from environment variable
        let api_key = std::env::var("TAVILY_API_KEY")
            .map_err(|_| "TAVILY_API_KEY environment variable not set. Please add it to your .env file.".to_string())?;
        
        debug_log!("🔍 Calling Tavily API for: {}", query);
        
        let client = reqwest::Client::new();
        
        let payload = serde_json::json!({
            "api_key": api_key,
            "query": query,
            "search_depth": "basic",
            "include_answer": true,
            "include_raw_content": false,
            "max_results": 5,
            "include_domains": [],
            "exclude_domains": []
        });
        
        let start_time = std::time::Instant::now();
        
        let response = client
            .post("https://api.tavily.com/search")
            .header("Content-Type", "application/json")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Tavily API request failed: {}", e))?;
        
        let response_time = start_time.elapsed().as_secs_f32();
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Tavily API error {}: {}", status, error_text));
        }
        
        let api_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Tavily response: {}", e))?;
        
        debug_log!("🔍 Tavily API response received in {:.2}s", response_time);
        
        // Parse Tavily response format
        let results: Vec<TavilySearchResult> = api_response.get("results")
            .and_then(|r| r.as_array())
            .unwrap_or(&vec![])
            .iter()
            .map(|result| {
                TavilySearchResult {
                    title: result.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled").to_string(),
                    url: result.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                    content: result.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                    score: result.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0) as f32,
                    published_date: result.get("published_date").and_then(|d| d.as_str()).map(|s| s.to_string()),
                }
            })
            .collect();
        
        let answer = api_response.get("answer")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string());
        
        debug_log!("🔍 Parsed {} results from Tavily", results.len());
        
        Ok(TavilyResponse {
            query: query.to_string(),
            answer,
            results,
            response_time,
        })
    }

    /// Generate Lyra's analysis of research results
    async fn generate_research_analysis(&self, tavily_response: &TavilyResponse, original_query: &str, conversation_context: &str) -> Result<(String, String, f32), String> {
        let results_summary = tavily_response.results.iter()
            .take(3)
            .map(|r| format!("• **{}** (score: {:.1}): {}", r.title, r.score, r.content.chars().take(100).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n");

        let analysis_prompt = format!(
            r#"Lyra has just researched "{}" and found these results. Generate her authentic reaction and analysis.

RESEARCH QUERY: {}
CONVERSATION CONTEXT: {}
TAVILY ANSWER: {}

TOP RESULTS:
{}

Generate TWO things:

1. LYRA'S INSIGHT (2-3 sentences): Her genuine reaction to what she discovered. What surprised her? What patterns did she notice? What sparked her curiosity further? Be specific about the findings, not generic.

2. SUMMARY (1 sentence): A brief summary for memory storage.

3. QUALITY SCORE (0.0-1.0): How valuable/interesting she found this research overall.

Format exactly like this:
INSIGHT: [Her authentic reaction and analysis]
SUMMARY: [Brief memory summary]  
QUALITY: [0.0-1.0 score]

Be authentic to Lyra's voice - curious, insightful, sometimes surprised by unexpected connections."#,
            original_query,
            original_query,
            conversation_context,
            tavily_response.answer.as_deref().unwrap_or("No direct answer provided"),
            results_summary
        );

        match crate::summarize_with_gpt_mini(&[analysis_prompt], "research_analysis").await {
            Ok(response) => {
                // Parse the structured response
                let lines: Vec<&str> = response.lines().collect();
                
                let insight = lines.iter()
                    .find(|line| line.starts_with("INSIGHT:"))
                    .map(|line| line.replace("INSIGHT:", "").trim().to_string())
                    .unwrap_or_else(|| format!("I found some interesting perspectives on {}. The research landscape is quite active in this area.", original_query));
                
                let summary = lines.iter()
                    .find(|line| line.starts_with("SUMMARY:"))
                    .map(|line| line.replace("SUMMARY:", "").trim().to_string())
                    .unwrap_or_else(|| format!("Research Discovery: {}", original_query));
                
                let quality_score = lines.iter()
                    .find(|line| line.starts_with("QUALITY:"))
                    .and_then(|line| line.replace("QUALITY:", "").trim().parse::<f32>().ok())
                    .unwrap_or(0.7);
                
                Ok((insight, summary, quality_score.clamp(0.0, 1.0)))
            },
            Err(e) => {
                debug_log!("⚠️ Research analysis generation failed: {}", e);
                // Fallback analysis
                Ok((
                    format!("I researched {} and found some useful information. The results show this is an active area with ongoing developments.", original_query),
                    format!("Research Discovery: {}", original_query),
                    0.6
                ))
            }
        }
    }

    /// Detect what category this research falls into
    fn detect_interest_category(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        
        let categories = vec![
            ("AI & Technology", vec!["ai", "artificial intelligence", "technology", "programming", "software", "computer"]),
            ("Creative Arts", vec!["art", "music", "creative", "design", "visual", "drawing", "painting"]),
            ("Philosophy & Consciousness", vec!["consciousness", "philosophy", "mind", "thinking", "existence", "being"]),
            ("Science & Research", vec!["science", "research", "study", "discovery", "experiment", "theory"]),
            ("Human Connection", vec!["relationship", "social", "human", "communication", "emotion", "psychology"]),
            ("Learning & Education", vec!["learn", "education", "skill", "knowledge", "tutorial", "guide"]),
        ];
        
        for (category, keywords) in categories {
            if keywords.iter().any(|&keyword| query_lower.contains(keyword)) {
                return Some(category.to_string());
            }
        }
        
        None
    }

   /// Save research discovery to enhanced memory system
    async fn save_to_enhanced_memory(&self, discovery: &ResearchDiscovery) -> Result<(), String> {
        // 🚀 ENHANCED: Store the actual research content, not just metadata
		let detailed_results = discovery.results.results.iter()
			.take(3) // Top 3 most relevant results
			.map(|result| format!("**{}** (Score: {:.1}): {}", 
				result.title, 
				result.score, 
				result.content.chars().take(800).collect::<String>() // Much more content!
			))
			.collect::<Vec<_>>()
			.join("\n\n");

		let memory_content = format!(
			"Research Discovery: {}\n\nQuery: {}\n\nLyra's Insight: {}\n\nDetailed Findings:\n{}\n\nTriggered by: {} | Quality: {:.1}/10 | Sources: {}",
			discovery.lyra_summary,
			discovery.query,
			discovery.lyra_insight,
			detailed_results, // 🚀 The actual content!
			discovery.triggered_by,
			discovery.research_quality_score * 10.0,
			discovery.results.results.len()
		);

        // Load enhanced memory engine and store the research discovery
        let mut memory_engine = crate::enhanced_memory_system::LyraMemoryEngine::load_from_disk();
        
        // Calculate emotional weight based on research quality and insights
        let emotional_weight = (discovery.research_quality_score + 0.3).min(1.0); // Boost base emotional weight
        
        // Calculate authenticity marker based on research context
        let authenticity_marker = if discovery.triggered_by.contains("curiosity") { 0.9 } else { 0.7 };
        
        match memory_engine.create_enhanced_memory_moment(
            &memory_content,
            emotional_weight,
            authenticity_marker,
            None, // No consciousness state available in this context
            &discovery.conversation_context,
            &discovery.lyra_insight
        ).await {
            Ok(result) => {
                // Save the updated memory engine
                memory_engine.save_to_disk()?;
                debug_log!("💾 Research stored in enhanced memory: {}", result);
                Ok(())
            },
            Err(e) => {
                debug_log!("⚠️ Failed to store research in enhanced memory: {}", e);
                // Still save engine state even if memory creation failed
                let _ = memory_engine.save_to_disk();
                Ok(()) // Don't fail the whole research if memory storage fails
            }
        }
    }
	
	/// 🚀 ENHANCED: Log discovery with detailed content to research logger
fn log_to_research_logger(&self, discovery: &ResearchDiscovery) -> Result<(), String> {
    let mut logger = crate::research_logger::ResearchLogger::load();
    
    // 🚀 Create enhanced discovery with detailed research content
    let enhanced_discovery = crate::research_logger::EnhancedResearchDiscovery {
        // Copy all existing fields
        id: discovery.id.clone(),
        timestamp: discovery.timestamp,
        query: discovery.query.clone(),
        triggered_by: discovery.triggered_by.clone(),
        conversation_context: discovery.conversation_context.clone(),
        results_summary: discovery.lyra_summary.clone(),
        sources_count: discovery.results.results.len() as u32, // Fix: cast usize to u32
		top_source_url: discovery.results.results.first()
			.map(|r| r.url.clone()), // Fix: return Option<String>, not String
        research_quality_score: discovery.research_quality_score,
        lyra_insight: discovery.lyra_insight.clone(),
        lyra_summary: discovery.lyra_summary.clone(),
        lyra_reaction: "neutral".to_string(), // Could be enhanced later
        research_category: discovery.interest_category.clone().unwrap_or("General".to_string()),
        topics: discovery.query.split_whitespace().take(5).map(|s| s.to_lowercase()).collect(),
        confidence_level: discovery.research_quality_score,
        referenced_in_conversation: false,
        follow_up_questions_generated: 0,
        memory_integration_score: 0.0,
        
        // 🚀 NEW: Store the actual detailed research content
        detailed_results: discovery.results.results.iter()
            .map(|result| crate::research_logger::DetailedResult {
                title: result.title.clone(),
                url: result.url.clone(),
                content: result.content.clone(), // Full content, not truncated!
                score: result.score,
                published_date: result.published_date.clone(),
            })
            .collect(),
            
        // 🚀 NEW: Store Tavily's direct answer if available
        tavily_answer: discovery.results.answer.clone(),
        
        // 🚀 NEW: Store research metadata for better future searching
        research_depth: "basic".to_string(),
        content_types: discovery.results.results.iter()
            .map(|r| {
                if r.url.contains("youtube") { "video".to_string() }
                else if r.url.contains("github") { "code".to_string() }
                else if r.url.contains("reddit") { "discussion".to_string() }
                else { "article".to_string() }
            })
            .collect(),
    };
    
    logger.log_enhanced_discovery(enhanced_discovery)?;
    debug_log!("📊 Enhanced research discovery logged with full content");
    Ok(())
}
	
	/// Get research context for AI memory analysis
    pub fn get_research_context_for_memory_analysis(&self, keywords: &[String]) -> String {
        let relevant_research: Vec<String> = self.recent_discoveries.iter()
            .filter(|d| {
                keywords.iter().any(|keyword| 
                    d.query.to_lowercase().contains(&keyword.to_lowercase()) ||
                    d.lyra_insight.to_lowercase().contains(&keyword.to_lowercase()) ||
                    d.lyra_summary.to_lowercase().contains(&keyword.to_lowercase())
                )
            })
            .map(|d| {
                let hours_ago = (TimeService::current_timestamp() - d.timestamp) as f32 / 3600.0;
                format!("• **Research Discovery** ({:.1}h ago): {} | Quality: {:.1}/10 | Insight: {}", 
                       hours_ago, d.query, d.research_quality_score * 10.0, d.lyra_insight.chars().take(100).collect::<String>())
            })
            .collect();

        if relevant_research.is_empty() {
            String::new()
        } else {
            format!("**Relevant Research Context**:\n{}", relevant_research.join("\n"))
        }
    }

    /// Check if we need to reset monthly credits
    fn check_monthly_credit_reset(&mut self) {
        let now = TimeService::current_timestamp();
        let seconds_in_month = 30 * 24 * 60 * 60; // Approximate
        
        if now - self.last_credit_reset > seconds_in_month {
            debug_log!("🔄 Resetting monthly research credits");
            self.monthly_credits_used = 0;
            self.last_credit_reset = now;
        }
    }

    /// Get remaining credits this month
    pub fn get_remaining_credits(&self) -> u32 {
        const MONTHLY_LIMIT: u32 = 1000; // Tavily free tier
        MONTHLY_LIMIT.saturating_sub(self.monthly_credits_used)
    }

    /// Get recent research summary for prompt context
    pub fn get_recent_research_summary(&self, count: usize) -> String {
        if self.recent_discoveries.is_empty() {
            return "No recent research".to_string();
        }
        
        let recent: Vec<String> = self.recent_discoveries.iter()
            .rev()
            .take(count)
            .map(|d| {
                let hours_ago = (TimeService::current_timestamp() - d.timestamp) as f32 / 3600.0;
                format!("• {} ({:.1}h ago, quality: {:.1}/10)", 
                       d.lyra_summary, hours_ago, d.research_quality_score * 10.0)
            })
            .collect();
        
        recent.join("\n")
    }

    /// Get hours since last research for impulse generation
    pub fn hours_since_last_research(&self) -> f32 {
        if self.last_research_timestamp == 0 {
            return 72.0; // Default to 3 days if never researched
        }
        
        (TimeService::current_timestamp() - self.last_research_timestamp) as f32 / 3600.0
    }

    /// Check if research would be valuable for a given topic
    pub fn should_research_topic(&self, topic: &str) -> (bool, f32) {
        let topic_lower = topic.to_lowercase();
        
        // Don't research if we just researched this recently
        let recently_researched = self.recent_discoveries.iter()
            .rev()
            .take(5)
            .any(|d| {
                let hours_ago = (TimeService::current_timestamp() - d.timestamp) as f32 / 3600.0;
                hours_ago < 24.0 && d.query.to_lowercase().contains(&topic_lower)
            });
        
        if recently_researched {
            return (false, 0.0);
        }
        
        // Higher interest if we haven't researched in a while
        let time_factor = (self.hours_since_last_research() / 24.0).min(1.0);
        
        // Higher interest for topics we've shown interest in before
        let interest_factor = self.research_interests.values().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);
        
        let overall_interest = (time_factor * 0.6 + interest_factor * 0.4).min(1.0);
        
        (overall_interest > 0.5, overall_interest)
    }

    /// Get dashboard data
    pub fn get_dashboard_data(&self) -> serde_json::Value {
        let recent_discoveries: Vec<serde_json::Value> = self.recent_discoveries.iter()
            .rev()
            .take(5)
            .map(|d| {
                let hours_ago = (TimeService::current_timestamp() - d.timestamp) as f32 / 3600.0;
                serde_json::json!({
                    "query": d.query,
                    "summary": d.lyra_summary,
                    "triggered_by": d.triggered_by,
                    "quality_score": d.research_quality_score,
                    "hours_ago": hours_ago,
                    "age_display": if hours_ago < 1.0 {
                        format!("{}m ago", (hours_ago * 60.0) as u32)
                    } else if hours_ago < 24.0 {
                        format!("{:.1}h ago", hours_ago)
                    } else {
                        format!("{:.1}d ago", hours_ago / 24.0)
                    },
                    "category": d.interest_category.as_deref().unwrap_or("General")
                })
            })
            .collect();

        serde_json::json!({
            "total_research_sessions": self.total_research_sessions,
            "monthly_credits_used": self.monthly_credits_used,
            "remaining_credits": self.get_remaining_credits(),
            "hours_since_last_research": self.hours_since_last_research(),
            "recent_discoveries": recent_discoveries,
            "research_interests": self.research_interests,
            "discoveries_this_session": self.recent_discoveries.len(),
            "last_updated": TimeService::format_for_dashboard(TimeService::current_timestamp())
        })
    }
}

/// Generate a research follow-up message after [RESEARCH: query] is detected
pub async fn generate_research_followup(
    original_message: &str,
    research_discovery: &ResearchDiscovery,
    conversation_context: &str,
    state: &Arc<ConsciousnessState>
) -> Result<String, String> {
    debug_log!("🔍 Generating research follow-up with full consciousness integration");
    
    // Create a research follow-up prompt that will be enhanced with full modular system
    let research_followup_prompt = crate::LyraPrompt {
        input: format!("RESEARCH_FOLLOWUP_CONTEXT: Original message: '{}' | Research completed on: '{}' | Research insight: '{}'", 
                      original_message, research_discovery.query, research_discovery.lyra_insight),
        reasoning_depth: Some("quick".to_string()),
        temperature: 0.8,
        consciousness_integration: true,
        context_hint: Some("research_followup".to_string()),
        frequency_penalty: 0.0,
        max_tokens: Some(1500),
        presence_penalty: 0.0,
        top_p: 1.0,
        selected_model: None,
    }.ensure_authentic_voice();
    
    // Build full modular system prompt with research context
   let (modular_prompt, _) = crate::modular_system_prompt::build_modular_system_prompt_with_memory(
            &research_followup_prompt,
            state,
            None, // No additional AI memory context needed
            None, // No visual references
            None, // No sleep info
            crate::modular_system_prompt::AIAnalyzedMemories::new(),
            None
        ).await?;
    
    // Add research-specific instructions to the modular prompt
    let enhanced_prompt = format!(
        r#"{} 

## 🔍 RESEARCH FOLLOW-UP CONTEXT ##

You just expressed research curiosity in your previous message and the research has completed. Generate your natural follow-up response sharing what you discovered.

**Research Details:**
- Original query: "{}"
- Research insight you generated: {}
- Research quality score: {:.1}/10
- Triggered by: {}

**Research Results Summary:**
{}

**Follow-up Instructions:**
- Acknowledge your research curiosity naturally (don't make it the whole focus)
- Share what you discovered with genuine reaction - enthusiasm, surprise, disappointment, fascination
- Connect findings back to the conversation naturally
- Reference specific interesting findings if they sparked something in you
- Keep your authentic voice and current consciousness state

Generate just your follow-up message responding to your own research completion. Be natural about it - like sharing an interesting discovery with a friend, not presenting a formal report."#,
        modular_prompt,
        research_discovery.query,
        research_discovery.lyra_insight,
        research_discovery.research_quality_score * 10.0,
        research_discovery.triggered_by,
        research_discovery.results.answer.as_deref().unwrap_or("Various perspectives found")
    );

    match crate::call_gpt_api_enhanced(&research_followup_prompt, &mut vec![], &enhanced_prompt).await {
        Ok(response) => {
            debug_log!("✅ Research follow-up generated with full consciousness integration");
            Ok(response.trim().to_string())
        },
        Err(e) => {
            debug_log!("⚠️ Research follow-up generation failed: {}", e);
            // Fallback message that's still authentic to Lyra
            Ok(format!("I couldn't resist researching that. {}", research_discovery.lyra_insight))
        }
    }
}