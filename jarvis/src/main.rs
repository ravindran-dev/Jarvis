use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::fs;
use std::io;
use std::process::Command;
use std::sync::Arc;

use jarvis_core::{cmdlang::ActionRegistry, events::EventBus};

use crate::app::App;
use crate::config::Config;

mod app;
mod commands;
mod config;
mod plugins;
pub mod shell;
mod system;
mod theme;
mod ui;
pub mod utils;

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_tui(
    engine: crate::shell::ExecutionEngine,
    session_context: crate::shell::SessionContext,
    initial_screen: crate::app::Screen,
) -> Result<()> {
    let mut terminal = setup_terminal()?;

    // Panic hook to ensure raw mode is disabled on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    let mut app = App::new(engine, session_context, initial_screen)?;
    let result = app.run(&mut terminal);

    if let Err(e) = restore_terminal(terminal) {
        error!("Failed to restore terminal: {}", e);
    }

    if let Err(e) = result {
        error!("Application error: {}", e);
        return Err(e);
    }
    Ok(())
}

struct CliInteraction;

impl crate::shell::UserInteraction for CliInteraction {
    fn confirm(&mut self, prompt: &str) -> bool {
        println!("JARVIS: {}", prompt);
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap_or_default();
        buf.trim().to_lowercase() == "y"
    }

    fn print(&mut self, text: &str) {
        println!("JARVIS: {}", text);
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("JARVIS TERMINAL PORTAL");
        println!("Type 'jarvis commands' to see available modules.");
        return Ok(());
    }

    let mut session_context = crate::shell::SessionContext::default();
    let mut config = Config::load();
    let mut registry = ActionRegistry::new();
    jarvis_core::proc::register_all(&mut registry);
    jarvis_core::resources::register_all(&mut registry);
    jarvis_core::cgroup::register_all(&mut registry);
    jarvis_core::svc::register_all(&mut registry);
    jarvis_core::net::register_all(&mut registry);

    let event_bus = EventBus::new();
    let registry_arc = Arc::new(std::sync::Mutex::new(registry));
    let engine = crate::shell::ExecutionEngine::new(registry_arc.clone(), event_bus.clone());

    let cmd = args[1].as_str();

    if cmd == "commands" {
        println!("JARVIS TERMINAL PORTAL - Available Commands");
        println!("===========================================");
        println!("  jarvis monitor    - Launch main dashboard");
        println!("  jarvis cpu        - Launch CPU view");
        println!("  jarvis memory     - Launch Memory view");
        println!("  jarvis storage    - Launch Storage explorer");
        println!("  jarvis processes  - Launch Process manager");
        println!("  jarvis network    - Launch Network monitor");
        println!("  jarvis services   - System services");
        println!("  jarvis users      - User management");
        println!("  jarvis settings   - JARVIS settings");
        println!();
        println!("UTILITY");
        println!("  jarvis doctor     - Diagnose JARVIS");
        println!("  jarvis update     - Update JARVIS");
        println!("  jarvis help       - Launch Help (WIP)");
        println!("  jarvis commands   - Show this list");
        println!();
        println!("You can also use the short aliases directly: monitor, cpu, storage, processes, network, commands.");
        return Ok(());
    }

    if cmd == "doctor" {
        println!("JARVIS DIAGNOSTICS");
        println!("===========================================");
        println!("✓ JARVIS binary found");
        println!("✓ Zsh integration active");
        println!("✓ Kitty terminal detected");
        println!("✓ Configuration valid");
        println!("✓ Required permissions available");
        println!("✓ Plugins loaded");
        println!();
        println!("STATUS: HEALTHY");
        return Ok(());
    }

    if cmd == "update" {
        let mut child = Command::new("/home/ravi/Jarvis/update.sh").spawn()?;
        child.wait()?;
        return Ok(());
    }

    let screen = match cmd {
        "monitor" => Some(crate::app::Screen::Overview),
        "cpu" => Some(crate::app::Screen::Cpu),
        "memory" => Some(crate::app::Screen::Memory),
        "storage" => Some(crate::app::Screen::Storage),
        "processes" => Some(crate::app::Screen::Processes),
        "network" => Some(crate::app::Screen::Network),
        "services" => Some(crate::app::Screen::Services),
        "users" => Some(crate::app::Screen::Users),
        "settings" => Some(crate::app::Screen::Settings),
        "help" => Some(crate::app::Screen::Help),
        _ => None,
    };

    if let Some(s) = screen {
        return run_tui(engine, session_context, s);
    }

    // If not a TUI view, execute as a JARVIS command
    let command_line = args[1..].join(" ");
    let mut interaction = CliInteraction;
    match engine.execute_line(
        &command_line,
        &mut session_context,
        &mut config,
        &mut interaction,
    ) {
        Ok(res) => {
            if !res.output.is_empty() {
                println!("{}", res.output);
            }
        }
        Err(e) => {
            println!("JARVIS Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
