# JARVIS Installation Guide

JARVIS is split into two components:
- **`jarvis`**: The unprivileged CLI/TUI client that you run as a normal user.
- **`jarvis-daemon`**: The privileged daemon running as a `systemd` service to safely execute root operations like cgroup v2 limits and firewall rules.

## Prerequisites
- Systemd
- Linux Kernel with cgroup v2 unified hierarchy
- `ufw` (for network rules)

## Installation Steps

### 1. Build Release Binaries
```bash
cargo build --workspace --release
```

### 2. Install Binaries
Copy the binaries to a system PATH accessible to all users:
```bash
sudo cp target/release/jarvis /usr/local/bin/
sudo cp target/release/jarvis-daemon /usr/local/bin/
```

### 3. Configure the Systemd Service
Copy the hardened systemd service file:
```bash
sudo cp deploy/jarvis-daemon.service /etc/systemd/system/
```

Authorize your primary unprivileged user to interact with the daemon by configuring `JARVIS_AUTHORIZED_UID`. Find your user ID (e.g. `id -u`) and update the service file:
```bash
# Edit /etc/systemd/system/jarvis-daemon.service
# Set Environment="JARVIS_AUTHORIZED_UID=1000" (replace 1000 with your UID)
sudo systemctl daemon-reload
```

### 4. Enable and Start the Daemon
```bash
sudo systemctl enable --now jarvis-daemon.service
```

### 5. Verify Installation
Verify the unprivileged CLI can communicate with the daemon:
```bash
jarvis limit sleep cpu 50
```

## Uninstallation
To remove JARVIS from your system:
```bash
sudo systemctl disable --now jarvis-daemon.service
sudo rm /etc/systemd/system/jarvis-daemon.service
sudo systemctl daemon-reload
sudo rm /usr/local/bin/jarvis /usr/local/bin/jarvis-daemon
```
