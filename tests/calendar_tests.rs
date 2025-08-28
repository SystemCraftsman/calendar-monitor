use calendar_monitor::calendar::CalendarService;
use calendar_monitor::meeting::Meeting;
use calendar_monitor::config::{Config, ServerConfig, IcsConfig, GoogleConfig};
use chrono::{NaiveDate, Utc};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cache_ttl_seconds: 300,
            },
            ics: IcsConfig {
                file_paths: vec![
                    "test_calendar1.ics".to_string(),
                    "test_calendar2.ics".to_string(),
                ],
            },
            google: GoogleConfig {
                client_id: None,
                client_secret: None,
                redirect_uri: None,
            },
        }
    }

    fn create_test_service() -> CalendarService {
        let config = create_test_config();
        CalendarService::new_from_config(&config)
    }

    #[test]
    fn test_calendar_service_from_config() {
        let config = create_test_config();
        let _service = CalendarService::new_from_config(&config);
        
        // Test that the service is created successfully
        // Since fields are private, we test that the service can be instantiated
        // and doesn't panic - this verifies the config is properly processed
        // We could test further by calling get_meetings_in_range if we had test ICS data
    }

    #[test]
    fn test_calendar_service_from_config_no_ics() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cache_ttl_seconds: 600,
            },
            ics: IcsConfig {
                file_paths: vec![], // Empty file paths
            },
            google: GoogleConfig {
                client_id: None,
                client_secret: None,
                redirect_uri: None,
            },
        };
        
        let _service = CalendarService::new_from_config(&config);
        
        // Test that the service is created successfully with empty ICS config
        // Since fields are private, we verify it doesn't panic on creation
    }

    #[test]
    fn test_parse_rrule_until() {
        let service = create_test_service();
        
        // Test valid UNTIL date
        let rrule = "FREQ=WEEKLY;WKST=MO;UNTIL=20250620T235959Z;BYDAY=MO,TU,WE,TH,FR";
        let until_date = service.parse_rrule_until(rrule);
        assert_eq!(until_date, Some(NaiveDate::from_ymd_opt(2025, 6, 20).unwrap()));
        
        // Test UNTIL without time component
        let rrule2 = "FREQ=WEEKLY;UNTIL=20251225";
        let until_date2 = service.parse_rrule_until(rrule2);
        assert_eq!(until_date2, Some(NaiveDate::from_ymd_opt(2025, 12, 25).unwrap()));
        
        // Test RRULE without UNTIL
        let rrule3 = "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR";
        let until_date3 = service.parse_rrule_until(rrule3);
        assert_eq!(until_date3, None);
        
        // Test invalid UNTIL format
        let rrule4 = "FREQ=WEEKLY;UNTIL=invalid-date";
        let until_date4 = service.parse_rrule_until(rrule4);
        assert_eq!(until_date4, None);
    }

    #[test]
    fn test_meeting_status() {
        let now = Utc::now();
        
        // Test current meeting (started but not ended)
        let current_meeting = Meeting {
            title: "Current Meeting".to_string(),
            start_time: now - chrono::Duration::minutes(30),
            end_time: now + chrono::Duration::minutes(30),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        assert!(current_meeting.is_active());
        assert!(!current_meeting.is_upcoming());
        assert!(!current_meeting.has_ended());
        
        // Test upcoming meeting
        let upcoming_meeting = Meeting {
            title: "Upcoming Meeting".to_string(),
            start_time: now + chrono::Duration::minutes(30),
            end_time: now + chrono::Duration::minutes(90),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        assert!(!upcoming_meeting.is_active());
        assert!(upcoming_meeting.is_upcoming());
        assert!(!upcoming_meeting.has_ended());
        
        // Test ended meeting
        let ended_meeting = Meeting {
            title: "Ended Meeting".to_string(),
            start_time: now - chrono::Duration::minutes(90),
            end_time: now - chrono::Duration::minutes(30),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        assert!(!ended_meeting.is_active());
        assert!(!ended_meeting.is_upcoming());
        assert!(ended_meeting.has_ended());
    }

    #[test]
    fn test_time_block_detection() {
        let meeting1 = Meeting {
            title: "[Time Block]".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        assert!(meeting1.is_time_block());
        
        let meeting2 = Meeting {
            title: "Regular Meeting".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(1),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        assert!(!meeting2.is_time_block());
    }

    #[test]
    fn test_meeting_countdown() {
        let now = Utc::now();
        let meeting = Meeting {
            title: "Test Meeting".to_string(),
            start_time: now - chrono::Duration::minutes(30),
            end_time: now + chrono::Duration::minutes(30),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        
        let countdown = meeting.time_until_end();
        assert!(countdown > 25 * 60); // Should be around 30 minutes (1800 seconds)
        assert!(countdown < 35 * 60); // But less than 35 minutes due to test execution time
    }

    #[test]
    fn test_meeting_builder() {
        let meeting = Meeting::new(
            "Test Meeting".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1),
        )
        .with_description("Test description".to_string())
        .with_location("Test location".to_string());
        
        assert_eq!(meeting.title, "Test Meeting");
        assert_eq!(meeting.description, Some("Test description".to_string()));
        assert_eq!(meeting.location, Some("Test location".to_string()));
    }
}
