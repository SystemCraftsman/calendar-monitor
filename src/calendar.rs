use crate::meeting::Meeting;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use ical::parser::ical::component::IcalEvent;
use ical::IcalParser;
use std::env;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct CalendarService {
    ics_file_paths: Vec<String>,
    cached_meetings: Arc<Mutex<Option<Vec<Meeting>>>>,
    last_fetch_time: Arc<Mutex<Option<SystemTime>>>,
    cache_duration_secs: u64,
}

impl CalendarService {
    pub fn new() -> Self {
        Self {
            ics_file_paths: Vec::new(),
            cached_meetings: Arc::new(Mutex::new(None)),
            last_fetch_time: Arc::new(Mutex::new(None)),
            cache_duration_secs: 300, // 5 minutes
        }
    }

    /// Initialize with single ICS file path
    pub fn new_with_ics_file(file_path: String) -> Self {
        Self {
            ics_file_paths: vec![file_path],
            cached_meetings: Arc::new(Mutex::new(None)),
            last_fetch_time: Arc::new(Mutex::new(None)),
            cache_duration_secs: 300, // 5 minutes
        }
    }

    /// Initialize with multiple ICS file paths
    pub fn new_with_ics_files(file_paths: Vec<String>) -> Self {
        Self {
            ics_file_paths: file_paths,
            cached_meetings: Arc::new(Mutex::new(None)),
            last_fetch_time: Arc::new(Mutex::new(None)),
            cache_duration_secs: 300, // 5 minutes
        }
    }

    /// Initialize from environment variables
    pub fn new_from_env() -> Self {
        let mut ics_paths = Vec::new();

        // Support single file: ICS_FILE_PATH=./calendar.ics
        if let Ok(single_path) = env::var("ICS_FILE_PATH") {
            tracing::info!("Found ICS_FILE_PATH: {}", single_path);
            ics_paths.push(single_path);
        } else {
            tracing::info!("ICS_FILE_PATH not found in environment");
        }

        // Support multiple files: ICS_FILE_PATHS=./work.ics,./personal.ics,./project.ics
        if let Ok(multiple_paths) = env::var("ICS_FILE_PATHS") {
            tracing::info!("Found ICS_FILE_PATHS: {}", multiple_paths);
            for path in multiple_paths.split(',') {
                let trimmed_path = path.trim().to_string();
                if !trimmed_path.is_empty() && !ics_paths.contains(&trimmed_path) {
                    ics_paths.push(trimmed_path);
                }
            }
        } else {
            tracing::info!("ICS_FILE_PATHS not found in environment");
        }

        tracing::info!("Initialized CalendarService with {} ICS paths: {:?}", ics_paths.len(), ics_paths);

        Self {
            ics_file_paths: ics_paths,
            cached_meetings: Arc::new(Mutex::new(None)),
            last_fetch_time: Arc::new(Mutex::new(None)),
            cache_duration_secs: 300, // 5 minutes cache
        }
    }

    /// Initialize from TOML configuration
    pub fn new_from_config(config: &crate::config::Config) -> Self {
        let mut ics_paths = config.ics.file_paths.clone();

        // If no paths in config, try environment variables as fallback
        if ics_paths.is_empty() {
            if let Ok(single_path) = env::var("ICS_FILE_PATH") {
                tracing::info!("Found ICS_FILE_PATH in environment: {}", single_path);
                ics_paths.push(single_path);
            }

            if let Ok(multiple_paths) = env::var("ICS_FILE_PATHS") {
                tracing::info!("Found ICS_FILE_PATHS in environment: {}", multiple_paths);
                for path in multiple_paths.split(',') {
                    let trimmed_path = path.trim().to_string();
                    if !trimmed_path.is_empty() && !ics_paths.contains(&trimmed_path) {
                        ics_paths.push(trimmed_path);
                    }
                }
            }
        }

        tracing::info!("Initialized CalendarService from config with {} ICS paths: {:?}", ics_paths.len(), ics_paths);

        Self {
            ics_file_paths: ics_paths,
            cached_meetings: Arc::new(Mutex::new(None)),
            last_fetch_time: Arc::new(Mutex::new(None)),
            cache_duration_secs: config.server.cache_ttl_seconds, // Use cache TTL from config
        }
    }

