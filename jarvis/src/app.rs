use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use jarvis_core::events::JarvisEvent;
use log::info;
use ratatui::{backend::Backend, Terminal};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::commands::CommandIndex;
use crate::config::Config;
use crate::plugins::PluginManager;
use crate::shell::{ExecutionEngine, SessionContext, UserInteraction};
use crate::system::{metrics::SystemMetrics, storage::StorageAnalyzer};
use crate::theme::Theme;
use crate::ui::layout;

struct TuiInteraction {
    output: Vec<String>,
}

impl UserInteraction for TuiInteraction {
    fn confirm(&mut self, prompt: &str) -> bool {
        let _ = crossterm::terminal::disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen);

        println!("\nJARVIS TUI Confirmation:\n{}\n[y/N]", prompt);

        let mut confirmed = false;
        let mut buf = String::new();
        if std::io::stdin().read_line(&mut buf).is_ok() {
            confirmed = buf.trim().to_lowercase() == "y";
        }

        let _ = crossterm::terminal::enable_raw_mode();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        );
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        );

        confirmed
    }

    fn print(&mut self, text: &str) {
        self.output.push(text.to_string());
    }
}
/// Represents the different screens/views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Cpu,
    Memory,
    Storage,
    Processes,
    Network,
    Services,
    Users,
    Commands,
    Events,
    Settings,
    Help,
}

impl Screen {
    /// Get the next screen in the navigation order
    pub fn next(&self) -> Self {
        match self {
            Screen::Overview => Screen::Cpu,
            Screen::Cpu => Screen::Memory,
            Screen::Memory => Screen::Storage,
            Screen::Storage => Screen::Processes,
            Screen::Processes => Screen::Network,
            Screen::Network => Screen::Services,
            Screen::Services => Screen::Users,
            Screen::Users => Screen::Commands,
            Screen::Commands => Screen::Events,
            Screen::Events => Screen::Settings,
            Screen::Settings => Screen::Overview,
            Screen::Help => Screen::Overview,
        }
    }

    /// Get the previous screen in the navigation order
    pub fn previous(&self) -> Self {
        match self {
            Screen::Overview => Screen::Settings,
            Screen::Cpu => Screen::Overview,
            Screen::Memory => Screen::Cpu,
            Screen::Storage => Screen::Memory,
            Screen::Processes => Screen::Storage,
            Screen::Network => Screen::Processes,
            Screen::Services => Screen::Network,
            Screen::Users => Screen::Services,
            Screen::Commands => Screen::Users,
            Screen::Events => Screen::Commands,
            Screen::Settings => Screen::Events,
            Screen::Help => Screen::Overview,
        }
    }
}

/// Main application state
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,
    /// Current active screen
    pub current_screen: Screen,
    /// System metrics collector
    pub metrics: SystemMetrics,
    /// Storage analyzer
    pub storage: StorageAnalyzer,
    /// Command index and search
    pub commands: CommandIndex,
    /// Plugin manager
    pub plugins: PluginManager,
    /// Persistent configuration
    pub config: Config,
    /// Current theme
    pub theme: Theme,
    /// Available themes
    themes: Vec<Theme>,
    /// Current theme index
    current_theme_index: usize,
    /// Last time metrics were updated
    last_update: Instant,
    /// Update interval in milliseconds
    update_interval: Duration,
    /// Selected item index in current view
    pub selected_index: usize,
    /// Scroll offset for current view
    pub scroll_offset: usize,
    /// Input buffer for search/commands
    pub input_buffer: String,
    /// Whether input mode is active
    pub input_mode: bool,
    /// Selected index in Settings
    pub settings_selected: usize,
    /// Cursor visibility for blinking effect
    pub cursor_visible: bool,
    /// Last cursor blink toggle time
    last_cursor_blink: Instant,
    /// Captured output from last executed command
    pub command_output: Vec<String>,
    /// Storage search buffer for filtering directories
    pub storage_search_buffer: String,
    /// Whether storage search feature is enabled
    pub storage_search_enabled: bool,
    /// Event bus receiver
    pub event_rx: tokio::sync::broadcast::Receiver<JarvisEvent>,
    /// Event log history
    pub event_log: Vec<JarvisEvent>,
    /// Execution Engine
    pub engine: ExecutionEngine,
    /// Session Context
    pub session_context: SessionContext,
    /// Network connections cache
    pub network_connections: Vec<jarvis_core::types::NetworkConnection>,
    /// Network pane scroll offset
    pub network_scroll: usize,
    /// The command the user selected to run, to be passed back to rustyline
    pub selected_command_to_run: Option<String>,
}

