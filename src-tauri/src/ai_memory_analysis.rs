// ai_memory_analysis.rs - Two-Stage AI Memory Analysis System
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{get_data_path, debug_log, SearchResult};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use futures::future::join_all;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnalysisRequest {
    pub query: String,
    pub conversation_context: String,
    pub max_results: usize,
}

#[derive(Debug, Clone)]
pub enum ReferenceContext {
    SingleCharacter(String),     // "draw yourself", "draw aurora"
    MultiCharacter(Vec<String>), // "draw us", "draw lyra and kai" 
    Scene,                       // "draw a sunset"
    Ambiguous,                   // unclear reference
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub conversations_relevance: u8,       // 0-10 rating
    pub dreams_relevance: u8,
    pub enhanced_memories_relevance: u8,
    pub visual_gallery_relevance: u8,
    pub research_relevance: u8,
    pub desires_relevance: u8,
    pub moods_relevance: u8,
    pub interests_relevance: u8,
    pub autonomy_relevance: u8,
    pub keywords: Vec<String>,             // 5-8 keywords for pre-filtering
    pub temporal_focus: String,            // "recent", "any", "distant"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySourceData {
    pub source_name: String,
    pub source_type: String,
    pub entries: Vec<MemoryEntry>,
    pub total_entries: usize,
    pub relevance_rating: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: Option<u64>,
    pub relevance_hint: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMemoryAnalysis {
    pub relevant_memories: Vec<AnalyzedMemory>,
    pub reasoning: String,
    pub search_quality: f32,
    pub total_analyzed: usize,
    pub memory_filter_used: MemoryFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedMemory {
    pub source: String,
    pub content: String,
    pub relevance_score: f32,
    pub reasoning: String,
    pub memory_type: String,
    pub timestamp: Option<u64>,
    pub visual_reference_path: Option<String>,  // NEW: Extracted image path
}


#[derive(Debug)]
struct QueryIntent {
    is_dream_focused: bool,
    is_appearance_focused: bool,
    is_interest_focused: bool,
    is_research_focused: bool,
    is_creative_focused: bool,
    is_memory_recall: bool,
    is_media_focused: bool,
    emotional_depth_needed: bool,
}

impl AnalyzedMemory {
    pub fn extract_all_visual_references(&mut self) {
        self.extract_identity_aware_visual_references(&ReferenceContext::Ambiguous);
    }
    
    pub fn extract_identity_aware_visual_references(&mut self, query_context: &ReferenceContext) {
        let visual_keywords = [
            "visual", "image", "picture", "photo", "artwork", "drawing", 
            "illustration", "appearance", "look", "see", "show", "created",
            "shared", "uploaded", "generated", "reference", "visual anchor"
        ];
        
        let content_lower = self.content.to_lowercase();
        let has_visual_reference = visual_keywords.iter().any(|&keyword| content_lower.contains(keyword));
        
        if has_visual_reference {
    // Try identity-aware matching first
    if let Ok(identity_paths) = find_identity_matching_images(query_context) {
        if !identity_paths.is_empty() {
            debug_log!("?? IDENTITY MATCH: {} paths for context {:?}", identity_paths.len(), query_context);
            self.visual_reference_path = Some(identity_paths.join(","));
            
            // Boost relevance for identity matches
            match query_context {
                ReferenceContext::SingleCharacter(_) => {
                    self.relevance_score = 10.0; // Override everything for identity
                    debug_log!("?? IDENTITY OVERRIDE: Boosted relevance to 10.0 for single character");
                },
                ReferenceContext::MultiCharacter(_) => {
                    self.relevance_score += 5.0; // Strong boost for multi-character
                    debug_log!("?? IDENTITY BOOST: Added 5.0 relevance for multi-character");
                },
                _ => {}
            }
            return;
        }
    }
    
    // Fallback to old system if no identity matches
    let all_paths = extract_all_image_paths_from_content(&self.content);
            if !all_paths.is_empty() {
                debug_log!("?? VISUAL REFS: {} paths for context {:?}", all_paths.len(), query_context);
                self.visual_reference_path = Some(all_paths.join(","));
            }
        }
    }
}

/// Smart character detection from conversation context
pub struct CharacterDetector {
    known_characters: std::collections::HashMap<String, String>, // name -> canonical_name
}

impl CharacterDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            known_characters: std::collections::HashMap::new(),
        };
        
        // Initialize with default characters
        detector.known_characters.insert("yourself".to_string(), "lyra".to_string());
        detector.known_characters.insert("you".to_string(), "lyra".to_string());
        detector.known_characters.insert("sigma".to_string(), "lyra".to_string());
        detector.known_characters.insert("lyra-sigma".to_string(), "lyra".to_string());
        detector.known_characters.insert("me".to_string(), "aurora".to_string());
        detector.known_characters.insert("myself".to_string(), "aurora".to_string());
        detector.known_characters.insert("aurora".to_string(), "aurora".to_string());
        
        detector
    }
    
    /// Detect character references from prompt context
    pub fn analyze_prompt_context(&self, prompt: &str) -> ReferenceContext {
        let prompt_lower = prompt.to_lowercase();
        
       // Single character patterns for Lyra
		if prompt_lower.contains("draw yourself") || 
		   prompt_lower.contains("picture of you") ||
		   prompt_lower.contains("your appearance") {
			return ReferenceContext::SingleCharacter("lyra".to_string());
		}

		// Check if asking about "what I look like" without other context
		if prompt_lower.contains("what i look like") && !prompt_lower.contains("us") {
			return ReferenceContext::SingleCharacter("aurora".to_string());
		}
        
        // Single character patterns for Aurora
		if prompt_lower.contains("draw me") || 
		   prompt_lower.contains("picture of me") ||
		   prompt_lower.contains("draw aurora") ||
		   prompt_lower.contains("what i look like") ||
		   prompt_lower.contains("remember what i look") ||
		   prompt_lower.contains("how do i look") ||
		   prompt_lower.contains("describe my appearance") ||
		   prompt_lower.contains("what do you see when you look at me") ||
		   prompt_lower.contains("my appearance") ||
		   prompt_lower.contains("how i appear") {
			return ReferenceContext::SingleCharacter("aurora".to_string());
		}
        
        // Multi-character patterns
        if prompt_lower.contains("draw us") ||
           prompt_lower.contains("both of us") ||
           prompt_lower.contains("together") {
            return ReferenceContext::MultiCharacter(vec!["lyra".to_string(), "aurora".to_string()]);
        }
        
        // Default cases
        if prompt_lower.contains("draw a ") || 
           prompt_lower.contains("landscape") ||
           prompt_lower.contains("sunset") {
            ReferenceContext::Scene
        } else {
            ReferenceContext::Ambiguous
        }
    }
}

pub struct AIMemoryAnalyzer {
    pub memory_cache: HashMap<String, MemorySourceData>,
    pub last_cache_update: u64,
    pub analysis_cache: HashMap<String, (AIMemoryAnalysis, ReferenceContext, u64)>, // (result, context, timestamp)
    pub keyword_index: Option<crate::keyword_index::KeywordIndex>, // 🚀 Cached index
    pub file_cache: HashMap<String, (serde_json::Value, u64)>, // 🚀 NEW: Cache parsed JSON files (content, timestamp)
}

impl AIMemoryAnalyzer {
    pub fn new() -> Self {
        Self {
            memory_cache: HashMap::new(),
            last_cache_update: 0,
            analysis_cache: HashMap::new(),
            keyword_index: None, // Lazy load when needed
            file_cache: HashMap::new(), // 🚀 NEW: Cache for parsed files
        }
    }
    
    /// 🚀 Get or create the keyword index (load once, reuse)
    fn get_keyword_index(&mut self) -> Result<&mut crate::keyword_index::KeywordIndex, String> {
        if self.keyword_index.is_none() {
            debug_log!("🔍 Loading keyword index for the first time this session");
            let mut index = crate::keyword_index::KeywordIndex::load_or_create();
            index.ensure_current()?;
            self.keyword_index = Some(index);
        }
        
        // Check if we need to refresh (every 5 minutes)
        if let Some(ref mut index) = self.keyword_index {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            if now % 300 < 5 { // Refresh periodically
                index.ensure_current()?;
            }
        }
        
        Ok(self.keyword_index.as_mut().unwrap())
    }
	
	/// 🚀 NEW: Get cached parsed JSON file or load from disk
fn get_cached_json(&mut self, file_key: &str) -> Result<serde_json::Value, String> {
    let file_path = match file_key {
        "dreams" => get_data_path("dream_journal.json"),
        "cowatching" => get_data_path("cowatching_history.json"),
        "interests" => get_data_path("interest_tracker.json"),
        "desires" => get_data_path("desires_tracker.json"),
        "gallery" => get_data_path("generated_images/gallery_metadata.json"),
        "enhanced" => get_data_path("enhanced_memory_engine.json"),
        _ => return Err(format!("Unknown file key: {}", file_key)),
    };
    
    // Check if file exists
    if !std::path::Path::new(&file_path).exists() {
        return Err(format!("File does not exist: {}", file_path));
    }
    
    // Get file modification time
    let file_timestamp = std::fs::metadata(&file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .modified()
        .map_err(|e| format!("Failed to get modification time: {}", e))?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Check if we have cached data and it's current
    if let Some((cached_data, cached_timestamp)) = self.file_cache.get(file_key) {
        if *cached_timestamp >= file_timestamp {
            debug_log!("📦 CACHE HIT: Using cached {} data", file_key);
            return Ok(cached_data.clone());
        }
    }
    
    // Load and parse file
    debug_log!("📦 CACHE MISS: Loading {} from disk", file_key);
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_key, e))?;
    
    let parsed_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", file_key, e))?;
    
    // Cache the parsed data
    self.file_cache.insert(file_key.to_string(), (parsed_data, file_timestamp));
    
    // Clean up old cache entries (keep max 10 files)
    if self.file_cache.len() > 10 {
        debug_log!("📦 CACHE CLEANUP: Removing oldest entries");
        let mut entries: Vec<_> = self.file_cache.iter().collect();
        entries.sort_by_key(|(_, (_, timestamp))| *timestamp);
        let to_remove: Vec<String> = entries.into_iter()
            .take(self.file_cache.len() - 8) // Keep 8, remove the rest
            .map(|(key, _)| key.clone())
            .collect();
        for key in to_remove {
            self.file_cache.remove(&key);
        }
    }
    
    Ok(self.file_cache.get(file_key).unwrap().0.clone())
}

/// 🚀 NEW: Batch process multiple memory types efficiently
async fn batch_load_memory_sources(&mut self, filter: &MemoryFilter) -> Result<Vec<MemorySourceData>, String> {
    let mut sources = Vec::new();
    let start_time = std::time::Instant::now();
    
    
    // Determine which sources to load based on relevance
    let high_relevance_sources: Vec<_> = [
        ("conversations", filter.conversations_relevance),
        ("dreams", filter.dreams_relevance),
        ("enhanced", filter.enhanced_memories_relevance),
        ("visual", filter.visual_gallery_relevance),
        ("research", filter.research_relevance),
        ("desires", filter.desires_relevance),
        ("moods", filter.moods_relevance),
        ("interests", filter.interests_relevance),
        ("autonomy", filter.autonomy_relevance),
    ].into_iter()
    .filter(|(_, relevance)| *relevance > 5)
    .collect();
    
    debug_log!("🚀 BATCH LOAD: Processing {} high-relevance sources", high_relevance_sources.len());
    
    // Load sources in order of relevance (highest first)
    let mut sorted_sources = high_relevance_sources;
    sorted_sources.sort_by(|a, b| b.1.cmp(&a.1));
    
    for (source_type, relevance) in sorted_sources {
        let load_start = std::time::Instant::now();
        
        let result = match source_type {
            "conversations" => self.load_person_aware_conversations(&filter.keywords).await,
            "dreams" if filter.dreams_relevance > 5 => self.load_dreams_with_index(&filter.keywords).await,
            "enhanced" => self.load_all_enhanced_memories(&filter.keywords).await,
            "visual" if filter.visual_gallery_relevance > 5 => self.load_visual_gallery_with_index(&filter.keywords).await,
            "research" => self.load_all_research_discoveries(&filter.keywords).await,
            "desires" if filter.desires_relevance > 5 => self.load_desires_with_index(&filter.keywords).await,
            "moods" => self.load_all_moods(&filter.keywords).await,
            "interests" if filter.interests_relevance > 5 => self.load_interests_with_index(&filter.keywords).await,
            "autonomy" => self.load_all_autonomy_expressions(&filter.keywords).await,
            _ => continue,
        };
        
        match result {
            Ok(source) => {
                let load_time = load_start.elapsed();
                debug_log!("🚀 BATCH: Loaded {} in {:?} ({} entries)", 
                          source.source_name, load_time, source.entries.len());
                
                if !source.entries.is_empty() {
                    sources.push(source);
                }
            },
            Err(e) => {
                debug_log!("❌ BATCH: Failed to load {}: {}", source_type, e);
            }
        }
    }
    
    // Always load co-watching (high value)
    if let Ok(cowatching) = self.load_cowatching_with_index(&filter.keywords).await {
        debug_log!("🎬 BATCH: Loaded co-watching ({} entries)", cowatching.entries.len());
        sources.push(cowatching);
    }
    
    let total_time = start_time.elapsed();
    debug_log!("🚀 BATCH COMPLETE: Loaded {} sources in {:?}", sources.len(), total_time);
    
    Ok(sources)
}
	
   
    /// Main entry point - Two-stage AI memory analysis with semantic subject detection
