use calendar_monitor::calendar::CalendarService;
use calendar_monitor::meeting::{Meeting, ResponseStatus};
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
        let current_meeting = Meeting::new(
            "Current Meeting".to_string(),
            now - chrono::Duration::minutes(30),
            now + chrono::Duration::minutes(30)
        );
        assert!(current_meeting.is_active());
        assert!(!current_meeting.is_upcoming());
        assert!(!current_meeting.has_ended());
        
        // Test upcoming meeting
        let upcoming_meeting = Meeting::new(
            "Upcoming Meeting".to_string(),
            now + chrono::Duration::minutes(30),
            now + chrono::Duration::minutes(90)
        );
        assert!(!upcoming_meeting.is_active());
        assert!(upcoming_meeting.is_upcoming());
        assert!(!upcoming_meeting.has_ended());
        
        // Test ended meeting
        let ended_meeting = Meeting::new(
            "Ended Meeting".to_string(),
            now - chrono::Duration::minutes(90),
            now - chrono::Duration::minutes(30)
        );
        assert!(!ended_meeting.is_active());
        assert!(!ended_meeting.is_upcoming());
        assert!(ended_meeting.has_ended());
    }

    #[test]
    fn test_time_block_detection() {
        let meeting1 = Meeting::new(
            "[Time Block]".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        );
        assert!(meeting1.is_time_block());
        
        let meeting2 = Meeting::new(
            "Regular Meeting".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        );
        assert!(!meeting2.is_time_block());
    }

    #[test]
    fn test_meeting_countdown() {
        let now = Utc::now();
        let meeting = Meeting::new(
            "Test Meeting".to_string(),
            now - chrono::Duration::minutes(30),
            now + chrono::Duration::minutes(30)
        );
        
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

    #[test]
    fn test_ics_attendee_status_parsing() {
        // Test different PARTSTAT values that would be found in ICS ATTENDEE properties
        // Note: This test is conceptual since we can't easily create mock ical::property::Property objects
        // In practice, this functionality is tested through integration with real ICS files
        
        // Test that a meeting with response status can be created and filtered
        let accepted_meeting = Meeting::new(
            "Accepted ICS Event".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Accepted);
        
        assert!(accepted_meeting.should_display());
        assert_eq!(accepted_meeting.response_status_label(), None);
        
        let declined_meeting = Meeting::new(
            "Declined ICS Event".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Declined);
        
        assert!(!declined_meeting.should_display());
        assert_eq!(declined_meeting.response_status_label(), Some("Declined".to_string()));
        
        let tentative_meeting = Meeting::new(
            "Tentative ICS Event".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Tentative);
        
        assert!(tentative_meeting.should_display());
        assert_eq!(tentative_meeting.response_status_label(), Some("Tentative".to_string()));
        
        let no_response_meeting = Meeting::new(
            "No Response ICS Event".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::NoResponse);
        
        assert!(no_response_meeting.should_display());
        assert_eq!(no_response_meeting.response_status_label(), Some("Not Responded".to_string()));
    }

    #[test]
    fn test_ics_filtering_consistency_with_google_calendar() {
        // Test that ICS events with different response statuses behave the same as Google Calendar events
        let test_cases = [
            (ResponseStatus::Accepted, true, None),
            (ResponseStatus::Declined, false, Some("Declined".to_string())),
            (ResponseStatus::Tentative, true, Some("Tentative".to_string())),
            (ResponseStatus::NoResponse, true, Some("Not Responded".to_string())),
        ];
        
        for (status, should_display, expected_label) in test_cases {
            let meeting = Meeting::new(
                format!("ICS Event - {:?}", status),
                Utc::now(),
                Utc::now() + chrono::Duration::hours(1)
            ).with_response_status(status.clone());
            
            assert_eq!(meeting.should_display(), should_display, 
                "Response status {:?} should_display mismatch", status);
            assert_eq!(meeting.response_status_label(), expected_label,
                "Response status {:?} label mismatch", status);
        }
    }

    #[test]
    fn test_ics_events_without_response_status() {
        // Test that ICS events without ATTENDEE/PARTSTAT behave like regular calendar events
        let meeting_no_status = Meeting::new(
            "Regular ICS Event".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(1)
        );
        
        // Should display (no filtering)
        assert!(meeting_no_status.should_display());
        // Should have no label (clean display like accepted events)
        assert_eq!(meeting_no_status.response_status_label(), None);
        // Should have no response status (None)
        assert_eq!(meeting_no_status.response_status, None);
    }

    #[test]
    fn test_ics_attendee_property_parsing() {
        use ical::property::Property;
        
        let service = create_test_service();
        
        // Test PARTSTAT=ACCEPTED
        let accepted_property = Property {
            name: "ATTENDEE".to_string(),
            params: Some(vec![
                ("PARTSTAT".to_string(), vec!["ACCEPTED".to_string()]),
                ("CN".to_string(), vec!["John Doe".to_string()]),
            ]),
            value: Some("mailto:john@example.com".to_string()),
        };
        
        let result = service.parse_ical_attendee_status(&accepted_property);
        assert_eq!(result, Some(ResponseStatus::Accepted));
        
        // Test PARTSTAT=DECLINED
        let declined_property = Property {
            name: "ATTENDEE".to_string(),
            params: Some(vec![
                ("PARTSTAT".to_string(), vec!["DECLINED".to_string()]),
                ("CN".to_string(), vec!["Jane Doe".to_string()]),
            ]),
            value: Some("mailto:jane@example.com".to_string()),
        };
        
        let result = service.parse_ical_attendee_status(&declined_property);
        assert_eq!(result, Some(ResponseStatus::Declined));
        
        // Test PARTSTAT=TENTATIVE
        let tentative_property = Property {
            name: "ATTENDEE".to_string(),
            params: Some(vec![
                ("PARTSTAT".to_string(), vec!["TENTATIVE".to_string()]),
                ("CN".to_string(), vec!["Bob Smith".to_string()]),
            ]),
            value: Some("mailto:bob@example.com".to_string()),
        };
        
        let result = service.parse_ical_attendee_status(&tentative_property);
        assert_eq!(result, Some(ResponseStatus::Tentative));
        
        // Test PARTSTAT=NEEDS-ACTION
        let needs_action_property = Property {
            name: "ATTENDEE".to_string(),
            params: Some(vec![
                ("PARTSTAT".to_string(), vec!["NEEDS-ACTION".to_string()]),
                ("CN".to_string(), vec!["Alice Johnson".to_string()]),
            ]),
            value: Some("mailto:alice@example.com".to_string()),
        };
        
        let result = service.parse_ical_attendee_status(&needs_action_property);
        assert_eq!(result, Some(ResponseStatus::NoResponse));
        
        // Test property without PARTSTAT parameter
        let no_partstat_property = Property {
            name: "ATTENDEE".to_string(),
            params: Some(vec![
                ("CN".to_string(), vec!["No Status User".to_string()]),
            ]),
            value: Some("mailto:nostatus@example.com".to_string()),
        };
        
        let result = service.parse_ical_attendee_status(&no_partstat_property);
        assert_eq!(result, None);
        
        // Test property with no params
        let no_params_property = Property {
            name: "ATTENDEE".to_string(),
            params: None,
            value: Some("mailto:noparams@example.com".to_string()),
        };
        
        let result = service.parse_ical_attendee_status(&no_params_property);
        assert_eq!(result, None);
    }

    #[test]
    fn test_ics_event_filtering_integration() {
        use ical::parser::ical::component::IcalEvent;
        use ical::property::Property;
        
        let service = create_test_service();
        
        // Create a mock ICS event with PARTSTAT=DECLINED
        let declined_event = IcalEvent {
            properties: vec![
                Property {
                    name: "SUMMARY".to_string(),
                    params: None,
                    value: Some("Declined Meeting".to_string()),
                },
                Property {
                    name: "DTSTART".to_string(),
                    params: None,
                    value: Some("20240115T100000Z".to_string()),
                },
                Property {
                    name: "DTEND".to_string(),
                    params: None,
                    value: Some("20240115T110000Z".to_string()),
                },
                Property {
                    name: "ATTENDEE".to_string(),
                    params: Some(vec![
                        ("PARTSTAT".to_string(), vec!["DECLINED".to_string()]),
                        ("CN".to_string(), vec!["Test User".to_string()]),
                    ]),
                    value: Some("mailto:test@example.com".to_string()),
                },
            ],
            alarms: vec![],
        };
        
        // Test that declined event returns empty vec (filtered out)
        let result = service.convert_ical_event_to_meeting(declined_event);
        assert!(result.is_ok());
        let meetings = result.unwrap();
        assert_eq!(meetings.len(), 0, "Declined ICS events should be filtered out");
        
        // Create a mock ICS event with PARTSTAT=ACCEPTED
        let accepted_event = IcalEvent {
            properties: vec![
                Property {
                    name: "SUMMARY".to_string(),
                    params: None,
                    value: Some("Accepted Meeting".to_string()),
                },
                Property {
                    name: "DTSTART".to_string(),
                    params: None,
                    value: Some("20240115T100000Z".to_string()),
                },
                Property {
                    name: "DTEND".to_string(),
                    params: None,
                    value: Some("20240115T110000Z".to_string()),
                },
                Property {
                    name: "ATTENDEE".to_string(),
                    params: Some(vec![
                        ("PARTSTAT".to_string(), vec!["ACCEPTED".to_string()]),
                        ("CN".to_string(), vec!["Test User".to_string()]),
                    ]),
                    value: Some("mailto:test@example.com".to_string()),
                },
            ],
            alarms: vec![],
        };
        
        // Test that accepted event is parsed and included
        let result = service.convert_ical_event_to_meeting(accepted_event);
        assert!(result.is_ok());
        let meetings = result.unwrap();
        assert_eq!(meetings.len(), 1, "Accepted ICS events should be included");
        
        let meeting = &meetings[0];
        assert_eq!(meeting.title, "Accepted Meeting");
        assert_eq!(meeting.response_status, Some(ResponseStatus::Accepted));
        assert!(meeting.should_display());
        assert_eq!(meeting.response_status_label(), None);
    }
}
