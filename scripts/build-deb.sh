#!/bin/bash
# Build DEB packages for nanoimage
# Usage: ./scripts/build-deb.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "=== Building DEB packages for nanoimage ==="

# Step 1: Build release
echo "[1/4] Building release binaries..."
cargo build --release --quiet

# Step 2: Create package structure
echo "[2/4] Creating package structure..."
rm -rf debian-package
mkdir -p debian-package/nanoimage/usr/bin debian-package/nanoimage/DEBIAN
mkdir -p debian-package/nanoimage-gui/usr/bin debian-package/nanoimage-gui/usr/share/applications debian-package/nanoimage-gui/DEBIAN

cp target/release/nanoimage debian-package/nanoimage/usr/bin/
cp target/release/nanoimage-gui debian-package/nanoimage-gui/usr/bin/
cp debian/nanoimage-gui.desktop debian-package/nanoimage-gui/usr/share/applications/

# Step 3: Generate DEB packages
echo "[3/4] Generating DEB packages..."
dpkg-deb --build --root-owner-group debian-package/nanoimage nanoimage_0.1.0_amd64.deb
dpkg-deb --build --root-owner-group debian-package/nanoimage-gui nanoimage-gui_0.1.0_amd64.deb

# Step 4: Cleanup
echo "[4/4] Cleaning up..."
rm -rf debian-package

echo ""
echo "=== Build complete ==="
echo "Packages:"
ls -lh nanoimage_0.1.0_amd64.deb nanoimage-gui_0.1.0_amd64.deb
echo ""
echo "To install:"
echo "  sudo dpkg -i nanoimage_0.1.0_amd64.deb"
echo "  sudo dpkg -i nanoimage-gui_0.1.0_amd64.deb"
