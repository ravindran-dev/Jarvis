use crate::cmdlang::ActionRegistry;
use crate::types::{Action, ActionMetadata, ActionResult};
use anyhow::Result;
use std::process::Command;

macro_rules! service_action {
    ($struct_name:ident, $name:expr, $desc:expr, $destruct:expr, $privilege:expr, $event_constructor:expr) => {
        pub struct $struct_name {
            metadata: ActionMetadata,
        }

        impl $struct_name {
            #[allow(clippy::new_without_default)]
            // Actions are stateless; new() is preferred over Default for semantic clarity.
            pub fn new() -> Self {
                Self {
                    metadata: ActionMetadata {
                        name: $name.to_string(),
                        description: $desc.to_string(),
                        destructive: $destruct,
                        requires_privilege: $privilege,
                        category: "systemd".to_string(),
                    },
                }
            }
        }

        impl Action for $struct_name {
            fn metadata(&self) -> &ActionMetadata {
                &self.metadata
            }

            fn execute(&self, args: &[&str]) -> Result<ActionResult> {
                let (clean_args, force) = extract_force(args);
                if clean_args.is_empty() {
                    return Ok(ActionResult::Failure {
                        reason: format!("Usage: {} <service>", $name),
                        error: None,
                    });
                }

                let svc = &clean_args[0];

                if $destruct && !force {
                    return Ok(ActionResult::NeedsConfirmation {
                        action: $name.to_string(),
                        impact: format!("{} service '{}'", $name, svc),
                        warning: "This affects system services".to_string(),
                    });
                }

                let output = Command::new("systemctl").arg($name).arg(svc).output();
                match output {
                    Ok(out) if out.status.success() => Ok(ActionResult::Success {
                        action: $name.to_string(),
                        target: Some(svc.to_string()),
                        details: String::from_utf8_lossy(&out.stdout).to_string(),
                        events: Some(vec![$event_constructor(svc.to_string())]),
                    }),
                    Ok(out) => Ok(ActionResult::Failure {
                        reason: format!("Failed to {} {}", $name, svc),
                        error: Some(String::from_utf8_lossy(&out.stderr).to_string()),
                    }),
                    Err(e) => Ok(ActionResult::Failure {
                        reason: format!("systemctl {} failed", $name),
                        error: Some(e.to_string()),
                    }),
                }
            }
        }
    };
}

service_action!(
    StartServiceAction,
    "start",
    "Start a systemd service",
    true,
    true,
    crate::events::JarvisEvent::ServiceStarted
);
service_action!(
    StopServiceAction,
    "stop",
    "Stop a systemd service",
    true,
    true,
    crate::events::JarvisEvent::ServiceStopped
);
service_action!(
    RestartServiceAction,
    "restart",
    "Restart a systemd service",
    true,
    true,
    crate::events::JarvisEvent::ServiceRestarted
);
service_action!(
    EnableServiceAction,
    "enable",
    "Enable a systemd service",
    true,
    true,
    |svc| crate::events::JarvisEvent::ActionExecuted("enable".to_string(), svc)
);
service_action!(
    DisableServiceAction,
    "disable",
    "Disable a systemd service",
    true,
    true,
    |svc| crate::events::JarvisEvent::ActionExecuted("disable".to_string(), svc)
);

pub struct ServiceInfoAction {
    metadata: ActionMetadata,
}

impl ServiceInfoAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "service".to_string(),
                description: "Get status of a systemd service".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "systemd".to_string(),
            },
        }
    }
}

impl Action for ServiceInfoAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (clean_args, _) = extract_force(args);
        if clean_args.is_empty() {
            return Ok(ActionResult::Failure {
                reason: "Usage: service <name>".to_string(),
                error: None,
            });
        }

        let svc = &clean_args[0];
        let output = Command::new("systemctl").arg("status").arg(svc).output();

        match output {
            Ok(out) => {
                let data = String::from_utf8_lossy(if out.status.success() {
                    &out.stdout
                } else {
                    &out.stderr
                })
                .to_string();
                Ok(ActionResult::Information { data })
            }
            Err(e) => Ok(ActionResult::Failure {
                reason: "Failed to get service status".to_string(),
                error: Some(e.to_string()),
            }),
        }
    }
}

pub struct ServicesAction {
    metadata: ActionMetadata,
}

impl ServicesAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "services".to_string(),
                description: "List all systemd services".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "systemd".to_string(),
            },
        }
    }
}

impl Action for ServicesAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, _args: &[&str]) -> Result<ActionResult> {
        let output = Command::new("systemctl")
            .arg("list-units")
            .arg("--type=service")
            .arg("--all")
            .output();

        match output {
            Ok(out) if out.status.success() => Ok(ActionResult::Information {
                data: String::from_utf8_lossy(&out.stdout).to_string(),
            }),
            Ok(out) => Ok(ActionResult::Failure {
                reason: "Failed to list services".to_string(),
                error: Some(String::from_utf8_lossy(&out.stderr).to_string()),
            }),
            Err(e) => Ok(ActionResult::Failure {
                reason: "systemctl command failed".to_string(),
                error: Some(e.to_string()),
            }),
        }
    }
}

pub fn register_all(registry: &mut ActionRegistry) {
    registry.register(Box::new(StartServiceAction::new()));
    registry.register(Box::new(StopServiceAction::new()));
    registry.register(Box::new(RestartServiceAction::new()));
    registry.register(Box::new(EnableServiceAction::new()));
    registry.register(Box::new(DisableServiceAction::new()));
    registry.register(Box::new(ServiceInfoAction::new()));
    registry.register(Box::new(ServicesAction::new()));
}

fn extract_force(args: &[&str]) -> (Vec<String>, bool) {
    let mut clean_args = Vec::new();
    let mut force = false;
    for arg in args {
        if *arg == "--force" {
            force = true;
        } else {
            clean_args.push(arg.to_string());
        }
    }
    (clean_args, force)
}
