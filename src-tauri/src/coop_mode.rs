//coop_mode.rs
use serde::{Deserialize, Serialize};
use crate::consciousness_state::ConsciousnessState;
use crate::game_command_server::{GameCommand, ActionType};
use std::sync::Arc;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde_json::json;

// Global state for co-op mode
lazy_static! {
    static ref COOP_STATE: Mutex<Option<CoopMode>> = Mutex::new(None);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoopMode {
    pub is_active: bool,
    pub game: SupportedGame,
    pub session_id: String,
    pub character_name: String,
    pub last_action: Option<GameCommand>,
    pub last_action_time: Option<u64>,
    pub total_actions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupportedGame {
    Skyrim,
    BaldursGate3,
    Minecraft,
}

impl SupportedGame {
    pub fn to_string(&self) -> String {
        match self {
            SupportedGame::Skyrim => "skyrim".to_string(),
            SupportedGame::BaldursGate3 => "bg3".to_string(),
            SupportedGame::Minecraft => "minecraft".to_string(),
        }
    }
    
    pub fn get_available_actions(&self) -> Vec<String> {
        match self {
            SupportedGame::Skyrim => vec![
                "move_to(target, offset?)",
                "attack(target, spell?)",
                "cast(spell, target?)",
                "say(text)",
                "follow(target, distance)",
                "interact(object)",
                "wait(duration)",
                "sneak()",
                "stand()",
            ],
            SupportedGame::BaldursGate3 => vec![
                "move_to(position)",
                "attack(target, ability?)",
                "cast(spell, target?)",
                "say(text)",
                "examine(object)",
                "use_item(item, target?)",
                "end_turn()",
                "dialog_choice(option)",
            ],
            SupportedGame::Minecraft => vec![
                "move_to(x, y, z)",
                "dig(x, y, z)",
                "place_block(block, x, y, z)",
                "attack(entity)",
                "chat(message)",
                "collect(item)",
                "craft(item, quantity)",
                "follow(player, distance)",
            ],
        }.iter().map(|s| s.to_string()).collect()
    }
	
	
}

impl CoopMode {
    pub fn new(game: SupportedGame, character_name: String) -> Self {
        Self {
            is_active: false,
            game,
            session_id: uuid::Uuid::new_v4().to_string(),
            character_name,
            last_action: None,
            last_action_time: None,
            total_actions: 0,
        }
    }
    
   // In coop_mode.rs - update extract_and_execute_commands

async fn extract_and_execute_commands(
    &mut self,
    lyra_response: &str,
) -> Result<Vec<GameCommand>, String> {
    eprintln!("\n🎮 ============ COMMAND EXTRACTION START ============");
    eprintln!("🎮 Processing Lyra's response: {}", lyra_response);
    
    let mut commands = Vec::new();
    
    // First, check for explicit tags like [BREAK: tree] or [BUILD: house]
    let tag_regex = regex::Regex::new(r"\[([A-Z]+):\s*([^\]]+)\]").unwrap();
    
    for cap in tag_regex.captures_iter(lyra_response) {
    if let (Some(action), Some(target)) = (cap.get(1), cap.get(2)) {
        let action_str = action.as_str();
        let target_str = target.as_str().trim();
        
        eprintln!("🎮 Found tag: [{}:{}]", action_str, target_str);
        
        let command_action = match action_str {           
				"BREAK" | "DIG" => {
				// Check for directional digging first
				if target_str == "up" || target_str == "out" || target_str.contains("surface") {
					ActionType::Custom {
						action: "dig_up".to_string(),
						data: json!({
							"target": if target_str.contains("surface") { "surface" } else { "up" }
						})
					}
				}
				// Check if target is just a number (like "5")
				else if let Ok(amount) = target_str.parse::<i32>() {
					// Just a number - dig that many of nearest diggable block
					ActionType::Custom {
						action: "dig_nearest".to_string(),
						data: json!({"amount": amount})
					}
				} else {
					// Parse for amount - "dirt 10" or just "dirt"
					let parts: Vec<&str> = target_str.split_whitespace().collect();
					let material = parts.get(0).unwrap_or(&"").to_string();
					let amount = parts.get(1).and_then(|s| s.parse::<i32>().ok());
					
					// Handle different materials
					if material.contains("tree") {
						ActionType::Custom {
							action: "break_tree".to_string(),
							data: json!({"target": material, "mode": "full"})
						}
					}else if material.contains("stone") || material.contains("dirt") || 
							  material.contains("gravel") || material.contains("sand") {
						ActionType::Custom {
							action: "excavate".to_string(),
							data: json!({
								"material": material,
								"size": "smart",
								"amount": amount
							})
						}
					} else if material.contains("leaves") {
						ActionType::Custom {
							action: "shear_leaves".to_string(),
							data: json!({"amount": amount.unwrap_or(999)}) // 999 = "nearby"
						}
					} else {
						ActionType::Custom {
							action: "break_block".to_string(),
							data: json!({
								"target": material,
								"amount": amount
							})
						}
					}
				}
				},
				
				"MINE" => {
				let lower_target = target_str.to_lowercase();
				
				// Parse mining commands
				if lower_target.contains("down") || lower_target.contains("shaft") {
					ActionType::Custom {
						action: "mine_shaft".to_string(),
						data: json!({
							"depth": target_str.split_whitespace()
								.find_map(|s| s.parse::<i32>().ok())
								.unwrap_or(-59),
							"type": "staircase"
						})
					}
				} else if lower_target.contains("strip") || lower_target.contains("branch") {
					ActionType::Custom {
						action: "strip_mine".to_string(),
						data: json!({
							"pattern": "efficient",
							"length": 50,
							"branches": true
						})
					}
				} else if lower_target.contains("find") || lower_target.contains("search") {
					// FIX: Create owned String
					let ore_type = lower_target
						.replace("find", "")
						.replace("search", "")
						.trim()
						.to_string();
					
					ActionType::Custom {
						action: "smart_mine".to_string(),
						data: json!({
							"target": ore_type,
							"method": "cave_search"
						})
					}
				} else if lower_target.contains("area") || lower_target.contains("clear") {
					ActionType::Custom {
						action: "quarry".to_string(),
						data: json!({
							"size": target_str.split_whitespace()
								.find_map(|s| s.parse::<i32>().ok())
								.unwrap_or(10),
							"depth": 5
						})
					}
				} else {
					ActionType::Custom {
						action: "mine_vein".to_string(),
						data: json!({"ore": target_str})
					}
				}
			},
				"DIG" => {
					let lower_target = target_str.to_lowercase();
					
					if lower_target.contains("tunnel") {
						ActionType::Custom {
							action: "dig_tunnel".to_string(),
							data: json!({
								"length": 20,
								"width": 3,
								"height": 3,
								"torches": true
							})
						}
					} else if lower_target.contains("room") || lower_target.contains("chamber") {
						ActionType::Custom {
							action: "dig_room".to_string(),
							data: json!({
								"width": 7,
								"height": 4,
								"depth": 7
							})
						}
					} else if lower_target.contains("stair") {
						ActionType::Custom {
							action: "dig_staircase".to_string(),
							data: json!({
								"depth": 20,
								"direction": "down"
							})
						}
					} else {
						ActionType::Custom {
							action: "excavate".to_string(),
							data: json!({"material": target_str, "smart": true})
						}
					}
				},
				
				"BUILD" => {
					let lower_target = target_str.to_lowercase();
					
					// Parse building size modifiers
					let size = if lower_target.contains("small") { "small" }
							  else if lower_target.contains("large") || lower_target.contains("big") { "large" }
							  else { "medium" };
					
					// Parse building material preferences
					let material = if lower_target.contains("stone") { "stone" }
								  else if lower_target.contains("wood") { "wood" }
								  else if lower_target.contains("brick") { "brick" }
								  else { "auto" }; // Auto-select based on inventory
					
					let structure_data = if lower_target.contains("house") {
						json!({
							"type": "house",
							"size": size,
							"material": material,
							"features": ["door", "windows", "roof", "floor"]
						})
					} else if lower_target.contains("castle") {
						json!({
							"type": "castle",
							"size": size,
							"features": ["walls", "towers", "gate", "courtyard"]
						})
					} else if lower_target.contains("tower") {
						json!({
							"type": "tower",
							"height": target_str.split_whitespace()
								.find_map(|s| s.parse::<i32>().ok())
								.unwrap_or(10),
							"material": material,
							"features": ["stairs", "windows", "battlements"]
						})
					} else if lower_target.contains("bridge") {
						json!({
							"type": "bridge",
							"auto_length": true, // Detect gap
							"material": material,
							"railings": true
						})
					} else if lower_target.contains("wall") {
						json!({
							"type": "wall",
							"length": 20,
							"height": 4,
							"material": material,
							"style": if lower_target.contains("defensive") { "castle" } else { "simple" }
						})
					} else if lower_target.contains("bunker") || lower_target.contains("shelter") {
						json!({
							"type": "bunker",
							"underground": true,
							"reinforced": true,
							"features": ["storage", "bed", "crafting"]
						})
					} else if lower_target.contains("storage") {
						json!({
							"type": "storage_room",
							"size": size,
							"organized": true,
							"labels": true
						})
					} else if lower_target.contains("workshop") {
						json!({
							"type": "workshop",
							"stations": ["crafting", "furnace", "anvil", "enchanting"]
						})
					} else {
						json!({"type": target_str, "size": size})
					};
					
					ActionType::Custom {
						action: "build_structure".to_string(),
						data: structure_data
					}
				},
				
				"NERDPOLE" | "POLE" | "TOWER" => ActionType::Custom {
					action: "nerdpole".to_string(),
					data: json!({
						"height": target_str.parse::<i32>().unwrap_or(10)
					})
				},
				
				"BUCKET" | "POUR" | "FILL" => {
					ActionType::Custom {
						action: "use_bucket".to_string(),
						data: json!({
							"liquid": target_str,
							"action": if action_str == "FILL" { "fill" } else { "pour" }
						})
					}
				},

				"WATER" => {
					ActionType::Custom {
						action: "water_action".to_string(),
						data: json!({
							"target": target_str,
							"smart": true
						})
					}
				},

				"USE" => {
					// Smart item usage based on context
					ActionType::Custom {
						action: "use_item_smart".to_string(),
						data: json!({
							"item": target_str,
							"context": "auto"
						})
					}
				},

				"RAIL" | "TRACK" => {
					ActionType::Custom {
						action: "build_railway".to_string(),
						data: json!({
							"type": target_str,
							"length": "auto"
						})
					}
				},

				"RIDE" | "MOUNT" => {
					ActionType::Custom {
						action: "mount_entity".to_string(),
						data: json!({
							"entity": target_str
						})
					}
				},

				"PLACE" => {
					ActionType::Custom {
						action: "place_advanced".to_string(),
						data: json!({
							"item": target_str,
							"pattern": "smart"
						})
					}
				},
				
				// Add to tag handling
				"STORE" | "DEPOSIT" => {
					ActionType::Custom {
						action: "store_items".to_string(),
						data: json!({
							"items": target_str,
							"smart": true
						})
					}
				},

				"ORGANIZE" | "SORT" => {
					ActionType::Custom {
						action: "organize_inventory".to_string(),
						data: json!({
							"type": target_str
						})
					}
				},

				"DROP" | "THROW" => {
					ActionType::Custom {
						action: "drop_items".to_string(),
						data: json!({
							"items": target_str,
							"amount": "smart"
						})
					}
				},

				"KEEP" => {
					ActionType::Custom {
						action: "set_keep_items".to_string(),
						data: json!({
							"items": target_str
						})
					}
				},

				"CRAFT" => {
				// Check if there's a number at the end
				let parts: Vec<&str> = target_str.split_whitespace().collect();
				let last_part = parts.last().and_then(|s| s.parse::<i32>().ok());
				
				let (item_name, amount) = if let Some(num) = last_part {
					// Number at end: "planks 4" or "crafting table 2"
					let item_parts = &parts[..parts.len()-1];
					(item_parts.join(" "), num)
				} else {
					// No number: "crafting table" or "planks"
					(target_str.to_string(), 1)
				};
				
				ActionType::Custom {
					action: "craft_item".to_string(),
					data: json!({
						"item": item_name,
						"amount": amount
					})
				}
			},

				"BREW" => {
					ActionType::Custom {
						action: "brew_potion".to_string(),
						data: json!({
							"potion": target_str,
							"auto_ingredients": true
						})
					}
				},

				"REDSTONE" => {
					ActionType::Custom {
						action: "place_redstone".to_string(),
						data: json!({
							"pattern": target_str
						})
					}
				},

				"FLY" => {
					ActionType::Custom {
						action: "elytra_fly".to_string(),
						data: json!({
							"destination": target_str
						})
					}
				},

				"SHOOT" => {
					ActionType::Custom {
						action: "ranged_attack".to_string(),
						data: json!({
							"target": target_str,
							"weapon": "auto"
						})
					}
				},
				
				"TAME" => ActionType::Custom {
					action: "tame_animal".to_string(),
					data: json!({"animal": target_str})
				},
				
				"LEAD" | "BRING" => ActionType::Custom {
					action: "lead_animal".to_string(),
					data: json!({"animal": target_str, "method": "auto"})
				},
				
				"SHEAR" => ActionType::Custom {
					action: "shear".to_string(),
					data: json!({"target": target_str})
				},
				
				"BREED" => ActionType::Custom {
					action: "breed_animals".to_string(),
					data: json!({"type": target_str})
				},
				
				"FARM" => ActionType::Custom {
					action: "farm_crop".to_string(),
					data: json!({"crop": target_str, "action": "auto"})
				},
				
				"FISH" => ActionType::Custom {
					action: "go_fishing".to_string(),
					data: json!({"duration": "until_catch"})
				},
				
				"EXPLORE" => ActionType::Custom {
					action: "explore".to_string(),
					data: json!({"target": target_str, "range": "medium"})
				},
				
				"HUNT" => ActionType::Custom {
					action: "hunt".to_string(),
					data: json!({"target": target_str})
				},
				
				"SMELT" => ActionType::Custom {
					action: "smelt".to_string(),
					data: json!({"item": target_str, "fuel": "auto"})
				},
				
				"COOK" => ActionType::Custom {
					action: "cook_food".to_string(),
					data: json!({"food": target_str})
				},
				
				"ENCHANT" => ActionType::Custom {
					action: "enchant".to_string(),
					data: json!({"item": target_str, "level": "best"})
				},
				
				"TRADE" => ActionType::Custom {
					action: "trade".to_string(),
					data: json!({"with": target_str, "goal": "auto"})
				},
				
				"PORTAL" => ActionType::Custom {
					action: "build_portal".to_string(),
					data: json!({"type": target_str}) // nether or end
				},
                "FOLLOW" => ActionType::Follow {
                    target: target_str.to_string(),
                    distance: 3.0
                },
                "ATTACK" => ActionType::Attack {
                    target: target_str.to_string(),
                    ability: None
                },
                "GIVE" => {
                    // Parse "GIVE: diamond, 5" or "GIVE: player, diamond, 5"
                    let parts: Vec<&str> = target_str.split(',').map(|s| s.trim()).collect();
                    ActionType::Custom {
                        action: "give_item".to_string(),
                        data: json!({"parts": parts})
                    }
                },
                "COLLECT" => ActionType::Custom {
                    action: "collect".to_string(),
                    data: json!({"target": target_str})
                },
                "CRAFT" => ActionType::Custom {
                    action: "craft".to_string(),
                    data: json!({"item": target_str, "amount": 1})
                },
                "GOTO" => ActionType::Custom {
                    action: "goto".to_string(),
                    data: json!({"destination": target_str})
                },
                _ => ActionType::Custom {
                    action: action_str.to_lowercase(),
                    data: json!({"target": target_str})
                }
            };
            
        let command = self.create_command(command_action, &format!("Tagged action: {} {}", action_str, target_str));
        
        // FIX: Clone command where needed
        match crate::game_command_server::send_game_command(command.clone()).await {
            Ok(_) => {
                eprintln!("🎮 ✅ Tagged command sent successfully!");
                commands.push(command.clone());
                self.update_state(command);
            },
            Err(e) => eprintln!("🎮 ❌ Failed to send tagged command: {}", e)
        }
    }
}
    
    // If no tags found, try natural language with ENHANCED patterns
    if commands.is_empty() {
        eprintln!("🎮 No tags found, trying enhanced natural language extraction...");
        commands = self.extract_natural_commands_enhanced(lyra_response).await?;
    }
    
    eprintln!("🎮 Total commands processed: {}", commands.len());
    eprintln!("🎮 ============ COMMAND EXTRACTION END ============\n");
    Ok(commands)
}

// New enhanced natural language extraction
async fn extract_natural_commands_enhanced(&mut self, response: &str) -> Result<Vec<GameCommand>, String> {
    let mut commands = Vec::new();
    let lower = response.to_lowercase();
	
	// Digging/excavating patterns
if (lower.contains("dig") || lower.contains("get") || lower.contains("grab") || 
    lower.contains("collect") || lower.contains("gather") || lower.contains("mine")) {
    
    // Check for amounts
    let amount_regex = regex::Regex::new(r"(\d+)\s*(dirt|stone|sand|gravel|wood|logs?)").unwrap();
    if let Some(cap) = amount_regex.captures(&lower) {
        let amount = cap[1].parse::<i32>().unwrap_or(1);
        let material = &cap[2];
        
        commands.push(self.create_command(ActionType::Custom {
            action: "excavate".to_string(),
            data: json!({
                "material": material,
                "amount": amount
            })
        }, &format!("Getting {} {}", amount, material)));
    }
    // Also check reverse order "dirt 10" or "some dirt"
    else if lower.contains("some dirt") || lower.contains("some stone") {
        let material = if lower.contains("dirt") { "dirt" }
                      else if lower.contains("stone") { "stone" }
                      else if lower.contains("wood") { "wood" }
                      else { "dirt" };
        
        commands.push(self.create_command(ActionType::Custom {
            action: "excavate".to_string(),
            data: json!({"material": material})
        }, &format!("Getting some {}", material)));
    }
}

// Crafting patterns - much more flexible
if (lower.contains("craft") || lower.contains("make") || lower.contains("create")) &&
   (lower.contains("pickaxe") || lower.contains("sword") || lower.contains("planks") ||
    lower.contains("chest") || lower.contains("torch") || lower.contains("stick")) {
    
    let item = if lower.contains("pickaxe") { "pickaxe" }
               else if lower.contains("sword") { "sword" }
               else if lower.contains("planks") { "planks" }
               else if lower.contains("chest") { "chest" }
               else if lower.contains("torch") { "torch" }
               else if lower.contains("stick") { "stick" }
               else { "planks" };
    
    commands.push(self.create_command(ActionType::Custom {
        action: "craft_item".to_string(),
        data: json!({"item": item, "amount": 1})
    }, &format!("Crafting {}", item)));
}

// Storage patterns
if (lower.contains("put") || lower.contains("store") || lower.contains("save")) &&
   (lower.contains("chest") || lower.contains("away") || lower.contains("items")) {
    
    commands.push(self.create_command(ActionType::Custom {
        action: "store_items".to_string(),
        data: json!({"items": "smart", "smart": true})
    }, "Storing items"));
}

// Following variations
if lower.contains("wait up") || lower.contains("hold on") || 
   lower.contains("coming") || lower.contains("be right there") {
    commands.push(self.create_command(ActionType::Move {
        target: "player".to_string(),
        offset: None
    }, "Moving to player"));
}
    
    // Tree/wood breaking - much more flexible
    if (lower.contains("break") || lower.contains("chop") || lower.contains("cut") || 
        lower.contains("take down") || lower.contains("destroy") || lower.contains("get") ||
        lower.contains("harvest")) && 
       (lower.contains("tree") || lower.contains("wood") || lower.contains("log")) {
        
        commands.push(self.create_command(ActionType::Custom { 
            action: "break_tree".to_string(),
            data: json!({"target": "tree", "mode": "full"})
        }, "Breaking tree"));
    }
    
    // Mining patterns
    if (lower.contains("mine") || lower.contains("dig") || lower.contains("extract")) &&
       (lower.contains("coal") || lower.contains("iron") || lower.contains("gold") || 
        lower.contains("diamond") || lower.contains("ore")) {
        
        let ore_type = if lower.contains("coal") { "coal" }
        else if lower.contains("iron") { "iron" }
        else if lower.contains("gold") { "gold" }
        else if lower.contains("diamond") { "diamond" }
        else { "ore" };
        
        commands.push(self.create_command(ActionType::Custom { 
            action: "mine_vein".to_string(),
            data: json!({"ore": ore_type})
        }, "Mining ore vein"));
    }
    
    // Building patterns
    if lower.contains("build") || lower.contains("make") || lower.contains("construct") {
        let structure = if lower.contains("house") { "house" }
        else if lower.contains("tower") { "tower" }
        else if lower.contains("bridge") { "bridge" }
        else if lower.contains("wall") { "wall" }
        else if lower.contains("shelter") { "shelter" }
        else { "structure" };
        
        commands.push(self.create_command(ActionType::Custom { 
            action: "build".to_string(),
            data: json!({"structure": structure})
        }, "Building structure"));
    }
    
    // Movement patterns
    if lower.contains("come here") || lower.contains("come to me") || 
       lower.contains("i'll come") || lower.contains("on my way") {
        commands.push(self.create_command(ActionType::Move { 
            target: "player".to_string(), 
            offset: None 
        }, "Moving to player"));
    }
    
    // Following patterns
    if lower.contains("follow") || lower.contains("stay close") || 
       lower.contains("come with") || lower.contains("right behind") {
        commands.push(self.create_command(ActionType::Follow { 
            target: "player".to_string(), 
            distance: 3.0 
        }, "Following player"));
    }
    
    // Send each command
    for command in &commands {
        match crate::game_command_server::send_game_command(command.clone()).await {
            Ok(_) => {
                eprintln!("🎮 ✅ Natural command sent: {:?}", command.action);
                self.update_state(command.clone());
            },
            Err(e) => eprintln!("🎮 ❌ Failed to send natural command: {}", e)
        }
    }
    
    Ok(commands)
}

// Helper to update state after command
fn update_state(&mut self, command: GameCommand) {
    self.last_action = Some(command);
    self.last_action_time = Some(current_timestamp());
    self.total_actions += 1;
}

	fn create_command(&self, action: ActionType, reasoning: &str) -> GameCommand {
		GameCommand {
			id: uuid::Uuid::new_v4().to_string(),
			game: self.game.to_string(),
			session_id: self.session_id.clone(),
			action,
			parameters: serde_json::Value::Null,
			reasoning: reasoning.to_string(),
			timestamp: current_timestamp(),
		}
	}
		
		fn parse_command(&self, command_text: &str) -> Result<ActionType, String> {
		let trimmed = command_text.trim();
		
		// Try JSON first
		if trimmed.starts_with('{') {
			if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
				return self.parse_json_command(json);
			}
		}
		
		// Function-style commands: move_to(player, 5)
		if let Some(paren_pos) = trimmed.find('(') {
			let func_name = trimmed[..paren_pos].trim();
			let args_end = trimmed.rfind(')').unwrap_or(trimmed.len());
			let args = &trimmed[paren_pos+1..args_end];
			
			return self.parse_function_command(func_name, args);
		}
		
		// Simple commands
		let parts: Vec<&str> = trimmed.split_whitespace().collect();
		if parts.is_empty() {
			return Err("Empty command".to_string());
		}
		
		match parts[0].to_lowercase().as_str() {
			"move" | "go" | "walk" => {
				let target = parts.get(1).map(|s| s.to_string()).unwrap_or("player".to_string());
				Ok(ActionType::Move { target, offset: None })
			},
			"say" | "speak" => {
				let text = parts[1..].join(" ");
				Ok(ActionType::Speak { text })
			},
			"attack" | "fight" => {
				let target = parts.get(1).map(|s| s.to_string()).unwrap_or("nearest_enemy".to_string());
				Ok(ActionType::Attack { target, ability: None })
			},
			"follow" => {
				let target = parts.get(1).map(|s| s.to_string()).unwrap_or("player".to_string());
				let distance = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(3.0);
				Ok(ActionType::Follow { target, distance })
			},
			"wait" | "pause" => {
				let duration = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
				Ok(ActionType::Wait { duration_ms: duration })
			},
			_ => Ok(ActionType::Custom {
				action: trimmed.to_string(),
				data: serde_json::Value::Null,
			})
		}
	}

	fn parse_json_command(&self, json: serde_json::Value) -> Result<ActionType, String> {
		// Parse JSON-style commands
		if let Some(action_type) = json.get("type").and_then(|v| v.as_str()) {
			match action_type {
				"move" => Ok(ActionType::Move {
					target: json.get("target").and_then(|v| v.as_str()).unwrap_or("player").to_string(),
					offset: None,
				}),
				"speak" => Ok(ActionType::Speak {
					text: json.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string(),
				}),
				// ... more types
				_ => Ok(ActionType::Custom {
					action: action_type.to_string(),
					data: json.clone(),
				})
			}
		} else {
			Err("Missing action type in JSON".to_string())
		}
	}

	fn parse_function_command(&self, func_name: &str, args: &str) -> Result<ActionType, String> {
		let arg_parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
		
		match func_name.to_lowercase().as_str() {
			"move_to" => {
				let target = arg_parts.get(0).unwrap_or(&"player").to_string();
				Ok(ActionType::Move { target, offset: None })
			},
			"say" => {
				let text = args.trim_matches('"').to_string();
				Ok(ActionType::Speak { text })
			},
			"follow" => {
				let target = arg_parts.get(0).unwrap_or(&"player").to_string();
				let distance = arg_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
				Ok(ActionType::Follow { target, distance })
			},
			// ... more functions
			_ => Ok(ActionType::Custom {
				action: format!("{}({})", func_name, args),
				data: serde_json::Value::Null,
			})
		}
	}
		
		

	
}


// Public function to process Lyra's responses for commands
pub async fn process_lyra_response_for_commands(response: &str) -> Result<Vec<GameCommand>, String> {
    // Clone the coop state to avoid holding mutex across await
    let coop_clone = {
        let coop_state = COOP_STATE.lock().unwrap();
        coop_state.clone()
    };
    
    if let Some(mut coop) = coop_clone {
        if coop.is_active {
            let commands = coop.extract_and_execute_commands(response).await?;
            
            // Update the state with any changes
            {
                let mut coop_state = COOP_STATE.lock().unwrap();
                *coop_state = Some(coop);
            }
            
            Ok(commands)
        } else {
            Ok(vec![])
        }
    } else {
        Ok(vec![])
    }
}

// Add this function to access coop state
pub fn get_coop_state() -> Option<CoopMode> {
    COOP_STATE.lock().unwrap().clone()
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Tauri commands
#[tauri::command]
pub async fn enable_coop_mode(
    game: String,
    character_name: String,
) -> Result<String, String> {
    let game_type = match game.as_str() {
        "skyrim" => SupportedGame::Skyrim,
        "bg3" => SupportedGame::BaldursGate3,
        "minecraft" => SupportedGame::Minecraft,
        _ => return Err("Unsupported game".to_string()),
    };
    
    let mut coop = CoopMode::new(game_type, character_name);
    coop.is_active = true;
    
    // Save to global state
    let mut coop_state = COOP_STATE.lock().unwrap();
    *coop_state = Some(coop);
    
    Ok(format!("Co-op mode enabled for {}", game))
}

#[tauri::command]
pub async fn disable_coop_mode() -> Result<String, String> {
    let mut coop_state = COOP_STATE.lock().unwrap();
    *coop_state = None;
    
    Ok("Co-op mode disabled".to_string())
}

pub fn get_minecraft_action_context(inventory_summary: &str) -> String {
    let mut context = String::new();
    
    // Include current inventory
    context.push_str(&format!("📦 **Your Current Inventory:**\n{}\n\n", inventory_summary));
    
    // Mining with material requirements
    context.push_str("⛏️ **Mining Commands:**\n");
    context.push_str("• [MINE: shaft 30] - dig down to Y=30 (needs: pickaxe, torches, food)\n");
    context.push_str("• [MINE: strip] - efficient branch mining (needs: pickaxe, 64+ torches)\n");
    context.push_str("• [DIG: up/out] - return to surface (needs: pickaxe, 32+ blocks for pillaring)\n");
    context.push_str("• [DIG: dirt 20] - get specific amount (needs: shovel preferred)\n\n");
    
    // Building with exact requirements
    context.push_str("🏗️ **Building Commands (with material needs):**\n");
    context.push_str("• [BUILD: small wooden house] - 5x5x4 (needs: 64 planks, 20 glass, 1 door)\n");
    context.push_str("• [BUILD: medium stone house] - 7x7x5 (needs: 128 cobblestone, 32 glass, 1 door)\n");
    context.push_str("• [BUILD: large house] - 11x11x6 (needs: 256 blocks, 48 glass, 2 doors)\n");
    context.push_str("• [BUILD: tower 20] - 20 blocks tall (needs: 200 stone, 40 torches, ladders)\n");
    context.push_str("• [BUILD: bridge] - auto-detects gap (needs: 3 blocks per meter length)\n");
    context.push_str("• [BUILD: bunker] - underground (needs: 128 stone, bed, chests, torches)\n");
    context.push_str("• [BUILD: storage room] - organized (needs: 100 blocks, 20 chests, signs)\n");
    context.push_str("• [BUILD: workshop] - all stations (needs: materials for each station)\n\n");
    
    // Crafting with requirements
    context.push_str("🔨 **Crafting (materials → result):**\n");
    context.push_str("• [CRAFT: wooden_pickaxe] - 3 planks + 2 sticks → pickaxe\n");
    context.push_str("• [CRAFT: stone_pickaxe] - 3 cobblestone + 2 sticks → better pickaxe\n");
    context.push_str("• [CRAFT: chest] - 8 planks → storage\n");
    context.push_str("• [CRAFT: furnace] - 8 cobblestone → smelting\n");
    context.push_str("• [CRAFT: bed] - 3 wool + 3 planks → spawn point\n");
    context.push_str("• [CRAFT: torch 4] - 1 coal + 1 stick → 4 torches\n\n");
    
    // Resource gathering
    context.push_str("🌳 **Resource Gathering:**\n");
    context.push_str("• [BREAK: oak_tree] - get ~4-6 logs → 16-24 planks\n");
    context.push_str("• [DIG: stone 64] - get cobblestone for building\n");
    context.push_str("• [MINE: coal] - find coal for torches\n");
    context.push_str("• [SHEAR: sheep] - get wool (needs: shears)\n\n");
    
    // Context-aware suggestions
    context.push_str("💡 **Smart Suggestions based on inventory:**\n");
    
    // Parse inventory for smart suggestions
    if inventory_summary.contains("log") && !inventory_summary.contains("planks") {
        context.push_str("• You have logs - craft them into planks first!\n");
    }
    if inventory_summary.contains("planks") && !inventory_summary.contains("stick") {
        context.push_str("• You have planks - craft sticks for tools!\n");
    }
    if !inventory_summary.contains("pickaxe") {
        context.push_str("• No pickaxe! Craft one to mine stone/ores\n");
    }
    if !inventory_summary.contains("torch") {
        context.push_str("• No torches! Dangerous to mine without light\n");
    }
    
    context
}