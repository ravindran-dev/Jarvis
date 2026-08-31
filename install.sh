#!/bin/bash
set -e

echo "Starting JARVIS v1.0 Installation..."

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run as root (e.g. sudo ./install.sh)"
  exit 1
fi

echo "Building release binaries..."
cargo build --workspace --release

echo "Installing binaries to /usr/bin/..."
cp target/release/jarvis /usr/bin/jarvis
cp target/release/jarvis-daemon /usr/bin/jarvis-daemon
chmod 755 /usr/bin/jarvis
chmod 755 /usr/bin/jarvis-daemon

echo "Installing systemd service..."
cp deploy/jarvis-daemon.service /usr/lib/systemd/system/jarvis-daemon.service
chmod 644 /usr/lib/systemd/system/jarvis-daemon.service

echo "Configuring JARVIS daemon authorization..."
mkdir -p /etc/jarvis
chmod 755 /etc/jarvis

if [ ! -f /etc/jarvis/daemon.env ]; then
  # Determine the target UID for authorization
  if [ -n "$SUDO_UID" ]; then
    TARGET_UID=$SUDO_UID
  else
    TARGET_UID=1000 # Fallback
  fi
  echo "Setting JARVIS_AUTHORIZED_UID=$TARGET_UID"
  echo "JARVIS_AUTHORIZED_UID=$TARGET_UID" > /etc/jarvis/daemon.env
  chmod 644 /etc/jarvis/daemon.env
else
  echo "/etc/jarvis/daemon.env already exists, preserving existing configuration."
fi

echo "Reloading systemd and enabling jarvis-daemon..."
systemctl daemon-reload
systemctl enable jarvis-daemon
systemctl restart jarvis-daemon

echo "Installation complete!"
echo "Check daemon status with: systemctl status jarvis-daemon"
