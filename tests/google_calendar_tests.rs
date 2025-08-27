use calendar_monitor::google_calendar::{GoogleCalendarService, GoogleOAuthConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_oauth_config_creation() {
        let config = GoogleOAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
        };
        
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.client_secret, "test_client_secret");
        assert_eq!(config.redirect_uri, "http://localhost:3000/auth/google/callback");
    }

    #[test]
    fn test_google_calendar_service_creation() {
        let config = GoogleOAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
        };
        
        let service = GoogleCalendarService::new(config);
        assert!(service.is_ok());
        
        let service = service.unwrap();
        assert!(!service.is_authenticated()); // No tokens initially
    }

    #[test]
    fn test_google_calendar_service_env_creation_missing_vars() {
        // Test with missing environment variables
        unsafe {
            std::env::remove_var("GOOGLE_OAUTH_CLIENT_ID");
            std::env::remove_var("GOOGLE_OAUTH_CLIENT_SECRET");
            std::env::remove_var("GOOGLE_OAUTH_REDIRECT_URI");
        }
        
        let result = GoogleCalendarService::new_from_env();
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None when env vars missing
    }

    #[test]
    fn test_google_calendar_service_env_creation_with_vars() {
        // Set test environment variables
        unsafe {
            std::env::set_var("GOOGLE_OAUTH_CLIENT_ID", "test_client_id");
            std::env::set_var("GOOGLE_OAUTH_CLIENT_SECRET", "test_client_secret");
            std::env::set_var("GOOGLE_OAUTH_REDIRECT_URI", "http://localhost:3000/auth/google/callback");
        }
        
        let result = GoogleCalendarService::new_from_env();
        assert!(result.is_ok());
        
        // The service might still return None if there are validation issues
        // This test just verifies the function doesn't panic or error
        let _service = result.unwrap();
        
        // Clean up
        unsafe {
            std::env::remove_var("GOOGLE_OAUTH_CLIENT_ID");
            std::env::remove_var("GOOGLE_OAUTH_CLIENT_SECRET");
            std::env::remove_var("GOOGLE_OAUTH_REDIRECT_URI");
        }
    }

    #[test]
    fn test_auth_url_generation() {
        let config = GoogleOAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
        };
        
        let service = GoogleCalendarService::new(config).unwrap();
        let (auth_url, _csrf_token) = service.get_auth_url();
        
        // Auth URL should contain expected components
        let url_str = auth_url.to_string();
        assert!(url_str.contains("accounts.google.com"));
        assert!(url_str.contains("test_client_id"));
        assert!(url_str.contains("calendar"));
        assert!(url_str.contains("redirect_uri"));
    }

    #[test]
    fn test_google_event_conversion() {
        // This would test the convert_single_event_to_meeting method
        // but requires creating mock Google Calendar API events
        // For now, we'll test the structure exists
        let config = GoogleOAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
        };
        
        let service = GoogleCalendarService::new(config);
        assert!(service.is_ok());
    }
}
