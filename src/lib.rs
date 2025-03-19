//! # steckrs
//!
//! A lightweight, trait-based plugin system for Rust applications and libraries.
//!
//! ## What is steckrs?
//!
//! "steckrs" is a wordplay combining the German word "Stecker" (meaning "plug" or "connector") and
//! the Rust file extension (.rs). The library provides a flexible, type-safe plugin architecture
//! for Rust applications, allowing developers to:
//!
//! - Define extension points in their applications
//! - Create plugins that integrate with these extension points
//! - Dynamically manage plugins (loading, enabling, disabling, unloading)
//! - Register and invoke hooks with proper type safety
//!
//! ## Core Concepts
//!
//! ### Extension Points
//!
//! [Extension points](crate::hook::ExtensionPoint) define interfaces where plugins can add functionality. Each extension point:
//! - Is defined as a trait that plugins implement
//! - Specifies the contract that plugins must fulfill
//! - Provides type-safe interaction between the core application and plugins
//!
//! ### Plugins
//!
//! [Plugins](Plugin) are self-contained modules that implement functionality for extension points.
//! Each plugin:
//! - Has a unique identifier
//! - Can be enabled or disabled at runtime
//! - Can register multiple hooks to different extension points
//! - Has lifecycle methods ([`on_load`](Plugin::on_load), [`on_unload`](Plugin::on_unload))
//!
//! ### Hooks
//!
//! [Hooks](crate::hook::Hook) are implementations of extension points that plugins register. They:
//! - Implement the trait defined by an extension point
//! - Are invoked when the application calls that extension point
//! - Can be uniquely identified by their plugin ID, extension point, and optional discriminator
//!
//! ## Logs
//!
//! This library logs certain events with the [`tracing`] library.
//!
//! ## Usage Example
//!
//! Here's a simple example of how to use steckrs to create a plugin-enabled application:
//!
//! ```rust
//! use steckrs::{extension_point, simple_plugin, PluginManager};
//!
//! // Define an extension point
//! extension_point!(
//!     GreeterExtension: GreeterTrait;
//!     fn greet(&self, name: &str) -> String;
//! );
//!
//! // Create a plugin
//! simple_plugin!(
//!     HelloPlugin,
//!     "hello_plugin",
//!     "A simple greeting plugin",
//!     hooks: [(GreeterExtension, EnglishGreeter)]
//! );
//!
//! // Implement a hook
//! struct EnglishGreeter;
//! impl GreeterTrait for EnglishGreeter {
//!     fn greet(&self, name: &str) -> String {
//!         format!("Hello, {}!", name)
//!     }
//! }
//!
//! // Create plugin manager
//! let mut plugin_manager = PluginManager::new();
//!
//! // Load and enable the plugin
//! plugin_manager.load_plugin(Box::new(HelloPlugin::new())).unwrap();
//! plugin_manager.enable_plugin(HelloPlugin::ID).unwrap();
//!
//! // Get all enabled hooks (plugins could be disabled)
//! let hooks = plugin_manager.get_enabled_hooks_by_ep::<GreeterExtension>();
//!
//! // execute all hooks relevant for this extension point
//! for (_id, hook) in hooks {
//!     println!("{}", hook.inner().greet("World"));
//! }
//! ```
//!
//! ## Macros
//!
//! steckrs provides several convenience macros to reduce boilerplate:
//!
//! - [`extension_point!`] - Defines an extension point and its associated trait
//! - [`simple_plugin!`] - Creates a simple plugin with minimal boilerplate
//! - [`register_hook!`] - Registers a hook with the hook registry
//!
//! Note that [`register_hook!`] is not needed if you generate your plugin with [`simple_plugin!`].
//!
//! ## Advanced Usage
//!
//! For more complex scenarios, you can implement the [`Plugin`] trait directly,
//! allowing for more customized plugin behavior and state management.

#![warn(missing_docs)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::pedantic)]
#![warn(clippy::all)]
#![warn(clippy::empty_docs)]

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub mod error;
pub mod hook;
pub mod macros;

use tracing::{error, warn};

