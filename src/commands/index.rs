use anyhow::{Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Represents a single command with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub command: String,
    pub description: String,
    pub example: String,
    pub category: String,
    #[serde(default)]
    pub dangerous: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Command index for searching and managing Linux commands
pub struct CommandIndex {
    /// All available commands
    commands: Vec<Command>,
    /// Current search results
    search_results: Vec<Command>,
    /// Fuzzy matcher
    matcher: SkimMatcherV2,
}

impl CommandIndex {
    /// Create a new CommandIndex
    pub fn new() -> Result<Self> {
        let commands = Self::load_commands()?;
        let search_results = commands.clone();

        Ok(Self {
            commands,
            search_results,
            matcher: SkimMatcherV2::default(),
        })
    }

    /// Load commands from embedded or external JSON file
    fn load_commands() -> Result<Vec<Command>> {
        // First try to load from config directory
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("jarvis").join("commands.json");
            if config_path.exists() {
                info!("Loading commands from: {}", config_path.display());
                let content = fs::read_to_string(&config_path)
                    .context("Failed to read commands.json")?;
                let commands: Vec<Command> = serde_json::from_str(&content)
                    .context("Failed to parse commands.json")?;
                return Ok(commands);
            }
        }

        // Fallback to embedded default commands
        info!("Loading default embedded commands");
        Ok(Self::get_default_commands())
    }

    /// Get default built-in commands
    fn get_default_commands() -> Vec<Command> {
        vec![
            Command {
                command: "df -h".to_string(),
                description: "Show disk space usage in human-readable format".to_string(),
                example: "df -h".to_string(),
                category: "Disk".to_string(),
                dangerous: false,
                tags: vec!["disk".to_string(), "space".to_string(), "storage".to_string()],
            },
            Command {
                command: "du -sh".to_string(),
                description: "Display disk usage of a directory".to_string(),
                example: "du -sh /var/log".to_string(),
                category: "Disk".to_string(),
                dangerous: false,
                tags: vec!["disk".to_string(), "usage".to_string(), "directory".to_string()],
            },
            Command {
                command: "free -h".to_string(),
                description: "Display amount of free and used memory".to_string(),
                example: "free -h".to_string(),
                category: "Memory".to_string(),
                dangerous: false,
                tags: vec!["memory".to_string(), "ram".to_string()],
            },
            Command {
                command: "top".to_string(),
                description: "Display Linux processes in real-time".to_string(),
                example: "top".to_string(),
                category: "Process".to_string(),
                dangerous: false,
                tags: vec!["process".to_string(), "cpu".to_string(), "monitor".to_string()],
            },
            Command {
                command: "htop".to_string(),
                description: "Interactive process viewer (better than top)".to_string(),
                example: "htop".to_string(),
                category: "Process".to_string(),
                dangerous: false,
                tags: vec!["process".to_string(), "cpu".to_string(), "monitor".to_string()],
            },
            Command {
                command: "ps aux".to_string(),
                description: "Show all running processes".to_string(),
                example: "ps aux | grep nginx".to_string(),
                category: "Process".to_string(),
                dangerous: false,
                tags: vec!["process".to_string(), "list".to_string()],
            },
            Command {
                command: "netstat -tuln".to_string(),
                description: "Show listening ports and network connections".to_string(),
                example: "netstat -tuln".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "port".to_string(), "connection".to_string()],
            },
            Command {
                command: "ss -tuln".to_string(),
                description: "Modern replacement for netstat".to_string(),
                example: "ss -tuln".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "socket".to_string()],
            },
            Command {
                command: "lsof -i".to_string(),
                description: "List open files and network connections".to_string(),
                example: "lsof -i :80".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "port".to_string(), "file".to_string()],
            },
            Command {
                command: "journalctl -xe".to_string(),
                description: "View system logs with explanations".to_string(),
                example: "journalctl -xe".to_string(),
                category: "Logs".to_string(),
                dangerous: false,
                tags: vec!["log".to_string(), "systemd".to_string(), "debug".to_string()],
            },
            Command {
                command: "systemctl status".to_string(),
                description: "Show status of systemd services".to_string(),
                example: "systemctl status nginx".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["systemd".to_string(), "service".to_string()],
            },
            Command {
                command: "docker ps".to_string(),
                description: "List running Docker containers".to_string(),
                example: "docker ps -a".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "container".to_string()],
            },
            Command {
                command: "docker images".to_string(),
                description: "List Docker images".to_string(),
                example: "docker images".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "image".to_string()],
            },
            Command {
                command: "find . -name".to_string(),
                description: "Search for files by name".to_string(),
                example: "find . -name '*.log'".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["find".to_string(), "search".to_string(), "file".to_string()],
            },
            Command {
                command: "grep -r".to_string(),
                description: "Search for text in files recursively".to_string(),
                example: "grep -r 'error' /var/log".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["search".to_string(), "text".to_string()],
            },
            Command {
                command: "tail -f".to_string(),
                description: "Follow log file in real-time".to_string(),
                example: "tail -f /var/log/syslog".to_string(),
                category: "Logs".to_string(),
                dangerous: false,
                tags: vec!["log".to_string(), "watch".to_string(), "monitor".to_string()],
            },
            Command {
                command: "uname -a".to_string(),
                description: "Display system information".to_string(),
                example: "uname -a".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["system".to_string(), "info".to_string(), "kernel".to_string()],
            },
            Command {
                command: "uptime".to_string(),
                description: "Show how long the system has been running".to_string(),
                example: "uptime".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["system".to_string(), "uptime".to_string()],
            },
            Command {
                command: "ncdu".to_string(),
                description: "NCurses Disk Usage - interactive disk usage analyzer".to_string(),
                example: "ncdu /var".to_string(),
                category: "Disk".to_string(),
                dangerous: false,
                tags: vec!["disk".to_string(), "usage".to_string(), "interactive".to_string()],
            },
            Command {
                command: "iotop".to_string(),
                description: "Monitor I/O usage by processes".to_string(),
                example: "sudo iotop".to_string(),
                category: "IO".to_string(),
                dangerous: false,
                tags: vec!["io".to_string(), "disk".to_string(), "monitor".to_string()],
            },
            Command {
                command: "iostat".to_string(),
                description: "Report CPU and I/O statistics".to_string(),
                example: "iostat -x 1".to_string(),
                category: "IO".to_string(),
                dangerous: false,
                tags: vec!["io".to_string(), "stats".to_string()],
            },
            Command {
                command: "vmstat".to_string(),
                description: "Report virtual memory statistics".to_string(),
                example: "vmstat 1".to_string(),
                category: "Memory".to_string(),
                dangerous: false,
                tags: vec!["memory".to_string(), "stats".to_string()],
            },
            Command {
                command: "lscpu".to_string(),
                description: "Display CPU architecture information".to_string(),
                example: "lscpu".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["cpu".to_string(), "hardware".to_string()],
            },
            Command {
                command: "lsblk".to_string(),
                description: "List block devices".to_string(),
                example: "lsblk -f".to_string(),
                category: "Disk".to_string(),
                dangerous: false,
                tags: vec!["disk".to_string(), "block".to_string()],
            },
            Command {
                command: "lspci".to_string(),
                description: "List all PCI devices".to_string(),
                example: "lspci -v".to_string(),
                category: "Hardware".to_string(),
                dangerous: false,
                tags: vec!["hardware".to_string(), "pci".to_string()],
            },
            Command {
                command: "lsusb".to_string(),
                description: "List USB devices".to_string(),
                example: "lsusb".to_string(),
                category: "Hardware".to_string(),
                dangerous: false,
                tags: vec!["hardware".to_string(), "usb".to_string()],
            },
            Command {
                command: "ip addr".to_string(),
                description: "Show network interfaces and IP addresses".to_string(),
                example: "ip addr show".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "ip".to_string()],
            },
            Command {
                command: "ip route".to_string(),
                description: "Show routing table".to_string(),
                example: "ip route show".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "route".to_string()],
            },
            Command {
                command: "ping".to_string(),
                description: "Send ICMP echo requests to network host".to_string(),
                example: "ping -c 4 google.com".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "connectivity".to_string()],
            },
            Command {
                command: "traceroute".to_string(),
                description: "Print route packets take to network host".to_string(),
                example: "traceroute google.com".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "route".to_string()],
            },
            Command {
                command: "nslookup".to_string(),
                description: "Query DNS for domain name or IP address".to_string(),
                example: "nslookup google.com".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "dns".to_string()],
            },
            Command {
                command: "dig".to_string(),
                description: "DNS lookup utility".to_string(),
                example: "dig google.com".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "dns".to_string()],
            },
            Command {
                command: "curl".to_string(),
                description: "Transfer data from or to a server".to_string(),
                example: "curl -I https://google.com".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "http".to_string()],
            },
            Command {
                command: "wget".to_string(),
                description: "Download files from the web".to_string(),
                example: "wget https://example.com/file.txt".to_string(),
                category: "Network".to_string(),
                dangerous: false,
                tags: vec!["network".to_string(), "download".to_string()],
            },
            Command {
                command: "rsync".to_string(),
                description: "Sync files and directories".to_string(),
                example: "rsync -avz source/ dest/".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["sync".to_string(), "backup".to_string()],
            },
            Command {
                command: "tar -czf".to_string(),
                description: "Create compressed tar archive".to_string(),
                example: "tar -czf archive.tar.gz folder/".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["archive".to_string(), "compress".to_string()],
            },
            Command {
                command: "tar -xzf".to_string(),
                description: "Extract compressed tar archive".to_string(),
                example: "tar -xzf archive.tar.gz".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["extract".to_string(), "archive".to_string()],
            },
            Command {
                command: "zip -r".to_string(),
                description: "Create zip archive".to_string(),
                example: "zip -r archive.zip folder/".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["archive".to_string(), "compress".to_string()],
            },
            Command {
                command: "unzip".to_string(),
                description: "Extract zip archive".to_string(),
                example: "unzip archive.zip".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["extract".to_string(), "archive".to_string()],
            },
            Command {
                command: "chmod".to_string(),
                description: "Change file permissions".to_string(),
                example: "chmod 755 script.sh".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["permissions".to_string(), "security".to_string()],
            },
            Command {
                command: "chown".to_string(),
                description: "Change file owner and group".to_string(),
                example: "chown user:group file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["ownership".to_string(), "security".to_string()],
            },
            Command {
                command: "ln -s".to_string(),
                description: "Create symbolic link".to_string(),
                example: "ln -s /path/to/file link".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["link".to_string(), "symlink".to_string()],
            },
            Command {
                command: "cat".to_string(),
                description: "Concatenate and display file contents".to_string(),
                example: "cat file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["read".to_string(), "view".to_string()],
            },
            Command {
                command: "less".to_string(),
                description: "View file contents with pagination".to_string(),
                example: "less file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["read".to_string(), "pager".to_string()],
            },
            Command {
                command: "head".to_string(),
                description: "Output the first part of files".to_string(),
                example: "head -n 20 file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["read".to_string(), "view".to_string()],
            },
            Command {
                command: "tail".to_string(),
                description: "Output the last part of files".to_string(),
                example: "tail -n 20 file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["read".to_string(), "view".to_string()],
            },
            Command {
                command: "wc -l".to_string(),
                description: "Count lines in file".to_string(),
                example: "wc -l file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["count".to_string(), "lines".to_string()],
            },
            Command {
                command: "diff".to_string(),
                description: "Compare files line by line".to_string(),
                example: "diff file1.txt file2.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["compare".to_string(), "diff".to_string()],
            },
            Command {
                command: "sort".to_string(),
                description: "Sort lines of text files".to_string(),
                example: "sort file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["sort".to_string(), "text".to_string()],
            },
            Command {
                command: "uniq".to_string(),
                description: "Report or omit repeated lines".to_string(),
                example: "sort file.txt | uniq".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["unique".to_string(), "duplicate".to_string()],
            },
            Command {
                command: "awk".to_string(),
                description: "Pattern scanning and text processing".to_string(),
                example: "awk '{print $1}' file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["text".to_string(), "processing".to_string()],
            },
            Command {
                command: "sed".to_string(),
                description: "Stream editor for filtering and transforming text".to_string(),
                example: "sed 's/old/new/g' file.txt".to_string(),
                category: "Files".to_string(),
                dangerous: false,
                tags: vec!["text".to_string(), "replace".to_string()],
            },
            Command {
                command: "systemctl start".to_string(),
                description: "Start a systemd service".to_string(),
                example: "systemctl start nginx".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["systemd".to_string(), "service".to_string()],
            },
            Command {
                command: "systemctl stop".to_string(),
                description: "Stop a systemd service".to_string(),
                example: "systemctl stop nginx".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["systemd".to_string(), "service".to_string()],
            },
            Command {
                command: "systemctl restart".to_string(),
                description: "Restart a systemd service".to_string(),
                example: "systemctl restart nginx".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["systemd".to_string(), "service".to_string()],
            },
            Command {
                command: "systemctl enable".to_string(),
                description: "Enable service to start on boot".to_string(),
                example: "systemctl enable nginx".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["systemd".to_string(), "autostart".to_string()],
            },
            Command {
                command: "systemctl disable".to_string(),
                description: "Disable service from starting on boot".to_string(),
                example: "systemctl disable nginx".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["systemd".to_string(), "autostart".to_string()],
            },
            Command {
                command: "journalctl -f".to_string(),
                description: "Follow system logs in real-time".to_string(),
                example: "journalctl -f -u nginx".to_string(),
                category: "Logs".to_string(),
                dangerous: false,
                tags: vec!["log".to_string(), "systemd".to_string()],
            },
            Command {
                command: "journalctl -u".to_string(),
                description: "Show logs for specific unit".to_string(),
                example: "journalctl -u nginx".to_string(),
                category: "Logs".to_string(),
                dangerous: false,
                tags: vec!["log".to_string(), "service".to_string()],
            },
            Command {
                command: "dmesg".to_string(),
                description: "Print kernel ring buffer messages".to_string(),
                example: "dmesg | tail".to_string(),
                category: "Logs".to_string(),
                dangerous: false,
                tags: vec!["kernel".to_string(), "log".to_string()],
            },
            Command {
                command: "last".to_string(),
                description: "Show listing of last logged in users".to_string(),
                example: "last -10".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["users".to_string(), "login".to_string()],
            },
            Command {
                command: "who".to_string(),
                description: "Show who is logged on".to_string(),
                example: "who".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["users".to_string(), "logged".to_string()],
            },
            Command {
                command: "w".to_string(),
                description: "Show who is logged on and what they are doing".to_string(),
                example: "w".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["users".to_string(), "activity".to_string()],
            },
            Command {
                command: "id".to_string(),
                description: "Print user identity".to_string(),
                example: "id username".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["user".to_string(), "identity".to_string()],
            },
            Command {
                command: "groups".to_string(),
                description: "Print group memberships".to_string(),
                example: "groups username".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["user".to_string(), "groups".to_string()],
            },
            Command {
                command: "useradd".to_string(),
                description: "Create a new user".to_string(),
                example: "useradd -m username".to_string(),
                category: "System".to_string(),
                dangerous: true,
                tags: vec!["user".to_string(), "admin".to_string()],
            },
            Command {
                command: "usermod".to_string(),
                description: "Modify user account".to_string(),
                example: "usermod -aG sudo username".to_string(),
                category: "System".to_string(),
                dangerous: true,
                tags: vec!["user".to_string(), "admin".to_string()],
            },
            Command {
                command: "userdel".to_string(),
                description: "Delete user account".to_string(),
                example: "userdel -r username".to_string(),
                category: "System".to_string(),
                dangerous: true,
                tags: vec!["user".to_string(), "admin".to_string()],
            },
            Command {
                command: "passwd".to_string(),
                description: "Change user password".to_string(),
                example: "passwd username".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["user".to_string(), "password".to_string()],
            },
            Command {
                command: "docker run".to_string(),
                description: "Run a command in a new container".to_string(),
                example: "docker run -d -p 80:80 nginx".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "container".to_string()],
            },
            Command {
                command: "docker exec".to_string(),
                description: "Execute command in running container".to_string(),
                example: "docker exec -it container_id bash".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "container".to_string()],
            },
            Command {
                command: "docker logs".to_string(),
                description: "Fetch logs of a container".to_string(),
                example: "docker logs -f container_id".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "logs".to_string()],
            },
            Command {
                command: "docker stop".to_string(),
                description: "Stop running container".to_string(),
                example: "docker stop container_id".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "container".to_string()],
            },
            Command {
                command: "docker rm".to_string(),
                description: "Remove container".to_string(),
                example: "docker rm container_id".to_string(),
                category: "Docker".to_string(),
                dangerous: true,
                tags: vec!["docker".to_string(), "remove".to_string()],
            },
            Command {
                command: "docker rmi".to_string(),
                description: "Remove Docker image".to_string(),
                example: "docker rmi image_id".to_string(),
                category: "Docker".to_string(),
                dangerous: true,
                tags: vec!["docker".to_string(), "image".to_string()],
            },
            Command {
                command: "docker-compose up".to_string(),
                description: "Create and start containers".to_string(),
                example: "docker-compose up -d".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "compose".to_string()],
            },
            Command {
                command: "docker-compose down".to_string(),
                description: "Stop and remove containers".to_string(),
                example: "docker-compose down".to_string(),
                category: "Docker".to_string(),
                dangerous: false,
                tags: vec!["docker".to_string(), "compose".to_string()],
            },
            Command {
                command: "git status".to_string(),
                description: "Show working tree status".to_string(),
                example: "git status".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "status".to_string()],
            },
            Command {
                command: "git log".to_string(),
                description: "Show commit logs".to_string(),
                example: "git log --oneline -10".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "history".to_string()],
            },
            Command {
                command: "git diff".to_string(),
                description: "Show changes between commits".to_string(),
                example: "git diff HEAD".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "diff".to_string()],
            },
            Command {
                command: "git add".to_string(),
                description: "Add file contents to index".to_string(),
                example: "git add .".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "stage".to_string()],
            },
            Command {
                command: "git commit".to_string(),
                description: "Record changes to repository".to_string(),
                example: "git commit -m 'message'".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "commit".to_string()],
            },
            Command {
                command: "git push".to_string(),
                description: "Update remote refs".to_string(),
                example: "git push origin main".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "push".to_string()],
            },
            Command {
                command: "git pull".to_string(),
                description: "Fetch and merge from remote".to_string(),
                example: "git pull origin main".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "pull".to_string()],
            },
            Command {
                command: "git branch".to_string(),
                description: "List, create, or delete branches".to_string(),
                example: "git branch -a".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "branch".to_string()],
            },
            Command {
                command: "git checkout".to_string(),
                description: "Switch branches or restore files".to_string(),
                example: "git checkout main".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "branch".to_string()],
            },
            Command {
                command: "git merge".to_string(),
                description: "Join two or more development histories".to_string(),
                example: "git merge feature-branch".to_string(),
                category: "Git".to_string(),
                dangerous: false,
                tags: vec!["git".to_string(), "merge".to_string()],
            },
            Command {
                command: "killall".to_string(),
                description: "Kill processes by name".to_string(),
                example: "killall firefox".to_string(),
                category: "Process".to_string(),
                dangerous: true,
                tags: vec!["kill".to_string(), "process".to_string()],
            },
            Command {
                command: "pkill".to_string(),
                description: "Signal processes based on name".to_string(),
                example: "pkill -9 chrome".to_string(),
                category: "Process".to_string(),
                dangerous: true,
                tags: vec!["kill".to_string(), "process".to_string()],
            },
            Command {
                command: "kill".to_string(),
                description: "Send signal to process".to_string(),
                example: "kill -9 12345".to_string(),
                category: "Process".to_string(),
                dangerous: true,
                tags: vec!["kill".to_string(), "signal".to_string()],
            },
            Command {
                command: "nice".to_string(),
                description: "Run program with modified scheduling priority".to_string(),
                example: "nice -n 10 command".to_string(),
                category: "Process".to_string(),
                dangerous: false,
                tags: vec!["priority".to_string(), "cpu".to_string()],
            },
            Command {
                command: "renice".to_string(),
                description: "Alter priority of running processes".to_string(),
                example: "renice -n 5 -p 12345".to_string(),
                category: "Process".to_string(),
                dangerous: false,
                tags: vec!["priority".to_string(), "process".to_string()],
            },
            Command {
                command: "nohup".to_string(),
                description: "Run command immune to hangups".to_string(),
                example: "nohup ./script.sh &".to_string(),
                category: "Process".to_string(),
                dangerous: false,
                tags: vec!["background".to_string(), "process".to_string()],
            },
            Command {
                command: "screen".to_string(),
                description: "Screen manager with VT100 emulation".to_string(),
                example: "screen -S session_name".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["terminal".to_string(), "multiplexer".to_string()],
            },
            Command {
                command: "tmux".to_string(),
                description: "Terminal multiplexer".to_string(),
                example: "tmux new -s session_name".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["terminal".to_string(), "multiplexer".to_string()],
            },
            Command {
                command: "crontab -l".to_string(),
                description: "List cron jobs".to_string(),
                example: "crontab -l".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["cron".to_string(), "schedule".to_string()],
            },
            Command {
                command: "crontab -e".to_string(),
                description: "Edit cron jobs".to_string(),
                example: "crontab -e".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["cron".to_string(), "schedule".to_string()],
            },
            Command {
                command: "at".to_string(),
                description: "Execute commands at a later time".to_string(),
                example: "echo 'command' | at now + 1 hour".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["schedule".to_string(), "delay".to_string()],
            },
            Command {
                command: "watch".to_string(),
                description: "Execute program periodically".to_string(),
                example: "watch -n 2 df -h".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["monitor".to_string(), "repeat".to_string()],
            },
            Command {
                command: "env".to_string(),
                description: "Display environment variables".to_string(),
                example: "env".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["environment".to_string(), "variables".to_string()],
            },
            Command {
                command: "export".to_string(),
                description: "Set environment variable".to_string(),
                example: "export PATH=$PATH:/new/path".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["environment".to_string(), "variables".to_string()],
            },
            Command {
                command: "alias".to_string(),
                description: "Create command alias".to_string(),
                example: "alias ll='ls -la'".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["alias".to_string(), "shortcut".to_string()],
            },
            Command {
                command: "history".to_string(),
                description: "Show command history".to_string(),
                example: "history | tail -20".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["history".to_string(), "commands".to_string()],
            },
            Command {
                command: "man".to_string(),
                description: "Display manual pages".to_string(),
                example: "man ls".to_string(),
                category: "Help".to_string(),
                dangerous: false,
                tags: vec!["help".to_string(), "documentation".to_string()],
            },
            Command {
                command: "apropos".to_string(),
                description: "Search manual page names and descriptions".to_string(),
                example: "apropos network".to_string(),
                category: "Help".to_string(),
                dangerous: false,
                tags: vec!["help".to_string(), "search".to_string()],
            },
            Command {
                command: "whatis".to_string(),
                description: "Display one-line manual page descriptions".to_string(),
                example: "whatis ls".to_string(),
                category: "Help".to_string(),
                dangerous: false,
                tags: vec!["help".to_string(), "documentation".to_string()],
            },
            Command {
                command: "which".to_string(),
                description: "Locate a command".to_string(),
                example: "which python".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["path".to_string(), "locate".to_string()],
            },
            Command {
                command: "whereis".to_string(),
                description: "Locate binary, source, and manual page files".to_string(),
                example: "whereis ls".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["locate".to_string(), "files".to_string()],
            },
            Command {
                command: "hostname".to_string(),
                description: "Show or set system hostname".to_string(),
                example: "hostname".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["hostname".to_string(), "network".to_string()],
            },
            Command {
                command: "date".to_string(),
                description: "Display or set system date and time".to_string(),
                example: "date '+%Y-%m-%d %H:%M:%S'".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["time".to_string(), "date".to_string()],
            },
            Command {
                command: "timedatectl".to_string(),
                description: "Control system time and date".to_string(),
                example: "timedatectl status".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["time".to_string(), "timezone".to_string()],
            },
            Command {
                command: "cal".to_string(),
                description: "Display calendar".to_string(),
                example: "cal".to_string(),
                category: "System".to_string(),
                dangerous: false,
                tags: vec!["calendar".to_string(), "date".to_string()],
            },
            Command {
                command: "bc".to_string(),
                description: "Command-line calculator".to_string(),
                example: "echo '2+2' | bc".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["calculator".to_string(), "math".to_string()],
            },
            Command {
                command: "echo".to_string(),
                description: "Display a line of text".to_string(),
                example: "echo 'Hello World'".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["print".to_string(), "text".to_string()],
            },
            Command {
                command: "printf".to_string(),
                description: "Format and print data".to_string(),
                example: "printf '%s\n' 'text'".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["print".to_string(), "format".to_string()],
            },
            Command {
                command: "xargs".to_string(),
                description: "Build and execute command lines from input".to_string(),
                example: "find . -name '*.txt' | xargs rm".to_string(),
                category: "Utility".to_string(),
                dangerous: true,
                tags: vec!["pipe".to_string(), "execute".to_string()],
            },
            Command {
                command: "tee".to_string(),
                description: "Read from stdin and write to stdout and files".to_string(),
                example: "command | tee output.txt".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["pipe".to_string(), "save".to_string()],
            },
            Command {
                command: "tr".to_string(),
                description: "Translate or delete characters".to_string(),
                example: "echo 'hello' | tr 'a-z' 'A-Z'".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["text".to_string(), "transform".to_string()],
            },
            Command {
                command: "cut".to_string(),
                description: "Remove sections from lines of files".to_string(),
                example: "cut -d':' -f1 /etc/passwd".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["text".to_string(), "extract".to_string()],
            },
            Command {
                command: "paste".to_string(),
                description: "Merge lines of files".to_string(),
                example: "paste file1 file2".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["text".to_string(), "merge".to_string()],
            },
            Command {
                command: "column".to_string(),
                description: "Format input into columns".to_string(),
                example: "mount | column -t".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["format".to_string(), "display".to_string()],
            },
            Command {
                command: "jq".to_string(),
                description: "JSON processor".to_string(),
                example: "cat file.json | jq '.key'".to_string(),
                category: "Utility".to_string(),
                dangerous: false,
                tags: vec!["json".to_string(), "parse".to_string()],
            },
        ]
    }

    /// Search commands by query
    pub fn search(&mut self, query: &str) -> Result<()> {
        if query.trim().is_empty() {
            self.search_results = self.commands.clone();
            return Ok(());
        }

        debug!("Searching commands for: {}", query);

        let query_lower = query.to_lowercase();

        // Score and filter commands - be strict about matching
        let mut scored_commands: Vec<(Command, i64)> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                let cmd_lower = cmd.command.to_lowercase();
                let _desc_lower = cmd.description.to_lowercase();
                
                let mut score: i64 = 0;
                
                // Exact command prefix match (highest priority)
                if cmd_lower.starts_with(&query_lower) {
                    score += 10000;
                }
                // Command contains as a complete word (separated by space or dash)
                else if cmd_lower.split(|c: char| c == ' ' || c == '-')
                    .any(|word| word.starts_with(&query_lower)) {
                    score += 5000;
                }
                // Command contains the query (case-insensitive)
                else if cmd_lower.contains(&query_lower) {
                    score += 2000;
                } else {
                    // No match in command name, skip unless description matches
                    score = 0;
                }
                
                // Only apply fuzzy matching if we already have some score
                // This prevents loose fuzzy matches
                if score > 0 {
                    // Apply fuzzy matching only as a tiebreaker, not primary match
                    if let Some(fuzzy_score) = self.matcher.fuzzy_match(&cmd.command, &query_lower) {
                        score += (fuzzy_score as i64) / 10; // Reduce fuzzy impact
                    }
                    return Some((cmd.clone(), score));
                }
                
                None
            })
            .collect();

        // Sort by score descending
        scored_commands.sort_by(|a, b| b.1.cmp(&a.1));

        self.search_results = scored_commands
            .into_iter()
            .map(|(cmd, _)| cmd)
            .take(50)
            .collect();

        info!("Found {} matching commands", self.search_results.len());
        Ok(())
    }

    /// Get current search results
    pub fn get_results(&self) -> &[Command] {
        &self.search_results
    }

    /// Get number of results
    pub fn get_results_count(&self) -> usize {
        self.search_results.len()
    }

    /// Get a specific command by index
    pub fn get_selected_command(&self, index: usize) -> Option<&Command> {
        self.search_results.get(index)
    }

    /// Export commands to JSON file
    #[allow(dead_code)]
    pub fn export_to_file(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.commands)?;
        fs::write(path, json)?;
        info!("Exported commands to: {}", path.display());
        Ok(())
    }
}