    /// Get current meeting (if any) and next upcoming meeting (excluding time blocks)
    pub async fn get_current_and_next_meetings(&self) -> Result<(Option<Meeting>, Option<Meeting>)> {
        let meetings = self.get_meetings_for_today_and_tomorrow().await?;
        
        // Filter out time blocks for regular meetings
        let regular_meetings: Vec<_> = meetings.iter()
            .filter(|m| !m.is_time_block())
            .collect();
        
        let current_meeting = regular_meetings
            .iter()
            .find(|m| m.is_active())
            .cloned()
            .cloned();

        let next_meeting = regular_meetings
            .iter()
            .find(|m| m.is_upcoming())
            .cloned()
            .cloned();
            


        Ok((current_meeting, next_meeting))
    }

    /// Get active time blocks only
    pub async fn get_active_time_blocks(&self) -> Result<Vec<Meeting>> {
        let meetings = self.get_meetings_for_today_and_tomorrow().await?;
        
        // Filter for active time blocks only
        let _all_time_blocks: Vec<_> = meetings.iter()
            .filter(|m| m.is_time_block())
            .collect();
            
        let active_time_blocks: Vec<Meeting> = meetings.iter()
            .filter(|m| m.is_time_block() && m.is_active())
            .cloned()
            .collect();
            


        Ok(active_time_blocks)
    }

    /// Get all meetings for today and tomorrow
    pub async fn get_meetings_for_today_and_tomorrow(&self) -> Result<Vec<Meeting>> {
        if !self.ics_file_paths.is_empty() {
            // Check if cache is still valid
            let now = SystemTime::now();
            let cache_valid = {
                let last_fetch = self.last_fetch_time.lock().unwrap();
                if let Some(last_time) = *last_fetch {
                    now.duration_since(last_time)
                        .map(|d| d.as_secs() < self.cache_duration_secs)
                        .unwrap_or(false)
                } else {
                    false
                }
            };

            if cache_valid {
                // Return cached data
                let cached = self.cached_meetings.lock().unwrap();
                if let Some(meetings) = cached.as_ref() {
                    tracing::debug!("Returning cached meetings ({} items)", meetings.len());
                    return Ok(meetings.clone());
                }
            }

            // Cache is expired or empty, fetch fresh data
            tracing::info!("Cache expired or empty, fetching fresh calendar data");
            let fresh_meetings = self.parse_multiple_ics_files_extended().await?;
            
            // Update cache with fresh data (even if empty)
            {
                let mut cached = self.cached_meetings.lock().unwrap();
                *cached = Some(fresh_meetings.clone());
                tracing::info!("Updated cache with {} fresh meetings", fresh_meetings.len());
            }
            {
                let mut last_fetch = self.last_fetch_time.lock().unwrap();
                *last_fetch = Some(now);
            }
            
            Ok(fresh_meetings)
        } else {
            // Fall back to mock data if no ICS files are configured
            self.get_mock_meetings().await
        }
    }

    /// Get all meetings for today
    pub async fn get_meetings_for_today(&self) -> Result<Vec<Meeting>> {
        // Get today and tomorrow meetings, then filter to today only
        let all_meetings = self.get_meetings_for_today_and_tomorrow().await?;
        let today = Utc::now().date_naive();
        
        Ok(all_meetings
            .into_iter()
            .filter(|m| m.start_time.date_naive() == today)
            .collect())
    }

    /// Parse multiple ICS files and merge all meetings (today and tomorrow)
    async fn parse_multiple_ics_files_extended(&self) -> Result<Vec<Meeting>> {
        let mut all_meetings = Vec::new();
        
        for ics_path in &self.ics_file_paths {
            match self.parse_ics_file_extended(ics_path).await {
                Ok(meetings) => {
                    let _count_before = all_meetings.len();
                    all_meetings.extend(meetings.clone());
                    let count_after = all_meetings.len();
                    tracing::info!("Loaded {} meetings from {} (total now: {})", 
                        meetings.len(), ics_path, count_after);
                    


                }
                Err(e) => {
                    tracing::warn!("Failed to parse ICS file '{}': {}", ics_path, e);
                    // Continue processing other files even if one fails
                    continue;
                }
            }
        }

        tracing::info!("Before sort/dedup: {} meetings", all_meetings.len());
        
        // Sort all meetings by start time
        all_meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        // Duplicates will be removed by custom logic below
        
        tracing::info!("After sort: {} meetings from {} ICS files", all_meetings.len(), self.ics_file_paths.len());
        Ok(all_meetings)
    }

