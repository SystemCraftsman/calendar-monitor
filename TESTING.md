# Testing Documentation

The Calendar Monitor application includes a comprehensive test suite with 15 unit tests covering all core functionality.

## Test Structure

```
tests/
├── mod.rs                    # Test module configuration
├── calendar_tests.rs         # Calendar and RRULE parsing tests (5 tests)
├── meeting_tests.rs          # Meeting logic and filtering tests (4 tests)
└── google_calendar_tests.rs # Google OAuth integration tests (6 tests)
```

## Running Tests

### Basic Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run in release mode (optimized)
cargo test --release

# Run a specific test
cargo test test_parse_rrule_until
```

### Specific Test Modules

```bash
# Calendar parsing tests
cargo test --test calendar_tests

# Meeting logic tests  
cargo test --test meeting_tests

# Google OAuth tests
cargo test --test google_calendar_tests
```

## Test Coverage

### Calendar Tests (5 tests)

| Test | Description |
|------|-------------|
| `test_parse_rrule_until` | Tests RRULE UNTIL clause parsing for recurring events |
| `test_meeting_status` | Tests meeting status detection (active, upcoming, ended) |
| `test_time_block_detection` | Tests time block vs regular meeting identification |
| `test_meeting_countdown` | Tests countdown timer calculations |
| `test_meeting_builder` | Tests Meeting struct builder pattern |

### Meeting Tests (4 tests)

| Test | Description |
|------|-------------|
| `test_meeting_deduplication_logic` | Tests duplicate event handling (keeps later end time) |
| `test_meeting_filtering_by_status` | Tests filtering by meeting status |
| `test_time_blocks_vs_regular_meetings` | Tests separation of time blocks and meetings |
| `test_meeting_sorting` | Tests chronological sorting of meetings |

### Google Calendar Tests (6 tests)

| Test | Description |
|------|-------------|
| `test_google_oauth_config_creation` | Tests OAuth configuration structure |
| `test_google_calendar_service_creation` | Tests service instantiation |
| `test_google_calendar_service_env_creation_missing_vars` | Tests behavior with missing environment variables |
| `test_google_calendar_service_env_creation_with_vars` | Tests behavior with environment variables present |
| `test_auth_url_generation` | Tests OAuth authorization URL generation |
| `test_google_event_conversion` | Tests service structure for event conversion |

## Test Examples

### RRULE Parsing Test

```rust
#[test]
fn test_parse_rrule_until() {
    let service = CalendarService::new();
    
    // Test valid UNTIL date
    let rrule = "FREQ=WEEKLY;WKST=MO;UNTIL=20250620T235959Z;BYDAY=MO,TU,WE,TH,FR";
    let until_date = service.parse_rrule_until(rrule);
    assert_eq!(until_date, Some(NaiveDate::from_ymd_opt(2025, 6, 20).unwrap()));
    
    // Test RRULE without UNTIL
    let rrule2 = "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR";
    let until_date2 = service.parse_rrule_until(rrule2);
    assert_eq!(until_date2, None);
}
```

### Meeting Status Test

```rust
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
}
```

### OAuth Configuration Test

```rust
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
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --release
      - name: Run tests with coverage
        run: cargo test --release -- --nocapture
```

### Local Testing Workflow

```bash
# 1. Check compilation
cargo check

# 2. Run all tests
cargo test

# 3. Run tests with detailed output
cargo test -- --nocapture

# 4. Test specific functionality
cargo test test_parse_rrule_until -- --nocapture

# 5. Build in release mode
cargo build --release
```

## Test Data

Tests use realistic data structures matching production usage:

- **Meeting times**: Use `Utc::now()` with relative offsets
- **RRULE patterns**: Real-world recurring event patterns
- **OAuth configs**: Realistic client IDs and redirect URIs
- **ICS data**: Simulated but valid calendar event structures

## Performance Testing

```bash
# Run tests in release mode for performance
cargo test --release

# Time individual tests
cargo test test_parse_rrule_until --release -- --nocapture --exact

# Memory usage profiling
cargo test --release -- --test-threads=1
```

## Coverage Goals

The test suite aims for:
- ✅ **100% coverage** of public API methods
- ✅ **Edge case testing** for RRULE parsing
- ✅ **Error handling** verification
- ✅ **OAuth flow** validation
- ✅ **Meeting logic** comprehensive testing

## Adding New Tests

When adding new functionality:

1. **Add unit tests** in the appropriate test file
2. **Test error conditions** and edge cases
3. **Use realistic test data** 
4. **Follow naming conventions**: `test_functionality_description`
5. **Include assertions** for all expected behaviors
6. **Test both success and failure paths**

## Debugging Tests

```bash
# Run a single test with debug output
cargo test test_name -- --nocapture

# Set environment variables for tests
RUST_LOG=debug cargo test -- --nocapture

# Use println! in tests for debugging
#[test]
fn test_example() {
    println!("Debug output: {:?}", some_value);
    assert_eq!(expected, actual);
}
```
