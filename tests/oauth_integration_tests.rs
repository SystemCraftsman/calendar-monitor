use calendar_monitor::google_calendar::{GoogleCalendarService, GoogleOAuthConfig, GoogleTokens};
use chrono::Utc;

#[test]
fn test_oauth_token_persistence() {
    let config = GoogleOAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
    };
    
    let mut service = GoogleCalendarService::new(config).unwrap();
    
    // Initially not authenticated
    assert!(!service.is_authenticated());
    
    // Set tokens
    let tokens = GoogleTokens {
        access_token: "test_access_token".to_string(),
        refresh_token: Some("test_refresh_token".to_string()),
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
    };
    
    service.set_tokens(tokens.clone());
    
    // Now should be authenticated
    assert!(service.is_authenticated());
    
    // Should be able to get tokens back
    let retrieved_tokens = service.get_tokens().unwrap();
    assert_eq!(retrieved_tokens.access_token, tokens.access_token);
    assert_eq!(retrieved_tokens.refresh_token, tokens.refresh_token);
}

#[test]
fn test_oauth_url_generation() {
    let config = GoogleOAuthConfig {
        client_id: "test_client_id".to_string(),
        client_secret: "test_client_secret".to_string(),
        redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
    };
    
    let service = GoogleCalendarService::new(config).unwrap();
    let (auth_url, _csrf_token) = service.get_auth_url();
    
    let auth_url_string = auth_url.to_string();
    assert!(auth_url_string.contains("accounts.google.com"));
    assert!(auth_url_string.contains("calendar.readonly"));
    assert!(auth_url_string.contains("test_client_id"));
}
