use crate::cmdlang::ActionRegistry;
use crate::types::{Action, ActionMetadata, ActionResult};
use anyhow::Result;
use procfs::net::{TcpState, tcp, tcp6, udp, udp6};
use procfs::process::all_processes;
use std::collections::HashMap;
use std::process::Command;

pub struct ConnectionsAction {
    metadata: ActionMetadata,
}

impl ConnectionsAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "connections".to_string(),
                description: "List active network connections".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "network".to_string(),
            },
        }
    }
}

impl Action for ConnectionsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, _args: &[&str]) -> Result<ActionResult> {
        let mut inode_to_pid = HashMap::new();
        let mut inode_to_name = HashMap::new();

        if let Ok(procs) = all_processes() {
            for p in procs.flatten() {
                let pid = p.pid;
                let name = p.stat().ok().map(|s| s.comm).unwrap_or_default();
                if let Ok(fds) = p.fd() {
                    for fd in fds.flatten() {
                        if let procfs::process::FDTarget::Socket(inode) = fd.target {
                            inode_to_pid.insert(inode, pid);
                            inode_to_name.insert(inode, name.clone());
                        }
                    }
                }
            }
        }

        let mut connections = Vec::new();

        if let Ok(entries) = tcp() {
            for entry in entries {
                let pid = inode_to_pid.get(&entry.inode).copied();
                let name = inode_to_name.get(&entry.inode).cloned();
                let local = format!(
                    "{}:{}",
                    entry.local_address.ip(),
                    entry.local_address.port()
                );
                let remote = format!(
                    "{}:{}",
                    entry.remote_address.ip(),
                    entry.remote_address.port()
                );
                let state = format!("{:?}", entry.state);
                connections.push(crate::types::NetworkConnection {
                    protocol: "tcp".to_string(),
                    local_addr: local,
                    remote_addr: remote,
                    state,
                    pid,
                    process_name: name,
                });
            }
        }
        if let Ok(entries) = tcp6() {
            for entry in entries {
                let pid = inode_to_pid.get(&entry.inode).copied();
                let name = inode_to_name.get(&entry.inode).cloned();
                let local = format!(
                    "{}:{}",
                    entry.local_address.ip(),
                    entry.local_address.port()
                );
                let remote = format!(
                    "{}:{}",
                    entry.remote_address.ip(),
                    entry.remote_address.port()
                );
                let state = format!("{:?}", entry.state);
                connections.push(crate::types::NetworkConnection {
                    protocol: "tcp6".to_string(),
                    local_addr: local,
                    remote_addr: remote,
                    state,
                    pid,
                    process_name: name,
                });
            }
        }
        if let Ok(entries) = udp() {
            for entry in entries {
                let pid = inode_to_pid.get(&entry.inode).copied();
                let name = inode_to_name.get(&entry.inode).cloned();
                let local = format!(
                    "{}:{}",
                    entry.local_address.ip(),
                    entry.local_address.port()
                );
                let remote = format!(
                    "{}:{}",
                    entry.remote_address.ip(),
                    entry.remote_address.port()
                );
                let state = format!("{:?}", entry.state);
                connections.push(crate::types::NetworkConnection {
                    protocol: "udp".to_string(),
                    local_addr: local,
                    remote_addr: remote,
                    state,
                    pid,
                    process_name: name,
                });
            }
        }
        if let Ok(entries) = udp6() {
            for entry in entries {
                let pid = inode_to_pid.get(&entry.inode).copied();
                let name = inode_to_name.get(&entry.inode).cloned();
                let local = format!(
                    "{}:{}",
                    entry.local_address.ip(),
                    entry.local_address.port()
                );
                let remote = format!(
                    "{}:{}",
                    entry.remote_address.ip(),
                    entry.remote_address.port()
                );
                let state = format!("{:?}", entry.state);
                connections.push(crate::types::NetworkConnection {
                    protocol: "udp6".to_string(),
                    local_addr: local,
                    remote_addr: remote,
                    state,
                    pid,
                    process_name: name,
                });
            }
        }

        Ok(ActionResult::NetworkConnections(connections))
    }
}

pub struct PortsAction {
    metadata: ActionMetadata,
}

impl PortsAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "ports".to_string(),
                description: "List listening ports".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "network".to_string(),
            },
        }
    }
}

impl Action for PortsAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, _args: &[&str]) -> Result<ActionResult> {
        let mut inode_to_pid = HashMap::new();
        let mut inode_to_name = HashMap::new();

        if let Ok(procs) = all_processes() {
            for p in procs.flatten() {
                let pid = p.pid;
                let name = p.stat().ok().map(|s| s.comm).unwrap_or_default();
                if let Ok(fds) = p.fd() {
                    for fd in fds.flatten() {
                        if let procfs::process::FDTarget::Socket(inode) = fd.target {
                            inode_to_pid.insert(inode, pid);
                            inode_to_name.insert(inode, name.clone());
                        }
                    }
                }
            }
        }

        let mut connections = Vec::new();

