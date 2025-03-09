// lib.rs
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub mod error;
pub mod hook;

use self::error::PluginResult;
use self::hook::HookRegistry;

/// Plugin identifier type
pub type PluginID = &'static str;

/// Plugin trait that must be implemented by plugins
/// This is typically defined in your main application and re-exported here
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the name of the plugin
    fn id(&self) -> PluginID;

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
        let id = plugin.id();
        if self.plugins.contains_key(id) {
            return Err(error::PluginError::AlreadyLoaded(id));
        }

        // Store the plugin
        self.plugins.insert(id, plugin);

        // Get a mutable reference to the just-stored plugin and call on_load
        if let Some(plugin) = self.plugins.get_mut(id) {
            // Load the plugin
            plugin.on_load()?;
        }

        Ok(())
    }

    /// Unload a plugin by ID
    pub fn unload_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        if let Some(mut plugin) = self.plugins.remove(id) {
            // Call on_unload for cleanup
            plugin.on_unload()?;

            // Remove all hooks registered by this plugin
            self.hook_registry.deregister_hooks_for_plugin(id);
        }
        Ok(())
    }

    /// Get a reference to a plugin by ID
    pub fn get_plugin(&self, id: PluginID) -> Option<&dyn Plugin> {
        self.plugins.get(id).map(|p| p.as_ref())
    }

    /// Get a mutable reference to a plugin by ID
    pub fn get_plugin_mut(&mut self, id: PluginID) -> Option<&mut dyn Plugin> {
        self.plugins.get_mut(id).map(|p| p.as_mut())
    }

    /// Get all plugin IDs
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

    /// Enable a plugin by ID
    pub fn enable_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        match self.plugins.get_mut(id) {
            Some(plugin) => {
                plugin.enable();
                Ok(())
            }
            None => Err(error::PluginError::NotFound(id)),
        }
    }

    /// Disable a plugin by ID
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

#[macro_export]
macro_rules! extension_point {
    ($name:ident: $trait_name:ident,
        $(fn $method_name:ident(&$self_param:tt $(, $param_name:ident: $param_type:ty)*) -> $return_type:ty),* $(,)?
    ) => {
        // Define the trait with all methods
        pub trait $trait_name: Send + Sync {
            $(
                fn $method_name(&$self_param $(, $param_name: $param_type)*) -> $return_type;
            )*
        }

        // Define the extension point struct
        pub struct $name;

        // Implement ExtensionPoint for the struct
        impl ExtensionPoint for $name {
            type HookTrait = dyn $trait_name;
        }
    };
}
