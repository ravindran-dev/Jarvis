use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success {
        action: String,
        target: Option<String>,
        details: String,
        events: Option<Vec<crate::events::JarvisEvent>>,
    },
    Information {
        data: String,
    },
    Warning {
        details: String,
    },
    NeedsConfirmation {
        action: String,
        impact: String,
        warning: String,
    },
    Failure {
        reason: String,
        error: Option<String>,
    },
    NetworkConnections(Vec<NetworkConnection>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
    pub pid: Option<i32>,
    pub process_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ActionMetadata {
    pub name: String,
    pub description: String,
    pub destructive: bool,
    pub requires_privilege: bool,
    pub category: String,
}

pub trait Action: Send + Sync {
    fn metadata(&self) -> &ActionMetadata;
    fn execute(&self, args: &[&str]) -> anyhow::Result<ActionResult>;
}
