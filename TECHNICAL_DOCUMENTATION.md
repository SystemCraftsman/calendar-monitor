# Calendar Monitor - Technical Documentation

## Table of Contents
1. [Project Overview](#project-overview)
2. [Architecture & Design](#architecture--design)
3. [File Structure](#file-structure)
4. [Key Dependencies](#key-dependencies)
5. [Core Data Structures](#core-data-structures)
6. [Function-by-Function Analysis](#function-by-function-analysis)
7. [Data Flow](#data-flow)
8. [Rust Concepts Explained](#rust-concepts-explained)
9. [Frontend Integration](#frontend-integration)

---

## Project Overview

**Calendar Monitor** is a real-time web application that displays:
- **Current Event**: Shows the currently active meeting/event with countdown
- **Next Event**: Displays the upcoming meeting/event 
- **Time Blocks**: Shows active time blocks (events with titles like `[Draft.dev]`)

The application reads calendar data from multiple ICS (iCalendar) files or URLs, processes recurring events, and provides real-time updates via WebSockets.

## Architecture & Design

The project follows a **modular architecture** with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Web Server    │    │  Calendar       │
│   (HTML/CSS/JS) │◄──►│   (Axum)        │◄──►│  Service        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   WebSocket     │    │   ICS Parser    │
                       │   Updates       │    │   (ical crate)  │
                       └─────────────────┘    └─────────────────┘
```

**Key Design Patterns:**
- **Async/Await**: All I/O operations are asynchronous
- **Caching**: Smart caching with 5-minute expiration
- **Real-time Updates**: WebSocket-based live updates every second
- **Error Handling**: Comprehensive error handling with `anyhow` and `Result<T>`

---

## File Structure

```
src/
├── main.rs           # Entry point, web server, routes
├── meeting.rs        # Meeting data structure and methods
├── calendar.rs       # Calendar service, ICS parsing, business logic
static/
├── app.js           # Frontend JavaScript (WebSocket client)
├── style.css        # CSS styling
templates/
├── index.html       # HTML template
Cargo.toml           # Dependencies and project metadata
```

---

## Key Dependencies

| Crate | Purpose | Why Used |
|-------|---------|----------|
| `axum` | Web framework | Modern, fast, type-safe HTTP server |
| `tokio` | Async runtime | Enables async/await functionality |
| `serde` | Serialization | JSON serialization/deserialization |
| `chrono` | Date/Time | Robust date/time handling with timezones |
| `ical` | ICS parsing | Parse iCalendar (.ics) files |
| `reqwest` | HTTP client | Download ICS files from URLs |
| `anyhow` | Error handling | Simplified error handling |
| `tracing` | Logging | Structured logging and debugging |

---

## Core Data Structures

### 1. Meeting Struct (`meeting.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub attendees: Vec<String>,
}
```

**Rust Concept**: `#[derive(...)]` automatically generates implementations for:
- `Debug`: Enables printing with `{:?}`
- `Clone`: Allows copying the struct
- `Serialize/Deserialize`: Enables JSON conversion

### 2. MeetingStatus Enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingStatus {
    Upcoming,
    InProgress,
    Ended,
}
```

**Rust Concept**: Enums in Rust are powerful - they can represent one of several possible states.

### 3. CalendarService Struct (`calendar.rs`)

```rust
pub struct CalendarService {
    ics_file_paths: Vec<String>,
    cached_meetings: Arc<Mutex<Option<Vec<Meeting>>>>,
    last_fetch_time: Arc<Mutex<Option<SystemTime>>>,
    cache_duration_secs: u64,
}
```

**Rust Concepts**:
- `Vec<String>`: Dynamic array of strings
- `Arc<Mutex<T>>`: Thread-safe shared data (`Arc` = atomic reference counting, `Mutex` = mutual exclusion)
- `Option<T>`: Represents a value that might not exist

---

## Function-by-Function Analysis

### 1. Main Function (`main.rs`)

#### `main()` - Application Entry Point

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    match dotenv::dotenv() {
        Ok(path) => info!("Loaded .env file from: {:?}", path),
        Err(e) => warn!("Could not load .env file: {}", e),
    }
    
    // Create web server with routes
    let app = Router::new()
        .route("/", get(index))              // Serve HTML page
        .route("/ws", get(websocket_handler)) // WebSocket endpoint
        .route("/api/meetings", get(get_meetings)) // REST API
        .nest_service("/static", get_service(ServeDir::new("static"))) // Static files
        .layer(CorsLayer::permissive()); // Enable CORS

    // Start server on localhost:3000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Rust Concepts**:
- `#[tokio::main]`: Attribute that sets up async runtime
- `async fn`: Asynchronous function that can be awaited
- `anyhow::Result<()>`: Return type that can contain an error
- `?` operator: Propagates errors upward (early return if error)

#### `index()` - Serve HTML Page

```rust
async fn index() -> impl IntoResponse {
    Html(include_str!("../templates/index.html"))
}
```

**Rust Concepts**:
- `impl IntoResponse`: Return type that can be converted to HTTP response
- `include_str!()`: Macro that includes file content at compile time

#### `websocket_handler()` - WebSocket Upgrade

```rust
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}
```

This function upgrades HTTP connections to WebSocket protocol.

#### `handle_socket()` - WebSocket Message Loop

```rust
async fn handle_socket(mut socket: WebSocket) {
    let mut interval = interval(Duration::from_secs(1)); // Tick every second
    let calendar_service = CalendarService::new_from_env();

    loop {
        interval.tick().await; // Wait for next second
        
        // Get current meetings and time blocks
        let meetings_result = calendar_service.get_current_and_next_meetings().await;
        let time_blocks_result = calendar_service.get_active_time_blocks().await;
        
        // Handle both results
        match (meetings_result, time_blocks_result) {
            (Ok((current, next)), Ok(active_time_blocks)) => {
                // Calculate countdown for current meeting
                let countdown_seconds = current.as_ref().map(|m| m.time_until_end());
                
                // Create update message
                let update = MeetingUpdate {
                    current_meeting: current,
                    next_meeting: next,
                    countdown_seconds,
                    active_time_blocks,
                };

                // Send JSON message to client
                if let Ok(message) = serde_json::to_string(&update) {
                    if socket.send(Message::Text(message)).await.is_err() {
                        break; // Client disconnected
                    }
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                warn!("Error fetching meetings or time blocks: {}", e);
            }
        }
    }
}
```

**Rust Concepts**:
- `mut`: Mutable reference (can be modified)
- `loop`: Infinite loop
- `match`: Pattern matching (similar to switch/case)
- `as_ref()`: Converts `Option<T>` to `Option<&T>`
- `map()`: Transforms the value inside Option if it exists

### 2. Meeting Methods (`meeting.rs`)

#### Constructor Functions

```rust
impl Meeting {
    pub fn new(title: String, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        Self {
            title,
            start_time,
            end_time,
            description: None,
            location: None,
            attendees: Vec::new(),
        }
    }
```

**Rust Concepts**:
- `impl`: Implementation block for a struct
- `Self`: Refers to the current struct type
- `Vec::new()`: Creates empty vector

#### Builder Pattern Methods

```rust
pub fn with_description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
}

pub fn with_location(mut self, location: String) -> Self {
    self.location = Some(location);
    self
}
```

**Rust Concepts**:
- `mut self`: Takes ownership and allows modification
- Returns `Self` for method chaining

#### Status Calculation

```rust
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
```

**Rust Concepts**:
- `&self`: Immutable reference to self
- Comparison operators work with DateTime types

#### Time Calculations

```rust
pub fn time_until_start(&self) -> i64 {
    (self.start_time - Utc::now()).num_seconds()
}

pub fn time_until_end(&self) -> i64 {
    (self.end_time - Utc::now()).num_seconds()
}
```

These calculate seconds until start/end. Negative values mean past events.

#### Activity Checks

```rust
pub fn is_active(&self) -> bool {
    matches!(self.status(), MeetingStatus::InProgress)
}

pub fn is_upcoming(&self) -> bool {
    matches!(self.status(), MeetingStatus::Upcoming)
}
```

**Rust Concepts**:
- `matches!()`: Macro for pattern matching that returns boolean

#### Time Block Detection

```rust
pub fn is_time_block(&self) -> bool {
    self.title.starts_with('[') && self.title.ends_with(']')
}

pub fn time_block_name(&self) -> Option<String> {
    if self.is_time_block() && self.title.len() > 2 {
        Some(self.title[1..self.title.len()-1].to_string())
    } else {
        None
    }
}
```

**Rust Concepts**:
- `[1..length-1]`: Slice syntax (excludes brackets)
- `to_string()`: Converts string slice to owned String

### 3. Calendar Service (`calendar.rs`)

#### Constructor from Environment

```rust
pub fn new_from_env() -> Self {
    let mut ics_paths = Vec::new();

    // Support single file: ICS_FILE_PATH=./calendar.ics
    if let Ok(single_path) = env::var("ICS_FILE_PATH") {
        tracing::info!("Found ICS_FILE_PATH: {}", single_path);
        ics_paths.push(single_path);
    }

    // Support multiple files: ICS_FILE_PATHS=./work.ics,./personal.ics
    if let Ok(multiple_paths) = env::var("ICS_FILE_PATHS") {
        for path in multiple_paths.split(',') {
            let trimmed_path = path.trim().to_string();
            if !trimmed_path.is_empty() && !ics_paths.contains(&trimmed_path) {
                ics_paths.push(trimmed_path);
            }
        }
    }

    Self {
        ics_file_paths: ics_paths,
        cached_meetings: Arc::new(Mutex::new(None)),
        last_fetch_time: Arc::new(Mutex::new(None)),
        cache_duration_secs: 300, // 5 minutes
    }
}
```

**Rust Concepts**:
- `if let Ok(value) = result`: Pattern matching for `Result` types
- `split(',')`: String method that returns iterator
- `trim()`: Removes whitespace
- `contains()`: Checks if vector contains element

#### Get Current and Next Meetings

```rust
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
```

**Rust Concepts**:
- `Result<(Type1, Type2)>`: Return tuple wrapped in Result
- `iter()`: Creates iterator over collection
- `filter()`: Keeps elements matching predicate
- `find()`: Returns first matching element
- `collect()`: Consumes iterator into collection
- `cloned().cloned()`: First clones the reference, then clones the Meeting

#### Smart Caching Implementation

```rust
pub async fn get_meetings_for_today_and_tomorrow(&self) -> Result<Vec<Meeting>> {
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
            return Ok(meetings.clone());
        }
    }

    // Cache expired, fetch fresh data
    let fresh_meetings = self.parse_multiple_ics_files_extended().await?;
    
    // Update cache
    {
        let mut cached = self.cached_meetings.lock().unwrap();
        *cached = Some(fresh_meetings.clone());
    }
    {
        let mut last_fetch = self.last_fetch_time.lock().unwrap();
        *last_fetch = Some(now);
    }
    
    Ok(fresh_meetings)
}
```

**Rust Concepts**:
- `lock().unwrap()`: Acquires mutex lock, panics if poisoned
- Scope blocks `{}` ensure locks are released early
- `*cached = value`: Dereferences and assigns new value
- `duration_since()`: Calculates time difference

#### ICS File Parsing

```rust
async fn parse_ics_file(&self, file_path: &str) -> Result<Vec<Meeting>> {
    let content = if file_path.starts_with("http://") || file_path.starts_with("https://") {
        // Download from URL
        let response = reqwest::get(file_path).await?;
        response.text().await?
    } else {
        // Read from local file
        std::fs::read_to_string(file_path)?
    };

    let reader = std::io::Cursor::new(content);
    let parser = IcalParser::new(reader);
    
    let mut meetings = Vec::new();
    for calendar in parser {
        let calendar = calendar?;
        for event in calendar.events {
            if let Some(meeting) = self.convert_ical_event_to_meeting(event)? {
                meetings.push(meeting);
            }
        }
    }
    
    Ok(meetings)
}
```

**Rust Concepts**:
- `&str`: String slice (borrowed string reference)
- `if condition { A } else { B }`: Conditional expression
- `for item in iterator`: For loop over iterator
- `if let Some(value) = option`: Pattern match for Some variant

#### Recurring Event Processing

```rust
fn convert_ical_event_to_meeting(&self, event: IcalEvent) -> Result<Vec<Meeting>> {
    let mut title = "Untitled Event".to_string();
    let mut start_time: Option<DateTime<Utc>> = None;
    let mut end_time: Option<DateTime<Utc>> = None;
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
                    rrule = Some(value);
                }
            }
            _ => {} // Ignore other properties
        }
    }

    // Process recurring rules
    if let Some(rrule_str) = rrule {
        if rrule_str.contains("FREQ=WEEKLY") {
            return self.expand_weekly_recurrence(title, start, end, &rrule_str);
        }
    }

    // Single event
    if let (Some(start), Some(end)) = (start_time, end_time) {
        Ok(vec![Meeting::new(title, start, end)])
    } else {
        Ok(vec![])
    }
}
```

**Rust Concepts**:
- `match expression`: Pattern matching on values
- `as_str()`: Converts String to &str
- `_ => {}`: Default case (ignore)
- Pattern matching on tuples: `(Some(a), Some(b))`

---

## Data Flow

```
1. Server Start
   ├── Load .env file
   ├── Initialize CalendarService
   └── Start web server

2. Client Connection
   ├── Serve HTML page
   ├── Establish WebSocket
   └── Start 1-second update loop

3. Data Update Cycle (every second)
   ├── Check cache validity (5-minute expiration)
   ├── If expired: Parse ICS files
   │   ├── Download from URLs or read local files
   │   ├── Parse ICS format
   │   ├── Expand recurring events
   │   ├── Filter for today/tomorrow
   │   └── Update cache
   ├── Filter meetings vs time blocks
   ├── Find current/next meetings
   ├── Find active time blocks
   └── Send JSON update to client

4. Frontend Update
   ├── Receive WebSocket message
   ├── Parse JSON data
   ├── Update DOM elements
   └── Apply animations (current meeting only)
```

---

## Rust Concepts Explained

### 1. Ownership System
- **Ownership**: Each value has exactly one owner
- **Borrowing**: Temporary access without taking ownership (`&T`)
- **Mutable Borrowing**: Temporary mutable access (`&mut T`)

### 2. Error Handling
```rust
// Result<T, E> - Either success (Ok) or error (Err)
let result: Result<i32, String> = Ok(42);

// ? operator - propagate errors
let value = some_function()?; // Returns early if error

// match on Result
match result {
    Ok(value) => println!("Success: {}", value),
    Err(error) => println!("Error: {}", error),
}
```

### 3. Option Type
```rust
// Option<T> - Either Some(value) or None
let maybe_value: Option<i32> = Some(42);

// Pattern matching
match maybe_value {
    Some(value) => println!("Value: {}", value),
    None => println!("No value"),
}

// if let syntax
if let Some(value) = maybe_value {
    println!("Value: {}", value);
}
```

### 4. Iterators
```rust
let numbers = vec![1, 2, 3, 4, 5];

let doubled: Vec<i32> = numbers
    .iter()                    // Create iterator
    .filter(|&n| n > 2)       // Keep elements > 2
    .map(|&n| n * 2)          // Double each element
    .collect();               // Collect into Vec
```

### 5. Async/Await
```rust
// Async function
async fn fetch_data() -> Result<String> {
    let response = reqwest::get("https://api.example.com").await?;
    let text = response.text().await?;
    Ok(text)
}

// Using async function
#[tokio::main]
async fn main() {
    let data = fetch_data().await.unwrap();
    println!("{}", data);
}
```

---

## Frontend Integration

### WebSocket Client (`app.js`)

```javascript
class CalendarMonitor {
    constructor() {
        this.ws = null;
        this.reconnectInterval = null;
        this.init();
    }

    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;
        
        this.ws = new WebSocket(wsUrl);
        
        this.ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            this.updateMeetingDisplay(data);
        };
    }

    updateMeetingDisplay(data) {
        this.updateCurrentMeeting(data.current_meeting, data.countdown_seconds);
        this.updateNextMeeting(data.next_meeting);
        this.updateActiveTimeBlocks(data.active_time_blocks);
    }
}
```

### CSS Grid Layout

```css
main {
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: auto 1fr;
    grid-template-areas: 
        "timeblock timeblock"
        "current next";
}

.time-blocks { grid-area: timeblock; }
.current-meeting { grid-area: current; }
.next-meeting { grid-area: next; }
```

---

## Performance Considerations

1. **Caching**: 5-minute cache reduces ICS parsing overhead
2. **Async I/O**: Non-blocking file/network operations
3. **WebSocket**: Real-time updates without polling
4. **Selective Updates**: Only fetches data when cache expires
5. **Memory Management**: Rust's ownership system prevents memory leaks

---

## Security Features

1. **Type Safety**: Rust prevents many common bugs at compile time
2. **Memory Safety**: No buffer overflows or use-after-free
3. **CORS**: Configurable cross-origin resource sharing
4. **Environment Variables**: Sensitive data stored in .env files

---

## Debugging and Logging

The application uses structured logging with the `tracing` crate:

```rust
// Different log levels
tracing::info!("Server starting");
tracing::warn!("Cache expired");
tracing::debug!("Processing {} meetings", count);
tracing::error!("Failed to parse ICS: {}", error);
```

Enable debug logging with:
```bash
RUST_LOG=debug cargo run
```

---

This documentation provides a comprehensive overview of the Calendar Monitor project, explaining both the high-level architecture and the detailed implementation of each function. The Rust-specific concepts are explained in context to help newcomers understand the language features being used.
