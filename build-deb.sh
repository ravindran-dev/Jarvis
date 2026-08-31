#!/bin/bash
set -e

VERSION="1.0.0"
ARCH="amd64"
PACKAGE="jarvis"
BUILD_DIR="target/deb-build"
DEB_NAME="${PACKAGE}_${VERSION}_${ARCH}.deb"

echo "Building release binaries..."
cargo build --workspace --release

echo "Setting up fakeroot directory..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/lib/systemd/system"
mkdir -p "$BUILD_DIR/DEBIAN"

echo "Copying files..."
cp target/release/jarvis "$BUILD_DIR/usr/bin/"
cp target/release/jarvis-daemon "$BUILD_DIR/usr/bin/"
cp deploy/jarvis-daemon.service "$BUILD_DIR/usr/lib/systemd/system/"
cp packaging/deb/control "$BUILD_DIR/DEBIAN/"
cp packaging/deb/postinst "$BUILD_DIR/DEBIAN/"
cp packaging/deb/prerm "$BUILD_DIR/DEBIAN/"

echo "Setting permissions..."
chmod 755 "$BUILD_DIR/usr/bin/jarvis"
chmod 755 "$BUILD_DIR/usr/bin/jarvis-daemon"
chmod 644 "$BUILD_DIR/usr/lib/systemd/system/jarvis-daemon.service"
chmod 755 "$BUILD_DIR/DEBIAN/postinst"
chmod 755 "$BUILD_DIR/DEBIAN/prerm"

echo "Building Debian package..."
dpkg-deb --root-owner-group --build "$BUILD_DIR" "$DEB_NAME"

echo "Successfully built $DEB_NAME"
