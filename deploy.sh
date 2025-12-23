#!/bin/bash
#
# Cross-compile and deploy Sphero RVR Rust library to Raspberry Pi
#
# Usage:
#   ./deploy.sh [OPTIONS]
#
# Options:
#   --pi-host HOST    Raspberry Pi hostname/IP (default: raspberrypi.local)
#   --pi-user USER    SSH username (default: pi)
#   --example NAME    Example to deploy (default: basic_connection)
#   --run            Run the binary after deployment
#   --release        Build with release optimizations (default: debug)
#   --help           Show this help message

set -e  # Exit on error

# Default configuration
PI_HOST="${PI_HOST:-raspberrypi.local}"
PI_USER="${PI_USER:-pi}"
EXAMPLE="basic_connection"
RUN_AFTER_DEPLOY=false
BUILD_MODE="debug"
TARGET="aarch64-unknown-linux-gnu"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --pi-host)
            PI_HOST="$2"
            shift 2
            ;;
        --pi-user)
            PI_USER="$2"
            shift 2
            ;;
        --example)
            EXAMPLE="$2"
            shift 2
            ;;
        --run)
            RUN_AFTER_DEPLOY=true
            shift
            ;;
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --help)
            grep '^#' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Sphero RVR Deployment Script${NC}"
echo "================================"
echo "Target: ${TARGET}"
echo "Pi Host: ${PI_USER}@${PI_HOST}"
echo "Example: ${EXAMPLE}"
echo "Build Mode: ${BUILD_MODE}"
echo ""

# Check if cross-compilation target is installed
echo -e "${YELLOW}[1/5] Checking Rust target...${NC}"
if ! rustup target list --installed | grep -q "${TARGET}"; then
    echo "Installing ${TARGET} target..."
    rustup target add "${TARGET}"
else
    echo "Target ${TARGET} already installed"
fi

# Build the project
echo -e "${YELLOW}[2/5] Building project...${NC}"
if [ "$BUILD_MODE" == "release" ]; then
    cargo build --target="${TARGET}" --release --example="${EXAMPLE}"
    BINARY_PATH="target/${TARGET}/release/examples/${EXAMPLE}"
else
    cargo build --target="${TARGET}" --example="${EXAMPLE}"
    BINARY_PATH="target/${TARGET}/debug/examples/${EXAMPLE}"
fi

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Binary not found at ${BINARY_PATH}${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"
echo "Binary size: $(du -h "$BINARY_PATH" | cut -f1)"

# Test SSH connection
echo -e "${YELLOW}[3/5] Testing SSH connection...${NC}"
if ! ssh -o ConnectTimeout=5 "${PI_USER}@${PI_HOST}" "echo 'Connected'" &>/dev/null; then
    echo -e "${RED}Error: Cannot connect to ${PI_USER}@${PI_HOST}${NC}"
    echo "Please check:"
    echo "  - Raspberry Pi is powered on and connected to network"
    echo "  - SSH is enabled on the Pi"
    echo "  - Hostname/IP is correct"
    exit 1
fi
echo -e "${GREEN}SSH connection successful${NC}"

# Create remote directory
echo -e "${YELLOW}[4/5] Creating remote directory...${NC}"
ssh "${PI_USER}@${PI_HOST}" "mkdir -p ~/sphero-rvr/examples"

# Copy binary to Pi
echo -e "${YELLOW}[5/5] Deploying binary...${NC}"
scp "$BINARY_PATH" "${PI_USER}@${PI_HOST}:~/sphero-rvr/examples/${EXAMPLE}"

# Make binary executable
ssh "${PI_USER}@${PI_HOST}" "chmod +x ~/sphero-rvr/examples/${EXAMPLE}"

echo -e "${GREEN}Deployment successful!${NC}"
echo ""
echo "Binary deployed to: ~/sphero-rvr/examples/${EXAMPLE}"
echo ""

# Run the binary if requested
if [ "$RUN_AFTER_DEPLOY" = true ]; then
    echo -e "${YELLOW}Running example on Pi...${NC}"
    echo "----------------------------------------"
    ssh "${PI_USER}@${PI_HOST}" "cd ~/sphero-rvr/examples && RUST_LOG=info ./${EXAMPLE}"
else
    echo "To run the example on the Pi:"
    echo "  ssh ${PI_USER}@${PI_HOST}"
    echo "  cd ~/sphero-rvr/examples"
    echo "  RUST_LOG=info ./${EXAMPLE}"
fi
