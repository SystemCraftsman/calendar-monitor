use axum::{
    extract::{Query, ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse},
    routing::get,
    http::{StatusCode, HeaderMap, header},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod config;
mod calendar;
mod meeting;
mod google_calendar;

use config::Config;
use calendar::CalendarService;
use meeting::Meeting;
use google_calendar::{GoogleCalendarService, GoogleTokens};

// Embed static files into the binary
const STYLE_CSS: &str = include_str!("../static/style.css");
const APP_JS: &str = include_str!("../static/app.js");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingUpdate {
    pub current_meeting: Option<Meeting>,
    pub next_meeting: Option<Meeting>,
    pub countdown_seconds: Option<i64>,
    pub active_time_blocks: Vec<Meeting>,
}

// Global state for Google Calendar tokens
type GoogleTokensStore = Arc<Mutex<Option<GoogleTokens>>>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub google_tokens: GoogleTokensStore,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file (for development)
    if let Err(_e) = dotenv::dotenv() {
        // .env file is optional - don't warn in production
    }

    info!("Starting Calendar Monitor application");

    // Load configuration
    let config = match Config::load() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            Arc::new(config)
        }
        Err(e) => {
            warn!("Failed to load configuration: {}", e);
            info!("Using default configuration with environment variables");
            // Fallback to default config with env vars
            let mut default_config = Config::default();
            if let Err(env_err) = default_config.apply_env_vars() {
                return Err(anyhow::anyhow!(
                    "Configuration error: {}. Original error: {}", 
                    env_err, e
                ));
            }
            if let Err(validation_err) = default_config.validate() {
                return Err(anyhow::anyhow!(
                    "Configuration validation failed: {}", 
                    validation_err
                ));
            }
            Arc::new(default_config)
        }
    };

    // Create shared state for Google tokens
    let app_state = AppState {
        config: config.clone(),
        google_tokens: Arc::new(Mutex::new(None)),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler))
        .route("/api/meetings", get(get_meetings))
        .route("/auth/google/login", get(google_auth_login))
        .route("/auth/google/callback", get(google_auth_callback))
        .route("/auth/google/status", get(google_auth_status))
        .route("/static/style.css", get(serve_css))
        .route("/static/app.js", get(serve_js))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Run the server
    let bind_address = config.bind_address();
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    info!("Server running on http://{}", bind_address);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> impl IntoResponse {
    Html(include_str!("../templates/index.html"))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut interval = interval(Duration::from_secs(1));
    let calendar_service = CalendarService::new_from_config(&state.config);

    loop {
        interval.tick().await;
        
        // Get regular meetings and active time blocks from ICS sources
        let meetings_result = calendar_service.get_current_and_next_meetings().await;
        let time_blocks_result = calendar_service.get_active_time_blocks().await;
        
        // Try to get Google Calendar events and merge them
        let google_meetings = match GoogleCalendarService::new_from_config(&state.config) {
            Ok(Some(mut google_service)) => {
                // Set stored tokens if available
                if let Ok(tokens_guard) = state.google_tokens.lock() {
                    if let Some(ref tokens) = *tokens_guard {
                        google_service.set_tokens(tokens.clone());
                    }
                }
                
                if google_service.is_authenticated() {
                    match google_service.get_calendar_events().await {
                        Ok(events) => {
                            tracing::info!("WebSocket: Fetched {} Google Calendar events", events.len());
                            events
                        },
                        Err(e) => {
                            tracing::warn!("WebSocket: Failed to fetch Google Calendar events: {}", e);
                            Vec::new()
                        }
                    }
                } else {
                    tracing::info!("WebSocket: Google Calendar not authenticated");
                    Vec::new()
                }
            }
            Ok(None) => {
                tracing::debug!("WebSocket: Google OAuth not configured");
                Vec::new()
            },
            Err(e) => {
                tracing::warn!("WebSocket: Failed to create Google Calendar service: {}", e);
                Vec::new()
            },
        };
        
        match (meetings_result, time_blocks_result) {
            (Ok((mut current, mut next)), Ok(active_time_blocks)) => {
                tracing::info!("WebSocket: ICS current: {:?}, ICS next: {:?}", 
                    current.as_ref().map(|m| &m.title), 
                    next.as_ref().map(|m| &m.title)
                );
                
                // Merge Google Calendar events with ICS events
                if !google_meetings.is_empty() {
                    // Find current/next from Google Calendar events
                    let google_current = google_meetings.iter().find(|m| m.is_active()).cloned();
                    let google_next = google_meetings.iter().find(|m| m.is_upcoming()).cloned();
                    
                    tracing::info!("WebSocket: Google current: {:?}, Google next: {:?}", 
                        google_current.as_ref().map(|m| &m.title), 
                        google_next.as_ref().map(|m| &m.title)
                    );
                    
                    // Prioritize Google Calendar events when available
                    // Use Google Calendar current event if it exists, or if no ICS current event, or if Google event is earlier
                    if google_current.is_some() && (current.is_none() || google_current.as_ref().unwrap().start_time < current.as_ref().unwrap().start_time) {
                        tracing::info!("WebSocket: Using Google current event: {}", google_current.as_ref().unwrap().title);
                        current = google_current;
                    }
                    
                    // Use Google Calendar next event if it exists, or if no ICS next event, or if Google event is earlier  
                    if google_next.is_some() && (next.is_none() || google_next.as_ref().unwrap().start_time < next.as_ref().unwrap().start_time) {
                        tracing::info!("WebSocket: Using Google next event: {}", google_next.as_ref().unwrap().title);
                        next = google_next;
                    }
                } else {
                    tracing::info!("WebSocket: No Google Calendar events to merge");
                }
                
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

async fn get_meetings(State(state): State<AppState>) -> impl IntoResponse {
    let calendar_service = CalendarService::new_from_config(&state.config);
    
    // Get regular meetings and active time blocks from ICS sources
    let meetings_result = calendar_service.get_current_and_next_meetings().await;
    let time_blocks_result = calendar_service.get_active_time_blocks().await;
    
    // Try to get Google Calendar events and merge them
    let google_meetings = match GoogleCalendarService::new_from_config(&state.config) {
        Ok(Some(mut google_service)) => {
            // Set stored tokens if available
            if let Ok(tokens_guard) = state.google_tokens.lock() {
                if let Some(ref tokens) = *tokens_guard {
                    google_service.set_tokens(tokens.clone());
                }
            }
            
            if google_service.is_authenticated() {
                match google_service.get_calendar_events().await {
                    Ok(events) => {
                        info!("Successfully fetched {} Google Calendar events", events.len());
                        events
                    },
                    Err(e) => {
                        warn!("Failed to fetch Google Calendar events: {}", e);
                        Vec::new()
                    }
                }
            } else {
                info!("Google Calendar not authenticated");
                Vec::new()
            }
        }
        Ok(None) => {
            info!("Google OAuth not configured");
            Vec::new()
        },
        Err(e) => {
            warn!("Failed to create Google Calendar service: {}", e);
            Vec::new()
        },
    };
    
    match (meetings_result, time_blocks_result) {
        (Ok((mut current, mut next)), Ok(active_time_blocks)) => {
            // Merge Google Calendar events with ICS events
            if !google_meetings.is_empty() {
                // Find current/next from Google Calendar events
                let google_current = google_meetings.iter().find(|m| m.is_active()).cloned();
                let google_next = google_meetings.iter().find(|m| m.is_upcoming()).cloned();
                
                // Prioritize Google Calendar events when available
                // Use Google Calendar current event if it exists, or if no ICS current event, or if Google event is earlier
                if google_current.is_some() && (current.is_none() || google_current.as_ref().unwrap().start_time < current.as_ref().unwrap().start_time) {
                    current = google_current;
                }
                
                // Use Google Calendar next event if it exists, or if no ICS next event, or if Google event is earlier
                if google_next.is_some() && (next.is_none() || google_next.as_ref().unwrap().start_time < next.as_ref().unwrap().start_time) {
                    next = google_next;
                }
            }
            
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

/// Google OAuth login endpoint
async fn google_auth_login(State(state): State<AppState>) -> impl IntoResponse {
    match GoogleCalendarService::new_from_config(&state.config) {
        Ok(Some(google_service)) => {
            let (auth_url, _csrf_token) = google_service.get_auth_url();
            // For now, just redirect to Google OAuth
            axum::response::Redirect::temporary(auth_url.as_str()).into_response()
        }
        Ok(None) => {
            Html("<h1>Google OAuth not configured</h1><p>Please configure Google OAuth in your TOML config file or set environment variables GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, and GOOGLE_REDIRECT_URI.</p>".to_string()).into_response()
        }
        Err(e) => {
            warn!("Failed to create Google OAuth client: {}", e);
            Html(format!("<h1>Error</h1><p>Failed to initialize Google OAuth: {}</p>", e)).into_response()
        }
    }
}

/// Google OAuth callback endpoint
async fn google_auth_callback(
    query: Query<HashMap<String, String>>,
    State(state): State<AppState>
) -> impl IntoResponse {
    if let (Some(code), Some(_state)) = (query.get("code"), query.get("state")) {
        info!("Received OAuth callback with authorization code");
        
        // Exchange authorization code for tokens
        match GoogleCalendarService::new_from_config(&state.config) {
            Ok(Some(mut google_service)) => {
                match google_service.exchange_code(code.clone(), oauth2::CsrfToken::new(_state.clone())).await {
                    Ok(()) => {
                        // Store tokens in global state
                        if let Some(tokens) = google_service.get_tokens() {
                            if let Ok(mut tokens_guard) = state.google_tokens.lock() {
                                *tokens_guard = Some(tokens);
                                info!("Successfully stored Google Calendar tokens");
                            }
                        }
                        
                        Html(format!(
                            "<h1>✅ Google Calendar Connected!</h1>
                             <p>Successfully authenticated with Google Calendar.</p>
                             <p>You can now see your Google Calendar events in the monitor.</p>
                             <p><a href='/'>← Back to Calendar Monitor</a></p>
                             <script>
                                setTimeout(() => window.location.href = '/', 3000);
                             </script>"
                        ))
                    }
                    Err(e) => {
                        warn!("Failed to exchange OAuth code for tokens: {}", e);
                        Html(format!(
                            "<h1>❌ Google OAuth Error</h1>
                             <p>Failed to exchange authorization code for tokens: {}</p>
                             <p><a href='/auth/google/login'>← Try again</a></p>",
                            e
                        ))
                    }
                }
            }
            Ok(None) => {
                Html("<h1>❌ Google OAuth not configured</h1><p>Please configure Google OAuth in your TOML config file or set environment variables.</p>".to_string())
            }
            Err(e) => {
                warn!("Failed to create Google OAuth client: {}", e);
                Html(format!(
                    "<h1>❌ Google OAuth Error</h1>
                     <p>Failed to initialize OAuth client: {}</p>
                     <p><a href='/auth/google/login'>← Try again</a></p>",
                    e
                ))
            }
        }
    } else if let Some(error) = query.get("error") {
        Html(format!(
            "<h1>❌ Google OAuth Error</h1>
             <p>Error: {}</p>
             <p><a href='/auth/google/login'>← Try again</a></p>",
            error
        ))
    } else {
        Html("<h1>❌ Invalid OAuth callback</h1><p>Missing required parameters.</p>".to_string())
    }
}

/// Debug endpoint to check Google authentication status
async fn google_auth_status(State(state): State<AppState>) -> impl IntoResponse {
    let mut response = format!("<h1>🔍 Google Calendar Debug Status</h1>");
    
    // Check environment variables
    let has_client_id = std::env::var("GOOGLE_CLIENT_ID").is_ok();
    let has_client_secret = std::env::var("GOOGLE_CLIENT_SECRET").is_ok();
    let has_redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").is_ok();
    
    response.push_str(&format!("<h2>Environment Variables:</h2>"));
    response.push_str(&format!("<ul>"));
    response.push_str(&format!("<li>GOOGLE_CLIENT_ID: {}</li>", if has_client_id { "✅ Set" } else { "❌ Missing" }));
    response.push_str(&format!("<li>GOOGLE_CLIENT_SECRET: {}</li>", if has_client_secret { "✅ Set" } else { "❌ Missing" }));
    response.push_str(&format!("<li>GOOGLE_REDIRECT_URI: {}</li>", if has_redirect_uri { "✅ Set" } else { "❌ Missing" }));
    response.push_str(&format!("</ul>"));
    
    // Check stored tokens
    let has_tokens = if let Ok(tokens_guard) = state.google_tokens.lock() {
        tokens_guard.is_some()
    } else {
        false
    };
    
    response.push_str(&format!("<h2>Authentication Status:</h2>"));
    response.push_str(&format!("<ul>"));
    response.push_str(&format!("<li>Stored Tokens: {}</li>", if has_tokens { "✅ Available" } else { "❌ None" }));
    response.push_str(&format!("</ul>"));
    
    // Test Google Calendar service creation
    let service_status = match GoogleCalendarService::new_from_config(&state.config) {
        Ok(Some(mut google_service)) => {
            // Try to restore tokens
            if let Ok(tokens_guard) = state.google_tokens.lock() {
                if let Some(ref tokens) = *tokens_guard {
                    google_service.set_tokens(tokens.clone());
                    format!("✅ Service created and tokens restored")
                } else {
                    format!("⚠️ Service created but no tokens to restore")
                }
            } else {
                format!("❌ Service created but failed to access token store")
            }
        },
        Ok(None) => format!("❌ Service creation returned None (environment issue)"),
        Err(e) => format!("❌ Service creation failed: {}", e),
    };
    
    response.push_str(&format!("<h2>Service Status:</h2>"));
    response.push_str(&format!("<ul><li>{}</li></ul>", service_status));
    
    response.push_str(&format!("<p><a href='/'>← Back to Calendar</a> | <a href='/auth/google/login'>🔗 Connect Google Calendar</a></p>"));
    
    Html(response)
}

/// Serve embedded CSS file
async fn serve_css() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/css".parse().unwrap());
    (StatusCode::OK, headers, STYLE_CSS)
}

/// Serve embedded JavaScript file
async fn serve_js() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/javascript".parse().unwrap());
    (StatusCode::OK, headers, APP_JS)
}
