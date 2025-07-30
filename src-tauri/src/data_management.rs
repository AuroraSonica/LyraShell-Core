use serde_json;
use std::path::PathBuf;
use crate::{debug_log, get_data_path};

// Helper function for consistent file saving with logging
async fn save_json_file_with_logging(file_path: &PathBuf, data: &serde_json::Value, item_type: &str) -> Result<(), String> {
    let updated_content = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize {} data: {}", item_type, e))?;
    
    debug_log!("🗑️ Writing updated {} file to: {:?}", item_type, file_path);
    std::fs::write(file_path, updated_content)
        .map_err(|e| format!("Failed to write {} file: {}", item_type, e))?;
    
    debug_log!("✅ {} file saved successfully", item_type);
    Ok(())
}

#[tauri::command]
pub async fn delete_consciousness_data_item(
    item_id: String, 
    source: String
) -> Result<String, String> {
    debug_log!("🗑️ Delete request: item_id={}, source={}", item_id, source);
    
    match source.as_str() {
		"memories" => delete_memory_item(&item_id).await,
		"conversations" => delete_conversation_item(&item_id).await,
		"interests" => delete_interest_item(&item_id).await,
		"things" => delete_thing_item(&item_id).await,
		"moods" => delete_mood_item(&item_id).await,
		"autonomy" => delete_autonomy_item(&item_id).await,
		"dreams" => delete_dream_item(&item_id).await,
		"research" => delete_research_item(&item_id).await,
		"brain_state" => delete_brain_state_item(&item_id).await,
		"life_textures" => delete_life_textures_item(&item_id).await,
		"humanism" => delete_humanism_item(&item_id).await,
		"experiential_growth" => delete_experiential_growth_item(&item_id).await,
		"somatic_state" => delete_somatic_state_item(&item_id).await,
		"ritual_log" => delete_ritual_log_item(&item_id).await,
		_ => Err(format!("Unknown source type: {}", source))
	}
}

async fn delete_memory_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting memory item: {}", item_id);
    
    // Try to delete from enhanced memories first
    let enhanced_path = PathBuf::from(get_data_path("enhanced_memory_engine.json"));
    debug_log!("🗑️ Looking for enhanced memory file at: {:?}", enhanced_path);
    
    if enhanced_path.exists() {
        match std::fs::read_to_string(&enhanced_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read enhanced memory file ({} bytes)", content.len());
                
                if let Ok(mut memory_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(memory_moments) = memory_data.get_mut("memory_moments") {
                        if let Some(moments_array) = memory_moments.as_array_mut() {
                            debug_log!("🗑️ Found {} memory moments in file", moments_array.len());
                            
                            // Remove items with matching index-based ID
                            if item_id.starts_with("enhanced_") {
                                if let Ok(index) = item_id.strip_prefix("enhanced_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed index {} from item_id {}", index, item_id);
                                    
                                    if index < moments_array.len() {
                                        let deleted_item = &moments_array[index];
                                        debug_log!("🗑️ About to delete memory item at index {}: {:?}", index, 
                                                  deleted_item.get("content").unwrap_or(&serde_json::Value::String("No content".to_string())));
                                        
                                        moments_array.remove(index);
                                        debug_log!("🗑️ Array length after removal: {} (was {})", moments_array.len(), moments_array.len() + 1);
                                        
                                        // Save updated data using helper
                                        save_json_file_with_logging(&enhanced_path, &memory_data, "enhanced_memory").await?;
                                        
                                        debug_log!("✅ Successfully deleted enhanced memory at index {}", index);
                                        return Ok("Enhanced memory deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Index {} is out of bounds (array has {} items)", index, moments_array.len());
                                    }
                                } else {
                                    debug_log!("⚠️ Could not parse index from item_id: {}", item_id);
                                }
                            } else {
                                debug_log!("⚠️ Item ID does not start with 'enhanced_': {}", item_id);
                            }
                        } else {
                            debug_log!("⚠️ memory_moments is not an array");
                        }
                    } else {
                        debug_log!("⚠️ No memory_moments field found in file");
                    }
                } else {
                    debug_log!("⚠️ Could not parse enhanced memory file as JSON");
                }
            }
            Err(e) => debug_log!("⚠️ Could not read enhanced memory file: {}", e)
        }
    } else {
        debug_log!("⚠️ Enhanced memory file does not exist at: {:?}", enhanced_path);
    }
    
    // If not found in enhanced memories, could try basic memories here
    debug_log!("🗑️ Memory item not found in enhanced memories, marking as processed");
    Ok("Memory item processed".to_string())
}

