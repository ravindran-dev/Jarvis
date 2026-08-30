use crate::cmdlang::ActionRegistry;
use crate::types::{Action, ActionMetadata, ActionResult};
use anyhow::Result;
use sysinfo::System;

pub struct FindProcessAction {
    metadata: ActionMetadata,
}

impl FindProcessAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "find".to_string(),
                description: "Find a process by name or PID".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "proc".to_string(),
            },
        }
    }
}

impl Action for FindProcessAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        if args.is_empty() {
            return Ok(ActionResult::Failure {
                reason: "Missing target process name or PID".to_string(),
                error: None,
            });
        }

        let target = args.join(" ").to_lowercase();
        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let mut found = Vec::new();
        for (pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            if name.to_lowercase().contains(&target) || pid.as_u32().to_string() == target {
                found.push((pid.as_u32(), name));
            }
        }

        if found.is_empty() {
            Ok(ActionResult::Failure {
                reason: format!("Could not find process matching '{}'", target),
                error: None,
            })
        } else {
            let main_pid = found[0].0;
            let main_name = found[0].1.clone();
            Ok(ActionResult::Success {
                action: "found".to_string(),
                target: Some(main_name),
                details: format!("Found {} processes. Main PID is {}.", found.len(), main_pid),
                events: None,
            })
        }
    }
}

pub struct KillProcessAction {
    metadata: ActionMetadata,
}

impl KillProcessAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "kill".to_string(),
                description: "Kill a process".to_string(),
                destructive: true,
                requires_privilege: false,
                category: "proc".to_string(),
            },
        }
    }
}

impl Action for KillProcessAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (target, force) = extract_force(args);
        if target.is_empty() {
            return Ok(ActionResult::Failure {
                reason: "Missing target process".to_string(),
                error: None,
            });
        }

        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let found = find_matching_processes(&sys, &target);
        if found.is_empty() {
            return Ok(ActionResult::Failure {
                reason: format!("Could not find process matching '{}'", target),
                error: None,
            });
        }

        if !force {
            return Ok(ActionResult::NeedsConfirmation {
                action: "kill".to_string(),
                impact: format!("kill {} process(es) matching '{}'", found.len(), target),
                warning: "This is a destructive operation".to_string(),
            });
        }

        let mut killed_count = 0;
        let target_name = found[0].1.clone();

        let mut events = Vec::new();
        for (pid, name) in &found {
            if sys
                .process(sysinfo::Pid::from_u32(*pid))
                .is_some_and(|p| p.kill())
            {
                killed_count += 1;
                events.push(crate::events::JarvisEvent::ProcessKilled(
                    *pid,
                    name.clone(),
                ));
            }
        }

        Ok(ActionResult::Success {
            action: "killed".to_string(),
            target: Some(target_name),
            details: format!("Successfully killed {} process(es).", killed_count),
            events: Some(events),
        })
    }
}

pub struct PauseProcessAction {
    metadata: ActionMetadata,
}

impl PauseProcessAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "pause".to_string(),
                description: "Pause a process (SIGSTOP)".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "proc".to_string(),
            },
        }
    }
}

impl Action for PauseProcessAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (target, force) = extract_force(args);
        if target.is_empty() {
            return Ok(ActionResult::Failure {
                reason: "Missing target process".to_string(),
                error: None,
            });
        }

        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let found = find_matching_processes(&sys, &target);
        if found.is_empty() {
            return Ok(ActionResult::Failure {
                reason: format!("Could not find process matching '{}'", target),
                error: None,
            });
        }

        if found.len() > 1 && !force {
            return Ok(ActionResult::NeedsConfirmation {
                action: "pause".to_string(),
                impact: format!("pause {} processes matching '{}'", found.len(), target),
                warning: "This affects multiple processes".to_string(),
            });
        }

        let mut paused_count = 0;
        let target_name = found[0].1.clone();

        let mut events = Vec::new();
        for (pid, name) in &found {
            if let Some(process) = sys.process(sysinfo::Pid::from_u32(*pid)) {
                #[cfg(unix)]
                if process.kill_with(sysinfo::Signal::Stop).unwrap_or(false) {
                    paused_count += 1;
                    events.push(crate::events::JarvisEvent::ProcessPaused(
                        *pid,
                        name.clone(),
                    ));
                }
            }
        }

        Ok(ActionResult::Success {
            action: "paused".to_string(),
            target: Some(target_name),
            details: format!("Successfully paused {} process(es).", paused_count),
            events: Some(events),
        })
    }
}

pub struct ResumeProcessAction {
    metadata: ActionMetadata,
}

impl ResumeProcessAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "resume".to_string(),
                description: "Resume a process (SIGCONT)".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "proc".to_string(),
            },
        }
    }
}

impl Action for ResumeProcessAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (target, force) = extract_force(args);
        if target.is_empty() {
            return Ok(ActionResult::Failure {
                reason: "Missing target process".to_string(),
                error: None,
            });
        }

        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let found = find_matching_processes(&sys, &target);
        if found.is_empty() {
            return Ok(ActionResult::Failure {
                reason: format!("Could not find process matching '{}'", target),
                error: None,
            });
        }

        if found.len() > 1 && !force {
            return Ok(ActionResult::NeedsConfirmation {
                action: "resume".to_string(),
                impact: format!("resume {} processes matching '{}'", found.len(), target),
                warning: "This affects multiple processes".to_string(),
            });
        }

        let mut resumed_count = 0;
        let target_name = found[0].1.clone();

        let mut events = Vec::new();
        for (pid, name) in &found {
            if let Some(process) = sys.process(sysinfo::Pid::from_u32(*pid)) {
                #[cfg(unix)]
                if process
                    .kill_with(sysinfo::Signal::Continue)
                    .unwrap_or(false)
                {
                    resumed_count += 1;
                    events.push(crate::events::JarvisEvent::ProcessResumed(
                        *pid,
                        name.clone(),
                    ));
                }
            }
        }

        Ok(ActionResult::Success {
            action: "resumed".to_string(),
            target: Some(target_name),
            details: format!("Successfully resumed {} process(es).", resumed_count),
            events: Some(events),
        })
    }
}

pub struct ProcsAction {
    metadata: ActionMetadata,
}

impl ProcsAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "procs".to_string(),
                description: "List running processes".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "proc".to_string(),
            },
        }
    }
}

impl Action for ProcsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, _args: &[&str]) -> Result<ActionResult> {
        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        let count = sys.processes().len();
        Ok(ActionResult::Information {
            data: format!("There are {} processes currently running.", count),
        })
    }
}

pub fn register_all(registry: &mut ActionRegistry) {
    registry.register(Box::new(FindProcessAction::new()));
    registry.register(Box::new(KillProcessAction::new()));
    registry.register(Box::new(PauseProcessAction::new()));
    registry.register(Box::new(ResumeProcessAction::new()));
    registry.register(Box::new(ProcsAction::new()));
}

fn extract_force(args: &[&str]) -> (String, bool) {
    let mut clean_args = Vec::new();
    let mut force = false;
    for arg in args {
        if *arg == "--force" {
            force = true;
        } else {
            clean_args.push(*arg);
        }
    }
    (clean_args.join(" ").to_lowercase(), force)
}

fn find_matching_processes(sys: &System, target: &str) -> Vec<(u32, String)> {
    let mut found = Vec::new();
    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();
        if name.to_lowercase().contains(target) || pid.as_u32().to_string() == target {
            found.push((pid.as_u32(), name));
        }
    }
    found
}
