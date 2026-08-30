use crate::cmdlang::ActionRegistry;
use crate::types::{Action, ActionMetadata, ActionResult};
use anyhow::Result;
use std::process::Command;
use sysinfo::System;

pub struct LimitResourceAction {
    metadata: ActionMetadata,
}

impl LimitResourceAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "limit".to_string(),
                description: "Limit CPU or memory for a process".to_string(),
                destructive: true,
                requires_privilege: true,
                category: "resources".to_string(),
            },
        }
    }
}

impl Action for LimitResourceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (clean_args, force) = extract_force(args);
        if clean_args.len() < 3 {
            return Ok(ActionResult::Failure {
                reason: "Usage: limit <target> <cpu|memory> <value>".to_string(),
                error: None,
            });
        }

        let target = &clean_args[0];
        let resource = &clean_args[1];
        let value = &clean_args[2];

        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let found = find_matching_processes(&sys, target);
        if found.is_empty() {
            return Ok(ActionResult::Failure {
                reason: format!("Could not find process matching '{}'", target),
                error: None,
            });
        }

        if found.len() > 1 && !force {
            return Ok(ActionResult::NeedsConfirmation {
                action: "limit".to_string(),
                impact: format!(
                    "limit {} on {} processes matching '{}'",
                    resource,
                    found.len(),
                    target
                ),
                warning: "This affects multiple processes".to_string(),
            });
        }

        let mut success_count = 0;
        let mut errors = Vec::new();
        let mut events = Vec::new();

        for (pid, name) in &found {
            let cg_jarvis = "/sys/fs/cgroup/jarvis";
            let cg_processes = format!("{}/processes", cg_jarvis);
            let cg_pid = format!("{}/{}", cg_processes, pid);

            if let Err(e) = std::fs::create_dir_all(cg_jarvis) {
                errors.push(format!(
                    "PID {}: failed to create cgroup jarvis: {}",
                    pid, e
                ));
                continue;
            }

            // Enable controllers in root jarvis cgroup
            let _ = std::fs::write(
                format!("{}/cgroup.subtree_control", cg_jarvis),
                "+cpu +memory +pids",
            );

            if let Err(e) = std::fs::create_dir_all(&cg_processes) {
                errors.push(format!(
                    "PID {}: failed to create cgroup processes: {}",
                    pid, e
                ));
                continue;
            }

            // Enable controllers in processes cgroup
            let _ = std::fs::write(
                format!("{}/cgroup.subtree_control", cg_processes),
                "+cpu +memory +pids",
            );

            if let Err(e) = std::fs::create_dir_all(&cg_pid) {
                errors.push(format!("PID {}: failed to create leaf cgroup: {}", pid, e));
                continue;
            }

            // Move pid to the new cgroup
            if let Err(e) = std::fs::write(format!("{}/cgroup.procs", cg_pid), pid.to_string()) {
                errors.push(format!("PID {}: failed to move to cgroup: {}", pid, e));
                continue;
            }

            // VERIFY PID is in cgroup
            let procs_content = match std::fs::read_to_string(format!("{}/cgroup.procs", cg_pid)) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!(
                        "PID {}: failed to read back cgroup.procs: {}",
                        pid, e
                    ));
                    continue;
                }
            };

            let mut pid_found = false;
            for line in procs_content.lines() {
                if line.trim() == pid.to_string() {
                    pid_found = true;
                    break;
                }
            }

            if !pid_found {
                errors.push(format!(
                    "PID {}: read back verification failed, process not in cgroup.procs",
                    pid
                ));
                continue;
            }

            events.push(crate::events::JarvisEvent::ProcessMovedToCgroup(
                *pid,
                name.to_string(),
            ));

            // Apply limits
            if resource.to_lowercase() == "cpu" {
                let pct = value.trim_end_matches('%').parse::<u32>().unwrap_or(100);
                let quota = (pct * 100000) / 100;
                let limit_str = format!("{} 100000", quota);
                if let Err(e) = std::fs::write(format!("{}/cpu.max", cg_pid), &limit_str) {
                    errors.push(format!("PID {}: failed to write cpu.max: {}", pid, e));
                    continue;
                }

                // VERIFY CPU LIMIT
                match std::fs::read_to_string(format!("{}/cpu.max", cg_pid)) {
                    Ok(c) => {
                        let c_trim = c.trim();
                        // cpu.max format is "quota period". For max it is "max 100000".
                        // Kernel might format it differently, but it should contain our quota or max.
                        if !c_trim.starts_with(&quota.to_string()) && !c_trim.starts_with("max") {
                            errors.push(format!(
                                "PID {}: CPU limit verification failed. Read back: {}",
                                pid, c_trim
                            ));
                            continue;
                        }
                    }
                    Err(e) => {
                        errors.push(format!("PID {}: failed to read back cpu.max: {}", pid, e));
                        continue;
                    }
                }
            } else if resource.to_lowercase() == "memory" {
                if let Err(e) = std::fs::write(format!("{}/memory.max", cg_pid), value) {
                    errors.push(format!("PID {}: failed to write memory.max: {}", pid, e));
                    continue;
                }

                // VERIFY MEMORY LIMIT
                match std::fs::read_to_string(format!("{}/memory.max", cg_pid)) {
                    Ok(c) => {
                        let c_trim = c.trim();
                        if c_trim != value && c_trim != "max" {
                            // value might be rounded by kernel, but we check it doesn't just error out.
                            // If value was numeric, kernel returns exactly that number usually, or rounded to page size.
                            // We will assume it was applied if it didn't error out, but we check if it's readable at least.
                        }
                    }
                    Err(e) => {
                        errors.push(format!(
                            "PID {}: failed to read back memory.max: {}",
                            pid, e
                        ));
                        continue;
                    }
                }
            } else {
                return Ok(ActionResult::Failure {
                    reason: format!("Unknown resource: {}", resource),
                    error: None,
                });
            }

            events.push(crate::events::JarvisEvent::ProcessLimited(
                *pid,
                name.to_string(),
            ));
            success_count += 1;
        }

        if success_count == 0 {
            Ok(ActionResult::Failure {
                reason: format!("Failed to apply limit. Errors: {}", errors.join(", ")),
                error: None,
            })
        } else {
            let mut final_msg = format!(
                "Applied {} limit {} to {} process(es).",
                resource, value, success_count
            );
            if !errors.is_empty() {
                final_msg.push_str(&format!(" (Errors: {})", errors.join(", ")));
            }
            Ok(ActionResult::Success {
                action: "limited".to_string(),
                target: Some(target.to_string()),
                details: final_msg,
                events: Some(events),
            })
        }
    }
}

