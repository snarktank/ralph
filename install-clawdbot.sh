#!/bin/bash
#
# ClawdBot One-Click Installer
# Installs all dependencies and launches ClawdBot onboarding
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Minimum Node.js version required
MIN_NODE_VERSION=22

print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║              ClawdBot One-Click Installer                 ║"
    echo "║         Personal AI Assistant Gateway Setup               ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_command() {
    command -v "$1" &> /dev/null
}

get_node_major_version() {
    if check_command node; then
        node -v | sed 's/v//' | cut -d. -f1
    else
        echo "0"
    fi
}

detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS_NAME=$NAME
        OS_ID=$ID
        OS_VERSION=$VERSION_ID
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS_NAME="macOS"
        OS_ID="macos"
        OS_VERSION=$(sw_vers -productVersion)
    else
        OS_NAME="Unknown"
        OS_ID="unknown"
        OS_VERSION="unknown"
    fi
    log_info "Detected OS: $OS_NAME $OS_VERSION"
}

check_compatibility() {
    log_info "Checking system compatibility..."

    case "$OS_ID" in
        ubuntu|debian|fedora|centos|rhel|arch|manjaro|opensuse*)
            log_success "Linux distribution supported"
            ;;
        macos)
            log_success "macOS supported"
            ;;
        *)
            if [[ "$OSTYPE" == "linux-gnu"* ]]; then
                log_warn "Linux variant detected - should work but not officially tested"
            else
                log_error "Unsupported OS: $OS_ID"
                log_error "ClawdBot supports: macOS, Linux, Windows (WSL2)"
                exit 1
            fi
            ;;
    esac
}

install_node_linux() {
    log_info "Installing Node.js $MIN_NODE_VERSION via NodeSource..."

    # Detect package manager
    if check_command apt-get; then
        PKG_MANAGER="apt"
    elif check_command dnf; then
        PKG_MANAGER="dnf"
    elif check_command yum; then
        PKG_MANAGER="yum"
    elif check_command pacman; then
        PKG_MANAGER="pacman"
    else
        log_error "No supported package manager found (apt, dnf, yum, pacman)"
        log_info "Please install Node.js $MIN_NODE_VERSION+ manually: https://nodejs.org/"
        exit 1
    fi

    case "$PKG_MANAGER" in
        apt)
            log_info "Using apt package manager..."
            # Install prerequisites
            sudo apt-get update -qq
            sudo apt-get install -y -qq ca-certificates curl gnupg

            # Setup NodeSource repo for Node 22
            sudo mkdir -p /etc/apt/keyrings
            curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg 2>/dev/null || true
            echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$MIN_NODE_VERSION.x nodistro main" | sudo tee /etc/apt/sources.list.d/nodesource.list > /dev/null

            sudo apt-get update -qq
            sudo apt-get install -y nodejs
            ;;
        dnf|yum)
            log_info "Using $PKG_MANAGER package manager..."
            curl -fsSL https://rpm.nodesource.com/setup_$MIN_NODE_VERSION.x | sudo bash -
            sudo $PKG_MANAGER install -y nodejs
            ;;
        pacman)
            log_info "Using pacman package manager..."
            sudo pacman -Sy --noconfirm nodejs npm
            ;;
    esac
}

install_node_macos() {
    log_info "Installing Node.js on macOS..."

    if check_command brew; then
        log_info "Using Homebrew..."
        brew install node@$MIN_NODE_VERSION
        brew link --overwrite node@$MIN_NODE_VERSION
    else
        log_warn "Homebrew not found. Installing Homebrew first..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        brew install node@$MIN_NODE_VERSION
        brew link --overwrite node@$MIN_NODE_VERSION
    fi
}

