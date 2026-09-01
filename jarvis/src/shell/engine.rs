use crate::config::Config;
use crate::shell::{CommandParser, ConversationalResponse, Intent, SessionContext};
use anyhow::Result;
use jarvis_core::cmdlang::ActionRegistry;
use jarvis_core::events::EventBus;
use std::process::Command;
use std::sync::Arc;

pub struct ExecutionResult {
    pub output: String,
    pub requires_exit: bool,
}

pub trait UserInteraction {
    fn confirm(&mut self, prompt: &str) -> bool;
    fn print(&mut self, text: &str);
}

pub struct ExecutionEngine {
    pub registry: Arc<std::sync::Mutex<ActionRegistry>>,
    pub event_bus: Arc<EventBus>,
}

impl ExecutionEngine {
    pub fn new(registry: Arc<std::sync::Mutex<ActionRegistry>>, event_bus: Arc<EventBus>) -> Self {
        Self {
            registry,
            event_bus,
        }
    }

    pub fn execute_line(
        &self,
        line: &str,
        session_context: &mut SessionContext,
        config: &mut Config,
        interaction: &mut dyn UserInteraction,
    ) -> Result<ExecutionResult> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(ExecutionResult {
                output: String::new(),
                requires_exit: false,
            });
        }

        if trimmed == "exit" || trimmed == "quit" {
            return Ok(ExecutionResult {
                output: String::new(),
                requires_exit: true,
            });
        }

        // 1. Macro Expansion
        if let Some(macro_val) = config.macros.get(trimmed) {
            let mut requires_exit = false;
            let mut final_output = String::new();
            let steps = macro_val.steps.clone();
            for step in steps {
                let res = self.execute_line(&step, session_context, config, interaction)?;
                if !res.output.is_empty() {
                    final_output.push_str(&res.output);
                    final_output.push('\n');
                }
                if res.requires_exit {
                    requires_exit = true;
                }
            }
            return Ok(ExecutionResult {
                output: final_output.trim_end().to_string(),
                requires_exit,
            });
        }

        // 2. Alias Expansion
        let mut actual_line = trimmed.to_string();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if !parts.is_empty() {
            if let Some(alias_val) = config.aliases.get(parts[0]) {
                let rest = if parts.len() > 1 {
                    &trimmed[parts[0].len()..]
                } else {
                    ""
                };
                actual_line = format!("{}{}", alias_val, rest);
            }
        }

        let intent = CommandParser::parse(&actual_line, session_context);
        let mut output = String::new();

        match intent {
            Intent::TuiView => {
                // TUI mode doesn't execute from engine, usually main.rs handles it.
                return Err(anyhow::anyhow!(
                    "TUI cannot be launched from inside engine execution."
                ));
            }
            Intent::MacroCommand { subcommand, args } => match subcommand.as_str() {
                "list" => {
                    output.push_str("JARVIS Macros:\n");
                    for (name, def) in &config.macros {
                        output.push_str(&format!("  {} - {}\n", name, def.description));
                        for (i, step) in def.steps.iter().enumerate() {
                            output.push_str(&format!("    {}. {}\n", i + 1, step));
                        }
                    }
                }
                "create" => {
                    if args.len() < 2 {
                        output.push_str("Usage: macro create <name> \"<description>\" \"<step1>\" \"<step2>\" ...");
                    } else {
                        let name = args[0].clone();
                        let desc = args[1].clone();
                        let steps = if args.len() > 2 {
                            args[2..].to_vec()
                        } else {
                            vec![]
                        };
                        config.macros.insert(
                            name.clone(),
                            crate::config::MacroDef {
                                description: desc,
                                steps,
                            },
                        );
                        if let Err(e) = config.save() {
                            output.push_str(&format!("Failed to save config: {}", e));
                        } else {
                            output.push_str(&format!("Macro '{}' created.", name));
                        }
                    }
                }
                "delete" => {
                    if args.is_empty() {
                        output.push_str("Usage: macro delete <name>");
                    } else {
                        let name = args[0].clone();
                        if config.macros.remove(&name).is_some() {
                            if let Err(e) = config.save() {
                                output.push_str(&format!("Failed to save config: {}", e));
                            } else {
                                output.push_str(&format!("Macro '{}' deleted.", name));
                            }
                        } else {
                            output.push_str(&format!("Macro '{}' not found.", name));
                        }
                    }
                }
                "run" => {
                    if args.is_empty() {
                        output.push_str("Usage: macro run <name>");
                    } else {
                        let name = args[0].clone();
                        if let Some(macro_val) = config.macros.get(&name) {
                            let steps = macro_val.steps.clone();
                            for step in steps {
                                match self.execute_line(&step, session_context, config, interaction)
                                {
                                    Ok(res) => {
                                        if !res.output.is_empty() {
                                            output.push_str(&res.output);
                                            output.push('\n');
                                        }
                                        if res.requires_exit {
                                            return Ok(ExecutionResult {
                                                output,
                                                requires_exit: true,
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        let msg = format!("Macro step failed, stopping: {}", e);
                                        interaction.print(&msg);
                                        return Err(anyhow::anyhow!("Macro step failed: {}", e));
                                    }
                                }
                            }
                        } else {
                            output.push_str(&format!("Macro '{}' not found.", name));
                        }
                    }
                }
                _ => {
                    output.push_str("Unknown macro subcommand. Use list, create, run, delete.");
                }
            },
            Intent::Action { action, args } => {
                let mut arg_strs: Vec<String> = args.clone();
                let is_privileged = {
                    let reg = self.registry.lock().unwrap();
                    reg.requires_privilege(&action)
                };

                let result = if is_privileged {
                    let daemon = jarvis_core::daemon::DaemonClient::new();
                    if daemon.is_running() {
                        let req_args: Vec<&str> = arg_strs.iter().map(|s| s.as_str()).collect();
                        if let Some(req) =
                            jarvis_core::daemon::DaemonRequest::from_cmd(&action, &req_args)
                        {
                            match daemon.send_request(req) {
                                Ok(resp) => match resp {
                                    jarvis_core::daemon::DaemonResponse::Success(action_result) => {
                                        let text = ConversationalResponse::generate(&action_result);
                                        output.push_str(&text);
                                        if let jarvis_core::types::ActionResult::Success {
                                            events: Some(ref evs),
                                            ..
                                        } = action_result
                                        {
                                            for e in evs {
                                                self.event_bus.publish(e.clone());
                                            }
                                        }
                                        if let jarvis_core::types::ActionResult::Failure {
                                            reason,
                                            ..
                                        } = action_result
                                        {
                                            return Err(anyhow::anyhow!(
                                                "Action failed: {}",
                                                reason
                                            ));
                                        }
                                        return Ok(ExecutionResult {
                                            output,
                                            requires_exit: false,
                                        });
                                    }
                                    jarvis_core::daemon::DaemonResponse::Error(msg) => {
                                        let err_msg = format!("I couldn't do that: {}", msg);
                                        interaction.print(&err_msg);
                                        return Err(anyhow::anyhow!("Daemon error: {}", msg));
                                    }
                                },
                                Err(e) => {
                                    return Err(anyhow::anyhow!(
                                        "Daemon communication failed: {}",
                                        e
                                    ));
                                }
                            }
                        } else {
                            return Err(anyhow::anyhow!(
                                "Invalid arguments for privileged action: {}",
                                action
                            ));
                        }
                    } else {
                        return Err(anyhow::anyhow!(
                            "Daemon not running. Privileged action '{}' cannot be executed safely.",
                            action
                        ));
                    }
                } else {
                    let arg_refs: Vec<&str> = arg_strs.iter().map(|s| s.as_str()).collect();
                    let reg = self.registry.lock().unwrap();
                    reg.execute(&action, &arg_refs)
                };

                let result = result?;
                if let jarvis_core::types::ActionResult::NeedsConfirmation {
                    impact, warning, ..
                } = &result
                {
                    let prompt = format!(
                        "This will {}. {}. Do you want me to continue? [y/N]",
                        impact, warning
                    );
                    if interaction.confirm(&prompt) {
                        arg_strs.push("--force".to_string());
                        let final_result = if is_privileged {
                            let daemon = jarvis_core::daemon::DaemonClient::new();
                            if daemon.is_running() {
                                let req_args: Vec<&str> =
                                    arg_strs.iter().map(|s| s.as_str()).collect();
                                if let Some(req) =
                                    jarvis_core::daemon::DaemonRequest::from_cmd(&action, &req_args)
                                {
                                    match daemon.send_request(req) {
                                        Ok(resp) => match resp {
                                            jarvis_core::daemon::DaemonResponse::Success(
                                                action_result,
                                            ) => Ok(action_result),
                                            jarvis_core::daemon::DaemonResponse::Error(msg) => {
                                                Err(anyhow::anyhow!("Daemon error: {}", msg))
                                            }
                                        },
                                        Err(e) => Err(anyhow::anyhow!(
                                            "Daemon communication failed: {}",
                                            e
                                        )),
                                    }
                                } else {
                                    Err(anyhow::anyhow!("Invalid arguments"))
                                }
                            } else {
                                Err(anyhow::anyhow!("Daemon not running"))
                            }
                        } else {
                            let arg_refs: Vec<&str> = arg_strs.iter().map(|s| s.as_str()).collect();
                            let reg = self.registry.lock().unwrap();
                            reg.execute(&action, &arg_refs)
                        };

                        let final_res = final_result?;
                        output.push_str(&ConversationalResponse::generate(&final_res));
                        if let jarvis_core::types::ActionResult::Success {
                            events: Some(ref evs),
                            ..
                        } = final_res
                        {
                            for e in evs {
                                self.event_bus.publish(e.clone());
                            }
                        }
                        if let jarvis_core::types::ActionResult::Failure { reason, .. } = final_res
                        {
                            return Err(anyhow::anyhow!("Action failed: {}", reason));
                        }
                    } else {
                        output.push_str("Action cancelled.");
                    }
                } else {
                    output.push_str(&ConversationalResponse::generate(&result));
                    if let jarvis_core::types::ActionResult::Success {
                        events: Some(ref evs),
                        ..
                    } = result
                    {
                        for e in evs {
                            self.event_bus.publish(e.clone());
                        }
                    }
                    if let jarvis_core::types::ActionResult::Failure { reason, .. } = result {
                        return Err(anyhow::anyhow!("Action failed: {}", reason));
                    }
                }
                if !args.is_empty() {
                    session_context.last_target = Some(args[0].clone());
                }
            }
            Intent::PassThrough { command } => {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "zsh".to_string());
                match Command::new(&shell).arg("-c").arg(&command).status() {
                    Ok(status) => {
                        if !status.success() {
                            let code = status.code().unwrap_or(1);
                            return Err(anyhow::anyhow!("Command failed with code {}", code));
                        }
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to execute command: {}", e));
                    }
                }
            }
        }
        Ok(ExecutionResult {
            output: output.trim_end().to_string(),
            requires_exit: false,
        })
    }
}