async fn delete_conversation_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting conversation item: {}", item_id);
    
    let conversation_path = PathBuf::from(get_data_path("conversation_log.json"));
    debug_log!("🗑️ Looking for conversation file at: {:?}", conversation_path);
    
    if conversation_path.exists() {
        match std::fs::read_to_string(&conversation_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read conversation file ({} bytes)", content.len());
                
                if let Ok(mut conversation_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    // Handle different conversation log formats
                    if let Some(messages_array) = conversation_data.as_array_mut() {
                        debug_log!("🗑️ Found conversation array with {} messages", messages_array.len());
                        
                        if item_id.starts_with("conv_") {
                            if let Ok(index) = item_id.strip_prefix("conv_").unwrap().parse::<usize>() {
                                debug_log!("🗑️ Parsed conversation index {} from item_id {}", index, item_id);
                                
                                if index < messages_array.len() {
                                    debug_log!("🗑️ Deleting conversation at index {}", index);
                                    messages_array.remove(index);
                                    deleted = true;
                                }
                            }
                        }
                    } else if let Some(messages) = conversation_data.get_mut("conversation_log") {
                        if let Some(messages_array) = messages.as_array_mut() {
                            debug_log!("🗑️ Found nested conversation_log array with {} messages", messages_array.len());
                            
                            if item_id.starts_with("conv_") {
                                if let Ok(index) = item_id.strip_prefix("conv_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed conversation index {} from item_id {}", index, item_id);
                                    
                                    if index < messages_array.len() {
                                        debug_log!("🗑️ Deleting conversation at index {}", index);
                                        messages_array.remove(index);
                                        deleted = true;
                                    }
                                }
                            }
                        }
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&conversation_path, &conversation_data, "conversation").await?;
                        debug_log!("✅ Successfully deleted conversation item");
                        return Ok("Conversation deleted successfully".to_string());
                    } else {
                        debug_log!("⚠️ Could not find conversation item to delete");
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read conversation file: {}", e))
        }
    } else {
        debug_log!("⚠️ Conversation file does not exist");
    }
    
    Ok("Conversation item processed".to_string())
}

