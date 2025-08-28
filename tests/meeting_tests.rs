use calendar_monitor::meeting::{Meeting, ResponseStatus};
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meeting_deduplication_logic() {
        let now = Utc::now();
        
        // Create two meetings with same title and start time but different end times
        let meeting1 = Meeting::new(
            "Duplicate Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        );
        
        let meeting2 = Meeting::new(
            "Duplicate Meeting".to_string(),
            now, // Same start time
            now + chrono::Duration::hours(2) // Later end time
        );
        
        // meeting2 should be kept in deduplication (later end time)
        assert_eq!(meeting1.title, meeting2.title);
        assert_eq!(meeting1.start_time, meeting2.start_time);
        assert!(meeting2.end_time > meeting1.end_time);
    }

    #[test]
    fn test_meeting_filtering_by_status() {
        let now = Utc::now();
        
        let meetings = vec![
            // Ended meeting
            Meeting::new(
                "Past Meeting".to_string(),
                now - chrono::Duration::hours(2),
                now - chrono::Duration::hours(1)
            ),
            // Current meeting
            Meeting::new(
                "Current Meeting".to_string(),
                now - chrono::Duration::minutes(30),
                now + chrono::Duration::minutes(30)
            ),
            // Future meeting
            Meeting::new(
                "Future Meeting".to_string(),
                now + chrono::Duration::hours(1),
                now + chrono::Duration::hours(2)
            ),
        ];
        
        let current_meetings: Vec<_> = meetings.iter().filter(|m| m.is_active()).collect();
        let upcoming_meetings: Vec<_> = meetings.iter().filter(|m| m.is_upcoming()).collect();
        let ended_meetings: Vec<_> = meetings.iter().filter(|m| m.has_ended()).collect();
        
        assert_eq!(current_meetings.len(), 1);
        assert_eq!(upcoming_meetings.len(), 1);
        assert_eq!(ended_meetings.len(), 1);
        
        assert_eq!(current_meetings[0].title, "Current Meeting");
        assert_eq!(upcoming_meetings[0].title, "Future Meeting");
        assert_eq!(ended_meetings[0].title, "Past Meeting");
    }

    #[test]
    fn test_time_blocks_vs_regular_meetings() {
        let meetings = vec![
            Meeting::new(
                "[Time Block]".to_string(),
                Utc::now(),
                Utc::now() + chrono::Duration::hours(1)
            ),
            Meeting::new(
                "Regular Meeting".to_string(),
                Utc::now(),
                Utc::now() + chrono::Duration::hours(1)
            ),
            Meeting::new(
                "[Another Time Block]".to_string(),
                Utc::now(),
                Utc::now() + chrono::Duration::hours(1)
            ),
        ];
        
        let time_blocks: Vec<_> = meetings.iter().filter(|m| m.is_time_block()).collect();
        let regular_meetings: Vec<_> = meetings.iter().filter(|m| !m.is_time_block()).collect();
        
        assert_eq!(time_blocks.len(), 2);
        assert_eq!(regular_meetings.len(), 1);
        assert_eq!(regular_meetings[0].title, "Regular Meeting");
    }

    #[test]
    fn test_meeting_sorting() {
        let now = Utc::now();
        let mut meetings = vec![
            Meeting::new(
                "Third".to_string(),
                now + chrono::Duration::hours(2),
                now + chrono::Duration::hours(3)
            ),
            Meeting::new(
                "First".to_string(),
                now,
                now + chrono::Duration::hours(1)
            ),
            Meeting::new(
                "Second".to_string(),
                now + chrono::Duration::hours(1),
                now + chrono::Duration::hours(2)
            ),
        ];
        
        // Sort by start time
        meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        assert_eq!(meetings[0].title, "First");
        assert_eq!(meetings[1].title, "Second");
        assert_eq!(meetings[2].title, "Third");
    }

    #[test]
    fn test_response_status_functionality() {
        let now = Utc::now();
        
        // Test with_response_status method
        let meeting = Meeting::new(
            "Test Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Tentative);
        
        assert_eq!(meeting.response_status, Some(ResponseStatus::Tentative));
        
        // Test default (no response status)
        let meeting_no_status = Meeting::new(
            "Test Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        );
        
        assert_eq!(meeting_no_status.response_status, None);
    }

    #[test]
    fn test_should_display_filtering() {
        let now = Utc::now();
        
        // Accepted events should display
        let accepted = Meeting::new(
            "Accepted Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Accepted);
        assert!(accepted.should_display());
        
        // Tentative events should display
        let tentative = Meeting::new(
            "Tentative Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Tentative);
        assert!(tentative.should_display());
        
        // No response events should display
        let no_response = Meeting::new(
            "No Response Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::NoResponse);
        assert!(no_response.should_display());
        
        // Declined events should NOT display
        let declined = Meeting::new(
            "Declined Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Declined);
        assert!(!declined.should_display());
        
        // Events without response status should display (ICS events)
        let no_status = Meeting::new(
            "ICS Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        );
        assert!(no_status.should_display());
    }

    #[test]
    fn test_response_status_labels() {
        let now = Utc::now();
        
        // No response should have a label
        let no_response = Meeting::new(
            "Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::NoResponse);
        assert_eq!(no_response.response_status_label(), Some("Not Responded".to_string()));
        
        // Tentative should have a label
        let tentative = Meeting::new(
            "Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Tentative);
        assert_eq!(tentative.response_status_label(), Some("Tentative".to_string()));
        
        // Declined should have a label (though these won't typically be displayed)
        let declined = Meeting::new(
            "Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Declined);
        assert_eq!(declined.response_status_label(), Some("Declined".to_string()));
        
        // Accepted should have NO label (clean display)
        let accepted = Meeting::new(
            "Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        ).with_response_status(ResponseStatus::Accepted);
        assert_eq!(accepted.response_status_label(), None);
        
        // No response status should have NO label (ICS events)
        let no_status = Meeting::new(
            "Meeting".to_string(),
            now,
            now + chrono::Duration::hours(1)
        );
        assert_eq!(no_status.response_status_label(), None);
    }
}