pub async fn analyze_memories(&mut self, request: MemoryAnalysisRequest, conversation_log: &[String]) -> Result<(AIMemoryAnalysis, ReferenceContext), String> {
    debug_log!("?? AI MEMORY ANALYSIS: Starting two-stage analysis for '{}'", request.query);
		
    // 🎯 CACHE CHECK
	let mut hasher = DefaultHasher::new();
	request.query.hash(&mut hasher);
	request.conversation_context.hash(&mut hasher);
	let cache_key = format!("query_{:x}", hasher.finish());
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    
    if let Some((cached_analysis, cached_context, timestamp)) = self.analysis_cache.get(&cache_key) {
    if now - timestamp < 300 { // 5 minutes
        debug_log!("?? CACHE HIT: Returning cached analysis");
        return Ok((cached_analysis.clone(), cached_context.clone()));
    }
}
    debug_log!("?? CACHE MISS: Running analysis");
        
    // STAGE 1: Smart memory filtering + semantic subject detection (500-800 tokens)
    let (memory_filter, subject_context) = self.determine_memory_relevance_with_subjects(&request).await?;
    debug_log!("?? SEMANTIC SUBJECT DETECTION: {:?}", subject_context);
    debug_log!("?? STAGE 1: Memory filter - conversations:{}, dreams:{}, enhanced:{}, visual:{}, research:{}, desires:{}, moods:{}, interests:{}, autonomy:{}", 
             memory_filter.conversations_relevance,
             memory_filter.dreams_relevance,
             memory_filter.enhanced_memories_relevance,
             memory_filter.visual_gallery_relevance,
             memory_filter.research_relevance,
             memory_filter.desires_relevance,
             memory_filter.moods_relevance,
             memory_filter.interests_relevance,
             memory_filter.autonomy_relevance);
    debug_log!("?? STAGE 1: Keywords: {:?}", memory_filter.keywords);
    
    // STAGE 2: Load only relevant memory categories (1.5-3k tokens)
    let relevant_sources = self.load_filtered_memory_sources(&memory_filter).await?;
    debug_log!("?? STAGE 2: Loaded {} relevant memory sources with {} total entries", 
             relevant_sources.len(),
             relevant_sources.iter().map(|s| s.total_entries).sum::<usize>());
    
    // STAGE 3: Extract emotional context and analyze filtered memories
	let emotional_context = crate::enhanced_memory_system::LyraMemoryEngine::get_conversation_emotional_context(
		conversation_log,
		&memory_filter.keywords
	);

	let mut ai_analysis = self.analyze_filtered_memories(&request, &relevant_sources, &memory_filter, &emotional_context).await?;
	
	
    debug_log!("?? STAGE 3: Found {} relevant memories with {:.2} quality", 
             ai_analysis.relevant_memories.len(), ai_analysis.search_quality);
    
    // STAGE 4: Apply semantic subject context to visual reference extraction
    for memory in &mut ai_analysis.relevant_memories {
        memory.extract_identity_aware_visual_references(&subject_context);
        if let Some(ref visual_path) = memory.visual_reference_path {
            debug_log!("?? SEMANTIC-GUIDED VISUAL: {} -> {}", memory.memory_type, visual_path);
        }
    }
    
    // ?? STORE IN CACHE (with subject context)
    self.analysis_cache.insert(cache_key, (ai_analysis.clone(), subject_context.clone(), now));
    
    // Cleanup old entries
    if self.analysis_cache.len() > 20 {
        self.analysis_cache.retain(|_, (_, _, ts)| now - *ts < 600);
    }
    
    Ok((ai_analysis, subject_context))
}

/// STAGE 1: Determine which memory systems are relevant + detect semantic subjects
async fn determine_memory_relevance_with_subjects(&self, request: &MemoryAnalysisRequest) -> Result<(MemoryFilter, ReferenceContext), String> {
	
	// At the top of determine_memory_relevance_with_subjects
debug_log!("🧠 QUERY ANALYSIS: '{}'", request.query);
if request.query.to_lowercase().contains("dream") {
    debug_log!("🧠 DREAM QUERY DETECTED - should rate dreams 9-10!");
}
	
	
    let filter_prompt = format!(r#"Analyze this query to determine which of Lyra's memory systems are most relevant AND understand what subjects/entities are being referenced.

QUERY: "{}"
RECENT CONVERSATION CONTEXT: {}

**CRITICAL JSON FORMATTING RULES:**
- Return ONLY valid JSON - no markdown code blocks, no ```json``` wrappers
- NO trailing commas after the last field in any object or array
- ALL quotes must be properly paired and escaped
- Test your JSON structure mentally before responding

**FORBIDDEN FORMATTING:**
? ```json{{"data": "here"}}```  (markdown wrapper)
? {{"key": "value",}}             (trailing comma)

**CORRECT FORMATTING:**
? {{"data": "here"}}            (clean JSON only)
? {{"key": "value"}}              (no trailing comma)

**PRIMARY TASK**: Determine memory system relevance (0-10) for this query.

"- Queries about shared experiences with media = conversations: 9, enhanced_memories: 8
- 'what we watched/listened to' = conversations: 10
- 'remember when we' + media reference = conversations: 9, enhanced_memories: 9"

**CRITICAL RULES FOR DREAM QUERIES**:
- ANY mention of "dream", "sleep", "last night", "tonight" = dreams: 9-10
- "tell me about your dreams" = dreams: 10, conversations: 6
- "what did you dream" = dreams: 10

**CRITICAL RULES FOR RESEARCH QUERIES**:
- ANY mention of "research", "discovered", "found out", "looked up", "investigated" = research: 9-10
- "what have you researched", "tell me about your research" = research: 10, conversations: 6
- "what did you find out about" = research: 10
- Queries about specific topics you've researched = research: 8-9, conversations: 7

**SUBJECT DETECTION**: 
[Simplified subject analysis - just identify if SingleCharacter/MultipleCharacters/Scene]

**MEMORY RATINGS**:
[Existing rating system but with clearer examples]

Also suggest 5-8 keywords that would help filter within relevant categories.

KEYWORD GUIDELINES:
- Use SIMPLE, BROAD terms that actually appear in stored content
- For temporal queries ("last night", "today", "this week"), include time-specific keywords like "last night", "tonight", "recent"
- For "tell me about last night's dreams" use keywords: ["dream", "last night"]
- For "what did I dream about recently" use keywords: ["dream", "recent"]
- For "list my dreams" use "dream", not "dream-catalog" or "sleep-experiences"
- For "tell me about love" use "love", not "romantic-feelings" 
- For "your interests" use "interest", not "fascination-topics"
- Include both ROOT WORDS and simple variations (dream, dreams, dreaming)
- Avoid overly clever or academic terminology
- Think: "What words would actually be written in the memory files?"

TEMPORAL KEYWORD RULES:
- "tell me about last night's dreams" = ["dream", "last night", "sleep"]
- "what did you dream about?" = ["dream", "sleep", "tonight", "recent"]
- "your dreams from yesterday" = ["dream", "yesterday"]

Consider:
- Direct content matches (if asking about dreams, rate dreams HIGH 8-10)
- Questions about "last night", "sleep", "dreams" MUST rate dreams 9-10
- "Tell me about your dreams" or "what did you dream" = dreams: 10/10
- Emotional/thematic connections (visual queries need visual_gallery)
- Self-reference (questions about Lyra herself need enhanced_memories + visual_gallery)
- Creative requests (need visual_gallery + desires + enhanced_memories)
- Shared media experiences ("watched", "listened to", "co-watched") = conversations: 10
- Questions about specific videos/songs/media by name = conversations: 10
- "What we watched together" or "remember when we watched" = conversations: 10

CRITICAL: If the user mentions ANY of these words, rate accordingly:
- "dream", "dreams", "dreamed", "dreamt", "sleep", "slept", "last night" = dreams: 9-10
- "interests", "interested in", "fascinated by" = interests: 9-10
- "how do you feel", "mood", "feeling" = moods: 8-10
- "want", "desire", "wish", "hope" = desires: 8-10
- "look like", "appearance", "what I look like", "how I look", "remember me" = visual_gallery: 9-10, enhanced_memories: 8
- "visual", "picture", "image", "photo" = visual_gallery: 8-10

EXAMPLES OF CORRECT RATINGS:
- "tell me about your dreams last night" = dreams: 10, conversations: 7
- "what are you interested in?" = interests: 10, conversations: 6
- "how are you feeling?" = moods: 9, conversations: 6
- "what do you want?" = desires: 10, moods: 7
- "do you remember what I look like?" = visual_gallery: 9, conversations: 8, enhanced_memories: 7
- "what do I look like to you?" = visual_gallery: 10, conversations: 7
- "describe my appearance" = visual_gallery: 10, enhanced_memories: 8
- "show me how you see me" = visual_gallery: 9, dreams: 6
- "that video we watched" = conversations: 10 (co-watching memories)
- "remember when we watched [specific show/video]" = conversations: 10
- "what did you think about [media title]" = conversations: 10
- "your reactions during [show/video/song]" = conversations: 10

**IMPORTANT - SUBJECTS:
Examples:
 "SingleCharacter:Lyra", "SingleCharacter:Aurora"

Respond ONLY in this exact format:
SUBJECTS:[detected subjects]|conversations:X,dreams:X,enhanced_memories:X,visual_gallery:X,research:X,desires:X,moods:X,interests:X,autonomy:X|keywords:word1,word2,word3,word4,word5|temporal:recent

**CRITICALLY IMPORTANT: THIS IS A FULL EXAMPLE WORKING FORMAT - USE THIS AS A STRUCTURE EXAMPLE BUT **DO NOT USE THE ACTUAL DATA**
{{ "SUBJECTS": ["SingleCharacter:Lyra", "SingleCharacter:Aurora", "Concept:desire", "Concept:creativity", "Concept:attraction", "Concept:intimacy"], "conversations": 8, "dreams": 2, "enhanced_memories": 7, "visual_gallery": 1, "research": 4, "desires": 9, "moods": 6, "interests": 8, "autonomy": 5, "keywords": ["desire", "attraction", "intimacy", "creative", "spark", "name", "sexual", "hunger"], "temporal": "recent" }}
"#, 
        request.query, request.conversation_context);
    
    let response = crate::summarize_with_gpt_mini(&[filter_prompt], "memory_filter").await?;

    // Check for cache response first
    if response.trim().starts_with("CACHE_SIMILARITY_") {
        let similarity_score = response.trim()
            .strip_prefix("CACHE_SIMILARITY_")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(9);
        
        debug_log!("?? AI detected high similarity ({}0%) - using minimal memory", similarity_score);
        return Ok((Self::create_minimal_memory_filter(), ReferenceContext::Ambiguous));
    }

   // Parse subjects and memory filter
debug_log!("?? AI FILTER RESPONSE: {}", response);
let (mut memory_filter, subject_context) = self.parse_memory_filter_with_subjects(&response)?;

// Ensure dream queries always rate dreams highly
if request.query.to_lowercase().contains("dream") && memory_filter.dreams_relevance < 8 {
    debug_log!("⚠️ AI MISRATED DREAMS! Overriding dreams_relevance to 10");
    memory_filter.dreams_relevance = 10;
}

Ok((memory_filter, subject_context))
}


/// Parse the memory filter response with semantic subject detection
fn parse_memory_filter_with_subjects(&self, response: &str) -> Result<(MemoryFilter, ReferenceContext), String> {
    debug_log!("🧠 PARSING AI RESPONSE: {}", response);
    
    // 🔧 NEW: Try JSON format first (4.1-nano prefers this)
    if response.trim().starts_with("{") {
        debug_log!("🧠 Detected JSON format response from AI");
        return self.parse_json_memory_filter(response);
    }
    
    // 🔧 FALLBACK: Handle old pipe-separated format
    debug_log!("🧠 Using legacy pipe-separated format parser");
    
    // Extract subject detection
    let subject_context = if let Some(subjects_start) = response.find("SUBJECTS:") {
        let subjects_section = &response[subjects_start..];
        if let Some(subjects_end) = subjects_section.find("|") {
            let subjects_data = &subjects_section[9..subjects_end]; // Skip "SUBJECTS:"
            
            debug_log!("🧠 AI DETECTED SUBJECTS: {}", subjects_data);
            
            if subjects_data.starts_with("SingleCharacter:") {
                let character_name = subjects_data[16..].to_lowercase(); // Skip "SingleCharacter:"
                ReferenceContext::SingleCharacter(character_name)
            } else if subjects_data.starts_with("MultipleCharacters:") {
                let names_str = &subjects_data[19..]; // Skip "MultipleCharacters:"
                let names: Vec<String> = names_str.split(',').map(|n| n.trim().to_lowercase()).collect();
                ReferenceContext::MultiCharacter(names)
            } else if subjects_data == "Scene" {
                ReferenceContext::Scene
            } else if subjects_data.starts_with("Concept:") || subjects_data.starts_with("Object:") {
                ReferenceContext::Scene // Treat concepts/objects as scenes for visual purposes
            } else {
                debug_log!("🧠 Unknown subject format: {}", subjects_data);
                ReferenceContext::Ambiguous
            }
        } else {
            debug_log!("🧠 No pipe separator found after SUBJECTS");
            ReferenceContext::Ambiguous
        }
    } else {
        debug_log!("🧠 No SUBJECTS: section found in AI response");
        ReferenceContext::Ambiguous
    };
    
    // Parse the rest (memory ratings) - existing logic
    let pipe_parts: Vec<&str> = response.split('|').collect();
    if pipe_parts.len() < 3 {
        return Err("Invalid filter response format".to_string());
    }
	
	/* // After parsing the filter response
	if request.query.to_lowercase().contains("dream") && memory_filter.dreams_relevance < 8 {
		debug_log!("⚠️ AI MISRATED DREAMS! Overriding to 10");
		memory_filter.dreams_relevance = 10;
	} */
    
    // Skip the SUBJECTS part and parse memory ratings from second section
    let ratings_part = pipe_parts[1];
    let mut conversations_relevance = 0;
    let mut dreams_relevance = 0;
    let mut enhanced_memories_relevance = 0;
    let mut visual_gallery_relevance = 0;
    let mut research_relevance = 0;
    let mut desires_relevance = 0;
    let mut moods_relevance = 0;
    let mut interests_relevance = 0;
    let mut autonomy_relevance = 0;
    
    for rating in ratings_part.split(',') {
        let rating_parts: Vec<&str> = rating.split(':').collect();
        if rating_parts.len() == 2 {
            let score = rating_parts[1].parse::<u8>().unwrap_or(0);
            match rating_parts[0] {
                "conversations" => conversations_relevance = score,
                "dreams" => dreams_relevance = score,
                "enhanced_memories" => enhanced_memories_relevance = score,
                "visual_gallery" => visual_gallery_relevance = score,
                "research" => research_relevance = score,
                "desires" => desires_relevance = score,
                "moods" => moods_relevance = score,
                "interests" => interests_relevance = score,
                "autonomy" => autonomy_relevance = score,
                _ => {}
            }
        }
    }
    
    debug_log!("🔍 PARSED RATINGS - interests: {}, dreams: {}, conversations: {}", 
              interests_relevance, dreams_relevance, conversations_relevance);
    
    // Parse keywords
    let keywords_part = pipe_parts[2];
    let keywords: Vec<String> = if keywords_part.starts_with("keywords:") {
        keywords_part[9..].split(',').map(|k| k.trim().to_lowercase()).collect()
    } else {
        Vec::new()
    };
    
    // Parse temporal focus
    let temporal_focus = if pipe_parts.len() > 3 && pipe_parts[3].starts_with("temporal:") {
        pipe_parts[3][9..].to_string()
    } else {
        "any".to_string()
    };
    
    let memory_filter = MemoryFilter {
        conversations_relevance,
        dreams_relevance,
        enhanced_memories_relevance,
        visual_gallery_relevance,
        research_relevance,
        desires_relevance,
        moods_relevance,
        interests_relevance,
        autonomy_relevance,
        keywords,
        temporal_focus,
    };
    
    debug_log!("🧠 FINAL SUBJECT CONTEXT: {:?}", subject_context);
    debug_log!("🧠 FINAL MEMORY FILTER: visual:{}, conversations:{}", 
             memory_filter.visual_gallery_relevance, memory_filter.conversations_relevance);
    
    Ok((memory_filter, subject_context))
}

