use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use log::info;
use ratatui::{backend::Backend, Terminal};
use std::time::{Duration, Instant};

use crate::commands::CommandIndex;
use crate::plugins::PluginManager;
use crate::system::{metrics::SystemMetrics, storage::StorageAnalyzer};
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
}

impl App {
    /// Create a new App instance
    pub fn new() -> Result<Self> {
        info!("Initializing Jarvis application");

        let metrics = SystemMetrics::new()?;
        let storage = StorageAnalyzer::new()?;
        let commands = CommandIndex::new()?;
        let plugins = PluginManager::new();

        Ok(Self {
            should_quit: false,
            current_screen: Screen::Metrics,
            metrics,
            storage,
            commands,
            plugins,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
            selected_index: 0,
            scroll_offset: 0,
            input_buffer: String::new(),
            input_mode: false,
        })
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
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
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
            KeyCode::Char('h') | KeyCode::Left => {
                self.current_screen = self.current_screen.previous();
                self.reset_selection();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.current_screen = self.current_screen.next();
                self.reset_selection();
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
        }
    }

    /// Move selection up
    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
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
}
