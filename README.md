# 📅 Calendar Monitor

A real-time calendar monitoring application built with Rust that displays your current and upcoming events with live countdowns. Perfect for keeping track of your schedule on a separate monitor or display.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![WebSocket](https://img.shields.io/badge/WebSocket-FF6600?style=for-the-badge&logo=websocket&logoColor=white)
![HTML5](https://img.shields.io/badge/html5-%23E34F26.svg?style=for-the-badge&logo=html5&logoColor=white)
![CSS3](https://img.shields.io/badge/css3-%231572B6.svg?style=for-the-badge&logo=css3&logoColor=white)
![JavaScript](https://img.shields.io/badge/javascript-%23323330.svg?style=for-the-badge&logo=javascript&logoColor=%23F7DF1E)

## ✨ Features

### 🎯 **Core Functionality**
- **Real-time Updates**: Live countdown timers with WebSocket connections
- **Current Event**: Shows active meeting/event with time remaining
- **Next Event**: Displays upcoming event with start countdown
- **Time Blocks**: Special support for time blocks (events with `[brackets]`)
- **Multi-Calendar Support**: Read from multiple ICS files simultaneously
- **Smart Caching**: Efficient 5-minute caching to reduce load times

### 📅 **Calendar Integration**
- **Google Calendar OAuth**: Easy "Login with Google" integration - no complex setup needed
- **ICS File Support**: Read from local `.ics` files or live URLs
- **Multi-Source Support**: Combine Google Calendar with ICS feeds seamlessly
- **Recurring Events**: Full support for weekly recurring events with `RRULE` and `UNTIL` clauses
- **Timezone Handling**: Proper timezone conversion (supports Europe/Istanbul)
- **Cross-midnight Events**: Handles events that span midnight correctly
- **Event Filtering**: Separate handling of regular events vs time blocks

### 🎨 **Modern UI**
- **Responsive Design**: Mobile and desktop friendly
- **Grid Layout**: Clean CSS Grid-based layout
- **Live Animations**: Heartbeat effect for active current events
- **Date Display**: Smart date formatting (Tomorrow, weekday names, etc.)
- **Status Indicators**: Visual connection status and event states

### ⚡ **Performance**
- **Async Architecture**: Non-blocking I/O operations
- **Efficient Parsing**: Fast ICS file processing with deduplication
- **Memory Safe**: Built with Rust's memory safety guarantees
- **Low Resource Usage**: Minimal CPU and memory footprint

## 🖼️ Interface Overview

The application displays three main sections:

```
┌─────────────────────────────────────────┐
│     📅 Calendar Monitor  🔗 Connect     │
│           📋 Time Block: Draft.dev      │
│        22:00-01:00 • 45:23 remaining    │
└─────────────────────────────────────────┘
┌──────────────────────┬──────────────────┐
│   🎯 Current Event   │   ⏭️ Next Event   │
│                      │                  │
│   Team Standup       │   Project Review │
│   10:00-10:30        │   Tomorrow       │
│   ⏰ 05:23 remaining  │   14:00-15:00    │
│                      │   Starts in 3h   │
└──────────────────────┴──────────────────┘
```

## 🚀 **Quick Installation**

### **One-Line Installation** (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/systemcraftsman/calendar-monitor/main/install.sh | sudo bash
```

This automatically:
- ✅ **Detects your architecture** (x86_64, ARM64, ARMv7)
- ✅ **Downloads the correct binary** for your system
- ✅ **Sets up systemd service** for auto-start
- ✅ **Creates configuration** at `/etc/calendar-monitor/config.toml`
- ✅ **Works on Raspberry Pi** (ARM64/ARMv7)

### **Supported Platforms**

| Platform | Architecture | Status |
|----------|-------------|--------|
| 🐧 **Linux x86_64** | Intel/AMD 64-bit | ✅ Fully Supported |
| 🍓 **Raspberry Pi 4** | ARM64 (aarch64) | ✅ Fully Supported |
| 🍓 **Raspberry Pi 3** | ARMv7 | ✅ Fully Supported |
| 🍎 **macOS Intel** | x86_64 | ✅ Fully Supported |
| 🍎 **macOS Apple Silicon** | ARM64 | ✅ Fully Supported |
| 🪟 **Windows 10/11** | x86_64 | ✅ Fully Supported |

### **After Installation**

1. **Configure your calendars**:
   ```bash
   sudo nano /etc/calendar-monitor/config.toml
   ```

2. **Start the service**:
   ```bash
   sudo systemctl start calendar-monitor
   sudo systemctl enable calendar-monitor  # Auto-start on boot
   ```

3. **Access the interface**: http://localhost:3000

4. **Connect Google Calendar** (optional): Click "Connect Google Calendar"

### **Alternative Installation Methods**

- 📦 **[Manual Installation](INSTALLATION.md)** - Download pre-built binaries
- 🔨 **[Build from Source](INSTALLATION.md#building-from-source)** - For developers
- 🐳 **Docker** - Coming soon

## ⚙️ Configuration

### **Configuration Files** (Recommended)

Calendar Monitor uses a flexible configuration system with **TOML** files:

**Configuration locations** (in order of priority):
1. `./calendar-monitor.toml` (current directory)
2. `~/.config/calendar-monitor/config.toml` (user config)  
3. `/etc/calendar-monitor/config.toml` (system config)

**Sample configuration:**
```toml
[server]
host = "0.0.0.0"         # Bind to all interfaces ("127.0.0.1" for localhost only)
port = 3000              # Web server port
cache_ttl_seconds = 300  # Cache duration

[ics]
file_paths = [
    "https://calendar.google.com/calendar/ical/your-id/public/basic.ics",
    "https://outlook.live.com/owa/calendar/your-id/calendar.ics", 
    "/path/to/local/calendar.ics",
]

[google]
# Optional: Google Calendar OAuth integration
client_id = "your-google-client-id"
client_secret = "your-google-client-secret"  
redirect_uri = "http://localhost:3000/auth/google/callback"
```

### **Environment Variables** (Override config files)

For dynamic configuration or deployment:

```bash
# Server configuration  
export CALENDAR_MONITOR_HOST="127.0.0.1"
export CALENDAR_MONITOR_PORT="8080"
export CALENDAR_MONITOR_CACHE_TTL="600"

# Calendar sources (comma-separated)
export ICS_FILE_PATHS="https://cal1.ics,https://cal2.ics,/local/cal.ics"

# Google OAuth (optional)
export GOOGLE_CLIENT_ID="your-client-id"
export GOOGLE_CLIENT_SECRET="your-client-secret"
export GOOGLE_REDIRECT_URI="http://localhost:3000/auth/google/callback"
```

### Supported Calendar Sources

#### 🔗 **Google Calendar (Recommended)**
Easiest setup - just click "Connect Google Calendar" button:
1. No complex configuration needed
2. Works around company restrictions
3. Automatic authentication flow
4. Direct API access

See [Google OAuth Setup Guide](GOOGLE_OAUTH_SETUP_SIMPLE.md) for details.

#### 📁 **Local ICS Files**
```env
ICS_FILE_PATHS=./calendars/work.ics,./calendars/personal.ics
```

#### 🌐 **Live Calendar URLs**
```env
ICS_FILE_PATHS=https://calendar.google.com/calendar/ical/your-calendar-id/basic.ics
```

#### 🔗 **Mixed Sources**
Combine Google Calendar OAuth with ICS feeds:
```env
ICS_FILE_PATHS=./local-calendar.ics,https://remote-calendar.com/feed.ics
# Plus Google OAuth for additional events
```

### Calendar Setup Examples

<details>
<summary><strong>📅 Google Calendar</strong></summary>

1. Open Google Calendar
2. Go to Settings → Your calendar → Integrate calendar
3. Copy the "Secret address in iCal format"
4. Configure your calendar source in `calendar-monitor.toml` or set environment variable:
   ```bash
   export ICS_FILE_PATHS=https://calendar.google.com/calendar/ical/your-id/private-key/basic.ics
   ```
</details>

<details>
<summary><strong>📅 Outlook Calendar</strong></summary>

1. Open Outlook.com
2. Go to Settings → View all Outlook settings → Calendar → Shared calendars
3. Publish your calendar and copy the ICS link
4. Configure your calendar source in `calendar-monitor.toml` or set environment variable:
   ```bash
   export ICS_FILE_PATHS=https://outlook.live.com/owa/calendar/your-calendar-id/calendar.ics
   ```
</details>

<details>
<summary><strong>📅 Apple iCloud Calendar</strong></summary>

1. Open iCloud Calendar
2. Click the share icon next to your calendar
3. Enable "Public Calendar" and copy the link
4. Change `webcal://` to `https://` in the URL
5. Configure your calendar source in `calendar-monitor.toml` or set environment variable:
   ```env
   ICS_FILE_PATHS=https://p01-calendarws.icloud.com/published/2/your-calendar-id
   ```
</details>

### Time Block Events

Time blocks are special events with titles enclosed in brackets:

```
[Draft.dev]        ← Detected as time block
[Red Hat Work]     ← Detected as time block
Regular Meeting    ← Regular event
```

Time blocks appear in the top section and don't interfere with regular meeting scheduling.

## 🏗️ Project Structure

```
calendar-monitor/
├── src/
│   ├── main.rs           # Web server, routes, WebSocket handling
│   ├── calendar.rs       # ICS parsing, calendar service, caching
│   └── meeting.rs        # Meeting data structure and methods
├── static/
│   ├── app.js           # Frontend JavaScript, WebSocket client
│   └── style.css        # CSS styling and responsive design
├── templates/
│   └── index.html       # HTML template
├── Cargo.toml           # Dependencies and project metadata
├── calendar-monitor.toml # Configuration file (optional)
└── README.md
```

## 🛠️ Technologies Used

### **Backend (Rust)**
- **[Axum](https://github.com/tokio-rs/axum)** - Modern web framework
- **[Tokio](https://tokio.rs/)** - Async runtime
- **[Serde](https://serde.rs/)** - JSON serialization
- **[Chrono](https://github.com/chronotope/chrono)** - Date/time handling
- **[iCal](https://github.com/Peltoche/ical-rs)** - ICS file parsing
- **[reqwest](https://github.com/seanmonstar/reqwest)** - HTTP client
- **[anyhow](https://github.com/dtolnay/anyhow)** - Error handling
- **[tracing](https://github.com/tokio-rs/tracing)** - Structured logging

### **Frontend**
- **HTML5** - Semantic markup
- **CSS3** - Grid layout, animations, responsive design
- **JavaScript (ES6+)** - WebSocket client, DOM manipulation
- **WebSocket** - Real-time communication

## 🔧 Development

### Running in Development Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Run with auto-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

### Building for Production

```bash
# Optimized release build
cargo build --release

# Run production binary
./target/release/calendar-monitor
```

### Testing

```bash
# Run unit tests
cargo test

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out html
```

## 📊 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Serve main HTML page |
| `/ws` | GET | WebSocket upgrade for real-time updates |
| `/api/meetings` | GET | JSON API for current meeting data |
| `/auth/google/login` | GET | Google OAuth login redirect |
| `/auth/google/callback` | GET | Google OAuth callback handler |
| `/static/*` | GET | Static assets (CSS, JS) |

### WebSocket Message Format

```json
{
  "current_meeting": {
    "title": "Team Standup",
    "start_time": "2024-01-15T10:00:00Z",
    "end_time": "2024-01-15T10:30:00Z",
    "description": null,
    "location": "Conference Room A"
  },
  "next_meeting": {
    "title": "Project Review",
    "start_time": "2024-01-15T14:00:00Z",
    "end_time": "2024-01-15T15:00:00Z"
  },
  "countdown_seconds": 1823,
  "active_time_blocks": [
    {
      "title": "[Draft.dev]",
      "start_time": "2024-01-15T19:00:00Z",
      "end_time": "2024-01-15T22:00:00Z"
    }
  ]
}
```

## 🐛 Troubleshooting

### Common Issues

<details>
<summary><strong>No events showing</strong></summary>

1. Check your configuration file or environment variables
2. Verify ICS file/URL accessibility
3. Check logs: `RUST_LOG=debug cargo run`
4. Ensure events are for today/tomorrow
</details>

<details>
<summary><strong>WebSocket connection failed</strong></summary>

1. Check if port 3000 is available
2. Verify firewall settings
3. Try a different browser or incognito mode
4. Check browser console for errors
</details>

<details>
<summary><strong>Recurring events not working</strong></summary>

1. Verify RRULE format in ICS file
2. Check timezone settings
3. Currently only supports `FREQ=WEEKLY` rules
4. Enable debug logging to see parsing details
</details>

<details>
<summary><strong>Time zones incorrect</strong></summary>

1. Check ICS file timezone specification
2. Currently hardcoded for Europe/Istanbul
3. Modify `parse_ical_datetime` for your timezone
4. See technical documentation for details
</details>

### Development & Production

The application is optimized for production with clean logging:

```bash
# Production mode (default)
cargo run

# Development with debug logging
RUST_LOG=debug cargo run

# Run comprehensive test suite
cargo test
```

**Production Benefits**:
- Clean, minimal logging output
- Optimized performance without debug overhead
- 15 comprehensive unit tests ensure reliability
- All debugging functionality covered by tests

## 📚 Documentation

- **[Installation Guide](INSTALLATION.md)** - Complete installation instructions for all platforms
- **[Google OAuth Setup](GOOGLE_OAUTH_SETUP_SIMPLE.md)** - Easy Google Calendar integration (recommended)
- **[Testing Guide](TESTING.md)** - Comprehensive test documentation and coverage
- **[Technical Documentation](TECHNICAL_DOCUMENTATION.md)** - Detailed code explanation and Rust concepts
- **[ICS Setup Guide](ICS_CALENDAR_SETUP.md)** - Manual calendar file setup

## 🤝 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch**
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make your changes**
4. **Add tests** if applicable
5. **Run the test suite**
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```
6. **Commit your changes**
   ```bash
   git commit -m 'Add amazing feature'
   ```
7. **Push to the branch**
   ```bash
   git push origin feature/amazing-feature
   ```
8. **Open a Pull Request**

### Development Guidelines

- Follow Rust naming conventions
- Add documentation for public functions
- Include tests for new functionality
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` to catch common issues

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[Rust Community](https://www.rust-lang.org/community)** - Amazing language and ecosystem
- **[Axum Framework](https://github.com/tokio-rs/axum)** - Excellent web framework
- **[iCal-rs](https://github.com/Peltoche/ical-rs)** - ICS parsing library
- **[Tokio](https://tokio.rs/)** - Async runtime

---

<div align="center">

**Built with ❤️ and 🦀 Rust**

[Report Bug](https://github.com/yourusername/calendar-monitor/issues) • [Request Feature](https://github.com/yourusername/calendar-monitor/issues) • [Documentation](TECHNICAL_DOCUMENTATION.md)

</div>
