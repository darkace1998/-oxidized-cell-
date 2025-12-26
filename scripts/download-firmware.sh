#!/bin/bash
# Download and install PS3 firmware for oxidized-cell
#
# This script downloads the official PS3 System Software from Sony's servers
# and places it in the firmware directory for the emulator to use.

set -e

FIRMWARE_URL="http://dus01.ps3.update.playstation.net/update/ps3/image/us/2025_0305_c179ad173bbc08b55431d30947725a4b/PS3UPDAT.PUP"
FIRMWARE_DIR="${1:-firmware}"
FIRMWARE_FILE="$FIRMWARE_DIR/PS3UPDAT.PUP"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  PS3 Firmware Downloader${NC}"
echo -e "${GREEN}  For oxidized-cell PS3 Emulator${NC}"
echo -e "${GREEN}========================================${NC}"
echo

# Check if firmware already exists
if [ -f "$FIRMWARE_FILE" ]; then
    echo -e "${YELLOW}Firmware already exists at: $FIRMWARE_FILE${NC}"
    read -p "Do you want to re-download? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Keeping existing firmware."
        exit 0
    fi
fi

# Create firmware directory
mkdir -p "$FIRMWARE_DIR"

echo -e "Downloading PS3 System Software from Sony's servers..."
echo -e "URL: ${YELLOW}$FIRMWARE_URL${NC}"
echo

# Download with progress
if command -v wget &> /dev/null; then
    wget --progress=bar:force -O "$FIRMWARE_FILE" "$FIRMWARE_URL"
elif command -v curl &> /dev/null; then
    curl -L --progress-bar -o "$FIRMWARE_FILE" "$FIRMWARE_URL"
else
    echo -e "${RED}Error: Neither wget nor curl is installed.${NC}"
    echo "Please install one of them and try again:"
    echo "  sudo apt install wget"
    echo "  sudo apt install curl"
    exit 1
fi

# Verify the download
if [ -f "$FIRMWARE_FILE" ]; then
    FILE_SIZE=$(stat -f%z "$FIRMWARE_FILE" 2>/dev/null || stat -c%s "$FIRMWARE_FILE" 2>/dev/null)
    
    # PS3 firmware is typically around 200MB
    if [ "$FILE_SIZE" -gt 100000000 ]; then
        echo
        echo -e "${GREEN}========================================${NC}"
        echo -e "${GREEN}  Firmware downloaded successfully!${NC}"
        echo -e "${GREEN}========================================${NC}"
        echo
        echo -e "Location: ${YELLOW}$FIRMWARE_FILE${NC}"
        echo -e "Size: $(numfmt --to=iec $FILE_SIZE 2>/dev/null || echo "$FILE_SIZE bytes")"
        echo
        echo "The emulator will automatically use this firmware to decrypt games."
        echo
    else
        echo -e "${RED}Warning: Downloaded file seems too small. Download may have failed.${NC}"
        exit 1
    fi
else
    echo -e "${RED}Error: Failed to download firmware.${NC}"
    exit 1
fi
