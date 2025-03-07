use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("{0} was already loaded")]
    AlreadyLoaded(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin execution error: {0}")]
    ExecutionError(anyhow::Error),
}

/// Plugin trait that must be implemented by plugins
/// This is typically defined in your main application and re-exported here
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the name of the plugin
    fn name(&self) -> &str;

    /// Returns a description of the plugin
    fn description(&self) -> &str;

    /// Returns whether the plugin is enabled
    fn is_enabled(&self) -> bool;

    /// Enables the plugin
    fn enable(&mut self);

    /// Disables the plugin
    fn disable(&mut self);

    /// Called when the plugin is loaded
    fn on_load(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Called when the plugin is unloaded
    fn on_unload(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait PluginHooks: Debug {}

/// Plugin manager that handles plugin loading, execution, and lifecycle
#[derive(Debug, Default)]
pub struct PluginManager<PH: PluginHooks> {
    plugins: HashMap<String, Box<dyn Plugin>>,
    hooks: PH,
}

impl<PH: PluginHooks> PluginManager<PH> {
    /// Create a new plugin manager
    pub fn new(hooks: PH) -> Self {
        Self {
            plugins: HashMap::new(),
            hooks,
        }
    }

    /// Register a statically linked plugin
    pub fn load_plugin(&mut self, plugin: Box<dyn Plugin>) -> PluginResult<()> {
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name));
        }

        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> PluginResult<()> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.on_unload()?;
        }
        Ok(())
    }

    /// Get a reference to a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Get a mutable reference to a plugin by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut dyn Plugin> {
        self.plugins.get_mut(name).map(|p| p.as_mut())
    }

    /// Get a typed reference to a plugin (with downcasting)
    pub fn get_plugin_as<T: 'static>(&self, name: &str) -> Option<&T> {
        self.get_plugin(name)
            .and_then(|p| p.as_any().downcast_ref::<T>())
    }

    /// Get a typed mutable reference to a plugin (with downcasting)
    pub fn get_plugin_as_mut<T: 'static>(&mut self, name: &str) -> Option<&mut T> {
        self.get_plugin_mut(name)
            .and_then(|p| p.as_any_mut().downcast_mut::<T>())
    }

    /// Get all plugin names
    pub fn plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get all plugins
    pub fn plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|p| p.as_ref()).collect()
    }

    /// Get all enabled plugins
    pub fn enabled_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins
            .values()
            .filter(|p| p.is_enabled())
            .map(|p| p.as_ref())
            .collect()
    }

    /// Enable a plugin by name
    pub fn enable_plugin(&mut self, name: &str) -> PluginResult<()> {
        match self.plugins.get_mut(name) {
            Some(plugin) => {
                plugin.enable();
                Ok(())
            }
            None => Err(PluginError::NotFound(name.to_string())),
        }
    }

    /// Disable a plugin by name
    pub fn disable_plugin(&mut self, name: &str) -> PluginResult<()> {
        match self.plugins.get_mut(name) {
            Some(plugin) => {
                plugin.disable();
                Ok(())
            }
            None => Err(PluginError::NotFound(name.to_string())),
        }
    }
}