use self::error::{PluginError, PluginResult};
use self::hook::{ExtensionPoint, HookRegistry};

/// Plugin identifier type.
///
/// Every plugin must have a unique identifier.
///
/// # Examples
///
/// ```
/// let id: steckrs::PluginID = "hello_world_plugin";
/// ```
pub type PluginID = &'static str;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PluginIDOwned {
    inner: String,
}

impl From<PluginID> for PluginIDOwned {
    fn from(value: PluginID) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<PluginIDOwned> for PluginID {
    fn from(value: PluginIDOwned) -> Self {
        value.inner.leak::<'static>()
    }
}

impl std::fmt::Display for PluginIDOwned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.inner, f)
    }
}

/// Plugin trait that must be implemented by all plugins.
///
/// This trait defines the interface for plugin lifecycle management,
/// including loading, enabling, disabling, and unloading operations.
///
/// # Macros
///
/// Most users will find the [`simple_plugin!`] macro sufficient.
///
///
/// ```
/// # use steckrs::{extension_point, simple_plugin, PluginManager};
/// # extension_point!(
/// #     GreeterExtension: GreeterTrait;
/// #     fn greet(&self, name: &str) -> String;
/// # );
/// #
/// # struct EnglishGreeter;
/// # impl GreeterTrait for EnglishGreeter {
/// #     fn greet(&self, name: &str) -> String {
/// #         format!("Hello, {}!", name)
/// #     }
/// # }
/// #
/// simple_plugin!(
///     HelloPlugin,
///     "hello_plugin",
///     "A simple greeting plugin",
///     hooks: [(GreeterExtension, EnglishGreeter)]
/// );
/// ```
///
/// # Examples
///
/// ```
/// use steckrs::{Plugin, error::PluginResult, hook::HookRegistry};
///
/// #[derive(Debug)]
/// struct MyPlugin {
///     enabled: bool,
/// }
///
/// impl Plugin for MyPlugin {
///     fn id(&self) -> steckrs::PluginID {
///         // recommendation: add an associated constant for the ID
///         "my_plugin"
///     }
///
///     fn description(&self) -> &str {
///         // recommendation: add an associated constant for the DESCRIPTION
///         "A custom plugin implementation"
///     }
///
///     fn is_enabled(&self) -> bool {
///         self.enabled
///     }
///
///     fn enable(&mut self) {
///         self.enabled = true;
///     }
///
///     fn disable(&mut self) {
///         self.enabled = false;
///     }
///
///     fn register_hooks(&self, registry: &mut HookRegistry) -> PluginResult<()> {
///         // Register hooks here
///         Ok(())
///     }
///
///     // optionally define on_load and on_unload
/// }
/// ```
pub trait Plugin: Any + Send + Sync + Debug {
    /// Returns the unique identifier for this plugin.
    ///
    /// The ID must be unique across all loaded plugins.
    fn id(&self) -> PluginID;

    /// Returns a human-readable description of the plugin.
    fn description(&self) -> &str;

    /// Returns whether the plugin is currently enabled.
    fn is_enabled(&self) -> bool;

    /// Enables the plugin, allowing its hooks to be used.
    fn enable(&mut self);

    /// Disables the plugin, preventing its hooks from being used.
    fn disable(&mut self);

    /// Registers this plugin's [Hooks](crate::hook::Hook) with the [`HookRegistry`].
    ///
    /// This method is called during plugin loading, and should register
    /// all hooks that the plugin provides.
    ///
    /// # Errors
    ///
    /// Returns a `PluginError` if hook registration fails./
    fn register_hooks(&self, registry: &mut HookRegistry) -> PluginResult<()>;