install_node() {
    log_info "Node.js $MIN_NODE_VERSION+ required but not found or version too old"

    if [[ "$OS_ID" == "macos" ]]; then
        install_node_macos
    else
        install_node_linux
    fi

    # Verify installation
    if check_command node; then
        NEW_VERSION=$(get_node_major_version)
        if [[ "$NEW_VERSION" -ge "$MIN_NODE_VERSION" ]]; then
            log_success "Node.js v$(node -v | sed 's/v//') installed successfully"
        else
            log_error "Node.js installation succeeded but version is still below $MIN_NODE_VERSION"
            exit 1
        fi
    else
        log_error "Node.js installation failed"
        log_info "Please install Node.js $MIN_NODE_VERSION+ manually: https://nodejs.org/"
        exit 1
    fi
}

check_node() {
    log_info "Checking Node.js installation..."

    if check_command node; then
        CURRENT_VERSION=$(get_node_major_version)
        log_info "Found Node.js v$(node -v | sed 's/v//')"

        if [[ "$CURRENT_VERSION" -ge "$MIN_NODE_VERSION" ]]; then
            log_success "Node.js version meets requirements (>= $MIN_NODE_VERSION)"
        else
            log_warn "Node.js version $CURRENT_VERSION is below minimum required ($MIN_NODE_VERSION)"
            install_node
        fi
    else
        log_warn "Node.js not found"
        install_node
    fi
}

check_npm() {
    log_info "Checking npm..."

    if check_command npm; then
        log_success "npm v$(npm -v) found"
    else
        log_error "npm not found (should be installed with Node.js)"
        exit 1
    fi
}

install_clawdbot() {
    log_info "Installing ClawdBot globally..."

    # Try to install globally, fall back to user-local if permission denied
    if npm install -g clawdbot@latest 2>/dev/null; then
        log_success "ClawdBot installed globally"
    else
        log_warn "Global install failed (permission issue), trying with sudo..."
        if sudo npm install -g clawdbot@latest; then
            log_success "ClawdBot installed globally with sudo"
        else
            log_error "Failed to install ClawdBot"
            log_info "Try running: sudo npm install -g clawdbot@latest"
            exit 1
        fi
    fi

    # Verify installation
    if check_command clawdbot; then
        log_success "ClawdBot command available"
    else
        log_warn "clawdbot command not in PATH, checking npm global bin..."
        NPM_BIN=$(npm bin -g)
        if [[ -f "$NPM_BIN/clawdbot" ]]; then
            log_info "Adding npm global bin to PATH for this session"
            export PATH="$NPM_BIN:$PATH"
        fi
    fi
}

create_config_dir() {
    log_info "Setting up ClawdBot config directory..."

    CONFIG_DIR="$HOME/.clawdbot"
    if [[ ! -d "$CONFIG_DIR" ]]; then
        mkdir -p "$CONFIG_DIR"
        log_success "Created config directory: $CONFIG_DIR"
    else
        log_info "Config directory already exists: $CONFIG_DIR"
    fi
}

run_onboarding() {
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Installation complete! Starting ClawdBot onboarding...   ${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""

    log_info "Launching onboarding wizard with daemon installation..."
    log_info "This will configure your gateway, workspace, channels, and skills"
    echo ""

    # Run the onboarding wizard
    clawdbot onboard --install-daemon
}

# Error handler
handle_error() {
    log_error "Installation failed at line $1"
    log_info "If you need help, visit: https://github.com/clawdbot/clawdbot/issues"
    exit 1
}

trap 'handle_error $LINENO' ERR

# Main execution
main() {
    print_banner

    log_info "Starting ClawdBot installation..."
    echo ""

    # Step 1: Detect and check OS
    detect_os
    check_compatibility
    echo ""

    # Step 2: Check/install Node.js
    check_node
    echo ""

    # Step 3: Check npm
    check_npm
    echo ""

    # Step 4: Install ClawdBot
    install_clawdbot
    echo ""

    # Step 5: Setup config directory
    create_config_dir
    echo ""

    # Step 6: Run onboarding
    run_onboarding
}

main "$@"
