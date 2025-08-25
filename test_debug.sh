#!/bin/bash

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}System Monitor Debug & Test Script${NC}"
echo "====================================="

# Step 1: Check if cargo is installed
echo -e "\n${YELLOW}1. Checking Rust installation...${NC}"
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓ Cargo found: $(cargo --version)${NC}"
else
    echo -e "${RED}✗ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Step 2: Clean and rebuild
echo -e "\n${YELLOW}2. Cleaning and rebuilding project...${NC}"
cargo clean
cargo build --release
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Step 3: Test the collectors
echo -e "\n${YELLOW}3. Running collector tests...${NC}"
cargo test --lib -- --nocapture
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed${NC}"
else
    echo -e "${YELLOW}⚠ Some tests failed (this might be okay)${NC}"
fi

# Step 4: Create the static directory if it doesn't exist
echo -e "\n${YELLOW}4. Setting up static directory...${NC}"
mkdir -p static
echo -e "${GREEN}✓ Static directory ready${NC}"

# Step 5: Test SSE endpoint with curl (in background)
echo -e "\n${YELLOW}5. Starting the server...${NC}"
echo -e "${GREEN}Starting server on http://127.0.0.1:8080${NC}"
echo -e "${GREEN}Press Ctrl+C to stop${NC}\n"

# Set environment variables for better logging
export RUST_LOG=info
export RUST_BACKTRACE=1

# Run the server
cargo run --release

# Alternative: If you want to test the SSE endpoint directly
# You can uncomment the following lines and run them in a separate terminal:
#
# echo "Testing SSE endpoint with curl:"
# curl -N -H "Accept: text/event-stream" http://127.0.0.1:8080/metrics/stream