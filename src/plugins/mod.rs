use log::debug;
use std::collections::HashMap;

/// Plugin trait for extensibility
#[allow(dead_code)]
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Update plugin state (called every refresh cycle)
    fn update(&mut self);

    /// Render plugin output as a string
    fn render(&self) -> String;

    /// Get plugin version
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Check if plugin is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Plugin manager to handle multiple plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin + 'static>>,
}

impl PluginManager {
    /// Create a new empty PluginManager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a new plugin
    #[allow(dead_code)]
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.name().to_string();
        debug!("Registering plugin: {}", name);
        self.plugins.insert(name, plugin);
    }

    /// Unregister a plugin by name
    #[allow(dead_code)]
    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn Plugin>> {
        debug!("Unregistering plugin: {}", name);
        self.plugins.remove(name)
    }

    /// Get a plugin by name
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Update all enabled plugins
    pub fn update_all(&mut self) {
        for plugin in self.plugins.values_mut() {
            if plugin.is_enabled() {
                plugin.update();
            }
        }
    }

    /// Get list of all plugin names
    #[allow(dead_code)]
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get number of registered plugins
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Render all enabled plugins
    #[allow(dead_code)]
    pub fn render_all(&self) -> Vec<(String, String)> {
        self.plugins
            .values()
            .filter(|p| p.is_enabled())
            .map(|p| (p.name().to_string(), p.render()))
            .collect()
    }
}

// Example plugin implementation for demonstration
#[allow(dead_code)]
pub struct ExamplePlugin {
    name: String,
    counter: u64,
}

impl ExamplePlugin {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Self { name, counter: 0 }
    }
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn update(&mut self) {
        self.counter += 1;
    }

    fn render(&self) -> String {
        format!("Example plugin [{}] - Counter: {}", self.name, self.counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new("test".to_string()));

        manager.register(plugin);
        assert_eq!(manager.count(), 1);
        assert!(manager.get("test").is_some());
    }

    #[test]
    fn test_plugin_update() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new("test".to_string()));
        manager.register(plugin);

        manager.update_all();

        if let Some(plugin) = manager.get("test") {
            let render = plugin.render();
            assert!(render.contains("Counter: 1"));
        }
    }
}
