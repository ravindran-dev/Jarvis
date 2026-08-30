use std::sync::Arc;
use tokio::sync::broadcast;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JarvisEvent {
    CpuHigh(f32),
    MemoryHigh(f32),
    ProcessKilled(u32, String),
    ProcessPaused(u32, String),
    ProcessResumed(u32, String),
    ProcessLimited(u32, String),
    ProcessLimitRemoved(u32, String),
    ProcessMovedToCgroup(u32, String),
    ServiceStarted(String),
    ServiceStopped(String),
    ServiceRestarted(String),
    NetworkBlocked(String),
    NetworkAllowed(String),
    ActionExecuted(String, String),
    Log(String),
}

impl std::fmt::Display for JarvisEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JarvisEvent::CpuHigh(val) => write!(f, "CpuHigh -> {:.1}%", val),
            JarvisEvent::MemoryHigh(val) => write!(f, "MemoryHigh -> {:.1}%", val),
            JarvisEvent::ProcessKilled(pid, name) => {
                write!(f, "ProcessKilled -> {} ({})", name, pid)
            }
            JarvisEvent::ProcessPaused(pid, name) => {
                write!(f, "ProcessPaused -> {} ({})", name, pid)
            }
            JarvisEvent::ProcessResumed(pid, name) => {
                write!(f, "ProcessResumed -> {} ({})", name, pid)
            }
            JarvisEvent::ProcessLimited(pid, name) => {
                write!(f, "ProcessLimited -> {} ({})", name, pid)
            }
            JarvisEvent::ProcessLimitRemoved(pid, name) => {
                write!(f, "ProcessLimitRemoved -> {} ({})", name, pid)
            }
            JarvisEvent::ProcessMovedToCgroup(pid, name) => {
                write!(f, "ProcessMovedToCgroup -> {} ({})", name, pid)
            }
            JarvisEvent::ServiceStarted(name) => write!(f, "ServiceStarted -> {}", name),
            JarvisEvent::ServiceStopped(name) => write!(f, "ServiceStopped -> {}", name),
            JarvisEvent::ServiceRestarted(name) => write!(f, "ServiceRestarted -> {}", name),
            JarvisEvent::NetworkBlocked(target) => write!(f, "NetworkBlocked -> {}", target),
            JarvisEvent::NetworkAllowed(target) => write!(f, "NetworkAllowed -> {}", target),
            JarvisEvent::ActionExecuted(action, target) => {
                write!(f, "ActionExecuted -> {} on {}", action, target)
            }
            JarvisEvent::Log(msg) => write!(f, "Log -> {}", msg),
        }
    }
}

pub struct EventBus {
    sender: broadcast::Sender<JarvisEvent>,
}

impl EventBus {
    pub fn new() -> Arc<Self> {
        let (sender, _) = broadcast::channel(100);
        Arc::new(Self { sender })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<JarvisEvent> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: JarvisEvent) {
        let _ = self.sender.send(event);
    }
}
