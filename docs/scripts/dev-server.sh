#!/bin/bash
# Development server script for Polymera OS documentation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m' 
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting Polymera OS Documentation Development Server${NC}"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js is not installed${NC}"
    echo "Please install Node.js 18+ to run the documentation server"
    exit 1
fi

# Check Node.js version
NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo -e "${YELLOW}Warning: Node.js version is $NODE_VERSION, but 18+ is recommended${NC}"
fi

# Change to docs directory
cd "$(dirname "$0")/../site"

# Install dependencies if needed
if [ ! -d "node_modules" ] || [ "package.json" -nt "node_modules" ]; then
    echo -e "${YELLOW}Installing Node.js dependencies...${NC}"
    npm install
fi

# Start development server
echo -e "${GREEN}Starting Docusaurus development server...${NC}"
echo -e "Documentation will be available at: ${GREEN}http://localhost:3000${NC}"
echo -e "Press ${YELLOW}Ctrl+C${NC} to stop the server"

npm run start
