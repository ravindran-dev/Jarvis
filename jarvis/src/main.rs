use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use jarvis_core::events::EventBus;
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io;
use std::sync::Arc;

mod app;
mod config;
use config::Config;
#[allow(dead_code)] // FUTURE_SCAFFOLD: Unfinished async event support
mod async_loop;
mod commands;
#[allow(dead_code)] // FUTURE_SCAFFOLD: Unfinished plugin architecture
mod plugins;
mod shell;
mod system;
mod theme;
mod ui;
mod utils;

use app::App;
use jarvis_core::cmdlang::ActionRegistry;
use plugins::loader::PluginLoader;
use shell::SessionContext;
use theme::Theme;

/// Initialize the terminal for TUI rendering
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal mode
fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_tui(
    engine: crate::shell::ExecutionEngine,
    session_context: SessionContext,
) -> Result<Option<String>> {
    let mut plugin_loader = PluginLoader::default();
    plugin_loader.validate_plugins_dir()?;
    let _ = plugin_loader.discover();

    let mut terminal = setup_terminal()?;
    let mut app = App::new(engine, session_context)?;
    let result = app.run(&mut terminal);

    if let Err(e) = restore_terminal(terminal) {
        error!("Failed to restore terminal: {}", e);
    }

    if let Err(e) = result {
        error!("Application error: {}", e);
        return Err(e);
    }
    Ok(app.selected_command_to_run)
}

struct CliInteraction<'a> {
    rl: Option<&'a mut DefaultEditor>,
}

impl<'a> crate::shell::UserInteraction for CliInteraction<'a> {
    fn confirm(&mut self, prompt: &str) -> bool {
        println!("\nJARVIS: {}", prompt);
        let confirm = if let Some(editor) = &mut self.rl {
            editor.readline("> ").unwrap_or_default()
        } else {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap_or_default();
            buf
        };
        confirm.trim().to_lowercase() == "y"
    }

