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

        // Score and filter commands
        let mut scored_commands: Vec<(Command, i64)> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                // Try fuzzy matching on multiple fields
                let desc_score = self.matcher.fuzzy_match(&cmd.description, &query_lower);
                let cmd_score = self.matcher.fuzzy_match(&cmd.command, &query_lower);
                let tag_score = cmd
                    .tags
                    .iter()
                    .filter_map(|tag| self.matcher.fuzzy_match(tag, &query_lower))
                    .max();

                // Take the best score
                let score = vec![desc_score, cmd_score, tag_score]
                    .into_iter()
                    .flatten()
                    .max();

                if let Some(s) = score {
                    if s > 0 {
                        return Some((cmd.clone(), s));
                    }
                }
                None
            })
            .collect();

        // Sort by score descending
        scored_commands.sort_by(|a, b| b.1.cmp(&a.1));

        self.search_results = scored_commands
            .into_iter()
            .map(|(cmd, _)| cmd)
            .take(20)
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
