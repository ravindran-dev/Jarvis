#!/usr/bin/env bash
# JARVIS Terminal Portal Updater
# Builds release binaries as the current user, then installs to /usr/bin via sudo.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==================================="
echo " JARVIS Terminal Portal Updater    "
echo "==================================="
echo

# 1. Build release binaries as the normal user (no sudo needed)
echo "[1/5] Building release binaries..."
cargo build --workspace --release

JARVIS_BIN="target/release/jarvis"
DAEMON_BIN="target/release/jarvis-daemon"

if [ ! -f "$JARVIS_BIN" ]; then
    echo "[-] Error: jarvis binary not found at $JARVIS_BIN"
    exit 1
fi

echo "[+] Build successful."
echo ""

# 2. Stop the daemon before replacing binaries (avoids "Text file busy")
echo "[2/5] Stopping jarvis-daemon..."
if systemctl is-active --quiet jarvis-daemon 2>/dev/null; then
    sudo systemctl stop jarvis-daemon
    echo "[+] Daemon stopped."
else
    echo "[*] Daemon was not running."
fi

# 3. Install binaries to /usr/bin
echo "[3/5] Installing binaries to /usr/bin/..."
sudo cp "$JARVIS_BIN" /usr/bin/jarvis
sudo chmod 755 /usr/bin/jarvis

if [ -f "$DAEMON_BIN" ]; then
    sudo cp "$DAEMON_BIN" /usr/bin/jarvis-daemon
    sudo chmod 755 /usr/bin/jarvis-daemon
fi

echo "[+] Binaries installed."

# 4. Restart the daemon
echo "[4/5] Restarting jarvis-daemon..."
if [ -f /usr/lib/systemd/system/jarvis-daemon.service ]; then
    sudo systemctl daemon-reload
    sudo systemctl start jarvis-daemon
    echo "[+] Daemon restarted."
else
    echo "[*] No systemd service found, skipping daemon restart."
fi

# 5. Verify
echo "[5/5] Verifying installation..."
echo ""

INSTALLED=$(/usr/bin/jarvis version 2>/dev/null || echo "unknown")
echo "  Installed jarvis: /usr/bin/jarvis"
echo "  Version:          $INSTALLED"
echo "  command -v jarvis: $(command -v jarvis)"
echo ""

# Verify daemon.env is preserved
if [ -f /etc/jarvis/daemon.env ]; then
    echo "  daemon.env:       preserved (/etc/jarvis/daemon.env)"
else
    echo "  daemon.env:       not found (run install.sh for first-time setup)"
fi

echo ""
echo "==================================="
echo " Update complete!                  "
echo "==================================="
echo ""
echo "Restart your terminal or run:"
echo "  source ~/.zshrc"
echo ""