impl App {
    fn open_directory_in_file_manager(path: &str) -> Result<()> {
        use std::process::Stdio;

        let path_clean = path.trim();

        if cfg!(target_os = "windows") {
            Command::new("explorer")
                .arg(path_clean)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        } else if cfg!(target_os = "macos") {
            Command::new("open")
                .arg(path_clean)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        } else {
            let managers = ["nautilus", "dolphin", "nemo", "thunar", "pcmanfm"];
            let mut opened = false;
            for mgr in &managers {
                if Command::new(mgr)
                    .arg(path_clean)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .is_ok()
                {
                    opened = true;
                    break;
                }
            }
            if !opened {
                info!("No file manager found; falling back to xdg-open");
                Command::new("xdg-open")
                    .arg(path_clean)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;
            }
        }
        Ok(())
    }

    fn is_interactive_command(cmd: &str) -> bool {
        let candidates = [
            "htop", "top", "less", "more", "vim", "nvim", "nano", "man", "ssh", "watch", "tail -f",
            "tmux", "screen",
        ];
        let lc = cmd.to_lowercase();
        candidates.iter().any(|c| lc.contains(c))
    }
    /// Create a new App instance
    pub fn new(engine: ExecutionEngine, session_context: SessionContext) -> Result<Self> {
        info!("Initializing Jarvis application");

        let metrics = SystemMetrics::new()?;
        let storage = StorageAnalyzer::new()?;
        let commands = CommandIndex::new(&engine.registry)?;
        let plugins = PluginManager::new();
        let themes = Theme::all();
        let theme = themes[0].clone();
        let config = Config::load();

        let mut app = Self {
            should_quit: false,
            current_screen: Screen::Overview,
            metrics,
            storage,
            commands,
            plugins,
            theme,
            themes,
            current_theme_index: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(config.refresh_interval_ms),
            selected_index: 0,
            scroll_offset: 0,
            input_buffer: String::new(),
            input_mode: false,
            storage_search_buffer: String::new(),
            storage_search_enabled: true,
            config,
            settings_selected: 0,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            command_output: Vec::new(),
            event_rx: engine.event_bus.subscribe(),
            event_log: Vec::new(),
            engine,
            session_context,
            network_connections: Vec::new(),
            network_scroll: 0,
            selected_command_to_run: None,
        };

        if app.config.theme_index < app.themes.len() {
            app.current_theme_index = app.config.theme_index;
            app.theme = app.themes[app.current_theme_index].clone();
        }

        let threshold_bytes = app
            .config
            .storage_min_threshold_mb
            .saturating_mul(1024 * 1024);
        app.storage.set_min_threshold_bytes(threshold_bytes);

        Ok(app)
    }

    /// Run the main application loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        info!("Starting main event loop");

