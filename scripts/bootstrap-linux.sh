#!/bin/bash
# Aetheris OS Linux Bootstrap Script
# Installs development dependencies for Linux development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${CYAN}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect package manager
detect_package_manager() {
    if command_exists apt-get; then
        echo "apt"
    elif command_exists yum; then
        echo "yum"
    elif command_exists dnf; then
        echo "dnf"
    elif command_exists pacman; then
        echo "pacman"
    elif command_exists zypper; then
        echo "zypper"
    else
        echo "unknown"
    fi
}

# Function to install packages
install_packages() {
    local pkg_manager="$1"
    shift
    local packages=("$@")
    
    case "$pkg_manager" in
        "apt")
            sudo apt-get update
            sudo apt-get install -y "${packages[@]}"
            ;;
        "yum")
            sudo yum install -y "${packages[@]}"
            ;;
        "dnf")
            sudo dnf install -y "${packages[@]}"
            ;;
        "pacman")
            sudo pacman -S --noconfirm "${packages[@]}"
            ;;
        "zypper")
            sudo zypper install -y "${packages[@]}"
            ;;
        *)
            print_error "Unknown package manager: $pkg_manager"
            return 1
            ;;
    esac
}

# Function to install Rust
install_rust() {
    if ! command_exists cargo; then
        print_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_success "Rust installed successfully"
    else
        print_success "Rust already installed: $(cargo --version)"
    fi
}

# Function to install Go
install_go() {
    if ! command_exists go; then
        print_info "Installing Go..."
        local go_version="1.22.0"
        local go_arch
        case "$(uname -m)" in
            x86_64) go_arch="amd64" ;;
            arm64|aarch64) go_arch="arm64" ;;
            *) go_arch="amd64" ;;
        esac
        
        local go_tarball="go${go_version}.linux-${go_arch}.tar.gz"
        local go_url="https://go.dev/dl/${go_tarball}"
        
        cd /tmp
        wget -q "$go_url"
        sudo tar -C /usr/local -xzf "$go_tarball"
        rm "$go_tarball"
        
        # Add Go to PATH
        echo 'export PATH=$PATH:/usr/local/go/bin' >> "$HOME/.bashrc"
        echo 'export PATH=$PATH:/usr/local/go/bin' >> "$HOME/.profile"
        export PATH=$PATH:/usr/local/go/bin
        
        print_success "Go installed successfully"
    else
        print_success "Go already installed: $(go version)"
    fi
}

# Function to install Node.js
install_nodejs() {
    if ! command_exists node; then
        print_info "Installing Node.js..."
        local pkg_manager="$1"
        
        case "$pkg_manager" in
            "apt")
                curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
                sudo apt-get install -y nodejs
                ;;
            "yum"|"dnf")
                curl -fsSL https://rpm.nodesource.com/setup_20.x | sudo bash -
                sudo "$pkg_manager" install -y nodejs
                ;;
            "pacman")
                sudo pacman -S --noconfirm nodejs npm
                ;;
            "zypper")
                sudo zypper addrepo https://download.opensuse.org/repositories/devel:languages:nodejs/openSUSE_Leap_15.4/ devel:languages:nodejs
                sudo zypper refresh
                sudo zypper install -y nodejs20 npm20
                ;;
            *)
                # Fallback to NodeSource
                curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
                sudo apt-get install -y nodejs
                ;;
        esac
        
        print_success "Node.js installed successfully"
    else
        print_success "Node.js already installed: $(node --version)"
    fi
}

# Function to install Python
install_python() {
    if ! command_exists python3.11; then
        print_info "Installing Python 3.11..."
        local pkg_manager="$1"
        
        case "$pkg_manager" in
            "apt")
                sudo apt-get install -y software-properties-common
                sudo add-apt-repository -y ppa:deadsnakes/ppa
                sudo apt-get update
                sudo apt-get install -y python3.11 python3.11-pip python3.11-venv python3.11-dev
                ;;
            "yum"|"dnf")
                sudo "$pkg_manager" install -y python3.11 python3.11-pip python3.11-devel
                ;;
            "pacman")
                sudo pacman -S --noconfirm python python-pip
                ;;
            "zypper")
                sudo zypper install -y python311 python311-pip python311-devel
                ;;
            *)
                print_warning "Installing Python 3.11 via package manager may not be available"
                ;;
        esac
        
        print_success "Python 3.11 installed successfully"
    else
        print_success "Python 3.11 already installed: $(python3.11 --version)"
    fi
}

