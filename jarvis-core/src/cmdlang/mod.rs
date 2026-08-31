use crate::types::{Action, ActionResult};
use anyhow::Result;
use std::collections::HashMap;

pub struct ActionRegistry {
    actions: HashMap<String, Box<dyn Action>>,
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionRegistry {
    #[allow(clippy::new_without_default)]
    // Actions are stateless; new() is preferred over Default for semantic clarity.
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: Box<dyn Action>) {
        let name = action.metadata().name.clone();
        self.actions.insert(name, action);
    }

    pub fn execute(&self, name: &str, args: &[&str]) -> Result<ActionResult> {
        if let Some(action) = self.actions.get(name) {
            action.execute(args)
        } else {
            Ok(ActionResult::Failure {
                reason: format!("Action '{}' not found", name),
                error: None,
            })
        }
    }

    pub fn get_action_names(&self) -> Vec<String> {
        self.actions.keys().cloned().collect()
    }

    pub fn get_metadata(&self) -> Vec<crate::types::ActionMetadata> {
        self.actions.values().map(|a| a.metadata().clone()).collect()
    }

    pub fn requires_privilege(&self, name: &str) -> bool {
        if let Some(action) = self.actions.get(name) {
            action.metadata().requires_privilege
        } else {
            false
        }
    }
}