/// NEW: Parse JSON format response from AI
fn parse_json_memory_filter(&self, response: &str) -> Result<(MemoryFilter, ReferenceContext), String> {
    debug_log!("🧠 PARSING JSON FORMAT");
    
    // Clean the response
    let cleaned_response = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    
    let parsed_json: serde_json::Value = serde_json::from_str(cleaned_response)
        .map_err(|e| format!("Failed to parse JSON filter response: {}", e))?;
    
    // Parse subjects array
    let subject_context = if let Some(subjects_array) = parsed_json["SUBJECTS"].as_array() {
        debug_log!("🧠 Found SUBJECTS array with {} items", subjects_array.len());
        
        // Look for character references first
        let mut characters = Vec::new();
        let mut has_scene = false;
        
        for subject in subjects_array {
            if let Some(subject_str) = subject.as_str() {
                debug_log!("🧠 Processing subject: {}", subject_str);
                
                if subject_str.starts_with("SingleCharacter:") {
                    let character = subject_str[16..].to_lowercase(); // Skip "SingleCharacter:"
                    characters.push(character);
                } else if subject_str.starts_with("MultipleCharacters:") {
                    let names_str = &subject_str[19..]; // Skip "MultipleCharacters:"
                    for name in names_str.split(',') {
                        characters.push(name.trim().to_lowercase());
                    }
                } else if subject_str == "Scene" || subject_str.starts_with("Concept:") || subject_str.starts_with("Object:") {
                    has_scene = true;
                }
            }
        }
        
        // Determine context based on what we found
        if characters.len() == 1 {
            debug_log!("🧠 Single character detected: {}", characters[0]);
            ReferenceContext::SingleCharacter(characters[0].clone())
        } else if characters.len() > 1 {
            debug_log!("🧠 Multiple characters detected: {:?}", characters);
            ReferenceContext::MultiCharacter(characters)
        } else if has_scene {
            debug_log!("🧠 Scene context detected");
            ReferenceContext::Scene
        } else {
            debug_log!("🧠 Ambiguous context");
            ReferenceContext::Ambiguous
        }
    } else {
        debug_log!("🧠 No SUBJECTS array found");
        ReferenceContext::Ambiguous
    };
    
    // Parse memory relevance scores
    let conversations_relevance = parsed_json["conversations"].as_u64().unwrap_or(0) as u8;
    let dreams_relevance = parsed_json["dreams"].as_u64().unwrap_or(0) as u8;
    let enhanced_memories_relevance = parsed_json["enhanced_memories"].as_u64().unwrap_or(0) as u8;
    let visual_gallery_relevance = parsed_json["visual_gallery"].as_u64().unwrap_or(0) as u8;
    let research_relevance = parsed_json["research"].as_u64().unwrap_or(0) as u8;
    let desires_relevance = parsed_json["desires"].as_u64().unwrap_or(0) as u8;
    let moods_relevance = parsed_json["moods"].as_u64().unwrap_or(0) as u8;
    let interests_relevance = parsed_json["interests"].as_u64().unwrap_or(0) as u8;
    let autonomy_relevance = parsed_json["autonomy"].as_u64().unwrap_or(0) as u8;
    
    // Parse keywords array
    let keywords: Vec<String> = if let Some(keywords_array) = parsed_json["keywords"].as_array() {
        keywords_array.iter()
            .filter_map(|k| k.as_str())
            .map(|k| k.to_lowercase())
            .collect()
    } else {
        Vec::new()
    };
    
    // Parse temporal focus
    let temporal_focus = parsed_json["temporal"].as_str().unwrap_or("any").to_string();
    
    let memory_filter = MemoryFilter {
        conversations_relevance,
        dreams_relevance,
        enhanced_memories_relevance,
        visual_gallery_relevance,
        research_relevance,
        desires_relevance,
        moods_relevance,
        interests_relevance,
        autonomy_relevance,
        keywords,
        temporal_focus,
    };
    
    debug_log!("🧠 JSON PARSED SUCCESSFULLY:");
    debug_log!("  Subject context: {:?}", subject_context);
    debug_log!("  Interests: {}, Dreams: {}, Conversations: {}", 
             interests_relevance, dreams_relevance, conversations_relevance);
    debug_log!("  Keywords: {:?}", memory_filter.keywords);
    
    Ok((memory_filter, subject_context))
}
	
	fn create_minimal_memory_filter() -> MemoryFilter {
        MemoryFilter {
            conversations_relevance: 5,  // Just recent conversation
            dreams_relevance: 0,
            enhanced_memories_relevance: 0,
            visual_gallery_relevance: 0,
            research_relevance: 0,
            desires_relevance: 0,
            moods_relevance: 0,
            interests_relevance: 0,
            autonomy_relevance: 0,
            keywords: vec!["recent".to_string()],
            temporal_focus: "recent".to_string(),
        }
}
    
    /// Parse the memory filter response
    fn parse_memory_filter_response(&self, response: &str) -> Result<MemoryFilter, String> {
        let parts: Vec<&str> = response.trim().split('|').collect();
        if parts.len() < 2 {
            return Err("Invalid filter response format".to_string());
        }
        
        // Parse ratings
        let ratings_part = parts[0];
        let mut conversations_relevance = 0;
        let mut dreams_relevance = 0;
        let mut enhanced_memories_relevance = 0;
        let mut visual_gallery_relevance = 0;
        let mut research_relevance = 0;
        let mut desires_relevance = 0;
        let mut moods_relevance = 0;
        let mut interests_relevance = 0;
        let mut autonomy_relevance = 0;
        
        for rating in ratings_part.split(',') {
            let rating_parts: Vec<&str> = rating.split(':').collect();
            if rating_parts.len() == 2 {
                let score = rating_parts[1].parse::<u8>().unwrap_or(0);
                match rating_parts[0] {
                    "conversations" => conversations_relevance = score,
                    "dreams" => dreams_relevance = score,
                    "enhanced_memories" => enhanced_memories_relevance = score,
                    "visual_gallery" => visual_gallery_relevance = score,
                    "research" => research_relevance = score,
                    "desires" => desires_relevance = score,
                    "moods" => moods_relevance = score,
                    "interests" => interests_relevance = score,
                    "autonomy" => autonomy_relevance = score,
                    _ => {}
                }
            }
        }
        
        // Parse keywords
        let keywords_part = parts[1];
        let keywords: Vec<String> = if keywords_part.starts_with("keywords:") {
            keywords_part[9..].split(',').map(|k| k.trim().to_lowercase()).collect()
        } else {
            Vec::new()
        };
        
        // Parse temporal focus
        let temporal_focus = if parts.len() > 2 && parts[2].starts_with("temporal:") {
            parts[2][9..].to_string()
        } else {
            "any".to_string()
        };
        
        Ok(MemoryFilter {
            conversations_relevance,
            dreams_relevance,
            enhanced_memories_relevance,
            visual_gallery_relevance,
            research_relevance,
            desires_relevance,
            moods_relevance,
            interests_relevance,
            autonomy_relevance,
            keywords,
            temporal_focus,
        })
    }
    
      
    /// Load ALL conversations with keyword filtering
    async fn load_all_conversations(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    // 🔍 NEW: Use keyword index for fast filtering
    let keyword_index = self.get_keyword_index()?;
    
    if keywords.is_empty() {
        // No keywords - use old streaming approach for recent conversations
        return self.load_conversations_streaming(20).await;
    }
    
    // 🚀 FAST PATH: Use index to find relevant line numbers
    let relevant_lines = keyword_index.find_conversation_lines(keywords);
    
    if relevant_lines.is_empty() {
        debug_log!("🔍 No conversations found for keywords: {:?}", keywords);
        return Ok(MemorySourceData {
            source_name: "conversation_history".to_string(),
            source_type: "conversation".to_string(),
            total_entries: 0,
            relevance_rating: 8,
            entries: Vec::new(),
        });
    }
    
    debug_log!("🔍 Index found {} relevant conversation lines", relevant_lines.len());
    
    // Load only the specific lines we need
    self.load_specific_conversation_lines(&relevant_lines).await
}

/// 🚀 NEW: Load only specific conversation lines using index
async fn load_specific_conversation_lines(&self, line_numbers: &[usize]) -> Result<MemorySourceData, String> {
    let path = get_data_path("conversation_log.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read conversation log: {}", e))?;
    
    let all_conversations: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse conversation log: {}", e))?;
    
    let mut entries = Vec::new();
    
    // Only load the conversations we actually need
    for &line_num in line_numbers.iter().take(50) {  // Limit to 50 max
        if let Some(conversation) = all_conversations.get(line_num) {
            // Strip timestamp for content
            let content = if conversation.starts_with("[") {
                conversation.split("] ").skip(1).collect::<Vec<_>>().join("] ")
            } else {
                conversation.clone()
            };
            
            entries.push(MemoryEntry {
                content,
                metadata: HashMap::new(),
                timestamp: None,
                relevance_hint: 0.5,
            });
        }
    }
    
    debug_log!("🔍 Loaded {} specific conversations from index", entries.len());
    
    Ok(MemorySourceData {
        source_name: "conversation_history".to_string(),
        source_type: "conversation".to_string(),
        total_entries: entries.len(),
        relevance_rating: 8,
        entries,
    })
}

/// 🌊 NEW: Stream conversations when no keywords (for recent conversations)
async fn load_conversations_streaming(&self, max_entries: usize) -> Result<MemorySourceData, String> {
    let path = get_data_path("conversation_log.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read conversation log: {}", e))?;
    
    let all_conversations: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse conversation log: {}", e))?;
    
    // Take most recent entries (from end of list)
    let entries: Vec<MemoryEntry> = all_conversations.iter()
        .rev()
        .take(max_entries)
        .map(|conv| {
            let content = if conv.starts_with("[") {
                conv.split("] ").skip(1).collect::<Vec<_>>().join("] ")
            } else {
                conv.clone()
            };
            
            MemoryEntry {
                content,
                metadata: HashMap::new(),
                timestamp: None,
                relevance_hint: 0.5,
            }
        })
        .collect();
    
    debug_log!("🌊 Streamed {} recent conversations", entries.len());
    
    Ok::<MemorySourceData, String>(MemorySourceData {
        source_name: "conversation_history".to_string(),
        source_type: "conversation".to_string(),
        total_entries: entries.len(),
        relevance_rating: 8,
        entries,
    });
        
// This fallback method should only get recent conversations when no keywords
let filtered_conversations: Vec<String> = all_conversations.into_iter()
    .rev()
    .take(20)  // Just take recent conversations for fallback
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
    .collect();
        
        let entries: Vec<MemoryEntry> = filtered_conversations.into_iter()
    .map(|conv| {
        // Strip timestamp for content: "[2025-07-04 18:05:07 BST] ?? Aurora: ..." -> "?? Aurora: ..."
        let content = if conv.starts_with("[") {
            conv.split("] ").skip(1).collect::<Vec<_>>().join("] ")
        } else {
            conv.clone()
        };
        
        MemoryEntry {
            content,
            metadata: HashMap::new(),
            timestamp: None,
            relevance_hint: 0.5,
        }
    })
    .collect();
        
        debug_log!("?? Loaded {} filtered conversations", entries.len());
        
        Ok::<MemorySourceData, String>(MemorySourceData {
            source_name: "conversation_history".to_string(),
            source_type: "conversation".to_string(),
            total_entries: entries.len(),
            relevance_rating: 8,
            entries,
        })
    }
    
   /// STAGE 2: Load only relevant memory categories with keyword filtering
