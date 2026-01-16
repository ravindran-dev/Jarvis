use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use log::info;
use ratatui::{backend::Backend, Terminal};
use std::time::{Duration, Instant};

use crate::commands::CommandIndex;
use crate::plugins::PluginManager;
use crate::config::Config;
use crate::system::{metrics::SystemMetrics, storage::StorageAnalyzer};
use crate::theme::Theme;
use crate::ui::layout;

/// Represents the different screens/views in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Storage,
    Metrics,
    Commands,
    Settings,
}

impl Screen {
    /// Get the next screen in the navigation order
    pub fn next(&self) -> Self {
        match self {
            Screen::Storage => Screen::Metrics,
            Screen::Metrics => Screen::Commands,
            Screen::Commands => Screen::Settings,
            Screen::Settings => Screen::Storage,
        }
    }

    /// Get the previous screen in the navigation order
    pub fn previous(&self) -> Self {
        match self {
            Screen::Storage => Screen::Settings,
            Screen::Metrics => Screen::Storage,
            Screen::Commands => Screen::Metrics,
            Screen::Settings => Screen::Commands,
        }
    }

    /// Convert to display string
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Screen::Storage => "Storage",
            Screen::Metrics => "Metrics",
            Screen::Commands => "Commands",
            Screen::Settings => "Settings",
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
}

impl App {
    /// Create a new App instance
    pub fn new() -> Result<Self> {
        info!("Initializing Jarvis application");

        let metrics = SystemMetrics::new()?;
        let storage = StorageAnalyzer::new()?;
        let commands = CommandIndex::new()?;
        let plugins = PluginManager::new();
        let themes = Theme::all();
        let theme = themes[0].clone();
        let config = Config::load();

        let mut app = Self {
            should_quit: false,
            current_screen: Screen::Metrics,
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
            config,
            settings_selected: 0,
        };

        // Apply theme from config
        if app.config.theme_index < app.themes.len() {
            app.current_theme_index = app.config.theme_index;
            app.theme = app.themes[app.current_theme_index].clone();
        }

        // Apply storage threshold
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
            // Update data if interval has passed
            if self.last_update.elapsed() >= self.update_interval {
                self.update()?;
                self.last_update = Instant::now();
            }

            // Render UI
            terminal.draw(|f| layout::render(f, self))?;

            // Handle events with timeout
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
        // Update system metrics
        self.metrics.update()?;

        // Update plugins
        self.plugins.update_all();

        Ok(())
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Global quit handlers
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }

        if key.code == KeyCode::Char('q') && !self.input_mode {
            self.should_quit = true;
            return Ok(());
        }

