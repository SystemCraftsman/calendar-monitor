# Installation Guide

Calendar Monitor supports multiple installation methods and runs on various architectures including Raspberry Pi.

## 🚀 **Quick Installation (Recommended)**

### **One-Line Installation**
```bash
curl -sSL https://raw.githubusercontent.com/systemcraftsman/calendar-monitor/main/install.sh -o install.sh ; chmod +x install.sh ; sudo ./install.sh
```

This script will:
- ✅ Detect your system architecture automatically
- ✅ Download the correct binary for your platform
- ✅ Install to `/usr/local/bin/calendar-monitor`
- ✅ Create configuration directory at `/etc/calendar-monitor/`
- ✅ Set up systemd service for auto-start
- ✅ Create dedicated user account for security

### **After Installation**
1. **Configure your calendars**:
   ```bash
   sudo vim /etc/calendar-monitor/config.toml
   ```

2. **Start the service**:
   ```bash
   sudo systemctl start calendar-monitor
   sudo systemctl enable calendar-monitor  # Auto-start on boot
   ```

3. **Check status**:
   ```bash
   sudo systemctl status calendar-monitor
   sudo journalctl -u calendar-monitor -f  # View live logs
   ```

4. **Access the interface**: http://localhost:3000

---

## 📦 **Manual Installation**

### **1. Download Pre-built Binaries**