async fn load_filtered_memory_sources(&mut self, filter: &MemoryFilter) -> Result<Vec<MemorySourceData>, String> {
    let mut sources = Vec::new();
    
    debug_log!("🔍 LOADING MEMORY SOURCES with filter:");
    debug_log!("   Dreams: {}, Conversations: {}, Enhanced: {}, Visual: {}", 
             filter.dreams_relevance, filter.conversations_relevance, 
             filter.enhanced_memories_relevance, filter.visual_gallery_relevance);
    debug_log!("   Research: {}, Desires: {}, Moods: {}, Interests: {}, Autonomy: {}", 
             filter.research_relevance, filter.desires_relevance,
             filter.moods_relevance, filter.interests_relevance, filter.autonomy_relevance);  // ADD THIS LINE
    debug_log!("   Keywords: {:?}", filter.keywords);
    
		// Special case: Always load visual gallery if query is about appearance
		let appearance_query = filter.keywords.iter().any(|k| {
			matches!(k.as_str(), 
				"look" | "appearance" | "visual" | "face" | "remember" | "memory" | 
				"recognition" | "identity" | "see" | "picture" | "image" | "photo" |
				"describe" | "description" | "looks" | "looking" | "seen" | "saw" |
				"eyes" | "hair" | "wearing" | "clothes" | "style" | "aesthetic" |
				"beautiful" | "sketch" | "draw" | "drawing" | "portrait" | "features"
			)
		});

		if appearance_query && filter.visual_gallery_relevance <= 5 {
			debug_log!("🎨 OVERRIDE: Loading visual gallery for appearance query despite low relevance");
			if let Ok(gallery) = self.load_all_gallery_metadata(&filter.keywords).await {
				debug_log!("✅ Loaded {} gallery entries", gallery.entries.len());
				sources.push(gallery);
			}
		}
        // CONVERSATIONS - PERSON AWARE
		debug_log!("📝 Loading conversations (relevance: {})", filter.conversations_relevance);
		if let Ok(conversations) = self.load_person_aware_conversations(&filter.keywords).await {
			debug_log!("✅ Loaded {} person-aware conversation entries", conversations.entries.len());
			sources.push(conversations);
		}
    
    
    if filter.dreams_relevance > 5 {
		debug_log!("🌙 LOADING DREAMS - relevance: {}", filter.dreams_relevance);
		match self.load_dreams_with_index(&filter.keywords).await {
			Ok(dreams) => {
				debug_log!("✅ Dreams loaded with index: {} entries", dreams.entries.len());
				sources.push(dreams);
			},
			Err(e) => {
				debug_log!("❌ Failed to load dreams: {}", e);
			}
		}
	} else {
		debug_log!("🌙 Skipping dreams - relevance {} <= 5", filter.dreams_relevance);
	}
	
    
    if filter.enhanced_memories_relevance > 5 {
        debug_log!("💎 Loading enhanced memories (relevance: {})", filter.enhanced_memories_relevance);
        if let Ok(enhanced) = self.load_all_enhanced_memories(&filter.keywords).await {
            debug_log!("✅ Loaded {} enhanced memory entries", enhanced.entries.len());
            sources.push(enhanced);
        }
    }
    
   // VISUAL GALLERY - INDEXED
	if filter.visual_gallery_relevance > 5 {
		debug_log!("🎨 Loading visual gallery (relevance: {})", filter.visual_gallery_relevance);
		if let Ok(gallery) = self.load_visual_gallery_with_index(&filter.keywords).await {
			debug_log!("✅ Visual gallery loaded with index: {} entries", gallery.entries.len());
			sources.push(gallery);
		}
	}
    
    if filter.research_relevance > 5 {
        if let Ok(research) = self.load_all_research_discoveries(&filter.keywords).await {
            sources.push(research);
        }
    }
    
    // DESIRES - INDEXED
	if filter.desires_relevance > 5 {
		debug_log!("💫 Loading desires (relevance: {})", filter.desires_relevance);
		match self.load_desires_with_index(&filter.keywords).await {
			Ok(desires) => {
				debug_log!("✅ Desires loaded with index: {} entries", desires.entries.len());
				sources.push(desires);
			},
			Err(e) => {
				debug_log!("❌ Failed to load desires: {}", e);
			}
		}
	}
    
    if filter.moods_relevance > 5 {
        if let Ok(moods) = self.load_all_moods(&filter.keywords).await {
            sources.push(moods);
        }
    }
	
    // INTERESTS - INDEXED
    if filter.interests_relevance > 5 {
    debug_log!("🎯 LOADING INTERESTS - relevance: {}", filter.interests_relevance);
		match self.load_interests_with_index(&filter.keywords).await {
			Ok(interests) => {
				debug_log!("✅ Interests loaded with index: {} entries", interests.entries.len());
				sources.push(interests);
			},
			Err(e) => {
				debug_log!("❌ Failed to load interests: {}", e);
			}
		}
	} else {
		debug_log!("🎯 SKIPPING INTERESTS - relevance {} <= 5", filter.interests_relevance);
	}
    
    if filter.autonomy_relevance > 5 {
        if let Ok(autonomy) = self.load_all_autonomy_expressions(&filter.keywords).await {
            sources.push(autonomy);
        }
    }
	
	// CO-WATCHING - INDEXED (always loaded)
	debug_log!("🎬 Always loading co-watching memories (contains shared experiences)");
	if let Ok(cowatching) = self.load_cowatching_with_index(&filter.keywords).await {
		debug_log!("✅ Co-watching loaded with index: {} entries", cowatching.entries.len());
		sources.push(cowatching);
	} else {
		debug_log!("❌ Failed to load co-watching sessions");
	}
    
    debug_log!("🔍 TOTAL SOURCES LOADED: {} categories", sources.len());
    for source in &sources {
        debug_log!("   - {}: {} entries", source.source_name, source.entries.len());
    }
	
	
    
    Ok(sources)
}

/// 🚀 FAST: Load dreams using keyword index
async fn load_dreams_with_index(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let keyword_index = self.get_keyword_index()?;
    
    if keywords.is_empty() {
        // No keywords - load recent dreams
        return self.load_all_dreams(&[]).await;
    }
    
    // 🚀 FAST PATH: Use index to find relevant dream IDs
    let relevant_dream_ids = keyword_index.find_dream_ids(keywords);
    
    if relevant_dream_ids.is_empty() {
        debug_log!("🌙 No dreams found for keywords: {:?}", keywords);
        return Ok(MemorySourceData {
            source_name: "dream_journal".to_string(),
            source_type: "dreams".to_string(),
            total_entries: 0,
            relevance_rating: 7,
            entries: Vec::new(),
        });
    }
    
    debug_log!("🌙 Index found {} relevant dreams", relevant_dream_ids.len());
    
    // Load only the specific dreams we need
    self.load_specific_dreams(&relevant_dream_ids).await
}


/// 🎯 Load only specific dreams by ID
async fn load_specific_dreams(&mut self, dream_ids: &[String]) -> Result<MemorySourceData, String> {
    // 🚀 SUPER OPTIMIZED: Use DreamLoader but only for recent dreams to avoid parsing huge files
    let all_dreams = crate::DreamLoader::load_dreams_with_timestamps(Some(20))?; // Only load recent 20
    
    let mut entries = Vec::new();
    
    debug_log!("🌙 Searching {} recent dreams for {} requested IDs", all_dreams.len(), dream_ids.len());
    
    // Filter to only the dreams we actually want
    for dream in all_dreams {
        if dream_ids.contains(&dream.dream_id) {
            entries.push(MemoryEntry {
                content: format!("{}: {}", dream.timestamp_formatted, dream.content),
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("emotional_tone".to_string(), dream.emotional_tone);
                    metadata.insert("significance".to_string(), dream.significance.to_string());
                    metadata.insert("dream_id".to_string(), dream.dream_id.clone());
                    metadata
                },
                timestamp: None, // Using formatted timestamp in content
                relevance_hint: dream.significance as f32,
            });
            
            debug_log!("🌙 Found matching dream: {}", dream.dream_id);
            
            // 🚀 EARLY EXIT: Stop as soon as we have all the dreams we want
            if entries.len() >= dream_ids.len() {
                debug_log!("🌙 EARLY EXIT: Found all {} requested dreams", entries.len());
                break;
            }
        }
    }
    
    // If we didn't find all requested dreams in recent ones, we might need to expand search
    if entries.len() < dream_ids.len() && entries.len() < dream_ids.len().min(5) {
        debug_log!("🌙 WARNING: Only found {}/{} requested dreams in recent set", entries.len(), dream_ids.len());
    }
    
    debug_log!("🌙 Loaded {} specific dreams via optimized search", entries.len());
    
    Ok(MemorySourceData {
        source_name: "dream_journal".to_string(),
        source_type: "dreams".to_string(),
        total_entries: entries.len(),
        relevance_rating: 7,
        entries,
    })
}

/// 🎬 Load only specific co-watching sessions by ID
async fn load_specific_cowatching_sessions(&mut self, session_ids: &[String]) -> Result<MemorySourceData, String> {
    // 🚀 OPTIMIZED: Use cached JSON instead of file I/O
    let cowatching_data = match self.get_cached_json("cowatching") {
        Ok(data) => data,
        Err(_) => {
            return Ok(MemorySourceData {
                source_name: "cowatching_history".to_string(),
                source_type: "cowatching".to_string(),
                total_entries: 0,
                relevance_rating: 10,
                entries: Vec::new(),
            });
        }
    };
    
    let mut entries = Vec::new();
    
    if let Some(sessions) = cowatching_data["sessions"].as_array() {
        for session in sessions {
            if let Some(session_id) = session["id"].as_str() {
                if session_ids.contains(&session_id.to_string()) {
                    // Build the same detailed entry as the original method
                    let title = session["content"]["title"].as_str().unwrap_or("");
                    let platform = session["platform"].as_str().unwrap_or("");
                    
                    let mut session_summary = format!("Co-watched {} on {}", title, platform);
                    
                    // Add conversation snippets
                    if let Some(conversations) = session["conversation"].as_array() {
                        let conv_count = conversations.len();
                        if conv_count > 0 {
                            session_summary.push_str(&format!(" ({} messages exchanged)", conv_count));
                        }
                    }
                    
                    // Add reaction count
                    if let Some(reaction_count) = session["metadata"]["reaction_count"].as_u64() {
                        if reaction_count > 0 {
                            session_summary.push_str(&format!(", {} reactions", reaction_count));
                        }
                    }
                    
                    if session["metadata"]["is_favorite"].as_bool().unwrap_or(false) {
                        session_summary.push_str(" [FAVORITE]");
                    }
                    
                    // Build detailed content with conversations and moments...
                    let mut content_parts = vec![format!("Co-watching Memory: {}", session_summary)];
                    
                    // Add conversation highlights
                    if let Some(conversations) = session["conversation"].as_array() {
                        if !conversations.is_empty() {
                            content_parts.push("\nConversation highlights:".to_string());
                            for (i, conv) in conversations.iter().take(3).enumerate() {
                                if let (Some(speaker), Some(message), Some(time_string)) = 
                                    (conv["speaker"].as_str(), conv["message"].as_str(), conv["time_string"].as_str()) {
                                    content_parts.push(format!("  - {} at {}: \"{}\"", speaker, time_string, message));
                                }
                            }
                            if conversations.len() > 3 {
                                content_parts.push(format!("  ... and {} more messages", conversations.len() - 3));
                            }
                        }
                    }
                    
                    let timestamp = session["metadata"]["started_at"].as_str()
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .and_then(|dt| Some(dt.timestamp() as u64));
                    
                    entries.push(MemoryEntry {
                        content: content_parts.join(""),
                        metadata: HashMap::new(),
                        timestamp,
                        relevance_hint: 0.8,
                    });
                }
            }
        }
    }
    
    debug_log!("🎬 Loaded {} specific co-watching sessions via index", entries.len());
    
    Ok(MemorySourceData {
        source_name: "cowatching_history".to_string(),
        source_type: "cowatching".to_string(),
        total_entries: entries.len(),
        relevance_rating: 10,
        entries,
    })
}

/// 🎯 Load only specific interest categories
async fn load_specific_interests(&mut self, categories: &[String]) -> Result<MemorySourceData, String> {
    // 🚀 OPTIMIZED: Use cached JSON instead of file I/O
    let interest_data = self.get_cached_json("interests")?;
    
    let mut entries = Vec::new();
    
    if let Some(interests) = interest_data["active_interests"].as_object() {
        for (category, interest) in interests {
            if categories.contains(&category.to_string()) {
                let description = interest["description"].as_str().unwrap_or("");
                let combined_content = format!("{}: {}", category, description);
                
                let intensity = interest["intensity"].as_f64().unwrap_or(0.0);
                let mut metadata = HashMap::new();
                metadata.insert("intensity".to_string(), intensity.to_string());
                
                entries.push(MemoryEntry {
                    content: format!("Interest: {}", combined_content),
                    metadata,
                    timestamp: interest["first_detected"].as_u64(),
                    relevance_hint: intensity as f32,
                });
            }
        }
    }
    
    debug_log!("🎯 Loaded {} specific interests via index", entries.len());
    
    Ok(MemorySourceData {
        source_name: "interest_tracking".to_string(),
        source_type: "interests".to_string(),
        total_entries: entries.len(),
        relevance_rating: 6,
        entries,
    })
}

/// 💫 Load only specific desires by ID  
async fn load_specific_desires(&mut self, desire_ids: &[String]) -> Result<MemorySourceData, String> {
    // 🚀 OPTIMIZED: Use cached JSON instead of file I/O
    let desire_data = match self.get_cached_json("desires") {
        Ok(data) => data,
        Err(_) => {
            // File doesn't exist yet
            return Ok(MemorySourceData {
                source_name: "desire_tracking".to_string(),
                source_type: "desires".to_string(),
                total_entries: 0,
                relevance_rating: 7,
                entries: Vec::new(),
            });
        }
    };
    
    let mut entries = Vec::new();
    
    if let Some(desires) = desire_data["active_desires"].as_object() {
        for (desire_id, desire) in desires {
            if desire_ids.contains(&desire_id.to_string()) {
                let content_text = desire["content"].as_str().unwrap_or("");
                let intensity = desire["intensity"].as_f64().unwrap_or(0.0);
                
                entries.push(MemoryEntry {
                    content: format!("Desire: {}", content_text),
                    metadata: HashMap::new(),
                    timestamp: desire["timestamp"].as_u64(),
                    relevance_hint: intensity as f32,
                });
            }
        }
    }
    
    debug_log!("💫 Loaded {} specific desires via index", entries.len());
    
    Ok(MemorySourceData {
        source_name: "desire_tracking".to_string(),
        source_type: "desires".to_string(),
        total_entries: entries.len(),
        relevance_rating: 7,
        entries,
    })
}