    fn print(&mut self, text: &str) {
        println!("JARVIS: {}", text);
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    let _theme = Theme::dark();
    let mut config = Config::load();

    let mut rl = DefaultEditor::new()?;
    let _ = rl.load_history(".jarvis_history");

    let mut session_context = SessionContext::default();
    let mut registry = ActionRegistry::new();
    jarvis_core::proc::register_all(&mut registry);
    jarvis_core::resources::register_all(&mut registry);
    jarvis_core::cgroup::register_all(&mut registry);
    jarvis_core::svc::register_all(&mut registry);
    jarvis_core::net::register_all(&mut registry);

    let event_bus = EventBus::new();

    let registry_arc = Arc::new(std::sync::Mutex::new(registry));
    let engine = crate::shell::ExecutionEngine::new(registry_arc.clone(), event_bus.clone());

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let command_line = args[1..].join(" ");
        // Check macro/alias for one-shot CLI too by splitting on ';'
        for part in command_line.split(';') {
            let mut interaction = CliInteraction { rl: None };
            if part.trim() == "dashboard" {
                match run_tui(
                    crate::shell::ExecutionEngine::new(registry_arc.clone(), event_bus.clone()),
                    session_context.clone(),
                ) {
                    Ok(Some(_cmd)) => {
                        println!("JARVIS: Command insertion not supported in one-shot mode.");
                    }
                    Ok(None) => {}
                    Err(e) => {
                        println!("JARVIS: Error running dashboard: {}", e);
                    }
                }
                continue;
            }
            match engine.execute_line(part, &mut session_context, &mut config, &mut interaction) {
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
        }
        return Ok(());
    }

    println!("JARVIS System Control Environment");
    println!("Type 'dashboard' to enter TUI, or run normal commands.");

    let mut initial_input: Option<String> = None;

    loop {
        let readline = if let Some(ref init) = initial_input {
            rl.readline_with_initial("jarvis ❯ ", (init.as_str(), ""))
        } else {
            rl.readline("jarvis ❯ ")
        };
        initial_input = None;

        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);

                // Allow entering multiple commands separated by ;
                let mut should_exit = false;
                for part in trimmed.split(';') {
                    if part.trim() == "dashboard" {
                        match run_tui(
                            crate::shell::ExecutionEngine::new(
                                registry_arc.clone(),
                                event_bus.clone(),
                            ),
                            session_context.clone(),
                        ) {
                            Ok(Some(cmd)) => {
                                initial_input = Some(cmd);
                                break;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                println!("JARVIS: Error running dashboard: {}", e);
                            }
                        }
                        continue;
                    }
                    let mut interaction = CliInteraction { rl: Some(&mut rl) };
                    match engine.execute_line(
                        part,
                        &mut session_context,
                        &mut config,
                        &mut interaction,
                    ) {
                        Ok(res) => {
                            if !res.output.is_empty() {
                                println!("{}", res.output);
                            }
                            if res.requires_exit {
                                should_exit = true;
                                break;
                            }
                        }
                        Err(e) => {
                            println!("JARVIS Error: {}", e);
                        }
                    }
                }
                if should_exit {
                    break;
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("JARVIS: Error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(".jarvis_history");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_core::types::Action;
    use jarvis_core::types::ActionMetadata;
    use jarvis_core::types::ActionResult;

    struct DummyAction {
        metadata: ActionMetadata,
        fail: bool,
    }

    impl DummyAction {
        fn new(name: &str, fail: bool) -> Self {
            Self {
                metadata: ActionMetadata {
                    name: name.to_string(),
                    description: "Dummy".to_string(),
                    destructive: false,
                    requires_privilege: false,
                    category: "test".to_string(),
                },
                fail,
            }
        }
    }

    impl Action for DummyAction {
        fn metadata(&self) -> &ActionMetadata {
            &self.metadata
        }

        fn execute(&self, _args: &[&str]) -> anyhow::Result<ActionResult> {
            if self.fail {
                Ok(ActionResult::Failure {
                    reason: format!("{} failed", self.metadata.name),
                    error: None,
                })
            } else {
                Ok(ActionResult::Success {
                    action: self.metadata.name.clone(),
                    target: Some("dummy".to_string()),
                    details: "Success".to_string(),
                    events: Some(vec![jarvis_core::events::JarvisEvent::ActionExecuted(
                        self.metadata.name.clone(),
                        "dummy".to_string(),
                    )]),
                })
            }
        }
    }

    #[test]
    fn test_macro_event_bus_and_failure_stop() {
        let mut session_context = SessionContext::default();
        let mut registry = ActionRegistry::new();
        registry.register(Box::new(DummyAction::new("procs", false)));
        registry.register(Box::new(DummyAction::new("status", true)));
        registry.register(Box::new(DummyAction::new("find", false)));

        let mut config = Config::default();
        config.macros.insert(
            "test_macro".to_string(),
            crate::config::MacroDef {
                description: "Test".to_string(),
                steps: vec![
                    "procs".to_string(),
                    "status".to_string(),
                    "find".to_string(),
                ],
            },
        );

        let event_bus = EventBus::new();
        let mut rx = event_bus.subscribe();

        let registry_arc = Arc::new(std::sync::Mutex::new(registry));
        let engine = crate::shell::ExecutionEngine::new(registry_arc, event_bus.clone());
        let mut interaction = super::CliInteraction { rl: None };

        let result = engine.execute_line(
            "macro run test_macro",
            &mut session_context,
            &mut config,
            &mut interaction,
        );

        assert!(result.is_err()); // Because status fails

        // Check events
        let ev1 = rx.try_recv().unwrap();
        if let jarvis_core::events::JarvisEvent::ActionExecuted(a, _) = ev1 {
            assert_eq!(a, "procs");
        } else {
            panic!("Expected ActionExecuted for procs");
        }

        // status fails, so it doesn't emit ActionExecuted
        // find never runs because status failed and stopped the macro
        assert!(rx.try_recv().is_err());
    }
}