    /// Called when the plugin is loaded.
    ///
    /// Provides an opportunity to perform initialization that should happen
    /// when the plugin is first loaded, before hooks are used.
    ///
    /// This function is always called after [`register_hooks`](Plugin::register_hooks).
    ///
    /// # Errors
    ///
    /// Returns a [`PluginError`] if loading fails.
    fn on_load(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Called when the plugin is unloaded.
    ///
    /// Provides an opportunity to perform cleanup before the plugin is removed.
    ///
    /// # Errors
    ///
    /// Returns a [`PluginError`] if the unloading cleanup fails.
    fn on_unload(&mut self) -> PluginResult<()> {
        Ok(())
    }
}

/// Manages plugin loading, execution, and lifecycle.
///
/// The [`PluginManager`] is the core component of the steckrs plugin system,
/// responsible for:
/// - Loading and unloading plugins
/// - Enabling and disabling plugins
/// - Maintaining the hook registry
/// - Tracking loaded plugins
///
/// # Examples
///
/// ```
/// use steckrs::{PluginManager, simple_plugin, extension_point};
///
/// // Define extension point
/// extension_point!(
///     ExampleExt: ExampleTrait;
///     fn do_something(&self) -> &'static str;
/// );
///
/// // Define plugin
/// simple_plugin!(
///     ExamplePlugin,
///     "example_plugin",
///     "An example plugin",
///     hooks: [(ExampleExt, ExampleHook)]
/// );
///
/// // Hook implementation
/// struct ExampleHook;
/// impl ExampleTrait for ExampleHook {
///     fn do_something(&self) -> &'static str {
///         "I did something!"
///     }
/// }
///
/// // Plugin management
/// let mut manager = PluginManager::new();
/// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
/// manager.enable_plugin(ExamplePlugin::ID).unwrap();
///
/// // Use plugin hooks
/// // Get all enabled hooks (plugins could be disabled)
/// let hooks = manager.get_enabled_hooks_by_ep::<ExampleExt>();
/// for (_id, hook) in hooks {
///     assert_eq!(hook.inner().do_something(), "I did something!");
/// }
/// ```
#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: HashMap<PluginID, Box<dyn Plugin>>,
    hook_registry: HookRegistry,
}