# Main bootstrap function
main() {
    echo -e "${GREEN}🚀 Aetheris OS Linux Bootstrap${NC}"
    echo -e "${GREEN}================================${NC}"
    echo ""
    
    # Detect package manager
    local pkg_manager
    pkg_manager=$(detect_package_manager)
    print_info "Detected package manager: $pkg_manager"
    
    # Install system dependencies
    print_status "Installing system dependencies..."
    case "$pkg_manager" in
        "apt")
            install_packages "apt" \
                git git-lfs curl wget build-essential \
                cmake ninja-build llvm clang \
                pkg-config libssl-dev libffi-dev \
                python3-dev python3-pip python3-venv
            ;;
        "yum"|"dnf")
            install_packages "$pkg_manager" \
                git git-lfs curl wget gcc gcc-c++ make \
                cmake ninja-build llvm clang \
                pkgconfig openssl-devel libffi-devel \
                python3-devel python3-pip
            ;;
        "pacman")
            install_packages "pacman" \
                git git-lfs curl wget base-devel \
                cmake ninja llvm clang \
                pkg-config openssl libffi \
                python python-pip
            ;;
        "zypper")
            install_packages "zypper" \
                git git-lfs curl wget gcc gcc-c++ make \
                cmake ninja llvm clang \
                pkg-config libopenssl-devel libffi-devel \
                python3-devel python3-pip
            ;;
        *)
            print_error "Unsupported package manager: $pkg_manager"
            exit 1
            ;;
    esac
    
    # Install language runtimes
    install_rust
    install_go
    install_nodejs "$pkg_manager"
    install_python "$pkg_manager"
    
    # Install additional tools
    print_status "Installing additional tools..."
    
    # Install Rust components
    if command_exists cargo; then
        print_info "Installing Rust components..."
        "$HOME/.cargo/bin/rustup" component add rustfmt clippy
        "$HOME/.cargo/bin/rustup" toolchain install stable
    fi
    
    # Install Go tools
    if command_exists go; then
        print_info "Installing Go tools..."
        go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
    fi
    
    # Install Node.js tools
    if command_exists npm; then
        print_info "Installing Node.js tools..."
        npm install -g typescript prettier markdown-link-check
    fi
    
    # Install Python tools
    if command_exists pip3; then
        print_info "Installing Python tools..."
        pip3 install --user black isort flake8 pytest ruff
    fi
    
    # Final checklist
    echo ""
    print_status "Final checklist..."
    
    local tools=(
        "git:Git"
        "git-lfs:Git LFS"
        "cargo:Rust"
        "go:Go"
        "node:Node.js"
        "python3:Python"
        "cmake:CMake"
        "ninja:Ninja"
        "clang:LLVM"
    )
    
    local all_installed=true
    for tool_info in "${tools[@]}"; do
        local cmd="${tool_info%%:*}"
        local name="${tool_info##*:}"
        
        if command_exists "$cmd"; then
            print_success "$name"
        else
            print_error "$name"
            all_installed=false
        fi
    done
    
    echo ""
    if $all_installed; then
        print_success "Bootstrap completed successfully!"
        echo ""
        print_info "Next steps:"
        echo "1. Restart your terminal or run: source ~/.bashrc"
        echo "2. Run: make help"
        echo "3. Run: make bootstrap"
        echo "4. Run: make test"
    else
        print_warning "Bootstrap completed with some issues."
        print_info "Please install missing tools manually and restart your terminal."
    fi
    
    echo ""
    print_info "Useful commands:"
    echo "  make help     - Show available targets"
    echo "  make fmt      - Format all code"
    echo "  make lint     - Lint all code"
    echo "  make test     - Run tests"
    echo "  make docs     - Validate documentation"
    echo "  make package  - Package CLI tools"
    
    echo ""
    print_info "PATH hints:"
    echo "  Rust:     ~/.cargo/bin"
    echo "  Go:       ~/go/bin"
    echo "  Node.js:  ~/.npm-global/bin (if configured)"
    echo "  Python:   ~/.local/bin"
}

# Run main function
main "$@"
