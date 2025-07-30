// inventory_tracker.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryState {
    pub items: HashMap<String, i32>,
    pub last_update: u64,
    pub total_slots_used: i32,
    pub total_items: i32,
}

impl InventoryState {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            last_update: 0,
            total_slots_used: 0,
            total_items: 0,
        }
    }
    
    pub fn update(&mut self, items: HashMap<String, i32>) {
        self.items = items;
        self.total_slots_used = self.items.len() as i32;
        self.total_items = self.items.values().sum();
        self.last_update = current_timestamp();
    }
    
    pub fn get_summary(&self) -> String {
        if self.items.is_empty() {
            return "Empty inventory".to_string();
        }
        
        // Group items by category for cleaner display
        let mut categories: HashMap<&str, Vec<(&String, &i32)>> = HashMap::new();
        
        for (item, count) in &self.items {
            let category = categorize_item(item);
            categories.entry(category).or_insert_with(Vec::new).push((item, count));
        }
        
        let mut summary = Vec::new();
        
        // Priority order for categories
        let category_order = ["tools", "weapons", "food", "building", "valuable", "misc"];
        
        for cat in &category_order {
            if let Some(items) = categories.get(cat) {
                let items_str: Vec<String> = items.iter()
                    .map(|(name, count)| format!("{} x{}", clean_item_name(name), count))
                    .collect();
                
                if !items_str.is_empty() {
                    summary.push(format!("{}: {}", capitalize(cat), items_str.join(", ")));
                }
            }
        }
        
        summary.join(" | ")
    }
    
    pub fn has_item(&self, item_name: &str) -> bool {
        self.items.keys().any(|k| k.contains(item_name))
    }
    
    pub fn count_item(&self, item_name: &str) -> i32 {
        self.items.iter()
            .filter(|(k, _)| k.contains(item_name))
            .map(|(_, v)| v)
            .sum()
    }
    
    pub fn can_craft(&self, item: &str, amount: i32) -> (bool, String) {
        // Basic crafting checks
        match item {
            name if name.contains("planks") => {
                let logs = self.count_item("log");
                let can_craft = logs * 4 >= amount;
                (can_craft, format!("Need {} logs, have {}", (amount + 3) / 4, logs))
            },
            "chest" => {
                let planks = self.count_item("planks");
                let can_craft = planks >= 8;
                (can_craft, format!("Need 8 planks, have {}", planks))
            },
            "stick" => {
                let planks = self.count_item("planks");
                let can_craft = planks * 2 >= amount;
                (can_craft, format!("Need {} planks, have {}", (amount + 1) / 2, planks))
            },
            "torch" => {
                let sticks = self.count_item("stick");
                let coal = self.count_item("coal");
                let can_craft = sticks >= amount && coal >= amount;
                (can_craft, format!("Need {} sticks and {} coal, have {} and {}", amount, amount, sticks, coal))
            },
            _ => (false, "Unknown recipe".to_string())
        }
    }
}

lazy_static! {
    static ref INVENTORY_STATE: Mutex<InventoryState> = Mutex::new(InventoryState::new());
}

pub fn update_inventory(items: HashMap<String, i32>) {
    let mut state = INVENTORY_STATE.lock().unwrap();
    state.update(items);
}

pub fn get_inventory_summary() -> String {
    let state = INVENTORY_STATE.lock().unwrap();
    state.get_summary()
}

pub fn get_inventory_for_crafting(item: &str, amount: i32) -> String {
    let state = INVENTORY_STATE.lock().unwrap();
    let (can_craft, reason) = state.can_craft(item, amount);
    
    if can_craft {
        format!("✓ Can craft {} x{}", item, amount)
    } else {
        format!("✗ Cannot craft: {}", reason)
    }
}

pub fn get_full_inventory() -> InventoryState {
    INVENTORY_STATE.lock().unwrap().clone()
}

fn categorize_item(item: &str) -> &'static str {
    let item_lower = item.to_lowercase();
    
    if item_lower.contains("pickaxe") || item_lower.contains("axe") || 
       item_lower.contains("shovel") || item_lower.contains("hoe") {
        "tools"
    } else if item_lower.contains("sword") || item_lower.contains("bow") || 
              item_lower.contains("arrow") || item_lower.contains("armor") {
        "weapons"
    } else if item_lower.contains("apple") || item_lower.contains("bread") || 
              item_lower.contains("carrot") || item_lower.contains("beef") ||
              item_lower.contains("porkchop") || item_lower.contains("chicken") {
        "food"
    } else if item_lower.contains("log") || item_lower.contains("planks") || 
              item_lower.contains("stone") || item_lower.contains("dirt") ||
              item_lower.contains("cobblestone") || item_lower.contains("sand") {
        "building"
    } else if item_lower.contains("diamond") || item_lower.contains("iron") || 
              item_lower.contains("gold") || item_lower.contains("emerald") {
        "valuable"
    } else {
        "misc"
    }
}

fn clean_item_name(name: &str) -> &str {
    // Remove minecraft: prefix and underscores
    name.trim_start_matches("minecraft:")
        .trim_start_matches("oak_")
        .trim_start_matches("birch_")
        .trim_start_matches("spruce_")
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}