impl PluginManager {
    /// Creates a new empty plugin manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::PluginManager;
    ///
    /// let manager = PluginManager::new();
    /// assert_eq!(manager.plugin_ids().len(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            hook_registry: HookRegistry::new(),
        }
    }

    /// Creates a new plugin manager with an existing hook registry.
    ///
    /// This allows sharing a hook registry between multiple plugin managers,
    /// which can be useful for complex applications.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, hook::HookRegistry};
    ///
    /// let registry = HookRegistry::new();
    /// let manager = PluginManager::with_registry(registry);
    /// ```
    #[must_use]
    pub fn with_registry(hook_registry: HookRegistry) -> Self {
        Self {
            plugins: HashMap::new(),
            hook_registry,
        }
    }

    /// Returns a reference to the hook registry.
    ///
    /// The hook registry contains all registered hooks from loaded plugins.
    #[must_use]
    pub fn hook_registry(&self) -> &HookRegistry {
        &self.hook_registry
    }

    /// Returns a mutable reference to the hook registry.
    ///
    /// This can be used to directly manipulate the hook registry if needed.
    #[must_use]
    pub fn hook_registry_mut(&mut self) -> &mut HookRegistry {
        &mut self.hook_registry
    }

    /// Loads a plugin into the plugin manager.
    ///
    /// This will:
    /// 1. Register the plugin's hooks in the hook registry
    /// 2. Call the plugin's `on_load` method
    /// 3. Store the plugin in the manager
    ///
    /// # Errors
    ///
    /// Returns a `PluginError` if:
    /// - A plugin with the same ID is already loaded
    /// - The plugin's [`on_load`](Plugin::register_hooks) method fails
    /// - The plugin's [`on_load`](Plugin::on_load) method fails
    ///
    /// If any of the steps fail, this function will try to unload the half-loaded plugin again,
    /// using [`unload_plugin`](Self::unload_plugin).
    ///
    /// # Panics
    ///
    /// If loading of the plugin and then unloading the half-loaded plugin both fail, this function
    /// will panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     ExamplePlugin,
    ///     "example_plugin",
    ///     "An example plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
    /// assert!(manager.get_plugin("example_plugin").is_some());
    /// ```
    pub fn load_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> PluginResult<()> {
        let id = plugin.id();
        if self.plugins.contains_key(id) {
            return Err(error::PluginError::AlreadyLoaded(id.into()));
        }

        // register the hooks
        if let Err(e) = plugin.register_hooks(self.hook_registry_mut()) {
            self.handle_error_during_load(&e, id);
            return Err(e);
        }
        // Load the plugin
        if let Err(e) = plugin.on_load() {
            self.handle_error_during_load(&e, id);
            return Err(e);
        }

        // Store the plugin
        self.plugins.insert(id, plugin);

        Ok(())
    }

    /// Internal helper to handle errors during plugin loading.
    ///
    /// If a plugin fails during loading, this will attempt to clean up
    /// by unloading the plugin.
    fn handle_error_during_load(&mut self, e: &PluginError, plugin_id: PluginID) {
        error!("Could not register hooks of plugin {plugin_id}: {e}");
        warn!("Trying to unload the plugin again... Will crash if this fails");
        self.unload_plugin(plugin_id)
            .expect("Could not unload bad plugin again");
    }

    /// Unloads a plugin by ID.
    ///
    /// This will:
    /// 1. Call the plugin's `on_unload` method for cleanup
    /// 2. Remove all hooks registered by the plugin
    /// 3. Remove the plugin from the manager
    ///
    /// # Errors
    ///
    /// Returns a [`PluginError`] if:
    /// - The plugin's [`on_unload`](Plugin::on_unload) method fails
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     ExamplePlugin,
    ///     "example_plugin",
    ///     "An example plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
    /// manager.unload_plugin("example_plugin").unwrap();
    /// assert!(manager.get_plugin("example_plugin").is_none());
    /// ```
    pub fn unload_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        if let Some(mut plugin) = self.plugins.remove(id) {
            // Call on_unload for cleanup
            plugin.on_unload()?;

            // Remove all hooks registered by this plugin
            self.hook_registry.deregister_hooks_for_plugin(id);
        }
        Ok(())
    }

    /// Gets a reference to a plugin by ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     ExamplePlugin,
    ///     "example_plugin",
    ///     "An example plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
    ///
    /// let plugin = manager.get_plugin("example_plugin");
    /// assert!(plugin.is_some());
    /// assert_eq!(plugin.unwrap().id(), "example_plugin");
    /// ```
    #[must_use]
    pub fn get_plugin(&self, id: PluginID) -> Option<&dyn Plugin> {
        self.plugins.get(id).map(std::convert::AsRef::as_ref)
    }

    /// Gets a mutable reference to a plugin by ID.
    ///
    /// This can be used to modify a plugin's state after it's been loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     ExamplePlugin,
    ///     "example_plugin",
    ///     "An example plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
    ///
    /// let plugin = manager.get_plugin_mut("example_plugin");
    /// assert!(plugin.is_some());
    /// ```
    #[must_use]
    pub fn get_plugin_mut(&mut self, id: PluginID) -> Option<&mut dyn Plugin> {
        self.plugins.get_mut(id).map(std::convert::AsMut::as_mut)
    }

    /// Gets all plugin IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     Plugin1,
    ///     "plugin1",
    ///     "First plugin",
    ///     hooks: []
    /// );
    ///
    /// simple_plugin!(
    ///     Plugin2,
    ///     "plugin2",
    ///     "Second plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(Plugin1::new())).unwrap();
    /// manager.load_plugin(Box::new(Plugin2::new())).unwrap();
    ///
    /// let ids = manager.plugin_ids();
    /// assert_eq!(ids.len(), 2);
    /// assert!(ids.contains(&"plugin1"));
    /// assert!(ids.contains(&"plugin2"));
    /// ```
    #[must_use]
    pub fn plugin_ids(&self) -> Vec<PluginID> {
        self.plugins.keys().copied().collect()
    }

    /// Gets all plugins.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     Plugin1,
    ///     "plugin1",
    ///     "First plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(Plugin1::new())).unwrap();
    ///
    /// let plugins = manager.plugins();
    /// assert_eq!(plugins.len(), 1);
    /// assert_eq!(plugins[0].id(), "plugin1");
    /// ```
    #[must_use]
    pub fn plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins
            .values()
            .map(std::convert::AsRef::as_ref)
            .collect()
    }

    /// Gets all enabled plugins.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     Plugin1,
    ///     "plugin1",
    ///     "First plugin",
    ///     hooks: []
    /// );
    ///
    /// simple_plugin!(
    ///     Plugin2,
    ///     "plugin2",
    ///     "Second plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(Plugin1::new())).unwrap();
    /// manager.load_plugin(Box::new(Plugin2::new())).unwrap();
    /// manager.enable_plugin("plugin1").unwrap();
    ///
    /// let enabled = manager.enabled_plugins();
    /// assert_eq!(enabled.len(), 1);
    /// assert_eq!(enabled[0].id(), "plugin1");
    /// ```
    #[must_use]
    pub fn enabled_plugins(&self) -> Vec<&dyn Plugin> {
        self.plugins
            .values()
            .filter(|p| p.is_enabled())
            .map(std::convert::AsRef::as_ref)
            .collect()
    }

    /// Quickly check if a [`Plugin`] with a specific [`PluginID`] is enabled.
    ///
    /// This will return [`None`] if the [`Plugin`] with that [`PluginID`] was not found, otherwise
    /// `Some(enabled)`, where `enabled` is gotten with [`Plugin::is_enabled`].
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     Plugin,
    ///     "plugin",
    ///     "Some plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(Plugin::new())).unwrap();
    /// manager.enable_plugin("plugin").unwrap();
    ///
    /// assert_eq!(manager.plugin_is_enabled("plugin"), Some(true));
    /// assert_eq!(manager.plugin_is_enabled("nope"), None);
    ///
    /// manager.disable_plugin("plugin").unwrap();
    ///
    /// assert_eq!(manager.plugin_is_enabled("plugin"), Some(false));
    /// ```
    #[inline]
    #[must_use]
    pub fn plugin_is_enabled(&self, id: PluginID) -> Option<bool> {
        Some(self.plugins.get(id)?.is_enabled())
    }

    /// Enables a plugin by ID.
    ///
    /// Note that plugins are disabled by default
    ///
    /// # Errors
    ///
    /// Returns a [`PluginError::NotFound`] if no plugin with the given ID is loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     ExamplePlugin,
    ///     "example_plugin",
    ///     "An example plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
    /// manager.enable_plugin("example_plugin").unwrap();
    ///
    /// let plugin = manager.get_plugin("example_plugin").unwrap();
    /// assert!(plugin.is_enabled());
    /// ```
    pub fn enable_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        match self.plugins.get_mut(id) {
            Some(plugin) => {
                plugin.enable();
                Ok(())
            }
            None => Err(error::PluginError::NotFound(id.into())),
        }
    }

    /// Disables a plugin by ID.
    ///
    /// # Errors
    ///
    /// Returns a [`PluginError::NotFound`] if no plugin with the given ID is loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{PluginManager, simple_plugin};
    ///
    /// simple_plugin!(
    ///     ExamplePlugin,
    ///     "example_plugin",
    ///     "An example plugin",
    ///     hooks: []
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(ExamplePlugin::new())).unwrap();
    /// manager.enable_plugin("example_plugin").unwrap();
    /// manager.disable_plugin("example_plugin").unwrap();
    ///
    /// let plugin = manager.get_plugin("example_plugin").unwrap();
    /// assert!(!plugin.is_enabled());
    /// ```
    pub fn disable_plugin(&mut self, id: PluginID) -> PluginResult<()> {
        match self.plugins.get_mut(id) {
            Some(plugin) => {
                plugin.disable();
                Ok(())
            }
            None => Err(error::PluginError::NotFound(id.into())),
        }
    }

    /// Gets all hooks of enabled [Plugins](Plugin) for a specific [`ExtensionPoint`] type.
    ///
    /// This method filters hooks by both extension point type and plugin enabled status,
    /// returning only hooks from enabled plugins.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The [`ExtensionPoint`] type
    ///
    /// # Returns
    ///
    /// A vector of tuples containing references to [`HookID`](crate::hook::HookID)s and hooks registered for the [`ExtensionPoint`]
    /// from enabled plugins.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, simple_plugin, PluginManager};
    ///
    /// extension_point!(
    ///     Logger: LoggerTrait;
    ///     fn log(&self, message: &str);
    /// );
    ///
    /// struct ConsoleLogger;
    /// impl LoggerTrait for ConsoleLogger {
    ///     fn log(&self, message: &str) {
    ///         // In a real implementation, this would print to console
    ///     }
    /// }
    ///
    /// simple_plugin!(
    ///     LoggerPlugin,
    ///     "logger_plugin",
    ///     "Basic logging plugin",
    ///     hooks: [(Logger, ConsoleLogger)]
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(LoggerPlugin::new())).unwrap();
    /// manager.enable_plugin(LoggerPlugin::ID).unwrap();
    ///
    /// // Get all enabled hooks for the Logger extension point
    /// let hooks = manager.get_enabled_hooks_by_ep::<Logger>();
    /// assert_eq!(hooks.len(), 1);
    ///
    /// // Use the hook
    /// for (id, hook) in hooks {
    ///     assert_eq!(id.plugin_id, "logger_plugin");
    ///     hook.inner().log("Hello from logger!");
    /// }
    /// ```
    #[must_use]
    pub fn get_enabled_hooks_by_ep<E: ExtensionPoint>(
        &self,
    ) -> Vec<(&hook::HookID, &hook::Hook<E>)> {
        self.hook_registry()
            .get_by_extension_point()
            .into_iter()
            .filter(|(id, _hook)| {
                if let Some(plugin) = self.plugins.get(id.plugin_id) {
                    plugin.is_enabled()
                } else {
                    false
                }
            })
            .collect()
    }

    /// Gets all mutable hooks of enabled [Plugins](Plugin) for a specific [`ExtensionPoint`] type.
    ///
    /// This method filters hooks by both extension point type and plugin enabled status,
    /// returning only hooks from enabled plugins.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The [`ExtensionPoint`] type
    ///
    /// # Returns
    ///
    /// A vector of tuples containing mutable references to [`HookID`](crate::hook::HookID)s and hooks registered for the [`ExtensionPoint`]
    /// from enabled plugins.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, simple_plugin, PluginManager};
    ///
    /// extension_point!(
    ///     Logger: LoggerTrait;
    ///     fn log(&self, message: &str);
    /// );
    ///
    /// struct ConsoleLogger;
    /// impl LoggerTrait for ConsoleLogger {
    ///     fn log(&self, message: &str) {
    ///         // In a real implementation, this would print to console
    ///     }
    /// }
    ///
    /// simple_plugin!(
    ///     LoggerPlugin,
    ///     "logger_plugin",
    ///     "Basic logging plugin",
    ///     hooks: [(Logger, ConsoleLogger)]
    /// );
    ///
    /// let mut manager = PluginManager::new();
    /// manager.load_plugin(Box::new(LoggerPlugin::new())).unwrap();
    /// manager.enable_plugin(LoggerPlugin::ID).unwrap();
    ///
    /// // Get all enabled hooks for the Logger extension point
    /// let hooks = manager.get_enabled_hooks_by_ep::<Logger>();
    /// assert_eq!(hooks.len(), 1);
    ///
    /// // Use the hook
    /// for (id, hook) in hooks {
    ///     assert_eq!(id.plugin_id, "logger_plugin");
    ///     hook.inner().log("Hello from logger!");
    /// }
    /// ```
    #[must_use]
    pub fn get_enabled_hooks_by_ep_mut<E: ExtensionPoint>(
        &mut self,
    ) -> Vec<(&hook::HookID, &mut hook::Hook<E>)> {
        let enabled_ids: Vec<PluginID> = self
            .plugins
            .iter()
            .filter_map(|(id, plug)| if plug.is_enabled() { Some(*id) } else { None })
            .collect();
        self.hook_registry_mut()
            .get_by_extension_point_mut()
            .into_iter()
            .filter(|(id, _hook)| enabled_ids.contains(&id.plugin_id))
            .collect()
    }
}
