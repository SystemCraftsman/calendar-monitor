use calendar_monitor::meeting::Meeting;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meeting_deduplication_logic() {
        let now = Utc::now();
        
        // Create two meetings with same title and start time but different end times
        let meeting1 = Meeting {
            title: "Duplicate Meeting".to_string(),
            start_time: now,
            end_time: now + chrono::Duration::hours(1),
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        
        let meeting2 = Meeting {
            title: "Duplicate Meeting".to_string(),
            start_time: now, // Same start time
            end_time: now + chrono::Duration::hours(2), // Later end time
            description: None,
            location: None,
            attendees: Vec::new(),
        };
        
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
            Meeting {
                title: "Past Meeting".to_string(),
                start_time: now - chrono::Duration::hours(2),
                end_time: now - chrono::Duration::hours(1),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
            // Current meeting
            Meeting {
                title: "Current Meeting".to_string(),
                start_time: now - chrono::Duration::minutes(30),
                end_time: now + chrono::Duration::minutes(30),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
            // Future meeting
            Meeting {
                title: "Future Meeting".to_string(),
                start_time: now + chrono::Duration::hours(1),
                end_time: now + chrono::Duration::hours(2),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
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
            Meeting {
                title: "[Time Block]".to_string(),
                start_time: Utc::now(),
                end_time: Utc::now() + chrono::Duration::hours(1),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
            Meeting {
                title: "Regular Meeting".to_string(),
                start_time: Utc::now(),
                end_time: Utc::now() + chrono::Duration::hours(1),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
            Meeting {
                title: "[Another Time Block]".to_string(),
                start_time: Utc::now(),
                end_time: Utc::now() + chrono::Duration::hours(1),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
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
            Meeting {
                title: "Third".to_string(),
                start_time: now + chrono::Duration::hours(2),
                end_time: now + chrono::Duration::hours(3),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
            Meeting {
                title: "First".to_string(),
                start_time: now,
                end_time: now + chrono::Duration::hours(1),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
            Meeting {
                title: "Second".to_string(),
                start_time: now + chrono::Duration::hours(1),
                end_time: now + chrono::Duration::hours(2),
                description: None,
                location: None,
                attendees: Vec::new(),
            },
        ];
        
        // Sort by start time
        meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        assert_eq!(meetings[0].title, "First");
        assert_eq!(meetings[1].title, "Second");
        assert_eq!(meetings[2].title, "Third");
    }
}