    /// Parse multiple ICS files and merge all meetings
    async fn parse_multiple_ics_files(&self) -> Result<Vec<Meeting>> {
        let mut all_meetings = Vec::new();
        
        for ics_path in &self.ics_file_paths {
            match self.parse_ics_file(ics_path).await {
                Ok(mut meetings) => {
                    all_meetings.append(&mut meetings);
                    tracing::info!("Loaded {} meetings from {}", meetings.len(), ics_path);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse ICS file '{}': {}", ics_path, e);
                    // Continue processing other files even if one fails
                    continue;
                }
            }
        }

        // Sort all meetings by start time
        all_meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        // Remove duplicates based on title and start time, keeping the one with later end time
        let mut i = 0;
        while i < all_meetings.len() {
            let mut j = i + 1;
            while j < all_meetings.len() {
                if all_meetings[i].title == all_meetings[j].title && 
                   all_meetings[i].start_time == all_meetings[j].start_time {
                    
                    tracing::debug!("Found duplicate events: '{}' at {}", 
                        all_meetings[i].title, all_meetings[i].start_time.format("%H:%M"));
                    tracing::debug!("  Event A: {} to {}", 
                        all_meetings[i].start_time.format("%H:%M"), 
                        all_meetings[i].end_time.format("%H:%M"));
                    tracing::debug!("  Event B: {} to {}", 
                        all_meetings[j].start_time.format("%H:%M"), 
                        all_meetings[j].end_time.format("%H:%M"));
                    
                    // Keep the one with later end time (remove the other)
                    if all_meetings[i].end_time < all_meetings[j].end_time {
                        tracing::debug!("  → Keeping Event B (later end time)");
                        all_meetings.remove(i);
                        // Don't increment i since we removed an element
                        break;
                    } else {
                        tracing::debug!("  → Keeping Event A (later end time)");
                        all_meetings.remove(j);
                        // Don't increment j since we removed an element
                    }
                } else {
                    j += 1;
                }
            }
            if j >= all_meetings.len() {
                i += 1;
            }
        }
        
        // Debug Draft.dev events after deduplication
        for (i, meeting) in all_meetings.iter().enumerate() {
            if meeting.title.contains("Draft.dev") {
                tracing::info!("AFTER DEDUP - Draft.dev #{}: {} to {} (dur: {} min)", 
                    i, 
                    meeting.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
                    meeting.end_time.format("%Y-%m-%d %H:%M:%S UTC"),
                    (meeting.end_time - meeting.start_time).num_minutes());
            }
        }
        
        tracing::info!("Total meetings loaded: {} from {} ICS files", all_meetings.len(), self.ics_file_paths.len());
        Ok(all_meetings)
    }

    /// Get mock meetings for development/testing
    async fn get_mock_meetings(&self) -> Result<Vec<Meeting>> {
        let now = Utc::now();
        
        // Create some sample meetings
        let mut meetings = vec![
            Meeting::new(
                "Daily Standup".to_string(),
                now - Duration::minutes(5), // Started 5 minutes ago
                now + Duration::minutes(25), // Ends in 25 minutes
            )
            .with_description("Daily team sync meeting".to_string())
            .with_location("Conference Room A".to_string()),
            
            Meeting::new(
                "Project Review".to_string(),
                now + Duration::minutes(30), // Starts in 30 minutes
                now + Duration::minutes(90), // Lasts 1 hour
            )
            .with_description("Quarterly project review with stakeholders".to_string())
            .with_location("Main Conference Room".to_string()),
            
            Meeting::new(
                "Client Call".to_string(),
                now + Duration::hours(3),
                now + Duration::hours(4),
            )
            .with_description("Weekly check-in with client".to_string())
            .with_location("Zoom".to_string()),
        ];

        // Sort meetings by start time
        meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        Ok(meetings)
    }

