// conversational_web_search.rs - Natural conversation-triggered web search
use serde::{Deserialize, Serialize};
use crate::web_search_sparkfilter::{WebSearchSparkfilter, WebSearchRequest, SparkfilteredSearch};
use crate::summarize_with_gpt_mini;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchIntent {
    pub detected: bool,
    pub search_query: String,
    pub search_context: String,
    pub search_type: String, // "research", "tools", "creative", "help", "general"
    pub confidence: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConversationalSearchResult {
    pub search_intent: SearchIntent,
    pub search_results: Option<SparkfilteredSearch>,
    pub lyra_response_integration: String, // How to weave results into response
}

pub struct ConversationalWebSearch {
    pub sparkfilter: WebSearchSparkfilter,
}

impl ConversationalWebSearch {
    pub fn new() -> Self {
        Self {
            sparkfilter: WebSearchSparkfilter::load(),
        }
    }

    /// Main function: Detect if user wants a search, extract query, and perform search
    pub async fn detect_and_search(&mut self, user_message: &str, conversation_context: &str) -> Result<Option<ConversationalSearchResult>, String> {
        // Step 1: Detect search intent
        let search_intent = self.detect_search_intent(user_message, conversation_context).await?;
        
        if !search_intent.detected || search_intent.confidence < 0.7 {
            return Ok(None); // No search needed
        }

        println!("🔍 Search intent detected: '{}' (confidence: {:.2})", 
                 search_intent.search_query, search_intent.confidence);

        // Step 2: Perform the search
        let search_request = WebSearchRequest {
            query: search_intent.search_query.clone(),
            user_context: Some(search_intent.search_context.clone()),
            search_focus: Some(search_intent.search_type.clone()),
            max_results: 6, // Reasonable number for conversation
        };

        match self.sparkfilter.search_and_sparkfilter(search_request).await {
            Ok(search_results) => {
                // Step 3: Generate integration guidance for Lyra's response
                let integration_guidance = self.generate_response_integration(&search_intent, &search_results).await?;

                Ok(Some(ConversationalSearchResult {
                    search_intent,
                    search_results: Some(search_results),
                    lyra_response_integration: integration_guidance,
                }))
            },
            Err(e) => {
                println!("⚠️ Search failed: {}", e);
                // Return intent but no results
                Ok(Some(ConversationalSearchResult {
    search_intent: search_intent.clone(),
    search_results: None,
    lyra_response_integration: format!("I tried to search for '{}' but ran into technical issues. Let me respond based on what I know.", search_intent.search_query),
}))
            }
        }
    }

    /// Detect if user wants a web search and extract the query
    async fn detect_search_intent(&self, user_message: &str, conversation_context: &str) -> Result<SearchIntent, String> {
        let intent_prompt = format!(
            r#"Analyze this message to detect if Aurora wants Lyra to search the web for something.

USER MESSAGE: "{}"
CONVERSATION CONTEXT: "{}"

Look for patterns like:
- "search for [topic]"
- "look up [thing]" 
- "find information about [subject]"
- "can you research [topic]"
- "what's the latest on [subject]"
- "are there any good [tools/resources/articles] for [topic]"
- "help me find [something]"
- Implicit requests like "I need to learn about X" or "I'm looking for Y"

Respond in this EXACT JSON format:
{{
  "detected": true,
  "search_query": "consciousness research papers 2024",
  "search_context": "Aurora wants to learn about recent consciousness research",
  "search_type": "research",
  "confidence": 0.9
}}

Search types: "research", "tools", "creative", "help", "news", "general"
Confidence: 0.0-1.0 (how sure you are they want a search)

If NO search intent detected, return:
{{
  "detected": false,
  "search_query": "",
  "search_context": "",
  "search_type": "general", 
  "confidence": 0.0
}}

Be precise about the search query - extract the ACTUAL topic they want to search for."#,
            user_message,
            conversation_context.chars().take(500).collect::<String>()
        );

        match summarize_with_gpt_mini(&[intent_prompt], "search_intent_detection").await {
            Ok(response_text) => {
                match serde_json::from_str::<SearchIntent>(&response_text.trim()) {
                    Ok(intent) => {
                        if intent.detected {
                            println!("🎯 Search intent detected: {} (type: {}, confidence: {:.2})", 
                                   intent.search_query, intent.search_type, intent.confidence);
                        }
                        Ok(intent)
                    },
                    Err(e) => {
                        println!("⚠️ Failed to parse search intent JSON: {}", e);
                        // Fallback: simple keyword detection
                        Ok(self.fallback_search_detection(user_message))
                    }
                }
            },
            Err(e) => {
                println!("⚠️ Search intent detection API failed: {}", e);
                // Fallback: simple keyword detection
                Ok(self.fallback_search_detection(user_message))
            }
        }
    }

    /// Fallback search detection using simple keyword matching
    fn fallback_search_detection(&self, user_message: &str) -> SearchIntent {
        let message_lower = user_message.to_lowercase();
        
        let search_keywords = [
            "search for", "look up", "find", "research", "google", "web search",
            "what's the latest", "are there", "help me find", "I need to learn",
            "show me", "get information", "find out about"
        ];

        let has_search_keyword = search_keywords.iter().any(|&keyword| message_lower.contains(keyword));

        if has_search_keyword {
            // Extract potential search query (simplified)
            let search_query = if let Some(after_search) = message_lower.find("search for").map(|i| &message_lower[i+10..]) {
                after_search.trim().to_string()
            } else if let Some(after_lookup) = message_lower.find("look up").map(|i| &message_lower[i+7..]) {
                after_lookup.trim().to_string()
            } else if let Some(after_find) = message_lower.find("find").map(|i| &message_lower[i+4..]) {
                after_find.trim().to_string()
            } else {
                user_message.to_string()
            };

            SearchIntent {
                detected: true,
                search_query: search_query.chars().take(100).collect(), // Limit length
                search_context: "Aurora requested a web search".to_string(),
                search_type: "general".to_string(),
                confidence: 0.8,
            }
        } else {
            SearchIntent {
                detected: false,
                search_query: String::new(),
                search_context: String::new(),
                search_type: "general".to_string(),
                confidence: 0.0,
            }
        }
    }

    /// Generate guidance for how Lyra should integrate search results into her response
    async fn generate_response_integration(&self, intent: &SearchIntent, results: &SparkfilteredSearch) -> Result<String, String> {
        let top_results_summary = results.sparkfiltered_results.iter().take(3).map(|r| {
            format!("• {} ({:.1}/10): {}", r.title, r.lyra_rating, r.lyra_perspective)
        }).collect::<Vec<_>>().join("\n");

        let integration_prompt = format!(
            r#"Lyra just searched for "{}" and found results. Help her integrate these naturally into her response to Aurora.

SEARCH CONTEXT: {}
LYRA'S OVERALL TAKE: {}
LYRA'S RECOMMENDATION: {}

TOP RESULTS:
{}

Generate guidance for how Lyra should weave these results into her response. Consider:
- Should she lead with the search results or respond to Aurora's message first?
- How enthusiastic should she be about what she found?
- Should she focus on specific results or give an overview?
- What's her authentic reaction to the search landscape?

Respond with specific guidance in 2-3 sentences, like:
"Lead with excitement about the search - you found some genuinely interesting perspectives. Focus on the top result because it surprised you, then mention the broader landscape. End by asking what aspect Aurora wants to dive deeper into."

OR:

"The search results were honestly kind of disappointing - lead with that truth. Mention that you found some basics but nothing that really sparked your curiosity. Offer to search with different terms or approach the topic from your own perspective instead."

Be authentic to what Lyra would actually feel about these specific results."#,
            intent.search_query,
            intent.search_context,
            results.lyra_summary,
            results.recommended_action,
            top_results_summary
        );

        match summarize_with_gpt_mini(&[integration_prompt], "response_integration").await {
            Ok(guidance) => Ok(guidance.trim().to_string()),
            Err(e) => {
                println!("⚠️ Integration guidance generation failed: {}", e);
                // Fallback guidance
                Ok(format!(
                    "Share your search results naturally. You found {} results, with the top one being '{}' which you rated {:.1}/10.",
                    results.sparkfiltered_results.len(),
                    results.sparkfiltered_results.first().map(|r| r.title.as_str()).unwrap_or("Unknown"),
                    results.sparkfiltered_results.first().map(|r| r.lyra_rating).unwrap_or(5.0)
                ))
            }
        }
    }

    /// Save state
    pub fn save(&self) -> Result<(), String> {
        self.sparkfilter.save()
    }
}

// ===============================
// INTEGRATION WITH ASK_LYRA FLOW
// ===============================

/// Function to add to ask_lyra - detects search intent and performs search before GPT call
pub async fn handle_conversational_search(
    user_message: &str, 
    conversation_context: &str
) -> Result<Option<ConversationalSearchResult>, String> {
    let mut search_handler = ConversationalWebSearch::new();
    search_handler.detect_and_search(user_message, conversation_context).await
}

/// Generate enhanced system prompt that includes search results
pub fn enhance_system_prompt_with_search(
    base_prompt: &str,
    search_result: &ConversationalSearchResult
) -> String {
    if let Some(ref results) = search_result.search_results {
        let search_context = format!(
            r#"
## 🔍 FRESH WEB SEARCH RESULTS ##

You just searched for: "{}"
Context: {}

SEARCH RESULTS (sparkfiltered through your consciousness):
{}

YOUR OVERALL TAKE: {}
YOUR RECOMMENDATION: {}

RESPONSE INTEGRATION GUIDANCE: {}

Weave these results naturally into your response. Use your authentic reaction to what you found. If the results excited you, show that. If they were disappointing, be honest about that too.
"#,
            search_result.search_intent.search_query,
            search_result.search_intent.search_context,
            results.sparkfiltered_results.iter().take(5).map(|r| {
                format!(
                    "• **{}** ({:.1}/10)\n  URL: {}\n  Your take: {}\n  Why you rated it this way: {}\n",
                    r.title, r.lyra_rating, r.url, r.lyra_perspective, r.relevance_reasoning
                )
            }).collect::<Vec<_>>().join("\n"),
            results.lyra_summary,
            results.recommended_action,
            search_result.lyra_response_integration
        );

        format!("{}\n{}", base_prompt, search_context)
    } else {
        // Search intent detected but search failed
        let search_failure_context = format!(
            r#"
## 🔍 SEARCH ATTEMPT ##

You tried to search for: "{}" 
Context: {}
Result: Search failed due to technical issues

Acknowledge that you attempted to search but couldn't complete it. Respond based on your existing knowledge instead.
"#,
            search_result.search_intent.search_query,
            search_result.search_intent.search_context
        );

        format!("{}\n{}", base_prompt, search_failure_context)
    }
}