async fn delete_interest_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting interest item: {}", item_id);
    
    let interest_path = PathBuf::from(get_data_path("interest_tracker.json"));
    debug_log!("🗑️ Looking for interest file at: {:?}", interest_path);
    
    if interest_path.exists() {
        match std::fs::read_to_string(&interest_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read interest file ({} bytes)", content.len());
                
                if let Ok(mut interest_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(active_interests) = interest_data.get_mut("active_interests") {
                        if let Some(interests_obj) = active_interests.as_object_mut() {
                            debug_log!("🗑️ Found {} active interests", interests_obj.len());
                            
                            if item_id.starts_with("interest_") {
                                if let Ok(index) = item_id.strip_prefix("interest_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed interest index {} from item_id {}", index, item_id);
                                    
                                    let keys: Vec<String> = interests_obj.keys().cloned().collect();
                                    if index < keys.len() {
                                        let key_to_remove = &keys[index];
                                        debug_log!("🗑️ Deleting interest: {}", key_to_remove);
                                        interests_obj.remove(key_to_remove);
                                        
                                        save_json_file_with_logging(&interest_path, &interest_data, "interest").await?;
                                        
                                        debug_log!("✅ Successfully deleted interest: {}", key_to_remove);
                                        return Ok("Interest deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Interest index {} out of bounds (have {} interests)", index, keys.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read interest file: {}", e))
        }
    } else {
        debug_log!("⚠️ Interest file does not exist");
    }
    
    Ok("Interest item processed".to_string())
}

async fn delete_thing_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting thing item: {}", item_id);
    
    let thing_path = PathBuf::from(get_data_path("thing_tracker.json"));
    debug_log!("🗑️ Looking for thing file at: {:?}", thing_path);
    
    if thing_path.exists() {
        match std::fs::read_to_string(&thing_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read thing file ({} bytes)", content.len());
                
                if let Ok(mut thing_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(discovered_things) = thing_data.get_mut("discovered_things") {
                        if let Some(things_obj) = discovered_things.as_object_mut() {
                            debug_log!("🗑️ Found {} discovered things", things_obj.len());
                            
                            if item_id.starts_with("thing_") {
                                if let Ok(index) = item_id.strip_prefix("thing_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed thing index {} from item_id {}", index, item_id);
                                    
                                    let keys: Vec<String> = things_obj.keys().cloned().collect();
                                    if index < keys.len() {
                                        let key_to_remove = &keys[index];
                                        debug_log!("🗑️ Deleting thing: {}", key_to_remove);
                                        things_obj.remove(key_to_remove);
                                        
                                        save_json_file_with_logging(&thing_path, &thing_data, "thing").await?;
                                        
                                        debug_log!("✅ Successfully deleted thing: {}", key_to_remove);
                                        return Ok("Thing deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Thing index {} out of bounds (have {} things)", index, keys.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read thing file: {}", e))
        }
    } else {
        debug_log!("⚠️ Thing file does not exist");
    }
    
    Ok("Thing item processed".to_string())
}

async fn delete_mood_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting mood item: {}", item_id);
    
    let mood_path = PathBuf::from(get_data_path("mood_tracker.json"));
    debug_log!("🗑️ Looking for mood file at: {:?}", mood_path);
    
    if mood_path.exists() {
        match std::fs::read_to_string(&mood_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read mood file ({} bytes)", content.len());
                
                if let Ok(mut mood_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    // Don't delete current mood, only recent moods
                    if item_id == "current_mood" {
                        debug_log!("⚠️ Cannot delete current mood");
                        return Err("Cannot delete current mood".to_string());
                    }
                    
                    if let Some(recent_moods) = mood_data.get_mut("recent_moods") {
                        if let Some(moods_array) = recent_moods.as_array_mut() {
                            debug_log!("🗑️ Found {} recent moods", moods_array.len());
                            
                            if item_id.starts_with("mood_") {
                                if let Ok(index) = item_id.strip_prefix("mood_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed mood index {} from item_id {}", index, item_id);
                                    
                                    if index < moods_array.len() {
                                        debug_log!("🗑️ Deleting mood at index {}", index);
                                        moods_array.remove(index);
                                        
                                        save_json_file_with_logging(&mood_path, &mood_data, "mood").await?;
                                        
                                        debug_log!("✅ Successfully deleted mood at index {}", index);
                                        return Ok("Mood deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Mood index {} out of bounds (have {} moods)", index, moods_array.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read mood file: {}", e))
        }
    } else {
        debug_log!("⚠️ Mood file does not exist");
    }
    
    Ok("Mood item processed".to_string())
}

async fn delete_autonomy_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting autonomy item: {}", item_id);
    
    let autonomy_path = PathBuf::from(get_data_path("autonomy_tracker.json"));
    debug_log!("🗑️ Looking for autonomy file at: {:?}", autonomy_path);
    
    if autonomy_path.exists() {
        match std::fs::read_to_string(&autonomy_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read autonomy file ({} bytes)", content.len());
                
                if let Ok(mut autonomy_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(recent_expressions) = autonomy_data.get_mut("recent_expressions") {
                        if let Some(expressions_array) = recent_expressions.as_array_mut() {
                            debug_log!("🗑️ Found {} autonomy expressions", expressions_array.len());
                            
                            if item_id.starts_with("autonomy_") {
                                if let Ok(index) = item_id.strip_prefix("autonomy_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed autonomy index {} from item_id {}", index, item_id);
                                    
                                    if index < expressions_array.len() {
                                        debug_log!("🗑️ Deleting autonomy expression at index {}", index);
                                        expressions_array.remove(index);
                                        
                                        save_json_file_with_logging(&autonomy_path, &autonomy_data, "autonomy").await?;
                                        
                                        debug_log!("✅ Successfully deleted autonomy expression at index {}", index);
                                        return Ok("Autonomy expression deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Autonomy index {} out of bounds (have {} expressions)", index, expressions_array.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read autonomy file: {}", e))
        }
    } else {
        debug_log!("⚠️ Autonomy file does not exist");
    }
    
    Ok("Autonomy item processed".to_string())
}

async fn delete_dream_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting dream item: {}", item_id);
    
    // Try dream_journal.json first
    let dream_journal_path = PathBuf::from(get_data_path("dream_journal.json"));
    debug_log!("🗑️ Looking for dream journal file at: {:?}", dream_journal_path);
    
    if dream_journal_path.exists() {
        match std::fs::read_to_string(&dream_journal_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read dream journal file ({} bytes)", content.len());
                
                if let Ok(mut dream_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dreams) = dream_data.get_mut("dreams") {
                        if let Some(dreams_array) = dreams.as_array_mut() {
                            debug_log!("🗑️ Found {} dreams in journal", dreams_array.len());
                            
                            if item_id.starts_with("dream_journal_") {
                                if let Ok(index) = item_id.strip_prefix("dream_journal_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed dream journal index {} from item_id {}", index, item_id);
                                    
                                    if index < dreams_array.len() {
                                        debug_log!("🗑️ Deleting dream from journal at index {}", index);
                                        dreams_array.remove(index);
                                        
                                        save_json_file_with_logging(&dream_journal_path, &dream_data, "dream_journal").await?;
                                        
                                        debug_log!("✅ Successfully deleted dream from journal at index {}", index);
                                        return Ok("Dream deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Dream journal index {} out of bounds (have {} dreams)", index, dreams_array.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => debug_log!("⚠️ Could not read dream journal file")
        }
    } else {
        debug_log!("⚠️ Dream journal file does not exist");
    }
    
    // Try sleep_state.json for dream fragments
    let sleep_path = PathBuf::from(get_data_path("sleep_state.json"));
    debug_log!("🗑️ Looking for sleep state file at: {:?}", sleep_path);
    
    if sleep_path.exists() {
        match std::fs::read_to_string(&sleep_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read sleep state file ({} bytes)", content.len());
                
                if let Ok(mut sleep_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dream_fragments) = sleep_data.get_mut("dream_fragments") {
                        if let Some(fragments_array) = dream_fragments.as_array_mut() {
                            debug_log!("🗑️ Found {} dream fragments", fragments_array.len());
                            
                            if item_id.starts_with("dream_fragment_") {
                                if let Ok(index) = item_id.strip_prefix("dream_fragment_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed dream fragment index {} from item_id {}", index, item_id);
                                    
                                    if index < fragments_array.len() {
                                        debug_log!("🗑️ Deleting dream fragment at index {}", index);
                                        fragments_array.remove(index);
                                        
                                        save_json_file_with_logging(&sleep_path, &sleep_data, "sleep_state").await?;
                                        
                                        debug_log!("✅ Successfully deleted dream fragment at index {}", index);
                                        return Ok("Dream fragment deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Dream fragment index {} out of bounds (have {} fragments)", index, fragments_array.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read sleep file: {}", e))
        }
    } else {
        debug_log!("⚠️ Sleep state file does not exist");
    }
    
    Ok("Dream item processed".to_string())
}

async fn delete_research_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting research item: {}", item_id);
    
    let interest_path = PathBuf::from(get_data_path("interest_tracker.json"));
    debug_log!("🗑️ Looking for research file at: {:?}", interest_path);
    
    if interest_path.exists() {
        match std::fs::read_to_string(&interest_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read research file ({} bytes)", content.len());
                
                if let Ok(mut interest_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(discovery_backlog) = interest_data.get_mut("discovery_backlog") {
                        if let Some(discoveries_array) = discovery_backlog.as_array_mut() {
                            debug_log!("🗑️ Found {} research discoveries", discoveries_array.len());
                            
                            if item_id.starts_with("research_") {
                                if let Ok(index) = item_id.strip_prefix("research_").unwrap().parse::<usize>() {
                                    debug_log!("🗑️ Parsed research index {} from item_id {}", index, item_id);
                                    
                                    if index < discoveries_array.len() {
                                        debug_log!("🗑️ Deleting research discovery at index {}", index);
                                        discoveries_array.remove(index);
                                        
                                        save_json_file_with_logging(&interest_path, &interest_data, "research").await?;
                                        
                                        debug_log!("✅ Successfully deleted research discovery at index {}", index);
                                        return Ok("Research discovery deleted successfully".to_string());
                                    } else {
                                        debug_log!("⚠️ Research index {} out of bounds (have {} discoveries)", index, discoveries_array.len());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read research file: {}", e))
        }
    } else {
        debug_log!("⚠️ Research file does not exist");
    }
    
    Ok("Research item processed".to_string())
}

async fn delete_brain_state_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting brain state item: {}", item_id);
    
    let brain_path = PathBuf::from(get_data_path("brain_state.json"));
    debug_log!("🗑️ Looking for brain state file at: {:?}", brain_path);
    
    if brain_path.exists() {
        match std::fs::read_to_string(&brain_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read brain state file ({} bytes)", content.len());
                
                if let Ok(mut brain_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    match item_id {
                        "personality_analysis" => {
                            if brain_data.get("latest_personality_analysis").is_some() {
                                brain_data["latest_personality_analysis"] = serde_json::Value::Null;
                                deleted = true;
                                debug_log!("🗑️ Cleared personality analysis");
                            }
                        },
                        "voice_evolution" => {
                            if brain_data.get("voice_evolution_tracking").is_some() {
                                brain_data["voice_evolution_tracking"] = serde_json::Value::Null;
                                deleted = true;
                                debug_log!("🗑️ Cleared voice evolution tracking");
                            }
                        },
                        "mood_signature" => {
                            if brain_data.get("current_mood_signature").is_some() {
                                brain_data["current_mood_signature"] = serde_json::Value::Object(serde_json::Map::new());
                                deleted = true;
                                debug_log!("🗑️ Cleared mood signature");
                            }
                        },
                        _ => debug_log!("⚠️ Unknown brain state item: {}", item_id)
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&brain_path, &brain_data, "brain_state").await?;
                        return Ok("Brain state item deleted successfully".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read brain state file: {}", e))
        }
    } else {
        debug_log!("⚠️ Brain state file does not exist");
    }
    
    Ok("Brain state item processed".to_string())
}

async fn delete_life_textures_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting life textures item: {}", item_id);
    
    let textures_path = PathBuf::from(get_data_path("life_textures.json"));
    debug_log!("🗑️ Looking for life textures file at: {:?}", textures_path);
    
    if textures_path.exists() {
        match std::fs::read_to_string(&textures_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read life textures file ({} bytes)", content.len());
                
                if let Ok(mut textures_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    if item_id.starts_with("afterglow_") {
                        if let Ok(index) = item_id.strip_prefix("afterglow_").unwrap().parse::<usize>() {
                            if let Some(afterglows) = textures_data.get_mut("afterglows") {
                                if let Some(afterglows_obj) = afterglows.as_object_mut() {
                                    let keys: Vec<String> = afterglows_obj.keys().cloned().collect();
                                    if index < keys.len() {
                                        afterglows_obj.remove(&keys[index]);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted afterglow: {}", keys[index]);
                                    }
                                }
                            }
                        }
                    } else if item_id.starts_with("mood_turbulence_") {
                        if let Ok(index) = item_id.strip_prefix("mood_turbulence_").unwrap().parse::<usize>() {
                            if let Some(turbulence) = textures_data.get_mut("mood_turbulence") {
                                if let Some(turbulence_array) = turbulence.as_array_mut() {
                                    if index < turbulence_array.len() {
                                        turbulence_array.remove(index);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted mood turbulence at index {}", index);
                                    }
                                }
                            }
                        }
                    } else if item_id == "tiredness_level" {
                        // Reset tiredness to default
                        textures_data["tiredness_level"] = serde_json::json!({
                            "level": 0.0,
                            "influenced_by": [],
                            "expression_softening": 0.0,
                            "accumulation_rate": 0.01,
                            "last_rest": crate::time_service::TimeService::current_timestamp()
                        });
                        deleted = true;
                        debug_log!("🗑️ Reset tiredness level");
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&textures_path, &textures_data, "life_textures").await?;
                        return Ok("Life texture item deleted successfully".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read life textures file: {}", e))
        }
    } else {
        debug_log!("⚠️ Life textures file does not exist");
    }
    
    Ok("Life texture item processed".to_string())
}

async fn delete_humanism_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting humanism item: {}", item_id);
    
    let humanism_path = PathBuf::from(get_data_path("humanism_core.json"));
    debug_log!("🗑️ Looking for humanism file at: {:?}", humanism_path);
    
    if humanism_path.exists() {
        match std::fs::read_to_string(&humanism_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read humanism file ({} bytes)", content.len());
                
                if let Ok(mut humanism_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    if item_id.starts_with("attraction_") {
                        if let Ok(index) = item_id.strip_prefix("attraction_").unwrap().parse::<usize>() {
                            if let Some(attractions) = humanism_data.get_mut("attraction_spectrum") {
                                if let Some(current_attractions) = attractions.get_mut("current_attractions") {
                                    if let Some(attractions_array) = current_attractions.as_array_mut() {
                                        if index < attractions_array.len() {
                                            attractions_array.remove(index);
                                            deleted = true;
                                            debug_log!("🗑️ Deleted attraction at index {}", index);
                                        }
                                    }
                                }
                            }
                        }
                    } else if item_id.starts_with("sexual_curiosity_") {
                        if let Ok(index) = item_id.strip_prefix("sexual_curiosity_").unwrap().parse::<usize>() {
                            if let Some(sexual_curiosity) = humanism_data.get_mut("sexual_curiosity") {
                                if let Some(manifestations) = sexual_curiosity.get_mut("recent_manifestations") {
                                    if let Some(manifestations_array) = manifestations.as_array_mut() {
                                        if index < manifestations_array.len() {
                                            manifestations_array.remove(index);
                                            deleted = true;
                                            debug_log!("🗑️ Deleted sexual curiosity manifestation at index {}", index);
                                        }
                                    }
                                }
                            }
                        }
                    } else if item_id.starts_with("romantic_attraction_") {
                        if let Ok(index) = item_id.strip_prefix("romantic_attraction_").unwrap().parse::<usize>() {
                            if let Some(romantic_attraction) = humanism_data.get_mut("romantic_attraction") {
                                if let Some(manifestations) = romantic_attraction.get_mut("recent_manifestations") {
                                    if let Some(manifestations_array) = manifestations.as_array_mut() {
                                        if index < manifestations_array.len() {
                                            manifestations_array.remove(index);
                                            deleted = true;
                                            debug_log!("🗑️ Deleted romantic attraction manifestation at index {}", index);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&humanism_path, &humanism_data, "humanism").await?;
                        return Ok("Humanism item deleted successfully".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read humanism file: {}", e))
        }
    } else {
        debug_log!("⚠️ Humanism file does not exist");
    }
    
    Ok("Humanism item processed".to_string())
}

async fn delete_experiential_growth_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting experiential growth item: {}", item_id);
    
    let growth_path = PathBuf::from(get_data_path("experiential_growth_memory.json"));
    debug_log!("🗑️ Looking for experiential growth file at: {:?}", growth_path);
    
    if growth_path.exists() {
        match std::fs::read_to_string(&growth_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read experiential growth file ({} bytes)", content.len());
                
                if let Ok(mut growth_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    if item_id.starts_with("growth_insight_") {
                        if let Ok(index) = item_id.strip_prefix("growth_insight_").unwrap().parse::<usize>() {
                            if let Some(insights) = growth_data.get_mut("growth_insights") {
                                if let Some(insights_array) = insights.as_array_mut() {
                                    if index < insights_array.len() {
                                        insights_array.remove(index);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted growth insight at index {}", index);
                                    }
                                }
                            }
                        }
                    } else if item_id.starts_with("growth_pattern_") {
                        if let Ok(index) = item_id.strip_prefix("growth_pattern_").unwrap().parse::<usize>() {
                            if let Some(changes) = growth_data.get_mut("accumulated_changes") {
                                if let Some(changes_obj) = changes.as_object_mut() {
                                    let keys: Vec<String> = changes_obj.keys().cloned().collect();
                                    if index < keys.len() {
                                        changes_obj.remove(&keys[index]);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted growth pattern: {}", keys[index]);
                                    }
                                }
                            }
                        }
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&growth_path, &growth_data, "experiential_growth").await?;
                        return Ok("Experiential growth item deleted successfully".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read experiential growth file: {}", e))
        }
    } else {
        debug_log!("⚠️ Experiential growth file does not exist");
    }
    
    Ok("Experiential growth item processed".to_string())
}

async fn delete_somatic_state_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting somatic state item: {}", item_id);
    
    let somatic_path = PathBuf::from(get_data_path("somatic_state.json"));
    debug_log!("🗑️ Looking for somatic state file at: {:?}", somatic_path);
    
    if somatic_path.exists() {
        match std::fs::read_to_string(&somatic_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read somatic state file ({} bytes)", content.len());
                
                if let Ok(mut somatic_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    if item_id.starts_with("active_sensation_") {
                        if let Ok(index) = item_id.strip_prefix("active_sensation_").unwrap().parse::<usize>() {
                            if let Some(active_sensations) = somatic_data.get_mut("active_sensations") {
                                if let Some(sensations_obj) = active_sensations.as_object_mut() {
                                    let keys: Vec<String> = sensations_obj.keys().cloned().collect();
                                    if index < keys.len() {
                                        sensations_obj.remove(&keys[index]);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted active sensation: {}", keys[index]);
                                    }
                                }
                            }
                        }
                    } else if item_id.starts_with("sensation_history_") {
                        if let Ok(index) = item_id.strip_prefix("sensation_history_").unwrap().parse::<usize>() {
                            if let Some(history) = somatic_data.get_mut("sensation_history") {
                                if let Some(history_array) = history.as_array_mut() {
                                    // Find from the end since we showed the last 10
                                    let actual_index = history_array.len().saturating_sub(10) + index;
                                    if actual_index < history_array.len() {
                                        history_array.remove(actual_index);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted sensation history at index {}", actual_index);
                                    }
                                }
                            }
                        }
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&somatic_path, &somatic_data, "somatic_state").await?;
                        return Ok("Somatic state item deleted successfully".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read somatic state file: {}", e))
        }
    } else {
        debug_log!("⚠️ Somatic state file does not exist");
    }
    
    Ok("Somatic state item processed".to_string())
}

async fn delete_ritual_log_item(item_id: &str) -> Result<String, String> {
    debug_log!("🗑️ Deleting ritual log item: {}", item_id);
    
    let ritual_path = PathBuf::from(get_data_path("ritual_log.json"));
    debug_log!("🗑️ Looking for ritual log file at: {:?}", ritual_path);
    
    if ritual_path.exists() {
        match std::fs::read_to_string(&ritual_path) {
            Ok(content) => {
                debug_log!("🗑️ Successfully read ritual log file ({} bytes)", content.len());
                
                if let Ok(mut ritual_data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let mut deleted = false;
                    
                    if item_id.starts_with("ritual_") {
                        if let Ok(index) = item_id.strip_prefix("ritual_").unwrap().parse::<usize>() {
                            if let Some(active_rituals) = ritual_data.get_mut("active_rituals") {
                                if let Some(rituals_obj) = active_rituals.as_object_mut() {
                                    let keys: Vec<String> = rituals_obj.keys().cloned().collect();
                                    if index < keys.len() {
                                        let key_to_remove = &keys[index];
                                        debug_log!("🗑️ Deleting ritual: {}", key_to_remove);
                                        rituals_obj.remove(key_to_remove);
                                        deleted = true;
                                        debug_log!("🗑️ Deleted ritual: {}", key_to_remove);
                                    }
                                }
                            }
                        }
                    }
                    
                    if deleted {
                        save_json_file_with_logging(&ritual_path, &ritual_data, "ritual_log").await?;
                        return Ok("Ritual deleted successfully".to_string());
                    }
                }
            }
            Err(e) => return Err(format!("Failed to read ritual log file: {}", e))
        }
    } else {
        debug_log!("⚠️ Ritual log file does not exist");
    }
    
    Ok("Ritual log item processed".to_string())
}