    /// Parse ICS file and return meetings for today
    async fn parse_ics_file(&self, file_path: &str) -> Result<Vec<Meeting>> {
        let ics_content = if file_path.starts_with("http://") || file_path.starts_with("https://") {
            // Download ICS from URL
            tracing::info!("Downloading ICS from URL: {}", file_path);
            let response = reqwest::get(file_path).await
                .map_err(|e| anyhow!("Failed to download ICS from URL {}: {}", file_path, e))?;
            
            if !response.status().is_success() {
                return Err(anyhow!("HTTP error {} when downloading ICS from {}", response.status(), file_path));
            }
            
            response.text().await
                .map_err(|e| anyhow!("Failed to read ICS content from {}: {}", file_path, e))?
        } else {
            // Read local file
            if !Path::new(file_path).exists() {
                return Err(anyhow!("ICS file not found: {}", file_path));
            }
            
            std::fs::read_to_string(file_path)
                .map_err(|e| anyhow!("Failed to read ICS file: {}", e))?
        };

        let reader = IcalParser::new(ics_content.as_bytes());

        let mut meetings = Vec::new();
        let today = Utc::now().date_naive();

        for line in reader {
            match line {
                Ok(calendar) => {
                    for event in calendar.events {
                        let event_meetings = self.convert_ical_event_to_meeting(event)?;
                        for meeting in event_meetings {
                            // Only include events for today
                            if meeting.start_time.date_naive() == today {
                                meetings.push(meeting);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error parsing ICS line: {}", e);
                    continue;
                }
            }
        }

        // Sort meetings by start time
        meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        Ok(meetings)
    }

    /// Parse ICS file and return meetings for today and tomorrow
    async fn parse_ics_file_extended(&self, file_path: &str) -> Result<Vec<Meeting>> {
        let ics_content = if file_path.starts_with("http://") || file_path.starts_with("https://") {
            // Download ICS from URL
            tracing::info!("Downloading ICS from URL: {}", file_path);
            let response = reqwest::get(file_path).await
                .map_err(|e| anyhow!("Failed to download ICS from URL {}: {}", file_path, e))?;
            
            if !response.status().is_success() {
                return Err(anyhow!("HTTP error {} when downloading ICS from {}", response.status(), file_path));
            }
            
            response.text().await
                .map_err(|e| anyhow!("Failed to read ICS content from {}: {}", file_path, e))?
        } else {
            // Read local file
            if !Path::new(file_path).exists() {
                return Err(anyhow!("ICS file not found: {}", file_path));
            }
            
            std::fs::read_to_string(file_path)
                .map_err(|e| anyhow!("Failed to read ICS file: {}", e))?
        };

        let reader = IcalParser::new(ics_content.as_bytes());

        let mut meetings = Vec::new();
        let today = Utc::now().date_naive();
        let tomorrow = today + Duration::days(1);

        for line in reader {
            match line {
                Ok(calendar) => {
                    for event in calendar.events {
                        let event_meetings = self.convert_ical_event_to_meeting(event)?;
                        for meeting in event_meetings {
                            // Include events for today and tomorrow
                            let meeting_date = meeting.start_time.date_naive();
                            if meeting_date == today || meeting_date == tomorrow {
                                meetings.push(meeting);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error parsing ICS line: {}", e);
                    continue;
                }
            }
        }

        // Sort meetings by start time
        meetings.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        Ok(meetings)
    }

    /// Convert ICS event to our Meeting struct, handling recurring events
    fn convert_ical_event_to_meeting(&self, event: IcalEvent) -> Result<Vec<Meeting>> {
        let mut title = "Untitled Event".to_string();
        let mut start_time: Option<DateTime<Utc>> = None;
        let mut end_time: Option<DateTime<Utc>> = None;
        let mut description: Option<String> = None;
        let mut location: Option<String> = None;
        let mut rrule: Option<String> = None;

        // Parse event properties
        for property in event.properties {
            match property.name.as_str() {
                "SUMMARY" => {
                    if let Some(value) = property.value {
                        title = value;
                    }
                }
                "DTSTART" => {
                    if let Some(value) = property.value {
                        start_time = self.parse_ical_datetime(&value)?;
                    }
                }
                "DTEND" => {
                    if let Some(value) = property.value {
                        end_time = self.parse_ical_datetime(&value)?;
                    }
                }
                "RRULE" => {
                    if let Some(value) = property.value {
                        tracing::debug!("Found RRULE for '{}': {}", title, value);
                        rrule = Some(value);
                    }
                }
                "DURATION" => {
                    if let Some(value) = property.value {
                        tracing::debug!("Found DURATION property: {}", value);
                        // Some ICS files use DURATION instead of DTEND
                    }
                }
                "DESCRIPTION" => {
                    if let Some(value) = property.value {
                        description = Some(value);
                    }
                }
                "LOCATION" => {
                    if let Some(value) = property.value {
                        location = Some(value);
                    }
                }
                _ => {} // Ignore other properties for now
            }
        }

        // Both start and end times are required
        if let (Some(start), Some(end)) = (start_time, end_time) {
            // Check if this is a recurring event
            if let Some(rrule_value) = rrule {
                self.expand_recurring_event(title, start, end, &rrule_value, description, location)
            } else {
                // Non-recurring event
                let mut meeting = Meeting::new(title.clone(), start, end);
                
                if let Some(desc) = description {
                    meeting = meeting.with_description(desc);
                }
                
                if let Some(loc) = location {
                    meeting = meeting.with_location(loc);
                }
                
                Ok(vec![meeting])
            }
        } else {
            tracing::debug!("Skipping event '{}' - missing start or end time", title);
            Ok(vec![]) // Skip events without proper time information
        }
    }

    /// Expand recurring events for today and tomorrow
    fn expand_recurring_event(
        &self,
        title: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        rrule: &str,
        description: Option<String>,
        location: Option<String>,
    ) -> Result<Vec<Meeting>> {
        let today = Utc::now().date_naive();
        let tomorrow = today + chrono::Duration::days(1);
        let mut meetings = Vec::new();

        // Parse RRULE (basic support for common patterns)
        if rrule.contains("FREQ=WEEKLY") {
            tracing::info!("Expanding RRULE for '{}': {}", title, rrule);
            tracing::info!("Original time: {} - {} (weekday: {:?})", 
                start.format("%Y-%m-%d %H:%M:%S UTC"), 
                end.format("%Y-%m-%d %H:%M:%S UTC"), 
                start.weekday());
            
            // Check for UNTIL clause and respect it
            if let Some(until_date) = self.parse_rrule_until(rrule) {
                tracing::info!("  UNTIL clause found: {}", until_date.format("%Y-%m-%d"));
                if today > until_date || tomorrow > until_date {
                    tracing::info!("  → Skipping recurring event - past UNTIL date");
                    return Ok(meetings); // Return empty if past the UNTIL date
                }
            }
                
            let duration = end - start;
            
            // Get the day of week from the original start time
            let original_weekday = start.weekday();
            
            // Check if event should occur today
            if self.should_occur_on_day(rrule, today, original_weekday) {
                let today_start = self.adjust_time_to_date(start, today);
                let today_end = self.adjust_time_to_date(end, today);
                
                // Handle cross-midnight events: check if original event crosses midnight
                // by comparing the day of the original start and end times
                let original_crosses_midnight = start.date_naive() != end.date_naive();
                let today_end = if original_crosses_midnight {
                    // If original event crossed midnight, move end to next day
                    today_end + chrono::Duration::days(1)
                } else {
                    today_end
                };
                
                tracing::info!("  → Generating for TODAY: {} -> {} (duration: {})", 
                    today_start.format("%Y-%m-%d %H:%M:%S UTC"), 
                    today_end.format("%Y-%m-%d %H:%M:%S UTC"),
                    duration.num_minutes());
                
                let mut meeting = Meeting::new(title.clone(), today_start, today_end);
                if let Some(desc) = description.as_ref() {
                    meeting = meeting.with_description(desc.clone());
                }
                if let Some(loc) = location.as_ref() {
                    meeting = meeting.with_location(loc.clone());
                }
                meetings.push(meeting);
                tracing::debug!("Generated recurring event for today: '{}' at {}", title, today_start);
            }
            
            // Check if event should occur tomorrow
            if self.should_occur_on_day(rrule, tomorrow, original_weekday) {
                let tomorrow_start = self.adjust_time_to_date(start, tomorrow);
                let tomorrow_end = self.adjust_time_to_date(end, tomorrow);
                
                // Handle cross-midnight events: check if original event crosses midnight
                // by comparing the day of the original start and end times
                let original_crosses_midnight = start.date_naive() != end.date_naive();
                let tomorrow_end = if original_crosses_midnight {
                    // If original event crossed midnight, move end to next day
                    tomorrow_end + chrono::Duration::days(1)
                } else {
                    tomorrow_end
                };
                
                let mut meeting = Meeting::new(title.clone(), tomorrow_start, tomorrow_end);
                if let Some(desc) = description.as_ref() {
                    meeting = meeting.with_description(desc.clone());
                }
                if let Some(loc) = location.as_ref() {
                    meeting = meeting.with_location(loc.clone());
                }
                meetings.push(meeting);
                tracing::debug!("Generated recurring event for tomorrow: '{}' at {}", title, tomorrow_start);
            }
        } else {
            tracing::debug!("Unsupported RRULE pattern: {}", rrule);
        }
        
        Ok(meetings)
    }

    /// Parse UNTIL date from RRULE string
    pub fn parse_rrule_until(&self, rrule: &str) -> Option<chrono::NaiveDate> {
        if let Some(start) = rrule.find("UNTIL=") {
            let start = start + 6; // Skip "UNTIL="
            let end = rrule[start..].find(';').map(|i| start + i).unwrap_or(rrule.len());
            let until_str = &rrule[start..end];
            
            // Parse different UNTIL formats
            // Format: 20250620T235959 or 20250620T235959Z
            if let Some(date_part) = until_str.get(0..8) {
                if let Ok(date) = chrono::NaiveDate::parse_from_str(date_part, "%Y%m%d") {
                    tracing::debug!("Parsed UNTIL date: {} from '{}'", date.format("%Y-%m-%d"), until_str);
                    return Some(date);
                }
            }
            
            tracing::warn!("Failed to parse UNTIL date: '{}'", until_str);
        }
        None
    }

    /// Check if a recurring event should occur on a given date
    fn should_occur_on_day(&self, rrule: &str, date: chrono::NaiveDate, original_weekday: chrono::Weekday) -> bool {
        let weekday = date.weekday();
        
        // Check for BYDAY restrictions
        if rrule.contains("BYDAY=") {
            let byday_pattern = if let Some(start) = rrule.find("BYDAY=") {
                let start = start + 6; // Skip "BYDAY="
                let end = rrule[start..].find(';').map(|i| start + i).unwrap_or(rrule.len());
                &rrule[start..end]
            } else {
                return weekday == original_weekday; // Default to original weekday
            };
            
            // Parse weekday codes (MO, TU, WE, TH, FR, SA, SU)
            let weekday_code = match weekday {
                chrono::Weekday::Mon => "MO",
                chrono::Weekday::Tue => "TU",
                chrono::Weekday::Wed => "WE",
                chrono::Weekday::Thu => "TH",
                chrono::Weekday::Fri => "FR",
                chrono::Weekday::Sat => "SA",
                chrono::Weekday::Sun => "SU",
            };
            
            return byday_pattern.contains(weekday_code);
        }
        
        // No BYDAY restriction, use original weekday
        weekday == original_weekday
    }

    /// Adjust a DateTime to occur on a specific date, keeping the same time
    fn adjust_time_to_date(&self, original_time: DateTime<Utc>, target_date: chrono::NaiveDate) -> DateTime<Utc> {
        let time = original_time.time();
        target_date.and_time(time).and_local_timezone(chrono::Utc).unwrap()
    }

    /// Parse ICS datetime string to chrono DateTime<Utc>
    fn parse_ical_datetime(&self, dt_str: &str) -> Result<Option<DateTime<Utc>>> {
        // Handle different ICS datetime formats
        
        // UTC format: 20231225T120000Z
        if dt_str.ends_with('Z') {
            let naive = chrono::NaiveDateTime::parse_from_str(dt_str, "%Y%m%dT%H%M%SZ")
                .map_err(|e| anyhow!("Failed to parse UTC datetime {}: {}", dt_str, e))?;
            return Ok(Some(Utc.from_utc_datetime(&naive)));
        }
        
        // Local format: 20231225T120000
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(dt_str, "%Y%m%dT%H%M%S") {
            // Convert from Europe/Istanbul timezone (UTC+3) to UTC
            // Istanbul time = UTC + 3 hours, so we subtract 3 hours to get UTC
            let utc_naive = naive - chrono::Duration::hours(3);
            return Ok(Some(Utc.from_utc_datetime(&utc_naive)));
        }
        
        // Date only format: 20231225
        if let Ok(date) = chrono::NaiveDate::parse_from_str(dt_str, "%Y%m%d") {
            let naive_datetime = date.and_hms_opt(0, 0, 0).unwrap();
            return Ok(Some(Utc.from_utc_datetime(&naive_datetime)));
        }
        
        tracing::warn!("Unable to parse datetime format: {}", dt_str);
        Ok(None)
    }


}

impl Default for CalendarService {
    fn default() -> Self {
        Self::new()
    }
}
