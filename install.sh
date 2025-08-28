#!/bin/bash
set -e

# Calendar Monitor Installation Script
# This script downloads and installs the Calendar Monitor binary for your system

GITHUB_REPO="systemcraftsman/calendar-monitor"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/calendar-monitor"
SYSTEMD_DIR="/etc/systemd/system"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "linux-x86_64"
            ;;
        aarch64)
            echo "linux-arm64"
            ;;
        armv7l)
            echo "linux-armv7"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Get latest release version
get_latest_version() {
    log_info "Fetching latest release version..."
    local version=$(curl -s "https://api.github.com/repos/$GITHUB_REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$version" ]; then
        log_error "Failed to fetch latest version"
        exit 1
    fi
    echo "$version"
}

# Download and install binary
install_binary() {
    local version=$1
    local arch=$2
    local download_url="https://github.com/$GITHUB_REPO/releases/download/$version/calendar-monitor-$arch.tar.gz"
    local temp_dir=$(mktemp -d)
    
    log_info "Downloading Calendar Monitor $version for $arch..."
    
    cd "$temp_dir"
    if ! curl -L -o "calendar-monitor.tar.gz" "$download_url"; then
        log_error "Failed to download release"
        exit 1
    fi
    
    log_info "Extracting binary..."
    tar -xzf calendar-monitor.tar.gz
    
    log_info "Installing binary to $INSTALL_DIR..."
    cp calendar-monitor "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/calendar-monitor"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_success "Binary installed successfully"
}

# Create configuration directory and sample config
setup_config() {
    log_info "Setting up configuration..."
    
    mkdir -p "$CONFIG_DIR"
    
    # Create sample configuration file if it doesn't exist
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        cat > "$CONFIG_DIR/config.toml" << 'EOF'
# Calendar Monitor Configuration
# Copy this file to ~/.config/calendar-monitor/config.toml for user-specific config
# or edit this file for system-wide configuration

[server]
host = "0.0.0.0"  # Bind to all interfaces (change to "127.0.0.1" for localhost only)
port = 3000
cache_ttl_seconds = 300

[ics]
# Add your ICS calendar URLs or file paths here
file_paths = [
    # "https://example.com/calendar.ics",
    # "/path/to/local/calendar.ics",
]

[google]
# Google OAuth configuration (optional)
# Get these from Google Cloud Console
# client_id = "your-google-client-id"
# client_secret = "your-google-client-secret"
# redirect_uri = "http://localhost:3000/auth/google/callback"
EOF
        
        log_info "Created sample configuration at $CONFIG_DIR/config.toml"
        log_warn "Please edit $CONFIG_DIR/config.toml to add your calendar sources"
    else
        log_info "Configuration file already exists at $CONFIG_DIR/config.toml"
    fi
}

# Create systemd service
create_systemd_service() {
    log_info "Creating systemd service..."
    
    cat > "$SYSTEMD_DIR/calendar-monitor.service" << EOF
[Unit]
Description=Calendar Monitor
Documentation=https://github.com/$GITHUB_REPO
After=network.target
Wants=network.target

[Service]
Type=simple
User=calendar-monitor
Group=calendar-monitor
ExecStart=$INSTALL_DIR/calendar-monitor
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=calendar-monitor

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$CONFIG_DIR

# Environment
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

    # Create calendar-monitor user if it doesn't exist
    if ! id "calendar-monitor" &>/dev/null; then
        log_info "Creating calendar-monitor user..."
        useradd --system --shell /bin/false --home-dir /var/lib/calendar-monitor calendar-monitor
    fi
    
    # Set permissions
    chown -R calendar-monitor:calendar-monitor "$CONFIG_DIR"
    
    systemctl daemon-reload
    log_success "Systemd service created"
}

# Main installation function
main() {
    log_info "Calendar Monitor Installation Script"
    log_info "======================================"
    
    check_root
    
    local arch=$(detect_arch)
    log_info "Detected architecture: $arch"
    
    local version=$(get_latest_version)
    log_info "Latest version: $version"
    
    install_binary "$version" "$arch"
    setup_config
    create_systemd_service
    
    log_success "Installation completed!"
    echo
    log_info "Next steps:"
    echo "1. Edit the configuration file: $CONFIG_DIR/config.toml"
    echo "2. Add your calendar sources (ICS URLs or file paths)"
    echo "3. Start the service: sudo systemctl start calendar-monitor"
    echo "4. Enable auto-start: sudo systemctl enable calendar-monitor"
    echo "5. Check status: sudo systemctl status calendar-monitor"
    echo "6. View logs: sudo journalctl -u calendar-monitor -f"
    echo "7. Access the web interface: http://localhost:3000"
    echo
    log_info "For Google Calendar integration, see the documentation:"
    log_info "https://github.com/$GITHUB_REPO/blob/main/GOOGLE_OAUTH_SETUP_SIMPLE.md"
}

# Run if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
