use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub response_status: Option<ResponseStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseStatus {
    Accepted,
    Declined,
    Tentative,
    NoResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingStatus {
    Upcoming,
    InProgress,
    Ended,
}

impl Meeting {
    pub fn new(
        title: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        Self {
            title,
            start_time,
            end_time,
            description: None,
            location: None,
            attendees: Vec::new(),
            response_status: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_response_status(mut self, response_status: ResponseStatus) -> Self {
        self.response_status = Some(response_status);
        self
    }

    /// Check if this meeting should be displayed based on response status
    /// Returns false only for declined events
    pub fn should_display(&self) -> bool {
        match &self.response_status {
            Some(ResponseStatus::Declined) => false,
            _ => true,
        }
    }

    /// Get a display label for the response status
    pub fn response_status_label(&self) -> Option<String> {
        match &self.response_status {
            Some(ResponseStatus::NoResponse) => Some("Not Responded".to_string()),
            Some(ResponseStatus::Tentative) => Some("Tentative".to_string()),
            Some(ResponseStatus::Accepted) => None, // No label needed for accepted
            Some(ResponseStatus::Declined) => Some("Declined".to_string()),
            None => None, // ICS events don't have response status
        }
    }

    pub fn with_attendees(mut self, attendees: Vec<String>) -> Self {
        self.attendees = attendees;
        self
    }

    /// Get the current status of the meeting
    pub fn status(&self) -> MeetingStatus {
        let now = Utc::now();
        
        if now < self.start_time {
            MeetingStatus::Upcoming
        } else if now >= self.start_time && now <= self.end_time {
            MeetingStatus::InProgress
        } else {
            MeetingStatus::Ended
        }
    }

    /// Get seconds until the meeting starts (negative if already started)
    pub fn time_until_start(&self) -> i64 {
        (self.start_time - Utc::now()).num_seconds()
    }

    /// Get seconds until the meeting ends (negative if already ended)
    pub fn time_until_end(&self) -> i64 {
        (self.end_time - Utc::now()).num_seconds()
    }

    /// Get the duration of the meeting in minutes
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    /// Check if the meeting is currently active
    pub fn is_active(&self) -> bool {
        matches!(self.status(), MeetingStatus::InProgress)
    }

    /// Check if the meeting is upcoming
    pub fn is_upcoming(&self) -> bool {
        matches!(self.status(), MeetingStatus::Upcoming)
    }

    /// Check if the meeting has ended
    pub fn has_ended(&self) -> bool {
        matches!(self.status(), MeetingStatus::Ended)
    }

    /// Format time remaining as a human-readable string (e.g., "15:30" for 15 minutes 30 seconds)
    pub fn format_time_remaining(&self) -> String {
        let seconds = match self.status() {
            MeetingStatus::InProgress => self.time_until_end(),
            MeetingStatus::Upcoming => self.time_until_start(),
            MeetingStatus::Ended => 0,
        };

        if seconds <= 0 {
            return "00:00".to_string();
        }

        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;

        if minutes >= 60 {
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            format!("{:02}:{:02}:{:02}", hours, remaining_minutes, remaining_seconds)
        } else {
            format!("{:02}:{:02}", minutes, remaining_seconds)
        }
    }

    /// Get a human-readable start time string
    pub fn formatted_start_time(&self) -> String {
        self.start_time.format("%H:%M").to_string()
    }

    /// Get a human-readable date string
    pub fn formatted_date(&self) -> String {
        self.start_time.format("%Y-%m-%d").to_string()
    }

    /// Get a human-readable time range string
    pub fn formatted_time_range(&self) -> String {
        format!(
            "{} - {}",
            self.start_time.format("%H:%M"),
            self.end_time.format("%H:%M")
        )
    }

    /// Check if this meeting is a time block (title starts with [ and ends with ])
    pub fn is_time_block(&self) -> bool {
        self.title.starts_with('[') && self.title.ends_with(']')
    }

    /// Get the time block name without brackets (if it's a time block)
    pub fn time_block_name(&self) -> Option<String> {
        if self.is_time_block() && self.title.len() > 2 {
            Some(self.title[1..self.title.len()-1].to_string())
        } else {
            None
        }
    }
}
