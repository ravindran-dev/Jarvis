# SysTide Quick Start Guide

## Installation

```bash
cd /path/to/Jarvis
cargo build --release
./target/release/systide
```

## Keyboard Shortcuts

### Global Navigation
- `Tab` - Next screen
- `Shift+Tab` - Previous screen
- `h` / `←` - Move left (previous screen)
- `l` / `→` - Move right (next screen)
- `q` / `Ctrl+C` - Quit application

### Storage Screen
- `j` / `↓` - Move down in list
- `k` / `↑` - Move up in list
- `r` - Rescan directories
- `Enter` - Select item (future: drill down)

### Metrics Screen
- `r` - Force refresh metrics

### Commands Screen
- `j` / `↓` - Move down in command list
- `k` / `↑` - Move up in command list
- `/` - Enter search mode
- `Enter` - Execute selected command
- `Esc` - Exit search mode

## Screens

1. **Storage** - Analyze disk usage with parallel scanning
2. **Metrics** - Real-time system metrics (CPU, memory, disk, network)
3. **Commands** - Search and execute Linux commands
4. **Settings** - Configuration options

## Features

### Storage Analyzer
- Scans home directory, /var/cache, /var/log, Docker folders
- Shows top 50 largest directories
- Parallel processing for fast scanning
- Non-blocking background scans

### System Metrics
- CPU usage (global and per-core)
- Memory and swap usage
- Disk usage for all mount points
- Network I/O statistics
- Temperature monitoring (when available)
- Auto-refresh every 1 second

### Command Assistant
- 30+ built-in Linux commands
- Fuzzy search by intent
- Category-based organization
- Safety warnings for dangerous commands
- Example usage for each command

### Custom Commands
Create `~/.config/systide/commands.json` to add your own commands:

```json
[
  {
    "command": "my-command",
    "description": "What it does",
    "example": "my-command --flag",
    "category": "Custom",
    "dangerous": false,
    "tags": ["custom", "example"]
  }
]
```

## Logging

Enable debug logging:
```bash
RUST_LOG=debug systide
```

## Tips

- Press `r` in Storage screen to trigger a fresh scan
- Use `/` in Commands screen to search by keywords or intent
- Temperature sensors may require kernel modules (coretemp, k10temp)
- Some operations may require sudo for full access

## Troubleshooting

**Temperature not showing?**
```bash
sudo modprobe coretemp  # Intel
sudo modprobe k10temp   # AMD
```

**Permission denied?**
```bash
sudo systide  # Run with elevated privileges
```

**Storage scan too slow?**
Edit `src/system/storage.rs` to exclude large directories.

## Performance

- Startup: < 100ms
- Memory: < 10MB RAM
- CPU (idle): < 1%
- Storage scan: ~1GB/s (I/O dependent)

Built with Rust for maximum performance and reliability.