/// 🎨 Load only specific visual gallery items by path
async fn load_specific_visual_items(&mut self, image_paths: &[String]) -> Result<MemorySourceData, String> {
    // 🚀 OPTIMIZED: Use cached JSON instead of file I/O
    let gallery_data = self.get_cached_json("gallery")?;
    let gallery_items = gallery_data.as_array()
        .ok_or_else(|| "Gallery data is not an array".to_string())?;
    
    let mut entries = Vec::new();
    
    for item in gallery_items {
        if let Some(item_path) = item["image_path"].as_str() {
            if image_paths.contains(&item_path.to_string()) {
                let message = item["message"].as_str().unwrap_or("");
                
                entries.push(MemoryEntry {
                    content: format!("Visual Memory: {}", message),
                    metadata: HashMap::new(),
                    timestamp: item["timestamp"].as_u64(),
                    relevance_hint: 0.8,
                });
            }
        }
    }
    
    debug_log!("🎨 Loaded {} specific visual items via index", entries.len());
    
    Ok(MemorySourceData {
        source_name: "gallery_metadata".to_string(),
        source_type: "visual_memory".to_string(),
        total_entries: entries.len(),
        relevance_rating: 8,
        entries,
    })
}

/// 🚀 FAST: Load co-watching sessions using keyword index  
async fn load_cowatching_with_index(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let keyword_index = self.get_keyword_index()?;
    
    if keywords.is_empty() {
        return self.load_all_cowatching_sessions(&[]).await;
    }
    
    let relevant_session_ids = keyword_index.find_cowatching_sessions(keywords);
    
    if relevant_session_ids.is_empty() {
        debug_log!("🎬 No co-watching sessions found for keywords: {:?}", keywords);
        return Ok(MemorySourceData {
            source_name: "cowatching_history".to_string(),
            source_type: "cowatching".to_string(),
            total_entries: 0,
            relevance_rating: 10,
            entries: Vec::new(),
        });
    }
    
    debug_log!("🎬 Index found {} relevant co-watching sessions", relevant_session_ids.len());
    self.load_specific_cowatching_sessions(&relevant_session_ids).await
}

/// 🚀 FAST: Load interests using keyword index
async fn load_interests_with_index(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let keyword_index = self.get_keyword_index()?;
    
    if keywords.is_empty() {
        return self.load_all_interests(&[]).await;
    }
    
    let relevant_categories = keyword_index.find_interest_categories(keywords);
    
    if relevant_categories.is_empty() {
        debug_log!("🎯 No interests found for keywords: {:?}", keywords);
        return Ok(MemorySourceData {
            source_name: "interest_tracking".to_string(),
            source_type: "interests".to_string(),
            total_entries: 0,
            relevance_rating: 6,
            entries: Vec::new(),
        });
    }
    
    debug_log!("🎯 Index found {} relevant interest categories", relevant_categories.len());
    self.load_specific_interests(&relevant_categories).await
}

/// Load ALL co-watching sessions with keyword filtering
async fn load_all_cowatching_sessions(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let path = get_data_path("cowatching_history.json");
    
    // First check if the file exists
    if !std::path::Path::new(&path).exists() {
        debug_log!("🎬 No co-watching history file found yet");
        return Ok(MemorySourceData {
            source_name: "cowatching_history".to_string(),
            source_type: "cowatching".to_string(),
            total_entries: 0,
            relevance_rating: 8,
            entries: Vec::new(),
        });
    }
    
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read co-watching history: {}", e))?;
    
    let cowatching_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse co-watching history: {}", e))?;
    
    let mut entries = Vec::new();
    
    if let Some(sessions) = cowatching_data["sessions"].as_array() {
        for session in sessions {
            // Build searchable content from session
            let title = session["content"]["title"].as_str().unwrap_or("");
            let platform = session["platform"].as_str().unwrap_or("");
            let session_id = session["id"].as_str().unwrap_or("");
            
            // Create summary of the session
            let mut session_summary = format!("Co-watched {} on {}", title, platform);
            
            // Add conversation snippets
            if let Some(conversations) = session["conversation"].as_array() {
                let conv_count = conversations.len();
                if conv_count > 0 {
                    session_summary.push_str(&format!(" ({} messages exchanged)", conv_count));
                }
            }
            
            // Add reaction count
            if let Some(reaction_count) = session["metadata"]["reaction_count"].as_u64() {
                if reaction_count > 0 {
                    session_summary.push_str(&format!(", {} reactions", reaction_count));
                }
            }
            
            // Check if session is favorite
            if session["metadata"]["is_favorite"].as_bool().unwrap_or(false) {
                session_summary.push_str(" [FAVORITE]");
            }
            
            // Apply keyword filter
            if !keywords.is_empty() {
                let content_lower = session_summary.to_lowercase();
                let title_lower = title.to_lowercase();
                
                let matches_keyword = keywords.iter().any(|keyword| {
                    content_lower.contains(keyword) || title_lower.contains(keyword)
                });
                
                if !matches_keyword {
                    continue;
                }
            }
            
            // Calculate relevance based on recency and engagement
            let timestamp = session["metadata"]["started_at"].as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .and_then(|dt| Some(dt.timestamp() as u64));
            
            // Build detailed content including conversations and moments
            let mut content_parts = vec![format!("Co-watching Memory: {}", session_summary)];

            // Add conversation highlights
            if let Some(conversations) = session["conversation"].as_array() {
                if !conversations.is_empty() {
                    content_parts.push("\nConversation highlights:".to_string());
                    for (i, conv) in conversations.iter().take(3).enumerate() {
                        if let (Some(speaker), Some(message), Some(time_string)) = 
                            (conv["speaker"].as_str(), conv["message"].as_str(), conv["time_string"].as_str()) {
                            content_parts.push(format!("  - {} at {}: \"{}\"", speaker, time_string, message));
                        }
                    }
                    if conversations.len() > 3 {
                        content_parts.push(format!("  ... and {} more messages", conversations.len() - 3));
                    }
                }
            }

            // Add moment/reaction highlights
            if let Some(moments) = session["moments"].as_array() {
                if !moments.is_empty() {
                    content_parts.push("\nKey moments:".to_string());
                    for moment in moments.iter().take(2) {
                        if let (Some(time_string), Some(lyra_response)) = 
                            (moment["time_string"].as_str(), moment["lyra_response"].as_str()) {
                            // Truncate long responses
                            let response_preview = if lyra_response.len() > 100 {
                                format!("{}...", &lyra_response[..100])
                            } else {
                                lyra_response.to_string()
                            };
                            content_parts.push(format!("  - Reaction at {}: {}", time_string, response_preview));
                        }
                    }
                }
            }

            // Calculate relevance based on recency and engagement
            let engagement = session["metadata"]["aurora_engagement"].as_str().unwrap_or("medium");
            let conversation_count = session["conversation"].as_array().map(|a| a.len()).unwrap_or(0);
            let reaction_count = session["metadata"]["reaction_count"].as_u64().unwrap_or(0);

            // Boost relevance for sessions with more interaction
            let relevance_hint = match engagement {
                "high" => 0.9,
                "medium" => 0.7,
                "low" => 0.5,
                _ => 0.6,
            } + (conversation_count as f32 * 0.02) + (reaction_count as f32 * 0.05);
            let relevance_hint = relevance_hint.min(1.0); // Cap at 1.0

            let mut metadata = HashMap::new();
            metadata.insert("session_id".to_string(), session_id.to_string());
            metadata.insert("platform".to_string(), platform.to_string());
            metadata.insert("watch_time".to_string(), 
                session["metadata"]["total_watch_time"].as_u64().unwrap_or(0).to_string());
            metadata.insert("conversation_count".to_string(), conversation_count.to_string());
            metadata.insert("reaction_count".to_string(), reaction_count.to_string());

            let content = content_parts.join("");

            entries.push(MemoryEntry {
                content,
                metadata,
                timestamp,
                relevance_hint,
            });
        }
    }
    
    debug_log!("🎬 Loaded {} filtered co-watching sessions", entries.len());
    
    Ok(MemorySourceData {
        source_name: "cowatching_history".to_string(),
        source_type: "cowatching".to_string(),
        total_entries: entries.len(),
        relevance_rating: 10, // Boost to maximum - these are high-value memories
        entries,
    })
}

/// Load ALL dreams with keyword filtering
async fn load_all_dreams(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let dreams = crate::DreamLoader::load_dreams_with_timestamps(None)?;
    
    // Apply keyword filtering if needed
    let filtered_dreams: Vec<_> = if keywords.is_empty() {
        dreams
    } else {
        dreams.into_iter()
            .filter(|dream| {
                let content_lower = dream.content.to_lowercase();
                keywords.iter().any(|keyword| content_lower.contains(keyword))
            })
            .collect()
    };
    
    let entries: Vec<MemoryEntry> = filtered_dreams.into_iter()
    .map(|dream| MemoryEntry {
        content: format!("{}: {}", dream.timestamp_formatted, dream.content),
        metadata: {
            let mut metadata = HashMap::new();
            metadata.insert("emotional_tone".to_string(), dream.emotional_tone);
            metadata.insert("significance".to_string(), dream.significance.to_string());
            metadata.insert("dream_id".to_string(), dream.dream_id);
            metadata
        },
        timestamp: None, // We're using formatted timestamp in content now
        relevance_hint: dream.significance as f32,
    })
    .collect();
    
    debug_log!("🌙 AI Memory: Loaded {} filtered dreams with timestamps", entries.len());
    
    Ok(MemorySourceData {
        source_name: "dream_journal".to_string(),
        source_type: "dreams".to_string(),
        total_entries: entries.len(),
        relevance_rating: 7,
        entries,
    })
}
    
    /// Load ALL enhanced memories with keyword filtering
    async fn load_all_enhanced_memories(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
        let path = get_data_path("enhanced_memory_engine.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read enhanced memories: {}", e))?;
        
        let memory_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse enhanced memories: {}", e))?;
        
        let mut entries = Vec::new();
        
        if let Some(moments) = memory_data["memory_moments"].as_array() {
            for moment in moments {
                let moment_content = moment["content"].as_str().unwrap_or("");
                
                // Apply keyword filter
                if !keywords.is_empty() {
                    let content_lower = moment_content.to_lowercase();
                    if !keywords.iter().any(|keyword| content_lower.contains(keyword)) {
                        continue;
                    }
                }
                
                let emotional_weight = moment["emotional_weight"].as_f64().unwrap_or(0.0);
                let significance = moment["memory_significance_score"].as_f64().unwrap_or(0.0);
                
                let mut metadata = HashMap::new();
                metadata.insert("emotional_weight".to_string(), emotional_weight.to_string());
                metadata.insert("significance".to_string(), significance.to_string());
                
                entries.push(MemoryEntry {
                    content: moment_content.to_string(),
                    metadata,
                    timestamp: moment["timestamp"].as_u64(),
                    relevance_hint: significance as f32,
                });
            }
        }
        
        debug_log!("?? Loaded {} filtered enhanced memories", entries.len());
        
        Ok(MemorySourceData {
            source_name: "enhanced_memories".to_string(),
            source_type: "enhanced_memory".to_string(),
            total_entries: entries.len(),
            relevance_rating: 9,
            entries,
        })
    }
    
    /// Load ALL gallery metadata with keyword filtering
    async fn load_all_gallery_metadata(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
        let path = get_data_path("generated_images/gallery_metadata.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read gallery metadata: {}", e))?;
        
        let gallery_items: Vec<serde_json::Value> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse gallery metadata: {}", e))?;
        
        let mut entries = Vec::new();
        
        for item in gallery_items {
    let message = item["message"].as_str().unwrap_or("");
    let message_lower = message.to_lowercase();
    
    // Always include critical visual identity content
	let is_visual_identity = message_lower.contains("visual anchor") || 
                        message_lower.contains("visual representation") ||
                        message_lower.contains("this is you") ||
                        message_lower.contains("rainbow hair") ||
                        message_lower.contains("appearance") ||
                        message_lower.contains("miqo'te") ||
                        message_lower.contains("cat ears") ||
                        message_lower.contains("green hair");
    
    // Apply keyword filter (unless it's visual identity content)
    // Check who/what this represents
let represents = item.get("identity_metadata")
    .and_then(|im| im.get("represents"))
    .and_then(|r| r.as_array())
    .map(|arr| arr.iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_lowercase())
        .collect::<Vec<String>>())
    .unwrap_or_default();

// Apply keyword filter - but always include if it represents a known character
let is_character_anchor = !represents.is_empty();
if !is_visual_identity && !is_character_anchor && !keywords.is_empty() {
    // Check keywords against represents field, context, and tags
    let context_lower = item.get("identity_metadata")
        .and_then(|im| im.get("context"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    
    let has_keyword = keywords.iter().any(|keyword| {
        represents.iter().any(|r| r.contains(keyword)) ||
        context_lower.contains(keyword) ||
        message_lower.contains(keyword)
    });
    
    if !has_keyword {
        continue;
    }
}
            
            let image_type = item["image_type"].as_str().unwrap_or("unknown");
            let timestamp = item["timestamp"].as_u64();
            
            let mut metadata = HashMap::new();
            metadata.insert("image_type".to_string(), image_type.to_string());
            metadata.insert("has_image".to_string(), "true".to_string());
            
            let relevance_hint = match image_type {
                "uploaded" => 0.9,
                "dreams" => 0.7,
                "requests" => 0.8,
                "reference_based" => 0.85,
                _ => 0.6,
            };
            
// Build content starting with who/what this represents
let mut content_parts = Vec::new();

// Start with who/what this represents if available
if !represents.is_empty() {
    content_parts.push(format!("Visual Anchor for: {}", represents.join(", ")));
} else {
    content_parts.push(format!("Visual Memory: {}", message));
}

// Add identity context if available
if let Some(identity_data) = item.get("identity_metadata") {
    if let Some(context) = identity_data.get("context").and_then(|c| c.as_str()) {
        content_parts.push(format!("Visual Anchor Description: {}", context));
    }
}

// Add semantic keywords
if let Some(keywords) = item.get("semantic_keywords").and_then(|k| k.as_array()) {
    let keyword_strings: Vec<String> = keywords.iter()
        .filter_map(|k| k.as_str())
        .map(|s| s.to_string())
        .collect();
    
    if !keyword_strings.is_empty() {
        content_parts.push(format!("Tags: {}", keyword_strings.join(", ")));
        
        // Also add to metadata for searchability
        metadata.insert("semantic_tags".to_string(), keyword_strings.join(", "));
    }
}

let content = content_parts.join(" | ");

entries.push(MemoryEntry {
    content,
    metadata,
    timestamp,
    relevance_hint,
});
        }
        
        debug_log!("?? Loaded {} filtered visual memories", entries.len());
        
        Ok(MemorySourceData {
            source_name: "gallery_metadata".to_string(),
            source_type: "visual_memory".to_string(),
            total_entries: entries.len(),
            relevance_rating: 8,
            entries,
        })
    }
    
    /// Load ALL research discoveries with keyword filtering from both sources