Download the appropriate binary for your system from the [releases page](https://github.com/systemcraftsman/calendar-monitor/releases):

| Platform | Architecture | Download |
|----------|-------------|----------|
| **Linux x86_64** | Intel/AMD 64-bit | `calendar-monitor-linux-x86_64.tar.gz` |
| **Linux ARM64** | Raspberry Pi 4, ARM64 | `calendar-monitor-linux-arm64.tar.gz` |
| **Linux ARMv7** | Raspberry Pi 3, ARMv7 | `calendar-monitor-linux-armv7.tar.gz` |
| **macOS x86_64** | Intel Mac | `calendar-monitor-macos-x86_64.tar.gz` |
| **macOS ARM64** | Apple Silicon Mac | `calendar-monitor-macos-arm64.tar.gz` |
| **Windows x86_64** | Windows 10/11 | `calendar-monitor-windows-x86_64.zip` |

### **2. Extract and Install**

**Linux/macOS:**
```bash
# Extract the binary
tar -xzf calendar-monitor-*.tar.gz

# Make executable and move to PATH
chmod +x calendar-monitor
sudo mv calendar-monitor /usr/local/bin/

# Verify installation
calendar-monitor --version
```

**Windows:**
```powershell
# Extract the zip file
# Move calendar-monitor.exe to a directory in your PATH
# Or run directly from the extracted location
```

---

## 🔧 **Configuration**

### **Configuration File Locations** (in order of priority)

1. `./calendar-monitor.toml` (current directory)
2. `~/.config/calendar-monitor/config.toml` (user config)
3. `/etc/calendar-monitor/config.toml` (system config)

### **Sample Configuration**

```toml
[server]
host = "0.0.0.0"        # Bind to all interfaces
port = 3000             # Web server port
cache_ttl_seconds = 300 # Cache duration

[ics]
file_paths = [
    "https://example.com/calendar.ics",
    "/path/to/local/calendar.ics",
]

[google]
# Optional: Google Calendar OAuth integration
client_id = "your-google-client-id"
client_secret = "your-google-client-secret"
redirect_uri = "http://localhost:3000/auth/google/callback"
```

### **Environment Variables** (Override config file)

```bash
# Server configuration
export CALENDAR_MONITOR_HOST="127.0.0.1"
export CALENDAR_MONITOR_PORT="8080"
export CALENDAR_MONITOR_CACHE_TTL="600"

# Calendar sources
export ICS_FILE_PATHS="https://cal1.ics,https://cal2.ics"

# Google OAuth (optional)
export GOOGLE_CLIENT_ID="your-client-id"
export GOOGLE_CLIENT_SECRET="your-client-secret"
export GOOGLE_REDIRECT_URI="http://localhost:3000/auth/google/callback"
```

---

## 🖥️ **Platform-Specific Instructions**

### **Raspberry Pi (ARM64/ARMv7)**

1. **Install using the script** (recommended):
   ```bash
   curl -sSL https://raw.githubusercontent.com/systemcraftsman/calendar-monitor/main/install.sh | sudo bash
   ```

2. **Or manually**:
   ```bash
   # Detect your Pi model
   uname -m  # aarch64 = ARM64, armv7l = ARMv7
   
   # Download appropriate binary
   wget https://github.com/systemcraftsman/calendar-monitor/releases/latest/download/calendar-monitor-linux-arm64.tar.gz
   
   # Install
   tar -xzf calendar-monitor-linux-arm64.tar.gz
   sudo mv calendar-monitor /usr/local/bin/
   ```

3. **Performance notes**:
   - Raspberry Pi 4 (ARM64): Excellent performance
   - Raspberry Pi 3 (ARMv7): Good performance, may take longer to start

### **macOS**

1. **Using the binary**:
   ```bash
   # Download and install
   curl -L -o calendar-monitor.tar.gz https://github.com/systemcraftsman/calendar-monitor/releases/latest/download/calendar-monitor-macos-arm64.tar.gz
   tar -xzf calendar-monitor.tar.gz
   sudo mv calendar-monitor /usr/local/bin/
   ```

2. **Create launch daemon** (auto-start):
   ```bash
   # Create plist file
   sudo tee /Library/LaunchDaemons/com.systemcraftsman.calendar-monitor.plist << 'EOF'
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>Label</key>
       <string>com.systemcraftsman.calendar-monitor</string>
       <key>ProgramArguments</key>
       <array>
           <string>/usr/local/bin/calendar-monitor</string>
       </array>
       <key>RunAtLoad</key>
       <true/>
       <key>KeepAlive</key>
       <true/>
   </dict>
   </plist>
   EOF
   
   # Load and start
   sudo launchctl load /Library/LaunchDaemons/com.systemcraftsman.calendar-monitor.plist
   ```

### **Windows**

1. **Manual installation**:
   - Download `calendar-monitor-windows-x86_64.zip`
   - Extract to `C:\Program Files\CalendarMonitor\`
   - Add to PATH or run directly

2. **Install as Windows Service** (optional):
   ```powershell
   # Using NSSM (Non-Sucking Service Manager)
   # Download NSSM from https://nssm.cc/
   
   nssm install CalendarMonitor "C:\Program Files\CalendarMonitor\calendar-monitor.exe"
   nssm start CalendarMonitor
   ```

---

## 🔧 **Building from Source**

### **Prerequisites**
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git

### **Build Steps**
```bash
# Clone repository
git clone https://github.com/systemcraftsman/calendar-monitor.git
cd calendar-monitor

# Build for your platform
cargo build --release

# Binary will be in target/release/calendar-monitor
```

### **Cross-compilation for Raspberry Pi**
```bash
# Install cross-compilation tools
cargo install cross

# Build for ARM64 (Raspberry Pi 4)
cross build --release --target aarch64-unknown-linux-gnu

# Build for ARMv7 (Raspberry Pi 3)
cross build --release --target armv7-unknown-linux-gnueabihf
```

---

## 🔍 **Troubleshooting**

### **Common Issues**

1. **"Permission denied" when running binary**:
   ```bash
   chmod +x calendar-monitor
   ```

2. **"No ICS file paths configured"**:
   - Edit config file and add calendar URLs
   - Or set `ICS_FILE_PATHS` environment variable

3. **Service won't start**:
   ```bash
   # Check logs
   sudo journalctl -u calendar-monitor -f
   
   # Test manually
   sudo -u calendar-monitor /usr/local/bin/calendar-monitor
   ```

4. **Web interface not accessible**:
   - Check if port 3000 is available: `sudo netstat -tlnp | grep 3000`
   - Try different port: `CALENDAR_MONITOR_PORT=8080 calendar-monitor`
   - Check firewall settings

### **Log Locations**
- **Systemd**: `sudo journalctl -u calendar-monitor -f`
- **Manual run**: Logs to console (use `RUST_LOG=debug` for verbose)

### **Performance Tuning**
```bash
# Increase cache time for better performance
export CALENDAR_MONITOR_CACHE_TTL="900"  # 15 minutes

# Reduce log verbosity
export RUST_LOG="warn"
```

---

## 📋 **Next Steps**

After installation:

1. **Configure your calendar sources** in the config file
2. **Set up Google Calendar OAuth** (optional) - see [Google OAuth Setup Guide](GOOGLE_OAUTH_SETUP_SIMPLE.md)
3. **Access the web interface** at http://localhost:3000
4. **Enable auto-start** with systemd/launchd/service manager

For more help, see:
- [Technical Documentation](TECHNICAL_DOCUMENTATION.md)
- [Google OAuth Setup](GOOGLE_OAUTH_SETUP_SIMPLE.md)
- [GitHub Issues](https://github.com/systemcraftsman/calendar-monitor/issues)
