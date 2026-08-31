#!/bin/bash
set -e

echo "Starting JARVIS v1.0 Uninstallation..."

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run as root (e.g. sudo ./uninstall.sh)"
  exit 1
fi

echo "Stopping and disabling jarvis-daemon..."
systemctl stop jarvis-daemon || true
systemctl disable jarvis-daemon || true

echo "Removing systemd service..."
rm -f /usr/lib/systemd/system/jarvis-daemon.service
systemctl daemon-reload

echo "Removing binaries..."
rm -f /usr/bin/jarvis
rm -f /usr/bin/jarvis-daemon

echo "Cleaning up runtime directories..."
# /run/jarvis should be removed by systemd automatically, but just in case:
rm -rf /run/jarvis

echo ""
read -p "Do you want to remove persistent configuration in /etc/jarvis? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  echo "Removing configuration..."
  rm -rf /etc/jarvis
else
  echo "Preserving configuration in /etc/jarvis."
fi

echo "Uninstallation complete!"
