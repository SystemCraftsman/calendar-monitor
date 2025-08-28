use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, AccessToken, RefreshToken,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

use crate::meeting::Meeting;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarEvent {
    pub id: String,
    pub summary: Option<String>,
    pub start: Option<GoogleEventTime>,
    pub end: Option<GoogleEventTime>,
    pub description: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleEventTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarResponse {
    pub items: Option<Vec<GoogleCalendarEvent>>,
}

pub struct GoogleCalendarService {
    client: BasicClient,
    tokens: Option<GoogleTokens>,
    http_client: reqwest::Client,
}

impl GoogleCalendarService {
    /// Create a new Google Calendar service using OAuth
    pub fn new(config: GoogleOAuthConfig) -> Result<Self> {
        let client = BasicClient::new(
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
            AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?,
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(config.redirect_uri)?);

        let http_client = reqwest::Client::new();

        tracing::info!("Successfully initialized Google Calendar OAuth client");

        Ok(Self {
            client,
            tokens: None,
            http_client,
        })
    }

    /// Create OAuth authorization URL
    pub fn get_auth_url(&self) -> (Url, CsrfToken) {
        self.client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/calendar.readonly".to_string()))
            .url()
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(&mut self, code: String, _csrf_token: CsrfToken) -> Result<()> {
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| anyhow!("OAuth token exchange failed: {}", e))?;

        let expires_at = token_result.expires_in().map(|duration| {
            Utc::now() + chrono::Duration::seconds(duration.as_secs() as i64)
        });

        self.tokens = Some(GoogleTokens {
            access_token: token_result.access_token().secret().clone(),
            refresh_token: token_result.refresh_token().map(|rt| rt.secret().clone()),
            expires_at,
        });

        tracing::info!("Successfully exchanged OAuth code for access token");
        Ok(())
    }

    /// Check if we have valid tokens
    pub fn is_authenticated(&self) -> bool {
        self.tokens.is_some()
    }

    /// Set tokens (for restoring from storage)
    pub fn set_tokens(&mut self, tokens: GoogleTokens) {
        self.tokens = Some(tokens);
    }

    /// Get current tokens
    pub fn get_tokens(&self) -> Option<GoogleTokens> {
        self.tokens.clone()
    }

    /// Create a new Google Calendar service from environment variables (OAuth)
    pub fn new_from_env() -> Result<Option<Self>> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID");
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET"); 
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI");

        match (client_id, client_secret, redirect_uri) {
            (Ok(client_id), Ok(client_secret), Ok(redirect_uri)) => {
                let config = GoogleOAuthConfig {
                    client_id,
                    client_secret,
                    redirect_uri,
                };
                match Self::new(config) {
                    Ok(service) => Ok(Some(service)),
                    Err(e) => Err(e),
                }
            }
            _ => {
                tracing::debug!("Google Calendar OAuth environment variables not found");
                Ok(None)
            }
        }
    }

    /// Create a new Google Calendar service from configuration (TOML or env)
    pub fn new_from_config(config: &crate::config::Config) -> Result<Option<Self>> {
        // Try to get Google OAuth config from the config struct
        if let (Some(client_id), Some(client_secret), Some(redirect_uri)) = 
            (&config.google.client_id, &config.google.client_secret, &config.google.redirect_uri) {
            let oauth_config = GoogleOAuthConfig {
                client_id: client_id.clone(),
                client_secret: client_secret.clone(),
                redirect_uri: redirect_uri.clone(),
            };
            match Self::new(oauth_config) {
                Ok(service) => Ok(Some(service)),
                Err(e) => Err(e),
            }
        } else {
            tracing::debug!("Google Calendar OAuth not configured in TOML config");
            Ok(None) // OAuth not configured
        }
    }

    /// Get calendar events for today and tomorrow
    pub async fn get_calendar_events(&self) -> Result<Vec<Meeting>> {
        let tokens = self.tokens.as_ref()
            .ok_or_else(|| anyhow!("No OAuth tokens available. Please authenticate first."))?;

        let now = Utc::now();
        let time_min = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let time_max = (now + chrono::Duration::days(1)).format("%Y-%m-%dT%H:%M:%SZ").to_string();

        tracing::debug!("Fetching Google Calendar events from {} to {}", time_min, time_max);

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime&maxResults=50",
            urlencoding::encode(&time_min),
            urlencoding::encode(&time_max)
        );

        let response = self.http_client
            .get(&url)
            .bearer_auth(&tokens.access_token)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch Google Calendar events: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Google Calendar API error {}: {}", status, text));
        }

        let calendar_response: GoogleCalendarResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Google Calendar response: {}", e))?;

        let meetings = self.convert_google_events_to_meetings(calendar_response)?;
        tracing::info!("Successfully fetched {} events from Google Calendar", meetings.len());
        Ok(meetings)
    }

    /// Convert Google Calendar events to our Meeting format
    fn convert_google_events_to_meetings(&self, response: GoogleCalendarResponse) -> Result<Vec<Meeting>> {
        let mut meetings = Vec::new();

        if let Some(events) = response.items {
            for event in events {
                if let Some(meeting) = self.convert_single_event_to_meeting(event)? {
                    meetings.push(meeting);
                }
            }
        }

        Ok(meetings)
    }

    /// Convert a single Google Calendar event to our Meeting struct
    fn convert_single_event_to_meeting(&self, event: GoogleCalendarEvent) -> Result<Option<Meeting>> {
        // Skip events without start/end times (all-day events, etc.)
        let (start_time, end_time) = match (event.start, event.end) {
            (Some(start), Some(end)) => {
                match (start.date_time, end.date_time) {
                    (Some(start_dt), Some(end_dt)) => {
                        let start_parsed = DateTime::parse_from_rfc3339(&start_dt)
                            .map_err(|e| anyhow!("Failed to parse start time: {}", e))?
                            .with_timezone(&Utc);
                        let end_parsed = DateTime::parse_from_rfc3339(&end_dt)
                            .map_err(|e| anyhow!("Failed to parse end time: {}", e))?
                            .with_timezone(&Utc);
                        (start_parsed, end_parsed)
                    }
                    _ => {
                        // Skip all-day events or events without proper times
                        tracing::debug!("Skipping all-day event: {:?}", event.summary);
                        return Ok(None);
                    }
                }
            }
            _ => {
                tracing::debug!("Skipping event without start/end times: {:?}", event.summary);
                return Ok(None);
            }
        };

        // Create the meeting
        let title = event.summary.unwrap_or_else(|| "Untitled Event".to_string());
        let mut meeting = Meeting::new(title, start_time, end_time);

        // Add description if available
        if let Some(description) = event.description {
            meeting = meeting.with_description(description);
        }

        // Add location if available
        if let Some(location) = event.location {
            meeting = meeting.with_location(location);
        }

        tracing::debug!("Converted Google event: {} ({} - {})", 
            meeting.title, 
            meeting.start_time.format("%H:%M"), 
            meeting.end_time.format("%H:%M"));

        Ok(Some(meeting))
    }
}