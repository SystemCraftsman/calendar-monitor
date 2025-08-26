use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},

    response::{Html, IntoResponse},
    routing::{get, get_service},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, warn};

mod calendar;
mod meeting;

use calendar::CalendarService;
use meeting::Meeting;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingUpdate {
    pub current_meeting: Option<Meeting>,
    pub next_meeting: Option<Meeting>,
    pub countdown_seconds: Option<i64>,
    pub active_time_blocks: Vec<Meeting>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    match dotenv::dotenv() {
        Ok(path) => info!("Loaded .env file from: {:?}", path),
        Err(e) => warn!("Could not load .env file: {}", e),
    }

    info!("Starting Calendar Monitor application");

    // Debug environment variables
    match std::env::var("ICS_FILE_PATH") {
        Ok(path) => info!("DEBUG: Found ICS_FILE_PATH: {}", path),
        Err(_) => info!("DEBUG: ICS_FILE_PATH not found"),
    }
    
    match std::env::var("ICS_FILE_PATHS") {
        Ok(paths) => info!("DEBUG: Found ICS_FILE_PATHS: {}", paths),
        Err(_) => info!("DEBUG: ICS_FILE_PATHS not found"),
    }

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler))
        .route("/api/meetings", get(get_meetings))
        .nest_service("/static", get_service(ServeDir::new("static")))
        .layer(CorsLayer::permissive());

    // Run the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../templates/index.html"))
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut interval = interval(Duration::from_secs(1));
    let calendar_service = CalendarService::new_from_env();

    loop {
        interval.tick().await;
        
        // Get regular meetings and active time blocks
        let meetings_result = calendar_service.get_current_and_next_meetings().await;
        let time_blocks_result = calendar_service.get_active_time_blocks().await;
        
        match (meetings_result, time_blocks_result) {
            (Ok((current, next)), Ok(active_time_blocks)) => {
                let countdown_seconds = current.as_ref().map(|m| m.time_until_end());
                
                let update = MeetingUpdate {
                    current_meeting: current,
                    next_meeting: next,
                    countdown_seconds,
                    active_time_blocks,
                };

                if let Ok(message) = serde_json::to_string(&update) {
                    if socket.send(Message::Text(message)).await.is_err() {
                        break;
                    }
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                warn!("Error fetching meetings or time blocks: {}", e);
            }
        }
    }
}

async fn get_meetings() -> impl IntoResponse {
    let calendar_service = CalendarService::new_from_env();
    
    // Get regular meetings and active time blocks
    let meetings_result = calendar_service.get_current_and_next_meetings().await;
    let time_blocks_result = calendar_service.get_active_time_blocks().await;
    
    match (meetings_result, time_blocks_result) {
        (Ok((current, next)), Ok(active_time_blocks)) => {
            let countdown_seconds = current.as_ref().map(|m| m.time_until_end());
            
            let update = MeetingUpdate {
                current_meeting: current,
                next_meeting: next,
                countdown_seconds,
                active_time_blocks,
            };
            
            Json(update)
        }
        _ => Json(MeetingUpdate {
            current_meeting: None,
            next_meeting: None,
            countdown_seconds: None,
            active_time_blocks: vec![],
        }),
    }
}