        while !self.should_quit {
            while let Ok(event) = self.event_rx.try_recv() {
                self.event_log.push(event);
                if self.event_log.len() > 100 {
                    self.event_log.remove(0);
                }
            }

            if self.last_update.elapsed() >= self.update_interval {
                self.update()?;
                self.last_update = Instant::now();
            }

            if self.last_cursor_blink.elapsed() >= Duration::from_millis(530) {
                self.cursor_visible = !self.cursor_visible;
                self.last_cursor_blink = Instant::now();
            }

            terminal.draw(|f| layout::render(f, self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key)?;
                }
            }
        }

        info!("Application shutting down");
        Ok(())
    }

    /// Update application state
    fn update(&mut self) -> Result<()> {
        self.metrics.update()?;

        if let Ok(jarvis_core::types::ActionResult::NetworkConnections(conns)) = self
            .engine
            .registry
            .lock()
            .unwrap()
            .execute("connections", &[])
        {
            self.network_connections = conns;
        }

        self.plugins.update_all();

        Ok(())
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }

        if key.code == KeyCode::Char('q') && !self.input_mode {
            if self.current_screen == Screen::Help {
                self.current_screen = Screen::Storage;
                self.reset_selection();
            } else {
                self.should_quit = true;
            }
            return Ok(());
        }

        if self.input_mode {
            match key.code {
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                    if self.current_screen == Screen::Commands {
                        self.commands.search(&self.input_buffer)?;
                        self.selected_index = 0;
                    } else if self.current_screen == Screen::Storage {
                        self.storage_search_buffer = self.input_buffer.clone();
                        self.selected_index = 0;
                        self.scroll_offset = 0;
                    }
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    if self.current_screen == Screen::Commands {
                        self.commands.search(&self.input_buffer)?;
                        self.selected_index = 0;
                    } else if self.current_screen == Screen::Storage {
                        self.storage_search_buffer = self.input_buffer.clone();
                        self.selected_index = 0;
                        self.scroll_offset = 0;
                    }
                }
                KeyCode::Enter => {
                    if self.current_screen == Screen::Storage {
                        self.input_mode = false;
                    } else {
                        self.handle_input_submit()?;
                        self.input_mode = false;
                        self.input_buffer.clear();
                    }
                }
                KeyCode::Esc => {
                    self.input_mode = false;
                    self.input_buffer.clear();
                    if self.current_screen == Screen::Storage {
                        self.storage_search_buffer.clear();
                        self.selected_index = 0;
                        self.scroll_offset = 0;
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        if self.current_screen == Screen::Settings {
            const REFRESH_STEPS: [u64; 4] = [250, 500, 1000, 2000];
            const LOG_LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];
            const THRESHOLD_STEPS_MB: [u64; 5] = [1, 10, 100, 500, 1024];

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.settings_selected < 3 {
                        self.settings_selected += 1;
                    }
                    return Ok(());
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.settings_selected > 0 {
                        self.settings_selected -= 1;
                    }
                    return Ok(());
                }
                KeyCode::Left | KeyCode::Char('-') => {
                    match self.settings_selected {
                        0 => {
                            if self.current_theme_index == 0 {
                                self.set_theme_by_index(self.themes.len() - 1);
                            } else {
                                self.set_theme_by_index(self.current_theme_index - 1);
                            }
                        }
                        1 => {
                            let mut idx = REFRESH_STEPS
                                .iter()
                                .position(|v| *v == self.config.refresh_interval_ms)
                                .unwrap_or(2);
                            idx = idx.saturating_sub(1);
                            self.set_refresh_interval_ms(REFRESH_STEPS[idx]);
                        }
                        2 => {
                            let mut idx = LOG_LEVELS
                                .iter()
                                .position(|v| *v == self.config.log_level)
                                .unwrap_or(2);
                            idx = idx.saturating_sub(1);
                            self.set_log_level(LOG_LEVELS[idx]);
                        }
                        3 => {
                            let mut idx = THRESHOLD_STEPS_MB
                                .iter()
                                .position(|v| *v == self.config.storage_min_threshold_mb)
                                .unwrap_or(0);
                            idx = idx.saturating_sub(1);
                            self.set_storage_threshold_mb(THRESHOLD_STEPS_MB[idx]);
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                KeyCode::Right | KeyCode::Char('+') => {
                    match self.settings_selected {
                        0 => {
                            self.set_theme_by_index(
                                (self.current_theme_index + 1) % self.themes.len(),
                            );
                        }
                        1 => {
                            let mut idx = REFRESH_STEPS
                                .iter()
                                .position(|v| *v == self.config.refresh_interval_ms)
                                .unwrap_or(2);
                            if idx + 1 < REFRESH_STEPS.len() {
                                idx += 1;
                            }
                            self.set_refresh_interval_ms(REFRESH_STEPS[idx]);
                        }
                        2 => {
                            let mut idx = LOG_LEVELS
                                .iter()
                                .position(|v| *v == self.config.log_level)
                                .unwrap_or(2);
                            if idx + 1 < LOG_LEVELS.len() {
                                idx += 1;
                            }
                            self.set_log_level(LOG_LEVELS[idx]);
                        }
                        3 => {
                            let mut idx = THRESHOLD_STEPS_MB
                                .iter()
                                .position(|v| *v == self.config.storage_min_threshold_mb)
                                .unwrap_or(0);
                            if idx + 1 < THRESHOLD_STEPS_MB.len() {
                                idx += 1;
                            }
                            self.set_storage_threshold_mb(THRESHOLD_STEPS_MB[idx]);
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Tab => {
                self.current_screen = self.current_screen.next();
                self.reset_selection();
            }
            KeyCode::BackTab => {
                self.current_screen = self.current_screen.previous();
                self.reset_selection();
            }

            KeyCode::Char('h') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.previous_theme();
                } else if self.current_screen != Screen::Settings {
                    self.current_screen = self.current_screen.previous();
                    self.reset_selection();
                }
            }
            KeyCode::Left => {
                if self.current_screen != Screen::Settings {
                    self.current_screen = self.current_screen.previous();
                    self.reset_selection();
                }
            }
            KeyCode::Char('l') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.next_theme();
                } else if self.current_screen != Screen::Settings {
                    self.current_screen = self.current_screen.next();
                    self.reset_selection();
                }
            }
            KeyCode::Right => {
                if self.current_screen != Screen::Settings {
                    self.current_screen = self.current_screen.next();
                    self.reset_selection();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection_up();
            }

            KeyCode::Enter => {
                self.handle_enter()?;
            }
            KeyCode::Backspace => {
                if self.current_screen == Screen::Storage {
                    if let Some(current_path) = self.storage.get_current_path() {
                        if let Some(parent) = current_path.parent() {
                            self.storage.set_current_path(Some(parent.to_path_buf()));
                            self.reset_selection();
                        } else {
                            self.storage.set_current_path(None);
                            self.reset_selection();
                        }
                    }
                }
            }
            KeyCode::Char('/') => {
                if self.current_screen == Screen::Commands {
                    self.input_mode = true;
                }
            }
            KeyCode::Char('r') => {
                self.handle_refresh()?;
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.next_theme();
            }
            KeyCode::Char('?') => {
                if self.current_screen == Screen::Storage {
                    self.storage_search_enabled = !self.storage_search_enabled;
                    if !self.storage_search_enabled {
                        self.storage_search_buffer.clear();
                        self.selected_index = 0;
                        self.scroll_offset = 0;
                    }
                } else if self.current_screen == Screen::Help {
                    self.current_screen = Screen::Storage;
                    self.reset_selection();
                } else {
                    self.current_screen = Screen::Help;
                    self.reset_selection();
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Reset selection when changing screens
    fn reset_selection(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Move selection down
    fn move_selection_down(&mut self) {
        if self.current_screen == Screen::Network {
            let max_items = self.network_connections.len();
            if max_items > 0 && self.network_scroll < max_items.saturating_sub(1) {
                self.network_scroll += 1;
            }
            return;
        }

        let max_items = self.get_max_items();
        if max_items > 0 && self.selected_index < max_items - 1 {
            self.selected_index += 1;
            let visible_lines = 20;
            let items_per_screen = visible_lines / 2;
            if self.selected_index >= self.scroll_offset + items_per_screen {
                self.scroll_offset = self.selected_index.saturating_sub(items_per_screen - 1);
            }
        }
    }

    /// Move selection up
    fn move_selection_up(&mut self) {
        if self.current_screen == Screen::Network {
            if self.network_scroll > 0 {
                self.network_scroll -= 1;
            }
            return;
        }

        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    /// Get maximum number of items in current view
    fn get_max_items(&self) -> usize {
        match self.current_screen {
            Screen::Storage => {
                let disks = self.metrics.get_disk_info();
                if self.storage_search_enabled && !self.storage_search_buffer.is_empty() {
                    let search = self.storage_search_buffer.to_lowercase();
                    disks
                        .into_iter()
                        .filter(|disk| {
                            disk.name.to_lowercase().contains(&search)
                                || disk.mount_point.to_lowercase().contains(&search)
                                || disk.file_system.to_lowercase().contains(&search)
                        })
                        .count()
                } else {
                    disks.len()
                }
            }
            Screen::Commands => self.commands.get_results_count(),
            _ => 0,
        }
    }

    /// Handle Enter key press
    fn handle_enter(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Storage => {
                // Storage screen is now just a view of mounts, no drill-down
            }
            Screen::Commands => {
                if let Some(cmd) = self.commands.get_selected_command(self.selected_index) {
                    info!("User selected command to insert: {}", cmd.command);
                    self.selected_command_to_run = Some(cmd.command.clone());
                    self.should_quit = true;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle input submission (search, etc.)
    fn handle_input_submit(&mut self) -> Result<()> {
        if self.current_screen == Screen::Commands {
            self.commands.search(&self.input_buffer)?;
            self.reset_selection();
        }
        Ok(())
    }

    /// Handle refresh action
    fn handle_refresh(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Storage => {
                // Now handled automatically via sysinfo update in background
            }
            Screen::Cpu | Screen::Memory | Screen::Network | Screen::Overview => {
                self.metrics.update()?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Cycle to next theme
    pub fn next_theme(&mut self) {
        self.current_theme_index = (self.current_theme_index + 1) % self.themes.len();
        self.theme = self.themes[self.current_theme_index].clone();
        self.config.theme_index = self.current_theme_index;
        let _ = self.config.save();
        info!("Switched to theme: {}", self.theme.name);
    }

    /// Cycle to previous theme
    pub fn previous_theme(&mut self) {
        if self.current_theme_index == 0 {
            self.current_theme_index = self.themes.len() - 1;
        } else {
            self.current_theme_index -= 1;
        }
        self.theme = self.themes[self.current_theme_index].clone();
        self.config.theme_index = self.current_theme_index;
        let _ = self.config.save();
        info!("Switched to theme: {}", self.theme.name);
    }

    /// Get all available themes
    pub fn get_available_themes(&self) -> &[Theme] {
        &self.themes
    }

    /// Get current theme index
    pub fn get_current_theme_index(&self) -> usize {
        self.current_theme_index
    }

    /// Apply and persist theme by index
    pub fn set_theme_by_index(&mut self, idx: usize) {
        if idx < self.themes.len() {
            self.current_theme_index = idx;
            self.theme = self.themes[idx].clone();
            self.config.theme_index = idx;
            let _ = self.config.save();
        }
    }

    /// Adjust refresh interval in ms and persist
    pub fn set_refresh_interval_ms(&mut self, ms: u64) {
        self.update_interval = Duration::from_millis(ms);
        self.config.refresh_interval_ms = ms;
        let _ = self.config.save();
    }

    /// Adjust storage min threshold (MB) and persist + apply
    pub fn set_storage_threshold_mb(&mut self, mb: u64) {
        self.config.storage_min_threshold_mb = mb;
        let bytes = mb.saturating_mul(1024 * 1024);
        self.storage.set_min_threshold_bytes(bytes);
        let _ = self.config.save();
    }

    /// Cycle log level value and persist (takes effect next run)
    pub fn set_log_level(&mut self, level: &str) {
        self.config.log_level = level.to_string();
        let _ = self.config.save();
    }
}
