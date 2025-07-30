//game_command_server.rs

use warp::{Filter, Reply};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::oneshot;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::debug_log;

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static mut SERVER_SHUTDOWN: Option<oneshot::Sender<()>> = None;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameCommand {
    pub id: String,
    pub game: String,
    pub session_id: String,
    pub action: ActionType,
    pub parameters: serde_json::Value,
    pub reasoning: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionType {
    Move { target: String, offset: Option<Vec3> },
    Attack { target: String, ability: Option<String> },
    Interact { object: String },
    Speak { text: String },
    UseItem { item: String },
    Cast { spell: String, target: Option<String> },
    Follow { target: String, distance: f32 },
    Wait { duration_ms: u64 },
    Custom { action: String, data: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone)]
pub struct GameConnection {
    pub game_id: String,
    pub connected_at: u64,
    pub tx: mpsc::UnboundedSender<GameCommand>,
}

pub struct GameCommandServer {
    port: u16,
    connections: Arc<Mutex<HashMap<String, GameConnection>>>,
}

impl GameCommandServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
		// Check if server is already running
		if SERVER_RUNNING.load(Ordering::SeqCst) {
			return Err("Server already running".into());
		}

		// Try to connect to see if port is in use
		if let Ok(_) = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", self.port)).await {
			return Err(format!("Port {} is already in use", self.port).into());
		}

		let connections = self.connections.clone();
		let connections_filter = warp::any().map(move || connections.clone());

		// WebSocket route
		let ws_route = warp::path!("ws" / String)
			.and(warp::ws())
			.and(connections_filter.clone())
			.map(|game_id: String, ws: warp::ws::Ws, connections| {
				ws.on_upgrade(move |socket| handle_websocket(socket, game_id, connections))
			});

		// Command route
		let command_route = warp::post()
			.and(warp::path("command"))
			.and(warp::body::json())
			.and(connections_filter.clone())
			.and_then(handle_command);

		// Status route
		let status_route = warp::get()
			.and(warp::path("status"))
			.and(connections_filter.clone())
			.and_then(handle_status);

		// Combine all routes
		let routes = ws_route.or(command_route).or(status_route);

		println!("🎮 Game Command Server starting on port {}", self.port);
		
		// Create shutdown channel
		let (tx, rx) = oneshot::channel::<()>();
		unsafe {
			SERVER_SHUTDOWN = Some(tx);
		}
		SERVER_RUNNING.store(true, Ordering::SeqCst);
		
		// Run server with graceful shutdown
		let (_, server) = warp::serve(routes)
			.bind_with_graceful_shutdown(([127, 0, 0, 1], self.port), async {
				rx.await.ok();
				println!("🎮 Game Command Server shutting down...");
			});
		
		tokio::spawn(async move {
			server.await;
			SERVER_RUNNING.store(false, Ordering::SeqCst);
			println!("🎮 Game Command Server stopped");
		});
		
		// Give it a moment to start
		tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		
		Ok(())
	}
}

async fn handle_websocket(
    ws: warp::ws::WebSocket,
    game_id: String,
    connections: Arc<Mutex<HashMap<String, GameConnection>>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Add connection
    {
        let mut conns = connections.lock().await;
        conns.insert(game_id.clone(), GameConnection {
            game_id: game_id.clone(),
            connected_at: current_timestamp(),
            tx,
        });
        println!("🎮 Game connected: {}", game_id);
    }

    // Spawn task to forward commands to websocket
    let game_id_clone = game_id.clone();
    tokio::spawn(async move {
        while let Some(command) = rx.recv().await {
            let msg = serde_json::to_string(&command).unwrap();
            if ws_tx.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // In handle_websocket function, where it handles incoming messages:
	while let Some(result) = ws_rx.next().await {
		match result {
			Ok(msg) => {
				if let Ok(text) = msg.to_str() {
					println!("📨 From {}: {}", game_id, text);
					
					// ADD THIS: Parse and forward inventory updates
					if let Ok(message) = serde_json::from_str::<serde_json::Value>(text) {
						if message["type"] == "status_update" {
							if let Some(inventory) = message["inventory"].as_object() {
								// Convert to HashMap<String, i32>
								let mut inventory_map = std::collections::HashMap::new();
								for (key, value) in inventory {
									if let Some(count) = value.as_i64() {
										inventory_map.insert(key.clone(), count as i32);
									}
								}
								
								// Update the inventory tracker
								crate::inventory_tracker::update_inventory(inventory_map);
								//debug_log!("📦 Updated inventory from bot");
							}
						}
					}
				}
			}
			Err(_) => break,
		}
	}

    // Remove connection on disconnect
    connections.lock().await.remove(&game_id);
    println!("🎮 Game disconnected: {}", game_id);
}

async fn handle_command(
    command: GameCommand,
    connections: Arc<Mutex<HashMap<String, GameConnection>>>,
) -> Result<impl Reply, warp::Rejection> {
    let conns = connections.lock().await;
    
    if let Some(connection) = conns.get(&command.game) {
        if connection.tx.send(command.clone()).is_ok() {
            Ok(warp::reply::json(&serde_json::json!({
                "status": "sent",
                "command_id": command.id,
                "game": command.game,
            })))
        } else {
            Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": "Failed to send command"
            })))
        }
    } else {
        Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": format!("Game '{}' not connected", command.game)
        })))
    }
}

async fn handle_status(
    connections: Arc<Mutex<HashMap<String, GameConnection>>>,
) -> Result<impl Reply, warp::Rejection> {
    let conns = connections.lock().await;
    let connected_games: Vec<_> = conns.keys().cloned().collect();
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "online",
        "connected_games": connected_games,
        "total_connections": connected_games.len(),
    })))
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Tauri commands
#[tauri::command]
pub async fn start_game_server(port: u16) -> Result<String, String> {
    // Check if already running
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        return Err("Server is already running".to_string());
    }
    
    let server = GameCommandServer::new(port);
    
    match server.start().await {
        Ok(_) => Ok(format!("Game command server started on port {}", port)),
        Err(e) => Err(format!("Failed to start server: {}", e))
    }
}

#[tauri::command]
pub async fn send_game_command(command: GameCommand) -> Result<String, String> {
	eprintln!("📤 Sending command to game server: {:?}", command.action);
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("http://localhost:8420/command"))
        .json(&command)
        .send()
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;
        
    if response.status().is_success() {
        Ok("Command sent successfully".to_string())
    } else {
        Err(format!("Command failed: {}", response.status()))
    }
}

#[tauri::command]
pub async fn get_game_server_status() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("http://localhost:8420/status")
        .send()
        .await
        .map_err(|_| "Server offline".to_string())?;
        
    if response.status().is_success() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to get status".to_string())
    }
}

#[tauri::command]
pub async fn stop_game_server() -> Result<String, String> {
    if !SERVER_RUNNING.load(Ordering::SeqCst) {
        return Err("Server not running".to_string());
    }
    
    unsafe {
        if let Some(shutdown) = SERVER_SHUTDOWN.take() {
            let _ = shutdown.send(());
            
            // Wait a bit for graceful shutdown
            let mut attempts = 0;
            while SERVER_RUNNING.load(Ordering::SeqCst) && attempts < 10 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                attempts += 1;
            }
            
            if SERVER_RUNNING.load(Ordering::SeqCst) {
                SERVER_RUNNING.store(false, Ordering::SeqCst);
                return Ok("Game command server force stopped".to_string());
            }
            
            Ok("Game command server stopped".to_string())
        } else {
            Err("No shutdown channel available".to_string())
        }
    }
}