        // Handle input mode
        if self.input_mode {
            match key.code {
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                    // Trigger search on every keystroke for Commands screen
                    if self.current_screen == Screen::Commands {
                        self.commands.search(&self.input_buffer)?;
                        self.selected_index = 0;
                    }
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    // Trigger search on every keystroke for Commands screen
                    if self.current_screen == Screen::Commands {
                        self.commands.search(&self.input_buffer)?;
                        self.selected_index = 0;
                    }
                }
                KeyCode::Enter => {
                    self.handle_input_submit()?;
                    self.input_mode = false;
                    self.input_buffer.clear();
                }
                KeyCode::Esc => {
                    self.input_mode = false;
                    self.input_buffer.clear();
                }
                _ => {}
            }
            return Ok(());
        }

        // Settings screen specific handling
        if self.current_screen == Screen::Settings {
            const REFRESH_STEPS: [u64; 4] = [250, 500, 1000, 2000];
            const LOG_LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"]; 
            const THRESHOLD_STEPS_MB: [u64; 5] = [1, 10, 100, 500, 1024];

            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    if self.settings_selected < 3 { self.settings_selected += 1; }
                    return Ok(());
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if self.settings_selected > 0 { self.settings_selected -= 1; }
                    return Ok(());
                }
                KeyCode::Left | KeyCode::Char('-') => {
                    match self.settings_selected {
                        0 => { // theme prev
                            if self.current_theme_index == 0 { self.set_theme_by_index(self.themes.len()-1); } else { self.set_theme_by_index(self.current_theme_index - 1); }
                        }
                        1 => { // refresh interval prev
                            let mut idx = REFRESH_STEPS.iter().position(|v| *v == self.config.refresh_interval_ms).unwrap_or(2);
                            if idx > 0 { idx -= 1; }
                            self.set_refresh_interval_ms(REFRESH_STEPS[idx]);
                        }
                        2 => { // log level prev
                            let mut idx = LOG_LEVELS.iter().position(|v| *v == self.config.log_level).unwrap_or(2);
                            if idx > 0 { idx -= 1; }
                            self.set_log_level(LOG_LEVELS[idx]);
                        }
                        3 => { // threshold prev
                            let mut idx = THRESHOLD_STEPS_MB.iter().position(|v| *v == self.config.storage_min_threshold_mb).unwrap_or(0);
                            if idx > 0 { idx -= 1; }
                            self.set_storage_threshold_mb(THRESHOLD_STEPS_MB[idx]);
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                KeyCode::Right | KeyCode::Char('+') => {
                    match self.settings_selected {
                        0 => { // theme next
                            self.set_theme_by_index((self.current_theme_index + 1) % self.themes.len());
                        }
                        1 => { // refresh interval next
                            let mut idx = REFRESH_STEPS.iter().position(|v| *v == self.config.refresh_interval_ms).unwrap_or(2);
                            if idx + 1 < REFRESH_STEPS.len() { idx += 1; }
                            self.set_refresh_interval_ms(REFRESH_STEPS[idx]);
                        }
                        2 => { // log level next
                            let mut idx = LOG_LEVELS.iter().position(|v| *v == self.config.log_level).unwrap_or(2);
                            if idx + 1 < LOG_LEVELS.len() { idx += 1; }
                            self.set_log_level(LOG_LEVELS[idx]);
                        }
                        3 => { // threshold next
                            let mut idx = THRESHOLD_STEPS_MB.iter().position(|v| *v == self.config.storage_min_threshold_mb).unwrap_or(0);
                            if idx + 1 < THRESHOLD_STEPS_MB.len() { idx += 1; }
                            self.set_storage_threshold_mb(THRESHOLD_STEPS_MB[idx]);
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                _ => {}
            }
        }

        // Navigation and screen-specific controls
        match key.code {
            // Screen navigation
            KeyCode::Tab => {
                self.current_screen = self.current_screen.next();
                self.reset_selection();
            }
            KeyCode::BackTab => {
                self.current_screen = self.current_screen.previous();
                self.reset_selection();
            }

            // Vim-style navigation
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

            // Actions
            KeyCode::Enter => {
                self.handle_enter()?;
            }
            KeyCode::Char('/') => {
                if self.current_screen == Screen::Commands {
                    self.input_mode = true;
                }
            }
            KeyCode::Char('r') => {
                // Refresh/rescan
                self.handle_refresh()?;
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                // Toggle through themes (without modifier for ease of use)
                self.next_theme();
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
        let max_items = self.get_max_items();
        if max_items > 0 && self.selected_index < max_items - 1 {
            self.selected_index += 1;
            // Keep selection visible by scrolling if needed
            // Each command takes 2 lines (title + description)
            let visible_lines = 20; // Approximate visible height
            let items_per_screen = visible_lines / 2;
            if self.selected_index >= self.scroll_offset + items_per_screen {
                self.scroll_offset = self.selected_index.saturating_sub(items_per_screen - 1);
            }
        }
    }

    /// Move selection up
    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Keep selection visible by scrolling if needed
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    /// Get maximum number of items in current view
    fn get_max_items(&self) -> usize {
        match self.current_screen {
            Screen::Storage => self.storage.get_results_count(),
            Screen::Commands => self.commands.get_results_count(),
            _ => 0,
        }
    }

    /// Handle Enter key press
    fn handle_enter(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Storage => {
                // Could implement drill-down into directory
                info!("Storage item selected: {}", self.selected_index);
            }
            Screen::Commands => {
                // Execute selected command
                if let Some(cmd) = self.commands.get_selected_command(self.selected_index) {
                    info!("Would execute command: {}", cmd.command);
                    // In production, add confirmation dialog and safe execution
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle input submission (search, etc.)
    fn handle_input_submit(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Commands => {
                self.commands.search(&self.input_buffer)?;
                self.reset_selection();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle refresh action
    fn handle_refresh(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Storage => {
                info!("Starting storage scan");
                self.storage.start_scan()?;
            }
            Screen::Metrics => {
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