        if let Ok(entries) = tcp() {
            for entry in entries {
                if entry.state == TcpState::Listen {
                    let pid = inode_to_pid.get(&entry.inode).copied();
                    let name = inode_to_name.get(&entry.inode).cloned();
                    let local = format!(
                        "{}:{}",
                        entry.local_address.ip(),
                        entry.local_address.port()
                    );
                    connections.push(crate::types::NetworkConnection {
                        protocol: "tcp".to_string(),
                        local_addr: local,
                        remote_addr: "-".to_string(),
                        state: "LISTEN".to_string(),
                        pid,
                        process_name: name,
                    });
                }
            }
        }
        if let Ok(entries) = tcp6() {
            for entry in entries {
                if entry.state == TcpState::Listen {
                    let pid = inode_to_pid.get(&entry.inode).copied();
                    let name = inode_to_name.get(&entry.inode).cloned();
                    let local = format!(
                        "{}:{}",
                        entry.local_address.ip(),
                        entry.local_address.port()
                    );
                    connections.push(crate::types::NetworkConnection {
                        protocol: "tcp6".to_string(),
                        local_addr: local,
                        remote_addr: "-".to_string(),
                        state: "LISTEN".to_string(),
                        pid,
                        process_name: name,
                    });
                }
            }
        }

        Ok(ActionResult::NetworkConnections(connections))
    }
}

pub struct NetAction {
    metadata: ActionMetadata,
}

impl NetAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "net".to_string(),
                description: "Manage network rules (block, allow)".to_string(),
                destructive: true,
                requires_privilege: true,
                category: "network".to_string(),
            },
        }
    }
}

impl Action for NetAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, args: &[&str]) -> Result<ActionResult> {
        let (clean_args, force) = extract_force(args);

        if clean_args.len() < 2 {
            return Ok(ActionResult::Failure {
                reason: "Usage: net <block|allow> <ip|port>".to_string(),
                error: None,
            });
        }

        let op = &clean_args[0];
        let target = &clean_args[1];

        if !force {
            return Ok(ActionResult::NeedsConfirmation {
                action: "net".to_string(),
                impact: format!("{} network traffic for {}", op, target),
                warning: "This affects system firewall rules".to_string(),
            });
        }

        // Using ufw for simplicity on Linux systems where available.
        if op.to_lowercase() == "block" {
            let output = Command::new("ufw")
                .arg("deny")
                .arg(target)
                .arg("comment")
                .arg("JARVIS")
                .output();
            match output {
                Ok(out) if out.status.success() => Ok(ActionResult::Success {
                    action: op.to_string(),
                    target: Some(target.to_string()),
                    details: String::from_utf8_lossy(&out.stdout).to_string(),
                    events: Some(vec![crate::events::JarvisEvent::NetworkBlocked(
                        target.to_string(),
                    )]),
                }),
                Ok(out) => Ok(ActionResult::Failure {
                    reason: format!("ufw {} {} failed", op, target),
                    error: Some(String::from_utf8_lossy(&out.stderr).to_string()),
                }),
                Err(e) => Ok(ActionResult::Failure {
                    reason: "Failed to run ufw".to_string(),
                    error: Some(e.to_string()),
                }),
            }
        } else if op.to_lowercase() == "allow" {
            if let Ok(status) = Command::new("ufw").arg("status").arg("numbered").output() {
                let stdout = String::from_utf8_lossy(&status.stdout);
                let mut indices = Vec::new();
                for line in stdout.lines() {
                    if line.contains("JARVIS")
                        && line.contains(target)
                        && let Some(start) = line.find('[')
                        && let Some(end) = line[start..].find(']')
                    {
                        let idx_str = line[start + 1..start + end].trim();
                        if let Ok(idx) = idx_str.parse::<u32>() {
                            indices.push(idx);
                        }
                    }
                }

                indices.sort_by(|a, b| b.cmp(a)); // reverse sort to delete from highest index

                let mut removed = 0;
                let mut last_error = None;
                for idx in indices {
                    match Command::new("ufw")
                        .arg("--force")
                        .arg("delete")
                        .arg(idx.to_string())
                        .output()
                    {
                        Ok(out) => {
                            if out.status.success() {
                                removed += 1;
                            } else {
                                last_error = Some(String::from_utf8_lossy(&out.stderr).to_string());
                            }
                        }
                        Err(e) => {
                            last_error = Some(e.to_string());
                        }
                    }
                }

                if removed > 0 {
                    Ok(ActionResult::Success {
                        action: "allow".to_string(),
                        target: Some(target.to_string()),
                        details: format!("Removed {} JARVIS rule(s) for {}", removed, target),
                        events: Some(vec![crate::events::JarvisEvent::NetworkAllowed(
                            target.to_string(),
                        )]),
                    })
                } else if let Some(err) = last_error {
                    Ok(ActionResult::Failure {
                        reason: format!("Failed to remove rules for {}", target),
                        error: Some(err),
                    })
                } else {
                    Ok(ActionResult::Failure {
                        reason: format!("No JARVIS rules found for {}", target),
                        error: None,
                    })
                }
            } else {
                Ok(ActionResult::Failure {
                    reason: "Failed to read ufw status".to_string(),
                    error: None,
                })
            }
        } else {
            Ok(ActionResult::Failure {
                reason: format!("Unknown net operation: {}", op),
                error: None,
            })
        }
    }
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

pub fn register_all(registry: &mut ActionRegistry) {
    registry.register(Box::new(NetAction::new()));
    registry.register(Box::new(ConnectionsAction::new()));
    registry.register(Box::new(PortsAction::new()));
}
