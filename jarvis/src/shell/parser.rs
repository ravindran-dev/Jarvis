use super::context::SessionContext;

#[derive(Debug, Clone)]
pub enum Intent {
    Action {
        action: String,
        args: Vec<String>,
    },
    TuiView,
    MacroCommand {
        subcommand: String,
        args: Vec<String>,
    },
    PassThrough {
        command: String,
    },
}

pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str, context: &SessionContext) -> Intent {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Intent::PassThrough {
                command: String::new(),
            };
        }

        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "dashboard" => Intent::TuiView,
            "status" => Intent::Action {
                action: "status".to_string(),
                args: vec![],
            },
            "procs" => Intent::Action {
                action: "procs".to_string(),
                args: vec![],
            },
            "find" | "info" | "kill" | "pause" | "resume" | "tree" | "priority" => {
                let mut resolved_args = vec![];
                if parts.len() > 1 {
                    let target_str = parts[1];
                    if let Some(resolved) = context.resolve_target(target_str) {
                        resolved_args.push(resolved.to_string());
                    } else {
                        resolved_args.push(target_str.to_string());
                    }
                    resolved_args.extend(parts[2..].iter().map(|s| s.to_string()));
                }
                Intent::Action {
                    action: cmd,
                    args: resolved_args,
                }
            }
            "limit" | "unlimit" | "limits" => {
                let mut resolved_args = vec![];
                if parts.len() > 1 {
                    let target_str = parts[1];
                    if let Some(resolved) = context.resolve_target(target_str) {
                        resolved_args.push(resolved.to_string());
                    } else {
                        resolved_args.push(target_str.to_string());
                    }
                    resolved_args.extend(parts[2..].iter().map(|s| s.to_string()));
                }
                Intent::Action {
                    action: cmd,
                    args: resolved_args,
                }
            }
            "services" | "service" | "start" | "stop" | "restart" | "enable" | "disable" => {
                let mut resolved_args = vec![];
                if parts.len() > 1 {
                    let target_str = parts[1];
                    resolved_args.push(target_str.to_string());
                    resolved_args.extend(parts[2..].iter().map(|s| s.to_string()));
                }
                Intent::Action {
                    action: cmd,
                    args: resolved_args,
                }
            }
            "network" | "connections" | "block" | "allow" | "blocked" => {
                let mut resolved_args = vec![];
                if parts.len() > 1 {
                    resolved_args.push(parts[1..].join(" "));
                }
                Intent::Action {
                    action: cmd,
                    args: resolved_args,
                }
            }
            "macro" => {
                if parts.len() > 1 {
                    Intent::MacroCommand {
                        subcommand: parts[1].to_string(),
                        args: parts[2..].iter().map(|s| s.to_string()).collect(),
                    }
                } else {
                    Intent::PassThrough {
                        command: input.to_string(),
                    }
                }
            }
            "actions" | "macros" | "history" => Intent::Action {
                action: cmd,
                args: vec![],
            },
            _ => Intent::PassThrough {
                command: trimmed.to_string(),
            },
        }
    }
}
