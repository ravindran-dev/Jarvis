use anyhow::{anyhow, Result};
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin version
    fn version(&self) -> &str;

    /// Initialize plugin
    fn init(&self) -> Result<()>;

    /// Execute plugin command
    fn execute(&self, args: Vec<String>) -> Result<String>;

    /// Cleanup plugin
    fn cleanup(&self) -> Result<()>;
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub enabled: bool,
}

/// Plugin loader for dynamic library loading
pub struct PluginLoader {
    plugins_dir: PathBuf,
    loaded_plugins: Vec<PluginInfo>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(plugins_dir: Option<PathBuf>) -> Self {
        let plugins_dir = plugins_dir.unwrap_or_else(|| {
            dirs::config_dir()
                .map(|d| d.join("jarvis").join("plugins"))
                .unwrap_or_else(|| PathBuf::from("./plugins"))
        });

        debug!("Plugin loader initialized with directory: {:?}", plugins_dir);

        Self {
            plugins_dir,
            loaded_plugins: Vec::new(),
        }
    }

    /// Discover available plugins
    pub fn discover(&mut self) -> Result<Vec<PluginInfo>> {
        if !self.plugins_dir.exists() {
            debug!("Plugins directory does not exist: {:?}", self.plugins_dir);
            return Ok(Vec::new());
        }

        let mut plugins = Vec::new();

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Look for .so (Linux) or .dll (Windows) or .dylib (macOS) files
            if let Some(extension) = path.extension() {
                if self.is_plugin_file(&extension) {
                    if let Ok(info) = self.extract_plugin_info(&path) {
                        debug!("Discovered plugin: {:?}", info);
                        plugins.push(info);
                    }
                }
            }
        }

        self.loaded_plugins = plugins.clone();
        info!("Discovered {} plugins", plugins.len());
        Ok(plugins)
    }

    /// Check if file is a valid plugin file
    fn is_plugin_file(&self, extension: &OsStr) -> bool {
        matches!(
            extension.to_str(),
            Some("so") | Some("dll") | Some("dylib")
        )
    }

    /// Extract plugin information from file path
    fn extract_plugin_info(&self, path: &Path) -> Result<PluginInfo> {
        let name = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("Invalid plugin file"))?
            .to_string();

        // Remove "lib" prefix if present
        let name = if name.starts_with("lib") {
            name[3..].to_string()
        } else {
            name
        };

        Ok(PluginInfo {
            name,
            version: "1.0.0".to_string(),
            path: path.to_path_buf(),
            enabled: true,
        })
    }

    /// List all discovered plugins
    pub fn list_plugins(&self) -> &[PluginInfo] {
        &self.loaded_plugins
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.loaded_plugins.iter_mut().find(|p| p.name == name) {
            plugin.enabled = true;
            info!("Enabled plugin: {}", name);
            Ok(())
        } else {
            Err(anyhow!("Plugin not found: {}", name))
        }
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.loaded_plugins.iter_mut().find(|p| p.name == name) {
            plugin.enabled = false;
            info!("Disabled plugin: {}", name);
            Ok(())
        } else {
            Err(anyhow!("Plugin not found: {}", name))
        }
    }

    /// Validate plugins directory structure
    pub fn validate_plugins_dir(&self) -> Result<()> {
        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)?;
            debug!("Created plugins directory: {:?}", self.plugins_dir);
        }
        Ok(())
    }

    /// Get plugins directory
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new(None);
        assert!(loader.plugins_dir.as_os_str().len() > 0);
    }

    #[test]
    fn test_is_plugin_file() {
        let loader = PluginLoader::new(None);
        assert!(loader.is_plugin_file(OsStr::new("so")));
        assert!(loader.is_plugin_file(OsStr::new("dll")));
        assert!(loader.is_plugin_file(OsStr::new("dylib")));
        assert!(!loader.is_plugin_file(OsStr::new("txt")));
    }

    #[test]
    fn test_extract_plugin_info() {
        let loader = PluginLoader::new(None);
        let path = PathBuf::from("libmyplugin.so");
        let info = loader.extract_plugin_info(&path);
        assert!(info.is_ok());
        assert_eq!(info.unwrap().name, "myplugin");
    }

    #[test]
    fn test_list_plugins() {
        let loader = PluginLoader::new(None);
        let plugins = loader.list_plugins();
        assert!(plugins.is_empty());
    }
}
