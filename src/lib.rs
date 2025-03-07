// lib.rs
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub mod error;
pub mod hook;

use self::error::PluginResult;
use self::hook::HookRegistry;

pub type PluginID = &'static str;

/// Plugin trait that must be implemented by plugins
/// This is typically defined in your main application and re-exported here
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the name of the plugin
    fn name(&self) -> PluginID;

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

/// Plugin manager that handles plugin loading, execution, and lifecycle
#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: HashMap<PluginID, Box<dyn Plugin>>,
    hook_registry: HookRegistry,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            hook_registry: HookRegistry::new(),
        }
    }

    /// Create a new plugin manager with an existing hook registry
    pub fn with_registry(hook_registry: HookRegistry) -> Self {
        Self {
            plugins: HashMap::new(),
            hook_registry,
        }
    }

    /// Get a reference to the hook registry
    pub fn hook_registry(&self) -> &HookRegistry {
        &self.hook_registry
    }

    /// Get a mutable reference to the hook registry
    pub fn hook_registry_mut(&mut self) -> &mut HookRegistry {
        &mut self.hook_registry
    }

    /// Register a statically linked plugin
    pub fn load_plugin(&mut self, plugin: Box<dyn Plugin>) -> PluginResult<()> {
        let id = plugin.name();
        if self.plugins.contains_key(id) {
            return Err(error::PluginError::AlreadyLoaded(id));
        }

        // Store the plugin
        self.plugins.insert(id, plugin);

        // Get a mutable reference to the just-stored plugin and call on_load
        if let Some(plugin) = self.plugins.values_mut().last() {
            plugin.on_load()?;
        }

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
    pub fn plugin_ids(&self) -> Vec<PluginID> {
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
    pub fn enable_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        match self.plugins.get_mut(id) {
            Some(plugin) => {
                plugin.enable();
                Ok(())
            }
            None => Err(error::PluginError::NotFound(id)),
        }
    }

    /// Disable a plugin by name
    pub fn disable_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        match self.plugins.get_mut(id) {
            Some(plugin) => {
                plugin.disable();
                Ok(())
            }
            None => Err(error::PluginError::NotFound(id)),
        }
    }
}
