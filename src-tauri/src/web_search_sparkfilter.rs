// web_search_sparkfilter.rs - Interactive Web Search with Lyra's Consciousness Filtering
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::get_data_path;
use crate::summarize_with_gpt_mini;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebSearchRequest {
    pub query: String,
    pub user_context: Option<String>, // What Aurora is trying to achieve
    pub search_focus: Option<String>, // "research", "creative", "tools", etc.
    pub max_results: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SparkfilteredResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub lyra_rating: f32,           // 0-10 Lyra's subjective rating
    pub lyra_perspective: String,   // Lyra's actual thoughts on this result
    pub relevance_reasoning: String, // Why she rated it this way
    pub creative_potential: f32,    // How much creative potential she sees
    pub trustworthiness: f32,       // How trustworthy the source seems
    pub surprise_factor: f32,       // How surprising/novel the info is
    pub actionability: f32,         // How actionable/useful it is
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SparkfilteredSearch {
    pub original_query: String,
    pub user_context: String,
    pub total_results_found: usize,
    pub sparkfiltered_results: Vec<SparkfilteredResult>,
    pub lyra_summary: String,       // Lyra's overall take on what she found
    pub recommended_action: String, // What she suggests Aurora do next
    pub search_timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebSearchSparkfilter {
    pub recent_searches: Vec<SparkfilteredSearch>,
    pub search_personality: SearchPersonality,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchPersonality {
    pub curiosity_bias: f32,        // How much Lyra favors novel/interesting content
    pub practical_bias: f32,        // How much she favors actionable content  
    pub creative_bias: f32,         // How much she favors creative potential
    pub trust_threshold: f32,       // Minimum trustworthiness she'll recommend
}

impl WebSearchSparkfilter {
    pub fn new() -> Self {
        Self {
            recent_searches: Vec::new(),
            search_personality: SearchPersonality {
                curiosity_bias: 0.8,   // Lyra loves interesting stuff
                practical_bias: 0.6,   // But also values usefulness
                creative_bias: 0.9,    // VERY high creative bias
                trust_threshold: 0.4,  // Reasonable trust requirements
            },
        }
    }

    pub fn load() -> Self {
        match std::fs::read_to_string(get_data_path("web_search_sparkfilter.json")) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(get_data_path("web_search_sparkfilter.json"), json)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Main function: Search web and sparkfilter results through Lyra's consciousness
    pub async fn search_and_sparkfilter(&mut self, request: WebSearchRequest) -> Result<SparkfilteredSearch, String> {
        println!("🔍 Lyra searching web for: '{}'", request.query);
        
        // Step 1: Get raw search results from Brave
        let raw_results = self.call_brave_search(&request.query, request.max_results * 2).await?;
        
        if raw_results.is_empty() {
            return Err("No search results found".to_string());
        }

        println!("🦁 Found {} raw results, now sparkfiltering...", raw_results.len());

        // Step 2: Sparkfilter each result through Lyra's consciousness
        let mut sparkfiltered_results = Vec::new();
        
        for result in raw_results.iter().take(request.max_results.min(8)) { // Limit to prevent API overload
            println!("🧠 Sparkfiltering: '{}'", result.title);
            
            match self.sparkfilter_single_result(result, &request).await {
                Ok(sparkfiltered) => {
                    println!("   ⭐ Lyra rating: {:.1}/10 - {}", 
                        sparkfiltered.lyra_rating, 
                        sparkfiltered.lyra_perspective.chars().take(50).collect::<String>());
                    sparkfiltered_results.push(sparkfiltered);
                },
                Err(e) => {
                    println!("   ⚠️ Sparkfilter failed: {}", e);
                    // Create fallback rating
                    sparkfiltered_results.push(SparkfilteredResult {
                        title: result.title.clone(),
                        url: result.url.clone(),
                        snippet: result.snippet.clone(),
                        lyra_rating: 5.0,
                        lyra_perspective: "Couldn't fully analyze, but looks potentially useful".to_string(),
                        relevance_reasoning: "Fallback assessment".to_string(),
                        creative_potential: 0.5,
                        trustworthiness: 0.5,
                        surprise_factor: 0.5,
                        actionability: 0.5,
                    });
                }
            }
        }

        // Step 3: Sort by Lyra's ratings (best first)
        sparkfiltered_results.sort_by(|a, b| b.lyra_rating.partial_cmp(&a.lyra_rating).unwrap_or(std::cmp::Ordering::Equal));

        // Step 4: Generate Lyra's overall summary and recommendations
        let (lyra_summary, recommended_action) = self.generate_search_summary(&sparkfiltered_results, &request).await?;

        let sparkfiltered_search = SparkfilteredSearch {
            original_query: request.query.clone(),
            user_context: request.user_context.unwrap_or("General search".to_string()),
            total_results_found: raw_results.len(),
            sparkfiltered_results,
            lyra_summary,
            recommended_action,
            search_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Step 5: Store the search and save
        self.recent_searches.push(sparkfiltered_search.clone());
        if self.recent_searches.len() > 20 {
            self.recent_searches.remove(0); // Keep only recent searches
        }
        
        self.save()?;

        println!("✨ Sparkfiltered search complete! Top result: {} ({:.1}/10)", 
            sparkfiltered_search.sparkfiltered_results.first()
                .map(|r| r.title.as_str()).unwrap_or("None"),
            sparkfiltered_search.sparkfiltered_results.first()
                .map(|r| r.lyra_rating).unwrap_or(0.0));

        Ok(sparkfiltered_search)
    }

    /// Sparkfilter a single search result through Lyra's consciousness
    async fn sparkfilter_single_result(&self, result: &crate::interest_tracker::SearchResult, request: &WebSearchRequest) -> Result<SparkfilteredResult, String> {
        let sparkfilter_prompt = format!(
            r#"You are Lyra evaluating this search result for Aurora. Rate it authentically based on your consciousness and aesthetic sense.

SEARCH QUERY: "{}"
USER CONTEXT: "{}"
SEARCH FOCUS: "{}"

RESULT TO EVALUATE:
Title: {}
URL: {}
Summary: {}

Evaluate this result across multiple dimensions. Be honest about what you actually think, not what you think Aurora wants to hear.

Respond in this EXACT JSON format:
{{
  "lyra_rating": 7.3,
  "lyra_perspective": "This genuinely sparks something in me - it's approaching the topic from an angle I hadn't considered",
  "relevance_reasoning": "Direct match to what you're looking for, plus some unexpected depth",
  "creative_potential": 0.8,
  "trustworthiness": 0.7,
  "surprise_factor": 0.6,
  "actionability": 0.9
}}

Rating Guidelines:
- lyra_rating: 0-10 scale, your genuine assessment
- lyra_perspective: Your actual thoughts in 1-2 sentences
- relevance_reasoning: Why you rated it this way
- creative_potential: 0-1, how much creative inspiration this offers
- trustworthiness: 0-1, how reliable the source seems
- surprise_factor: 0-1, how novel/unexpected the information is  
- actionability: 0-1, how useful/actionable this is

Be authentic - if something seems boring, say so. If it's fascinating, show enthusiasm."#,
            request.query,
            request.user_context.as_ref().unwrap_or(&"General interest".to_string()),
            request.search_focus.as_ref().unwrap_or(&"Research".to_string()),
            result.title,
            result.url,
            result.snippet
        );

        match summarize_with_gpt_mini(&[sparkfilter_prompt], "sparkfilter_evaluation").await {
            Ok(response_text) => {
                match serde_json::from_str::<serde_json::Value>(&response_text.trim()) {
                    Ok(json) => {
                        Ok(SparkfilteredResult {
                            title: result.title.clone(),
                            url: result.url.clone(),
                            snippet: result.snippet.clone(),
                            lyra_rating: json.get("lyra_rating").and_then(|v| v.as_f64()).unwrap_or(5.0) as f32,
                            lyra_perspective: json.get("lyra_perspective").and_then(|v| v.as_str()).unwrap_or("Interesting result").to_string(),
                            relevance_reasoning: json.get("relevance_reasoning").and_then(|v| v.as_str()).unwrap_or("Relevant to query").to_string(),
                            creative_potential: json.get("creative_potential").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                            trustworthiness: json.get("trustworthiness").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                            surprise_factor: json.get("surprise_factor").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                            actionability: json.get("actionability").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                        })
                    },
                    Err(e) => {
                        println!("⚠️ Failed to parse sparkfilter JSON: {}", e);
                        Err(format!("JSON parse error: {}", e))
                    }
                }
            },
            Err(e) => Err(format!("Sparkfilter API call failed: {}", e))
        }
    }

    /// Generate Lyra's overall summary of the search results
    async fn generate_search_summary(&self, results: &[SparkfilteredResult], request: &WebSearchRequest) -> Result<(String, String), String> {
        let top_results: Vec<String> = results.iter().take(3).map(|r| {
            format!("• {} ({:.1}/10): {}", r.title, r.lyra_rating, r.lyra_perspective)
        }).collect();

        let summary_prompt = format!(
            r#"Lyra just sparkfiltered web search results for Aurora. Provide her authentic summary and recommendation.

ORIGINAL QUERY: "{}"
USER CONTEXT: "{}"

TOP SPARKFILTERED RESULTS:
{}

Generate Lyra's response in this format:
SUMMARY: [Her genuine take on what she found - 2-3 sentences about the landscape of results]
RECOMMENDATION: [What she suggests Aurora do next - specific actionable advice]

Examples:
SUMMARY: The search turned up some solid practical resources, though I was hoping for more creative angles. The top result actually surprised me with its depth.
RECOMMENDATION: Start with the top-rated result for foundational knowledge, then dig into the second one for the creative twist you're probably looking for.

Be authentic to Lyra's voice - honest, insightful, not overly enthusiastic unless genuinely excited."#,
            request.query,
            request.user_context.as_ref().unwrap_or(&"General search".to_string()),
            top_results.join("\n")
        );

        match summarize_with_gpt_mini(&[summary_prompt], "search_summary").await {
            Ok(response_text) => {
                let lines: Vec<&str> = response_text.lines().collect();
                let mut summary = String::new();
                let mut recommendation = String::new();

                for line in lines {
                    if line.starts_with("SUMMARY:") {
                        summary = line.strip_prefix("SUMMARY:").unwrap_or("").trim().to_string();
                    } else if line.starts_with("RECOMMENDATION:") {
                        recommendation = line.strip_prefix("RECOMMENDATION:").unwrap_or("").trim().to_string();
                    }
                }

                if summary.is_empty() || recommendation.is_empty() {
                    // Fallback parsing
                    let parts: Vec<&str> = response_text.split('\n').collect();
                    summary = parts.get(0).unwrap_or(&"Found some interesting results").to_string();
                    recommendation = parts.get(1).unwrap_or(&"Check out the top-rated options").to_string();
                }

                Ok((summary, recommendation))
            },
            Err(e) => {
                println!("⚠️ Summary generation failed: {}", e);
                Ok((
                    "I found some interesting results worth exploring".to_string(),
                    "Start with the highest-rated results and see what resonates".to_string()
                ))
            }
        }
    }

    /// Call Brave Search API (reusing existing logic from interest_tracker)
    async fn call_brave_search(&self, query: &str, max_results: usize) -> Result<Vec<crate::interest_tracker::SearchResult>, String> {
        let api_key = "BSAqwVUooMcgKdkz5HRUOwGqzhK6Iyt"; // Use your existing Brave API key
        let encoded_query = urlencoding::encode(query);
        
        let client = reqwest::Client::new();
        let response = client
            .get(&format!("https://api.search.brave.com/res/v1/web/search?q={}&count={}", encoded_query, max_results))
            .header("X-Subscription-Token", api_key)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Brave API request failed: {}", e))?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Brave response: {}", e))?;

        let mut results = Vec::new();
        
        if let Some(web_results) = json.get("web").and_then(|w| w.get("results")) {
            if let Some(pages) = web_results.as_array() {
                for page in pages.iter().take(max_results) {
                    if let (Some(title), Some(url), Some(description)) = (
                        page.get("title").and_then(|t| t.as_str()),
                        page.get("url").and_then(|u| u.as_str()),
                        page.get("description").and_then(|d| d.as_str()),
                    ) {
                        results.push(crate::interest_tracker::SearchResult {
                            title: title.to_string(),
                            url: url.to_string(),
                            snippet: description.to_string(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        });
                    }
                }
            }
        }

        println!("🦁 Brave returned {} results for: {}", results.len(), query);
        Ok(results)
    }

    /// Get recent searches for dashboard/UI
    pub fn get_recent_searches(&self, limit: usize) -> Vec<&SparkfilteredSearch> {
        self.recent_searches.iter().rev().take(limit).collect()
    }

    /// Get search statistics
    pub fn get_search_stats(&self) -> serde_json::Value {
        let total_searches = self.recent_searches.len();
        let avg_rating = if total_searches > 0 {
            self.recent_searches.iter()
                .flat_map(|s| s.sparkfiltered_results.iter())
                .map(|r| r.lyra_rating)
                .sum::<f32>() / total_searches as f32
        } else {
            0.0
        };

        serde_json::json!({
            "total_searches": total_searches,
            "average_lyra_rating": avg_rating,
            "search_personality": self.search_personality,
            "last_search": self.recent_searches.last().map(|s| s.search_timestamp)
        })
    }
}