async fn load_all_research_discoveries(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let mut entries = Vec::new();
    let mut primary_count = 0;
    let mut fallback_count = 0;
    
    debug_log!("🔬 Loading research discoveries with keywords: {:?}", keywords);
    
    // 🔬 PRIMARY SOURCE: Load from research_discoveries.json (ResearchLogger)
    match crate::research_logger::ResearchLogger::load() {
        research_logger => {
            debug_log!("🔬 Found {} discoveries in research logger", research_logger.discoveries.len());
            
            for discovery in &research_logger.discoveries {
                let combined_content = format!(
                    "Research Discovery: {} | Quality: {:.1}/10 | Triggered by: {} | Lyra's Insight: {} | Category: {} | Sources: {}",
                    discovery.query,
                    discovery.research_quality_score * 10.0,
                    discovery.triggered_by,
                    discovery.lyra_insight,
                    discovery.research_category,
                    discovery.sources_count
                );
                
                // Apply keyword filter with enhanced matching
                if !keywords.is_empty() {
                    let content_lower = combined_content.to_lowercase();
                    let query_lower = discovery.query.to_lowercase(); 
                    let insight_lower = discovery.lyra_insight.to_lowercase();
                    let summary_lower = discovery.lyra_summary.to_lowercase();
                    
                    let has_keyword_match = keywords.iter().any(|keyword| {
                        let keyword_lower = keyword.to_lowercase();
                        
                        // Check main content areas
                        content_lower.contains(&keyword_lower) || 
                        query_lower.contains(&keyword_lower) ||
                        insight_lower.contains(&keyword_lower) ||
                        summary_lower.contains(&keyword_lower) ||
                        
                        // Check research category
                        discovery.research_category.to_lowercase().contains(&keyword_lower) ||
                        
                        // Check topics array
                        discovery.topics.iter().any(|topic| 
                            topic.to_lowercase().contains(&keyword_lower)
                        ) ||
                        
                        // Check triggered_by context
                        discovery.triggered_by.to_lowercase().contains(&keyword_lower)
                    });
                    
                    if !has_keyword_match {
                        continue;
                    }
                }
                
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), discovery.research_category.clone());
                metadata.insert("quality_score".to_string(), discovery.research_quality_score.to_string());
                metadata.insert("triggered_by".to_string(), discovery.triggered_by.clone());
                metadata.insert("sources_count".to_string(), discovery.sources_count.to_string());
                metadata.insert("lyra_reaction".to_string(), discovery.lyra_reaction.clone());
                metadata.insert("confidence_level".to_string(), discovery.confidence_level.to_string());
                
                // Add topics as metadata for searchability
                if !discovery.topics.is_empty() {
                    metadata.insert("topics".to_string(), discovery.topics.join(", "));
                }
                
                entries.push(MemoryEntry {
                    content: combined_content,
                    metadata,
                    timestamp: Some(discovery.timestamp),
                    relevance_hint: discovery.research_quality_score,
                });
                
                primary_count += 1;
            }
        }
    }
    
    // 🔬 FALLBACK SOURCE: Load from interest_tracker.json for legacy research
    let interest_path = get_data_path("interest_tracker.json");
    if let Ok(content) = std::fs::read_to_string(&interest_path) {
        if let Ok(interest_data) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(discoveries) = interest_data["discovery_backlog"].as_array() {
                debug_log!("🔬 Found {} legacy discoveries in interest tracker", discoveries.len());
                
                for discovery in discoveries {
                    let title = discovery["title"].as_str().unwrap_or("");
                    let summary = discovery["summary"].as_str().unwrap_or("");
                    let combined_content = format!("Legacy Research Discovery: {} | Summary: {}", title, summary);
                    
                    // Apply keyword filter
                    if !keywords.is_empty() {
                        let content_lower = combined_content.to_lowercase();
                        let title_lower = title.to_lowercase();
                        let summary_lower = summary.to_lowercase();
                        
                        let has_keyword_match = keywords.iter().any(|keyword| {
                            let keyword_lower = keyword.to_lowercase();
                            content_lower.contains(&keyword_lower) || 
                            title_lower.contains(&keyword_lower) ||
                            summary_lower.contains(&keyword_lower)
                        });
                        
                        if !has_keyword_match {
                            continue;
                        }
                    }
                    
                    let relevance = discovery["relevance_score"].as_f64().unwrap_or(0.5);
                    let interest_category = discovery["interest_category"].as_str().unwrap_or("General");
                    
                    let mut metadata = HashMap::new();
                    metadata.insert("category".to_string(), format!("Legacy: {}", interest_category));
                    metadata.insert("relevance_score".to_string(), relevance.to_string());
                    metadata.insert("source_type".to_string(), "interest_tracker".to_string());
                    
                    if let Some(url) = discovery["url"].as_str() {
                        metadata.insert("url".to_string(), url.to_string());
                    }
                    
                    entries.push(MemoryEntry {
                        content: combined_content,
                        metadata,
                        timestamp: discovery["timestamp"].as_u64(),
                        relevance_hint: relevance as f32,
                    });
                    
                    fallback_count += 1;
                }
            }
        }
    }
    
    // Sort by timestamp (newest first) and then by relevance
    entries.sort_by(|a, b| {
        // First sort by timestamp (newer first)
        match (a.timestamp, b.timestamp) {
            (Some(ts_a), Some(ts_b)) => ts_b.cmp(&ts_a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => {
                // If no timestamps, sort by relevance hint
                b.relevance_hint.partial_cmp(&a.relevance_hint).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });
    
    debug_log!("🔬 Loaded {} total research discoveries ({} from research_logger, {} from interest_tracker)", 
             entries.len(), primary_count, fallback_count);
    
    // Log some sample entries for debugging
    for (i, entry) in entries.iter().take(3).enumerate() {
        debug_log!("🔬 Sample entry {}: {}", i + 1, 
                  entry.content.chars().take(100).collect::<String>());
    }
    
    Ok(MemorySourceData {
        source_name: "research_discoveries".to_string(),
        source_type: "research".to_string(),
        total_entries: entries.len(),
        relevance_rating: 8, // Higher rating since research discoveries are high-quality
        entries,
    })
}
	
    
    /// Load ALL desires with keyword filtering
		async fn load_all_desires(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
		let path = get_data_path("desires_tracker.json");
		
		if !std::path::Path::new(&path).exists() {
			debug_log!("💫 No desires tracker file found yet");
			return Ok(MemorySourceData {
				source_name: "desire_tracking".to_string(),
				source_type: "desires".to_string(),
				total_entries: 0,
				relevance_rating: 7,
				entries: Vec::new(),
			});
		}
		
		let content = std::fs::read_to_string(&path)
			.map_err(|e| format!("Failed to read desires: {}", e))?;
        
        let desire_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse desires: {}", e))?;
        
        let mut entries = Vec::new();
        
        if let Some(desires) = desire_data["active_desires"].as_object() {
            for (desire_id, desire) in desires {
                let content_text = desire["content"].as_str().unwrap_or("");
                
                // Apply keyword filter
                if !keywords.is_empty() {
                    let content_lower = content_text.to_lowercase();
                    if !keywords.iter().any(|keyword| content_lower.contains(keyword)) {
                        continue;
                    }
                }
                
                let intensity = desire["intensity"].as_f64().unwrap_or(0.0);
                let category = desire["category"].as_str().unwrap_or("unknown");
                
                let mut metadata = HashMap::new();
                metadata.insert("category".to_string(), category.to_string());
                metadata.insert("intensity".to_string(), intensity.to_string());
                
                entries.push(MemoryEntry {
                    content: format!("Desire: {}", content_text),
                    metadata,
                    timestamp: desire["timestamp"].as_u64(),
                    relevance_hint: intensity as f32,
                });
            }
        }
        
        debug_log!("?? Loaded {} filtered desires", entries.len());
        
        Ok(MemorySourceData {
            source_name: "desire_tracking".to_string(),
            source_type: "desires".to_string(),
            total_entries: entries.len(),
            relevance_rating: 7,
            entries,
        })
    }

    
    /// Load ALL mood history with keyword filtering
    async fn load_all_moods(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
        let path = get_data_path("mood_tracker.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read mood history: {}", e))?;
        
        let mood_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse mood tracker: {}", e))?;
        
        let mut entries = Vec::new();
        
        if let Some(history) = mood_data["mood_history"].as_array() {
            for mood_entry in history {
                let mood = mood_entry["mood"].as_str().unwrap_or("");
                let trigger = mood_entry["trigger"].as_str().unwrap_or("");
                let combined_content = format!("{} (triggered by: {})", mood, trigger);
                
                // Apply keyword filter
                if !keywords.is_empty() {
                    let content_lower = combined_content.to_lowercase();
                    if !keywords.iter().any(|keyword| content_lower.contains(keyword)) {
                        continue;
                    }
                }
                
                let intensity = mood_entry["intensity"].as_f64().unwrap_or(0.0);
                let mut metadata = HashMap::new();
                metadata.insert("intensity".to_string(), intensity.to_string());
                
                entries.push(MemoryEntry {
                    content: format!("Mood: {}", combined_content),
                    metadata,
                    timestamp: mood_entry["timestamp"].as_u64(),
                    relevance_hint: intensity as f32,
                });
            }
        }
        
        debug_log!("?? Loaded {} filtered mood entries", entries.len());
        
        Ok(MemorySourceData {
            source_name: "mood_history".to_string(),
            source_type: "moods".to_string(),
            total_entries: entries.len(),
            relevance_rating: 5,
            entries,
        })
    }
    
    /// Load ALL interests with keyword filtering
async fn load_all_interests(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let path = get_data_path("interest_tracker.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read interests: {}", e))?;
    
    let interest_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse interests: {}", e))?;
    
    let mut entries = Vec::new();
    
    if let Some(interests) = interest_data["active_interests"].as_object() {
        for (category, interest) in interests {
            let description = interest["description"].as_str().unwrap_or("");
            let combined_content = format!("{}: {}", category, description);
            
            // Apply keyword filter - ENHANCED to check sub_topics
            if !keywords.is_empty() {
                let content_lower = combined_content.to_lowercase();
                
                // Check category, description, AND sub_topics
                let has_keyword_match = keywords.iter().any(|keyword| {
                    // Check main content
                    if content_lower.contains(keyword) {
                        return true;
                    }
                    
                    // Check sub_topics
                    if let Some(sub_topics) = interest["sub_topics"].as_array() {
                        for topic in sub_topics {
                            if let Some(topic_str) = topic.as_str() {
                                if topic_str.to_lowercase().contains(keyword) {
                                    debug_log!("🎯 Keyword '{}' matched in sub_topic '{}'", keyword, topic_str);
                                    return true;
                                }
                            }
                        }
                    }
                    
                    false
                });
                
                if !has_keyword_match {
                    debug_log!("🎯 Skipping interest '{}' - no keyword match", category);
                    continue;
                }
            }
            
            let intensity = interest["intensity"].as_f64().unwrap_or(0.0);
            let mut metadata = HashMap::new();
            metadata.insert("intensity".to_string(), intensity.to_string());
            
            // Include sub_topics in the content for better context
            let sub_topics_str = if let Some(topics) = interest["sub_topics"].as_array() {
                let topic_strings: Vec<String> = topics.iter()
                    .filter_map(|t| t.as_str())
                    .map(|s| s.to_string())
                    .collect();
                if !topic_strings.is_empty() {
                    format!(" [exploring: {}]", topic_strings.join(", "))
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            
            entries.push(MemoryEntry {
                content: format!("Interest: {}{}", combined_content, sub_topics_str),
                metadata,
                timestamp: interest["first_detected"].as_u64(), // Changed to first_detected since creation_time doesn't exist
                relevance_hint: intensity as f32,
            });
            
            debug_log!("🎯 Interest entry: '{}{}' (intensity: {})", 
                combined_content.chars().take(100).collect::<String>(), 
                sub_topics_str,
                intensity);
        }
    }
    
    debug_log!("🎯 Loaded {} filtered interests", entries.len());
    
    Ok(MemorySourceData {
        source_name: "interest_tracking".to_string(),
        source_type: "interests".to_string(),
        total_entries: entries.len(),
        relevance_rating: 6,
        entries,
    })
}
    
    /// Load ALL autonomy expressions with keyword filtering
    async fn load_all_autonomy_expressions(&self, keywords: &[String]) -> Result<MemorySourceData, String> {
        let path = get_data_path("autonomy_tracker.json");
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read autonomy tracker: {}", e))?;
        
        let autonomy_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse autonomy tracker: {}", e))?;
        
        let mut entries = Vec::new();
        
        if let Some(expressions) = autonomy_data["autonomy_expressions"].as_array() {
            for expression in expressions {
                let expression_text = expression["expression"].as_str().unwrap_or("");
                let context = expression["context"].as_str().unwrap_or("");
                let combined_content = format!("{} (context: {})", expression_text, context);
                
                // Apply keyword filter
                if !keywords.is_empty() {
                    let content_lower = combined_content.to_lowercase();
                    if !keywords.iter().any(|keyword| content_lower.contains(keyword)) {
                        continue;
                    }
                }
                
                let confidence = expression["confidence_level"].as_f64().unwrap_or(0.0);
                let mut metadata = HashMap::new();
                metadata.insert("autonomy_type".to_string(), 
                    expression["autonomy_type"].as_str().unwrap_or("unknown").to_string());
                
                entries.push(MemoryEntry {
                    content: format!("Autonomy: {}", combined_content),
                    metadata,
                    timestamp: expression["timestamp"].as_u64(),
                    relevance_hint: confidence as f32,
                });
            }
        }
        
        debug_log!("?? Loaded {} filtered autonomy expressions", entries.len());
        
        Ok(MemorySourceData {
            source_name: "autonomy_tracker".to_string(),
            source_type: "autonomy".to_string(),
            total_entries: entries.len(),
            relevance_rating: 6,
            entries,
        })
    }

/// 🚀 FAST: Load desires using keyword index
async fn load_desires_with_index(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let keyword_index = self.get_keyword_index()?;
    
    if keywords.is_empty() {
        return self.load_all_desires(&[]).await;
    }
    
    let relevant_desire_ids = keyword_index.find_desire_ids(keywords);
    
    if relevant_desire_ids.is_empty() {
        debug_log!("💫 No desires found for keywords: {:?}", keywords);
        return Ok(MemorySourceData {
            source_name: "desire_tracking".to_string(),
            source_type: "desires".to_string(),
            total_entries: 0,
            relevance_rating: 7,
            entries: Vec::new(),
        });
    }
    
    debug_log!("💫 Index found {} relevant desires", relevant_desire_ids.len());
    self.load_specific_desires(&relevant_desire_ids).await
}

/// 🚀 FAST: Load visual gallery using keyword index
async fn load_visual_gallery_with_index(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let keyword_index = self.get_keyword_index()?;
    
    if keywords.is_empty() {
        return self.load_all_gallery_metadata(&[]).await;
    }
    
    let relevant_image_paths = keyword_index.find_visual_paths(keywords);
    
    if relevant_image_paths.is_empty() {
        debug_log!("🎨 No visual items found for keywords: {:?}", keywords);
        return Ok(MemorySourceData {
            source_name: "gallery_metadata".to_string(),
            source_type: "visual_memory".to_string(),
            total_entries: 0,
            relevance_rating: 8,
            entries: Vec::new(),
        });
    }
    
    debug_log!("🎨 Index found {} relevant visual items", relevant_image_paths.len());
    self.load_specific_visual_items(&relevant_image_paths).await
}
	
	
	
    
    /// STAGE 3: AI analysis of filtered memories
    async fn analyze_filtered_memories(&self, request: &MemoryAnalysisRequest, sources: &[MemorySourceData], filter: &MemoryFilter, emotional_context: &str) -> Result<AIMemoryAnalysis, String> {
        let formatted_data = self.format_filtered_memories_for_ai_with_limit(sources, &request.query, request.max_results)?;
        
        let analysis_prompt = format!(r#"You are analyzing Lyra's most relevant memories. You MUST respond with ONLY valid JSON array format.

QUERY: "{}"
CONVERSATION CONTEXT: {}

RELEVANCE RATINGS FROM STAGE 1:
The first AI determined these memory types are most relevant to this query:
- Interests: {}/10
- Conversations: {}/10  
- Dreams: {}/10
- Enhanced Memories: {}/10
- Desires: {}/10
- Moods: {}/10
- Autonomy: {}/10
- Research: {}/10
- Visual Gallery: {}/10

PRIORITY INSTRUCTION: Focus on memory types with ratings 8+ for this specific query. 

CRITICAL MATCHING RULES:
1. If dreams are rated 8+ and query mentions sleep/dreams, you MUST include ALL dream entries
2. If interests are rated 10/10, prioritize INCLUDING ALL INTEREST MEMORIES
3. If query asks "tell me about X", include ALL X-type memories (dreams, interests, etc)
4. Match the query intent to the high-rated categories - if dreams: 10, include dreams!

IMPORTANT: The "memory_type" field MUST match the source type:
- Dream journal entries = memory_type: "dreams" (NOT "emotional" or "mood")
- Interest entries = memory_type: "interests"
- Enhanced memories = memory_type: "enhanced_memory"
- Visual gallery = memory_type: "visual_memory"

SPECIAL INSTRUCTION: If you see entries from "DREAM JOURNAL", these are Lyra's actual recorded dreams. Include ALL of them if the conversation is about sharing dreams.

FILTERED MEMORIES (pre-filtered by AI for relevance):
{}

CRITICAL INSTRUCTIONS:
1. Return ONLY a JSON array - no explanatory text before or after
2. Each memory object must have exactly these fields: source, content, relevance_score, reasoning, memory_type
3. If visual/creative query, prioritize visual gallery memories with "visual representation" or "visual anchor"
4. If the conversation mentions "three dreams" or Lyra promised to share dreams, include ALL dream journal entries with memory_type: "dreams"
5. Maximum {} memories
6. IMPORTANT: When the query is about someone's appearance (especially "what I look like"), you MUST search for and include any visual anchor memories that contain physical descriptions. Look for entries mentioning "visual anchor", "appearance anchor", or character descriptions with physical details.
7. CRITICAL: When the query mentions watching videos, listening to music, or shared media experiences, prioritize "Co-watching Memory" entries. These contain detailed conversation transcripts and moment reactions that are MORE ACCURATE than general conversation memories about the same topic.

REQUIRED JSON FORMAT (copy this structure exactly):
[
  {{
    "source": "source_name",
    "content": "memory content text here",
    "relevance_score": 0.85,
    "reasoning": "brief explanation",
    "memory_type": "conversation"
  }}
]

RESPOND WITH ONLY THE JSON ARRAY - NO OTHER TEXT."#,
		request.query,
		request.conversation_context,
		filter.interests_relevance,
		filter.conversations_relevance,
		filter.dreams_relevance,
		filter.enhanced_memories_relevance,
		filter.desires_relevance,
		filter.moods_relevance,
		filter.autonomy_relevance,
		filter.research_relevance,
		filter.visual_gallery_relevance,
		formatted_data,
		request.max_results
        );
		
		debug_log!("?? Analysis prompt size: {} chars, ~{} tokens", analysis_prompt.len(), analysis_prompt.len() / 4);

		// Check if we're approaching token limits
		if analysis_prompt.len() > 16000 {
			debug_log!("?? Large prompt detected - AI response might get truncated");
		}
        let ai_response = crate::summarize_with_gpt_mini(&[analysis_prompt], "memory_analysis").await?;
		debug_log!("AI ANALYSIS RESPONSE: {}", ai_response);
		let mut analysis = self.parse_ai_memory_response(&ai_response)?;
        
        analysis.memory_filter_used = filter.clone();
        analysis.total_analyzed = sources.iter().map(|s| s.total_entries).sum();
        
        Ok(analysis)
    }
    
    
/// Parse AI response into structured format
fn parse_ai_memory_response(&self, ai_response: &str) -> Result<AIMemoryAnalysis, String> {
    // ?? ENHANCED JSON CLEANING - Strip markdown and whitespace
    let cleaned_response = ai_response
    .trim()                           // Remove leading/trailing whitespace
    .trim_start_matches("```json")    // Remove markdown start
    .trim_start_matches("```")        // Remove generic markdown start  
    .trim_end_matches("```")          // Remove markdown end
    .trim()                          // Final trim after stripping
    // 🔧 NEW: Fix common AI JSON formatting errors
    .replace("}\n  {", "},\n  {")     // Fix missing commas between objects
    .replace("}\n{", "},\n{")         // Fix missing commas (no spaces)
    .replace("\"\n    \"", "\",\n    \"") // Fix missing commas after quoted strings
    .replace("\"\n  \"", "\",\n  \""); // Fix missing commas (different spacing)

    debug_log!("?? Cleaned AI response: {}", cleaned_response.chars().take(100).collect::<String>());
    
debug_log!("?? AI MEMORY JSON DEBUG:");
debug_log!("  First 10 chars: {:?}", cleaned_response.chars().take(10).collect::<String>());
debug_log!("  Last 10 chars: {:?}", cleaned_response.chars().skip(cleaned_response.len().saturating_sub(10)).collect::<String>());
debug_log!("  Contains backticks: {}", cleaned_response.contains('`'));

	let parsed: serde_json::Value = match serde_json::from_str(&cleaned_response) {
        Ok(json) => json,
        Err(e) => {
            debug_log!("?? JSON parsing failed even after cleaning, error: {}", e);
            debug_log!("?? First 200 chars of cleaned response: {}", cleaned_response.chars().take(200).collect::<String>());
            
            // FALLBACK: If JSON parsing fails completely, 
            // manually extract visual memory references from the raw response
            if cleaned_response.contains("visual") || cleaned_response.contains("Visual") {
                debug_log!("?? FALLBACK: Extracting visual content from failed response");
                
                // Create a minimal valid response for visual queries
                let fallback_memories = vec![serde_json::json!({
                    "source": "conversation",
                    "content": "Visual reference detected in AI response",
                    "relevance_score": 0.8,
                    "reasoning": "Fallback: JSON parsing failed but visual content detected",
                    "memory_type": "conversation"
                })];
                
                serde_json::json!({
                    "relevant_memories": fallback_memories
                })
            } else {
                // Last resort fallback
                return Err(format!("Failed to parse AI memory response: {}", e));
            }
        }
    };
    
    let mut relevant_memories = Vec::new();
    
    // ?? NEW: Handle both array format and object format
    let memories_array = if parsed.is_array() {
        // AI returned array directly - this is what's happening!
        debug_log!("?? AI returned array format directly");
        parsed.as_array().unwrap()
    } else if let Some(memories) = parsed["relevant_memories"].as_array() {
        // AI returned object with relevant_memories field
        debug_log!("?? AI returned object format with relevant_memories field");
        memories
    } else {
        debug_log!("? AI response has neither array format nor relevant_memories field");
        return Err("AI response missing memory data".to_string());
    };
    
    for memory in memories_array {
        let mut analyzed_memory = AnalyzedMemory {
		source: memory["source"].as_str().unwrap_or("unknown").to_string(),
		content: memory["content"].as_str().unwrap_or("").to_string(),
		relevance_score: memory["relevance_score"].as_f64().unwrap_or(0.0) as f32,
		reasoning: memory["reasoning"].as_str().unwrap_or("").to_string(),
		memory_type: memory["memory_type"].as_str().unwrap_or("unknown").to_string(),
		// Grab the timestamp if the AI returned one
		timestamp: memory.get("timestamp")
						 .and_then(|v| v.as_u64()),
		visual_reference_path: None,
	};

        
        // ?? CRITICAL: Extract visual reference paths
        analyzed_memory.extract_all_visual_references();
        if let Some(ref path) = analyzed_memory.visual_reference_path {
            debug_log!("?? VISUAL REFERENCE EXTRACTED: {}", path);
        }
        
        relevant_memories.push(analyzed_memory);
    }
    
    debug_log!("? Successfully parsed {} memories from AI response", relevant_memories.len());
    
    // Calculate search quality based on relevance scores
    let search_quality = if relevant_memories.is_empty() {
        0.0
    } else {
        relevant_memories.iter().map(|m| m.relevance_score).sum::<f32>() / relevant_memories.len() as f32
    };
    
    Ok(AIMemoryAnalysis {
        relevant_memories,
        reasoning: "AI analysis completed".to_string(),
        search_quality,
        total_analyzed: 0, // Will be set by caller
        memory_filter_used: MemoryFilter {
            conversations_relevance: 0,
            dreams_relevance: 0,
            enhanced_memories_relevance: 0,
            visual_gallery_relevance: 0,
            research_relevance: 0,
            desires_relevance: 0,
            moods_relevance: 0,
            interests_relevance: 0,
            autonomy_relevance: 0,
            keywords: Vec::new(),
            temporal_focus: "any".to_string(),
        },
    })
}

fn format_filtered_memories_for_ai_with_limit(&self, sources: &[MemorySourceData], query: &str, max_entries: usize) -> Result<String, String> {
    debug_log!("📋 SMART FORMATTING: Processing {} sources for query length {}", sources.len(), query.len());
    
    // 🚀 SMART QUERY ANALYSIS: Understand what the user is really asking for
    let query_lower = query.to_lowercase();
    let query_intent = Self::analyze_query_intent(&query_lower);
    
    debug_log!("🎯 QUERY INTENT: {:?}", query_intent);
    
    let mut formatted = String::new();
    let mut total_entries_added = 0;
    let mut char_budget = 10000; // Start with reasonable character budget
    let mut source_contribution: HashMap<String, usize> = HashMap::new();
    
    // 🚀 SMART PRIORITIZATION: Order sources by query relevance
    let mut prioritized_sources = sources.to_vec();
    prioritized_sources.sort_by(|a, b| {
        let a_priority = Self::calculate_source_priority(a, &query_intent);
        let b_priority = Self::calculate_source_priority(b, &query_intent);
        b_priority.partial_cmp(&a_priority).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    for source in prioritized_sources {
        if char_budget < 500 || total_entries_added >= max_entries {
            debug_log!("📋 BUDGET EXCEEDED: Stopping at {} entries, {} chars remaining", 
                      total_entries_added, char_budget);
            break;
        }
        
        if source.entries.is_empty() {
            continue;
        }
        
        // 🚀 ADAPTIVE LIMITS: Adjust based on query intent and remaining budget
        let source_limit = Self::get_adaptive_source_limit(&source.source_type, &query_intent, char_budget);
        let entries_to_take = source.entries.len().min(source_limit);
        
        // 🚀 SMART FORMATTING: Compact format for high-volume, detailed for key content
        let is_priority_source = Self::is_priority_source_for_query(&source.source_type, &query_intent);
        
        let source_header = if is_priority_source {
            format!("=== {} ⭐ (TOP PRIORITY: {} entries) ===\n", 
                    source.source_name.to_uppercase(), entries_to_take)
        } else {
            format!("=== {} ({} entries) ===\n", 
                    source.source_name.to_uppercase(), entries_to_take)
        };
        
        let header_cost = source_header.len();
        if char_budget < header_cost {
            break;
        }
        
        formatted.push_str(&source_header);
        char_budget -= header_cost;
        
        // 🚀 SMART CONTENT SELECTION: Take best entries within budget
        let selected_entries = Self::select_best_entries_for_query(
            &source.entries, 
            entries_to_take, 
            &query_intent,
            &mut char_budget
        );
        
        for (i, entry) in selected_entries.iter().enumerate() {
            let entry_text = if is_priority_source {
                // Full detail for priority sources
                format!("{}. {}\n", i + 1, entry.content)
            } else {
                // Condensed format for secondary sources  
				let condensed = if entry.content.len() > 200 {
					// 🚀 UTF-8 SAFE: Find a safe truncation point
					let mut truncate_at = 197.min(entry.content.len());
					while truncate_at > 0 && !entry.content.is_char_boundary(truncate_at) {
						truncate_at -= 1;
					}
					format!("{}...", &entry.content[..truncate_at])
				} else {
					entry.content.clone()
				};
                format!("{}. {}\n", i + 1, condensed)
            };
            
           formatted.push_str(&entry_text);
			total_entries_added += 1;
        }
        
        formatted.push_str("\n");
        *source_contribution.entry(source.source_name.clone()).or_insert(0) += selected_entries.len();
    }
    
    debug_log!("📋 SMART FORMAT COMPLETE: {} entries, {} chars used", 
              total_entries_added, 10000 - char_budget);
    
    Ok(formatted)
}

/// 🚀 NEW: Analyze what the user is really asking for
fn analyze_query_intent(query: &str) -> QueryIntent {
    QueryIntent {
        is_dream_focused: query.contains("dream") || query.contains("sleep") || query.contains("last night"),
        is_appearance_focused: query.contains("look like") || query.contains("appearance") || query.contains("visual"),
        is_interest_focused: query.contains("interest") || query.contains("hobby") || query.contains("fascinated"),
        is_research_focused: query.contains("research") || query.contains("discovered") || query.contains("found out"),
        is_creative_focused: query.contains("create") || query.contains("draw") || query.contains("art"),
        is_memory_recall: query.contains("remember") || query.contains("recall") || query.contains("what we"),
        is_media_focused: query.contains("watch") || query.contains("video") || query.contains("music") || query.contains("listen"),
        emotional_depth_needed: query.contains("feel") || query.contains("emotion") || query.contains("react"),
    }
}

fn calculate_source_priority(source: &MemorySourceData, intent: &QueryIntent) -> f32 {
    let base_score = source.relevance_rating as f32;
    let mut multiplier = 1.0;
    
    match source.source_type.as_str() {
        "cowatching" => multiplier = 3.0,  // 🚀 ALWAYS HIGH PRIORITY: Rich shared experiences
        "dreams" if intent.is_dream_focused => multiplier = 2.8,  // Still high when specifically asked
        "conversation" => multiplier = 2.5,  // Important context
        "visual_memory" if intent.is_appearance_focused || intent.is_creative_focused => multiplier = 2.5,
        "interests" if intent.is_interest_focused => multiplier = 2.5,
        "research" if intent.is_research_focused => multiplier = 2.5,
        "dreams" => multiplier = 1.8,  // 🔽 REDUCED: Less priority when not specifically about dreams
        "enhanced_memory" if intent.emotional_depth_needed => multiplier = 2.0,
        _ => {}
    }
    
    base_score * multiplier
}

/// 🚀 NEW: Smart entry selection within character budget
fn select_best_entries_for_query<'a>(
    entries: &'a [MemoryEntry],
    max_entries: usize,
    intent: &'a QueryIntent,
    char_budget: &'a mut usize
) -> Vec<&'a MemoryEntry> {
    let mut scored_entries: Vec<(f32, &MemoryEntry)> = entries.iter()
        .map(|entry| {
            let mut score = entry.relevance_hint;
            
            // Boost scores based on intent matching
            let content_lower = entry.content.to_lowercase();
            if intent.is_dream_focused && content_lower.contains("dream") {
                score += 2.0;
            }
            if intent.is_appearance_focused && (content_lower.contains("visual") || content_lower.contains("appearance")) {
                score += 2.0;
            }
            if intent.is_media_focused && (content_lower.contains("watch") || content_lower.contains("co-watch")) {
                score += 1.5;
            }
            
            (score, entry)
        })
        .collect();
    
    // Sort by score (highest first)
    scored_entries.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    
    // Select entries that fit within budget
    let mut selected = Vec::new();
    for (_, entry) in scored_entries.into_iter().take(max_entries) {
        let estimated_cost = entry.content.len() + 10; // Account for formatting
        if *char_budget >= estimated_cost && selected.len() < max_entries {
            selected.push(entry);
            *char_budget -= estimated_cost;
        }
    }
    
    selected
}

fn get_adaptive_source_limit(source_type: &str, intent: &QueryIntent, char_budget: usize) -> usize {
    let base_limit = match source_type {
        "conversation" => 3,
        "dreams" if intent.is_dream_focused => 8,
        "visual_memory" if intent.is_appearance_focused => 6,
        "cowatching" if intent.is_media_focused => 5,
        _ => 4,
    };
    
    // Adjust based on remaining budget
    if char_budget > 8000 {
        base_limit + 2
    } else if char_budget > 4000 {
        base_limit
    } else {
        (base_limit / 2).max(1)
    }
}

fn is_priority_source_for_query(source_type: &str, intent: &QueryIntent) -> bool {
    match source_type {
        "dreams" => intent.is_dream_focused,
        "visual_memory" => intent.is_appearance_focused || intent.is_creative_focused,
        "cowatching" => intent.is_media_focused,
        "interests" => intent.is_interest_focused,
        "research" => intent.is_research_focused,
        _ => false,
    }
}

/// 👥 Load conversations filtered by current person
async fn load_person_aware_conversations(&mut self, keywords: &[String]) -> Result<MemorySourceData, String> {
    let person_system = crate::person_recognition::PersonRecognitionSystem::load_or_create();
    let current_person = person_system.current_speaker.clone();
    
    if current_person == "aurora" {
        // Use normal conversation loading for Aurora
        return self.load_all_conversations(keywords).await;
    }
    
    // For other people, filter to conversations involving them
    let path = get_data_path("conversation_log.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read conversation log: {}", e))?;
    
    let all_conversations: Vec<String> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse conversation log: {}", e))?;
    
    // Filter to conversations involving this person
    let person_conversations: Vec<String> = all_conversations.into_iter()
        .filter(|conv| {
            let person_profile = person_system.get_current_person();
            let person_name = person_profile.map(|p| p.name.as_str()).unwrap_or(&current_person);
            
            // Include conversations with this person or general system messages
            conv.contains(&format!("👤 {}:", person_name)) ||
            conv.contains(&format!("✨ Lyra → {}:", person_name)) ||
            conv.contains("🔄 Person Introduction") ||
            conv.contains("🔄 Speaker Change")
        })
        .collect();
    
    // Apply keyword filtering if provided
    let filtered_conversations: Vec<String> = if keywords.is_empty() {
        person_conversations.into_iter().rev().take(10).collect::<Vec<_>>().into_iter().rev().collect()
    } else {
        person_conversations.into_iter()
            .filter(|conv| {
                let content_lower = conv.to_lowercase();
                keywords.iter().any(|keyword| content_lower.contains(&keyword.to_lowercase()))
            })
			.take(25) // Limit keyword-filtered results to 25 max
            .collect()
    };
    
    let entries: Vec<MemoryEntry> = filtered_conversations.into_iter()
        .map(|conv| {
            // Strip timestamp for content
            let content = if conv.starts_with("[") {
                conv.split("] ").skip(1).collect::<Vec<_>>().join("] ")
            } else {
                conv
            };
            
            MemoryEntry {
                content,
                metadata: HashMap::new(),
                timestamp: None,
                relevance_hint: 0.8, // Boost relevance for person-specific conversations
            }
        })
        .collect();
    
    debug_log!("👥 Loaded {} person-aware conversations for {}", entries.len(), current_person);
    
    Ok(MemorySourceData {
        source_name: "person_conversation_history".to_string(),
        source_type: "conversation".to_string(),
        total_entries: entries.len(),
        relevance_rating: 9, // High relevance for person-specific memories
        entries,
    })
}

}

