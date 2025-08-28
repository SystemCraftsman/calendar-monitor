use calendar_monitor::google_calendar::{GoogleCalendarService, GoogleOAuthConfig, GoogleCalendarEvent, GoogleEventTime, GoogleEventAttendee};
use calendar_monitor::config::{Config, ServerConfig, IcsConfig, GoogleConfig};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config_with_oauth() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cache_ttl_seconds: 300,
            },
            ics: IcsConfig {
                file_paths: vec![],
            },
            google: GoogleConfig {
                client_id: Some("test_client_id".to_string()),
                client_secret: Some("test_client_secret".to_string()),
                redirect_uri: Some("http://localhost:3000/auth/google/callback".to_string()),
            },
        }
    }

    fn create_test_config_without_oauth() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cache_ttl_seconds: 300,
            },
            ics: IcsConfig {
                file_paths: vec![],
            },
            google: GoogleConfig {
                client_id: None,
                client_secret: None,
                redirect_uri: None,
            },
        }
    }

    #[test]
    fn test_google_calendar_service_from_config_with_oauth() {
        let config = create_test_config_with_oauth();
        let result = GoogleCalendarService::new_from_config(&config);
        
        assert!(result.is_ok());
        let service_option = result.unwrap();
        assert!(service_option.is_some());
        
        let service = service_option.unwrap();
        assert!(!service.is_authenticated()); // No tokens initially
    }

    #[test]
    fn test_google_calendar_service_from_config_without_oauth() {
        let config = create_test_config_without_oauth();
        let result = GoogleCalendarService::new_from_config(&config);
        
        assert!(result.is_ok());
        let service_option = result.unwrap();
        assert!(service_option.is_none()); // Should return None when no OAuth config
    }

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
    fn test_google_calendar_service_legacy_env_creation_missing_vars() {
        // Test legacy new_from_env method with missing environment variables
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
    fn test_google_calendar_service_legacy_env_creation_with_vars() {
        // Test legacy new_from_env method with environment variables
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

    fn create_test_google_event_with_attendees(attendees: Option<Vec<GoogleEventAttendee>>) -> GoogleCalendarEvent {
        GoogleCalendarEvent {
            id: "test_event_id".to_string(),
            summary: Some("Test Meeting".to_string()),
            start: Some(GoogleEventTime {
                date_time: Some("2024-01-15T10:00:00Z".to_string()),
                date: None,
                time_zone: None,
            }),
            end: Some(GoogleEventTime {
                date_time: Some("2024-01-15T11:00:00Z".to_string()),
                date: None,
                time_zone: None,
            }),
            description: Some("Test description".to_string()),
            location: Some("Test location".to_string()),
            attendees,
        }
    }

    #[test]
    fn test_google_event_attendee_structures() {
        // Test accepted attendee
        let accepted_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("accepted".to_string()),
            is_self: Some(true),
        };
        
        assert_eq!(accepted_attendee.response_status, Some("accepted".to_string()));
        assert_eq!(accepted_attendee.is_self, Some(true));
        
        // Test declined attendee
        let declined_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("declined".to_string()),
            is_self: Some(true),
        };
        
        assert_eq!(declined_attendee.response_status, Some("declined".to_string()));
        
        // Test tentative attendee
        let tentative_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("tentative".to_string()),
            is_self: Some(true),
        };
        
        assert_eq!(tentative_attendee.response_status, Some("tentative".to_string()));
        
        // Test no response attendee
        let no_response_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("needsAction".to_string()),
            is_self: Some(true),
        };
        
        assert_eq!(no_response_attendee.response_status, Some("needsAction".to_string()));
    }

    #[test]
    fn test_google_event_with_different_response_statuses() {
        // Test event with accepted status
        let accepted_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("accepted".to_string()),
            is_self: Some(true),
        };
        let accepted_event = create_test_google_event_with_attendees(Some(vec![accepted_attendee]));
        assert_eq!(accepted_event.attendees.as_ref().unwrap()[0].response_status, Some("accepted".to_string()));

        // Test event with declined status
        let declined_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("declined".to_string()),
            is_self: Some(true),
        };
        let declined_event = create_test_google_event_with_attendees(Some(vec![declined_attendee]));
        assert_eq!(declined_event.attendees.as_ref().unwrap()[0].response_status, Some("declined".to_string()));

        // Test event with tentative status
        let tentative_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("tentative".to_string()),
            is_self: Some(true),
        };
        let tentative_event = create_test_google_event_with_attendees(Some(vec![tentative_attendee]));
        assert_eq!(tentative_event.attendees.as_ref().unwrap()[0].response_status, Some("tentative".to_string()));

        // Test event with no response status
        let no_response_attendee = GoogleEventAttendee {
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            response_status: Some("needsAction".to_string()),
            is_self: Some(true),
        };
        let no_response_event = create_test_google_event_with_attendees(Some(vec![no_response_attendee]));
        assert_eq!(no_response_event.attendees.as_ref().unwrap()[0].response_status, Some("needsAction".to_string()));

        // Test event with no attendees (personal event)
        let personal_event = create_test_google_event_with_attendees(None);
        assert!(personal_event.attendees.is_none());
    }

    #[test]
    fn test_google_event_time_parsing() {
        let event_time = GoogleEventTime {
            date_time: Some("2024-01-15T10:00:00Z".to_string()),
            date: None,
            time_zone: None,
        };
        
        assert_eq!(event_time.date_time, Some("2024-01-15T10:00:00Z".to_string()));
        assert!(event_time.date.is_none());
    }
}
