use crate::cmdlang::ActionRegistry;
use crate::types::{Action, ActionMetadata, ActionResult};
use anyhow::Result;
use sysinfo::System;

pub struct StatusAction {
    metadata: ActionMetadata,
}

impl StatusAction {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            metadata: ActionMetadata {
                name: "status".to_string(),
                description: "Get current system status".to_string(),
                destructive: false,
                requires_privilege: false,
                category: "system".to_string(),
            },
        }
    }
}

impl Action for StatusAction {
    fn metadata(&self) -> &ActionMetadata {
        &self.metadata
    }

    fn execute(&self, _args: &[&str]) -> Result<ActionResult> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let mem_usage = (used_mem as f64 / total_mem as f64) * 100.0;

        Ok(ActionResult::Information {
            data: format!(
                "Everything looks stable. CPU usage is {:.0}% and memory usage is {:.0}%.",
                cpu_usage, mem_usage
            ),
        })
    }
}

pub fn register_all(registry: &mut ActionRegistry) {
    registry.register(Box::new(StatusAction::new()));
}