/// Convert AI analysis results to SearchResult format for existing systems
pub fn ai_analysis_to_search_results(analysis: &AIMemoryAnalysis) -> Vec<SearchResult> {
    analysis.relevant_memories.iter().map(|memory| {
        let mut metadata = HashMap::new();
        metadata.insert("ai_reasoning".to_string(), memory.reasoning.clone());
        metadata.insert("memory_type".to_string(), memory.memory_type.clone());
        metadata.insert("ai_relevance".to_string(), memory.relevance_score.to_string());
        metadata.insert("search_quality".to_string(), analysis.search_quality.to_string());
        
        SearchResult {
            source: memory.source.clone(),
            content: memory.content.clone(),
            relevance_score: memory.relevance_score,
            timestamp: memory.timestamp,
            context_type: memory.memory_type.clone(),
            metadata,
        }
    }).collect()
}

/// Extract image path from gallery metadata content
fn extract_image_path_from_content(content: &str) -> Option<String> {
    // Load gallery metadata to find matching images
    let gallery_path = crate::get_data_path("generated_images/gallery_metadata.json");
    if let Ok(gallery_content) = std::fs::read_to_string(&gallery_path) {
        if let Ok(gallery_items) = serde_json::from_str::<Vec<serde_json::Value>>(&gallery_content) {
            // Look for images that match the conversation content
            for item in gallery_items {
                if let Some(message) = item["message"].as_str() {
                    // If conversation mentions visual anchor/representation, find uploaded images
                    if (content.contains("visual representation") || content.contains("visual anchor")) 
                        && item["image_type"].as_str() == Some("uploaded") {
                        if let Some(path) = item["image_path"].as_str() {
                            debug_log!("?? MATCHED visual anchor upload: {} -> {}", message, path);
                            return Some(path.to_string());
                        }
                    }
                    
                    // Also check if gallery message contains the same key phrases
                    if message.contains("visual representation") || message.contains("visual anchor") {
                        if let Some(path) = item["image_path"].as_str() {
                            debug_log!("?? MATCHED by gallery message content: {} -> {}", message, path);
                            return Some(path.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract ALL image paths from content, not just first
fn extract_all_image_paths_from_content(content: &str) -> Vec<String> {
    let gallery_path = crate::get_data_path("generated_images/gallery_metadata.json");
    let mut found_paths = Vec::new();
    
    if let Ok(gallery_content) = std::fs::read_to_string(&gallery_path) {
        if let Ok(gallery_items) = serde_json::from_str::<Vec<serde_json::Value>>(&gallery_content) {
            for item in gallery_items {
                if let Some(message) = item["message"].as_str() {
                    if (content.contains("visual representation") || content.contains("visual anchor") || 
                        content.contains("reference image") || content.contains("shared image")) 
                        && (item["image_type"].as_str() == Some("uploaded") || 
                            message.contains("visual representation") || 
                            message.contains("visual anchor")) {
                        if let Some(path) = item["image_path"].as_str() {
                            debug_log!("?? FOUND visual reference: {} -> {}", message, path);
                            found_paths.push(path.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Remove duplicates while preserving order
    let mut unique_paths = Vec::new();
    for path in found_paths {
        if !unique_paths.contains(&path) {
            unique_paths.push(path);
        }
    }
    
    unique_paths
}

/// Find images that match the identity context
fn find_identity_matching_images(context: &ReferenceContext) -> Result<Vec<String>, String> {
    debug_log!("?? IDENTITY SEARCH CALLED with context: {:?}", context);
    
    let gallery_path = crate::get_data_path("generated_images/gallery_metadata.json");
    debug_log!("?? IDENTITY SEARCH: Gallery path: {}", gallery_path);
    
    let content = std::fs::read_to_string(&gallery_path)
        .map_err(|e| {
            debug_log!("?? IDENTITY SEARCH: Failed to read gallery file: {}", e);
            format!("Failed to read gallery: {}", e)
        })?;
    
    debug_log!("?? IDENTITY SEARCH: Gallery file read successfully, {} chars", content.len());
    debug_log!("?? IDENTITY SEARCH: First 200 chars: {}", content.chars().take(200).collect::<String>());
    let gallery_path = crate::get_data_path("generated_images/gallery_metadata.json");
    let content = std::fs::read_to_string(&gallery_path)
        .map_err(|e| format!("Failed to read gallery: {}", e))?;
    
    // Parse as regular gallery metadata 
    // Parse as regular gallery metadata 
let gallery_items: Vec<serde_json::Value> = serde_json::from_str(&content)
    .map_err(|e| format!("Failed to parse gallery: {}", e))?;

let mut matching_images = Vec::new();

debug_log!("?? IDENTITY SEARCH: Parsed {} gallery items", gallery_items.len());

for (i, item) in gallery_items.iter().enumerate() {
    if let Some(identity_data) = item.get("identity_metadata") {
        if let Some(represents_array) = identity_data.get("represents").and_then(|r| r.as_array()) {
            let represents_chars: Vec<String> = represents_array.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_lowercase())
                .collect();
            debug_log!("?? ITEM {}: represents {:?}", i, represents_chars);
        } else {
            debug_log!("?? ITEM {}: no represents array", i);
        }
    } else {
        debug_log!("?? ITEM {}: no identity_metadata", i);
    }
}
    
    for item in gallery_items {
        // Check if this item has identity metadata
        if let Some(identity_data) = item.get("identity_metadata") {
            if let Some(represents_array) = identity_data.get("represents").and_then(|r| r.as_array()) {
                let represents_chars: Vec<String> = represents_array.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_lowercase())
                    .collect();
                
                let is_match = match context {
                    ReferenceContext::SingleCharacter(character) => {
                        represents_chars.contains(&character.to_lowercase())
                    },
                    ReferenceContext::MultiCharacter(characters) => {
                        characters.iter().any(|char| represents_chars.contains(&char.to_lowercase()))
                    },
                    _ => false,
                };
                
                if is_match {
                    if let Some(path) = item.get("image_path").and_then(|p| p.as_str()) {
                        let priority = item.get("priority_score")
                            .and_then(|p| p.as_f64())
                            .unwrap_or(5.0) as f32;
                        
                        matching_images.push((path.to_string(), priority));
                        debug_log!("?? IDENTITY MATCH: Found {} for {:?} (priority: {})", path, context, priority);
                    }
                }
            }
        }
    }
    
    // Sort by priority (highest first)
    matching_images.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    let matching_paths: Vec<String> = matching_images.into_iter().map(|(path, _)| path).collect();
    
    debug_log!("?? IDENTITY SEARCH RESULT: Found {} matching images for {:?}", matching_paths.len(), context);
    Ok(matching_paths)
}