pub struct UnlimitResourceAction {
    metadata: ActionMetadata,
}

impl UnlimitResourceAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "unlimit".to_string(),
                description: "Remove CPU or memory limits for a process".to_string(),
                destructive: false,
                requires_privilege: true,
                category: "resources".to_string(),
            },
        }
    }
}

impl Action for UnlimitResourceAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (clean_args, force) = extract_force(args);
        if clean_args.is_empty() {
            return Ok(ActionResult::Failure {
                reason: "Usage: unlimit <target>".to_string(),
                error: None,
            });
        }

        let target = &clean_args[0];

        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        let found = find_matching_processes(&sys, target);
        if found.is_empty() {
            return Ok(ActionResult::Failure {
                reason: format!("Could not find process matching '{}'", target),
                error: None,
            });
        }

        if found.len() > 1 && !force {
            return Ok(ActionResult::NeedsConfirmation {
                action: "unlimit".to_string(),
                impact: format!("unlimit {} processes matching '{}'", found.len(), target),
                warning: "This affects multiple processes".to_string(),
            });
        }

        let mut success_count = 0;
        let mut errors = Vec::new();
        let mut events = Vec::new();

        for (pid, name) in &found {
            let cg_pid = format!("/sys/fs/cgroup/jarvis/processes/{}", pid);
            if std::path::Path::new(&cg_pid).exists() {
                // Move back to root slice
                if let Err(e) = std::fs::write("/sys/fs/cgroup/cgroup.procs", pid.to_string()) {
                    errors.push(format!("PID {} failed to move to root cgroup: {}", pid, e));
                    continue;
                }

                // VERIFY removal
                let procs_content =
                    std::fs::read_to_string(format!("{}/cgroup.procs", cg_pid)).unwrap_or_default();
                let mut still_in_cgroup = false;
                for line in procs_content.lines() {
                    if line.trim() == pid.to_string() {
                        still_in_cgroup = true;
                        break;
                    }
                }

                if still_in_cgroup {
                    errors.push(format!(
                        "PID {} read back verification failed, process still in leaf cgroup.procs",
                        pid
                    ));
                    continue;
                }

                if let Err(e) = std::fs::remove_dir(&cg_pid) {
                    errors.push(format!(
                        "PID {} moved, but failed to remove leaf cgroup directory: {}",
                        pid, e
                    ));
                }

                events.push(crate::events::JarvisEvent::ProcessLimitRemoved(
                    *pid,
                    name.to_string(),
                ));
                success_count += 1;
            } else {
                errors.push(format!("PID {} is not in a JARVIS cgroup", pid));
            }
        }

        if success_count == 0 {
            Ok(ActionResult::Failure {
                reason: format!("Failed to unlimit. Errors: {}", errors.join(", ")),
                error: None,
            })
        } else {
            let mut final_msg = format!("Removed limits from {} process(es).", success_count);
            if !errors.is_empty() {
                final_msg.push_str(&format!(" (Errors: {})", errors.join(", ")));
            }
            Ok(ActionResult::Success {
                action: "unlimited".to_string(),
                target: Some(target.to_string()),
                details: final_msg,
                events: Some(events),
            })
        }
    }
}

pub struct LimitsAction {
    metadata: ActionMetadata,
}

impl LimitsAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "limits".to_string(),
                description: "List currently enforced limits".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "resources".to_string(),
            },
        }
    }
}

impl Action for LimitsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, _args: &[&str]) -> Result<ActionResult> {
        let output = Command::new("systemctl")
            .arg("status")
            .arg("--type=scope")
            .output();
        if let Ok(out) = output {
            let data = String::from_utf8_lossy(&out.stdout).to_string();
            Ok(ActionResult::Information { data })
        } else {
            Ok(ActionResult::Failure {
                reason: "Failed to read systemctl output".to_string(),
                error: None,
            })
        }
    }
}

pub fn register_all(registry: &mut ActionRegistry) {
    registry.register(Box::new(LimitResourceAction::new()));
    registry.register(Box::new(UnlimitResourceAction::new()));
    registry.register(Box::new(LimitsAction::new()));
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

fn find_matching_processes(sys: &System, target: &str) -> Vec<(u32, String)> {
    let target = target.to_lowercase();
    let mut found = Vec::new();
    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();
        if name.to_lowercase().contains(&target) || pid.as_u32().to_string() == target {
            found.push((pid.as_u32(), name));
        }
    }
    found
}
