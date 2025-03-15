//! # Macros
//!
//! This module provides convenience macros for defining extension points,
//! creating plugins, and working with hooks.
//!
//! ## Overview
//!
//! - [`extension_point!`](crate::extension_point): Defines a new extension point and its associated trait
//! - [`simple_plugin!`](crate::simple_plugin): Creates a simple plugin implementation with minimal boilerplate
//! - [`register_hook!`](crate::register_hook): Registers a hook with a hook registry
//!
//! These macros reduce the amount of boilerplate code needed to work with the
//! steckrs plugin system, making it easier to define and use plugins.

/// Defines a new [`ExtensionPoint`](crate::hook::ExtensionPoint) and its associated trait.
///
/// This macro generates:
/// - An extension point struct
/// - A trait for the extension point
/// - An implementation of the [`ExtensionPoint`](crate::hook::ExtensionPoint) trait for the struct
///
/// # Parameters
///
/// - `$name_meta`: Attributes for the extension point, like documentation and derives
/// - `$name`: The name of the extension point struct
/// - `$trait_meta`: Attributes for the trait, like documentation and derives
/// - `$trait_name`: The name of the trait that hooks will implement
/// - `$($fn_sig:tt)*`: The function signatures for the trait
///
/// # Examples
///
/// ```
/// use steckrs::{extension_point,Plugin};
/// use steckrs::hook::{Hook, ExtensionPoint};
///
/// // Define an extension point with a single method
/// extension_point!(
///     Logger: LoggerTrait; // Name of EP, Name of it's trait
///     fn log(&self, message: &str); // the methods the trait of the EP implements
/// );
///
/// // Define an extension point with multiple methods
/// extension_point!(
///     /// struct documentation for Formatter
///     Formatter:
///     /// trait documentation for FormatterTrait
///     FormatterTrait;
///     /// Function documentation for format
///     fn format(&self, text: &str) -> String;
///     fn supports_format(&self, format_name: &str) -> bool;
/// );
///
/// // Implement the trait for a concrete type
/// struct ConsoleLogger;
/// impl LoggerTrait for ConsoleLogger {
///     fn log(&self, message: &str) {
///         println!("Log: {}", message);
///     }
/// }
///
///
/// let hook = Hook::<Logger>::new(Box::new(ConsoleLogger), "myhook");
/// hook.inner().log("Hello from hook!");
/// ```
#[macro_export]
macro_rules! extension_point {
    (
    $(#[$name_meta:meta])*
    $name:ident:
    $(#[$trait_meta:meta])*
    $trait_name:ident;
        $($trait_item:tt)*
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name: Send + Sync  {
            $(
                $trait_item
            )*
        }

        $(#[$name_meta])*
        #[derive(Debug, Ord, Eq, PartialEq, PartialOrd, Copy, Clone, Hash)]
        pub struct $name;

        impl $crate::hook::ExtensionPoint for $name {
            type HookTrait = dyn $trait_name;
        }
    };
}

/// Creates a simple [Plugin](crate::Plugin) with a specified set of hooks.
///
/// This macro generates a plugin struct with the following features:
/// - Implements the `Plugin` trait
/// - Has a static `ID` constant
/// - Provides a `new()` method
/// - Registers the specified hooks
///
/// # Parameters
///
/// - `$plugin_meta`: Attributes for the plugin, like documentation and derives
/// - `$name`: The name of the plugin struct
/// - `$id`: The unique ID of the plugin (as a string literal)
/// - `$description`: A description of the plugin (as a string literal)
/// - `hooks: [($ext_point:ty, $hook_impl:ty)]`: A list of extension point and hook implementation pairs
///
/// # Examples
///
/// ```
/// use steckrs::{extension_point, simple_plugin, Plugin};
///
/// // Define an extension point
/// extension_point!(
///     Greeter: GreeterTrait;
///     fn greet(&self, name: &str) -> String;
/// );
///
/// // Implement the extension point
/// struct FormalGreeter;
/// impl GreeterTrait for FormalGreeter {
///     fn greet(&self, name: &str) -> String {
///         format!("Good day, {}!", name)
///     }
/// }
///
/// // Create a plugin with the implementation
/// simple_plugin!(
///     FormalGreetingPlugin,
///     "formal_greeting_plugin",
///     "A plugin that provides formal greetings",
///     hooks: [(Greeter, FormalGreeter)]
/// );
///
/// // Create an instance of the plugin
/// let plugin = FormalGreetingPlugin::new();
/// assert_eq!(plugin.id(), "formal_greeting_plugin");
/// assert_eq!(plugin.description(), "A plugin that provides formal greetings");
///
/// // Create a plugin with multiple hooks
/// struct CasualGreeter;
/// impl GreeterTrait for CasualGreeter {
///     fn greet(&self, name: &str) -> String {
///         format!("Hey, {}!", name)
///     }
/// }
///
/// extension_point!(
///     Farewell: FarewellTrait;
///     fn say_goodbye(&self, name: &str) -> String;
/// );
///
/// struct SimpleFarewell;
/// impl FarewellTrait for SimpleFarewell {
///     fn say_goodbye(&self, name: &str) -> String {
///         format!("Goodbye, {}!", name)
///     }
/// }
///
/// simple_plugin!(
///     /// Document your plugin like this if you want
///     GreetingPlugin,
///     "greeting_plugin",
///     "A plugin with multiple greeting implementations",
///     hooks: [
///         (Greeter, CasualGreeter, "casual"), // if there are two hooks for an extension point
///         (Greeter, FormalGreeter, "formal"), // you need to add a discriminant
///         (Farewell, SimpleFarewell)
///     ]
/// );
/// ```
///
/// # Panics
///
/// The generated [`register_hooks`](crate::Plugin::register_hooks) method may panic if hook registration fails.
#[macro_export]
macro_rules! simple_plugin {
        (
        $(#[$plugin_meta:meta])*
        $plugin_name:ident,
        $plugin_id:expr,
        $description:expr,
        hooks: [$(($extension_point:ident, $hook_impl:expr $(, $discrim:expr)?)),* $(,)?]) => {
        $(#[$plugin_meta])*
        #[derive(Debug)]
        pub struct $plugin_name {
            enabled: bool,
        }

        impl $plugin_name {
            #[doc = concat!("ID of ", stringify!($plugin_name))]
            pub const ID: $crate::PluginID = $plugin_id;
            #[doc = concat!("Description of ", stringify!($plugin_name))]
            pub const DESCRIPTION: &'static str = $description;

            #[doc = concat!("Creates a new ", stringify!($plugin_name))]
            pub fn new() -> Self {
                Self { enabled: false }
            }
        }

        impl $crate::Plugin for $plugin_name {
            fn id(&self) -> $crate::PluginID {
                Self::ID
            }

            fn description(&self) -> &str {
                Self::DESCRIPTION
            }

            fn is_enabled(&self) -> bool {
                self.enabled
            }

            fn enable(&mut self) {
                self.enabled = true;
            }

            fn disable(&mut self) {
                self.enabled = false;
            }


            fn register_hooks(&self, registry: &mut $crate::hook::HookRegistry) -> $crate::error::PluginResult<()> {
                $(
                    $crate::register_hook!(registry, Self::ID, $extension_point, $hook_impl $(, $discrim)?);
                )*

                Ok(())
            }
        }
    };
}

/// Registers a [`Hook`](crate::hook::Hook) with a [`HookRegistry`](crate::hook::HookRegistry).
///
/// This macro simplifies the process of creating and registering a hook
/// with a hook registry.
///
/// # Parameters
///
/// - `$registry`: The hook registry to register with
/// - `$plugin_id`: The ID of the plugin
/// - `$ext_point_id`: The ID of the extension point
/// - `$discriminator`: An optional discriminator (or `None`)
/// - `$hook_trait`: The trait type for the hook
/// - `$hook_impl`: The implementation type for the hook
///
/// # Panics
///
/// This macro will panic if [`crate::hook::HookRegistry::register`] fails.
///
/// # Examples
///
/// ```
/// use steckrs::{
///     extension_point,
///     hook::{ExtensionPoint, HookRegistry},
///     register_hook,
/// };
///
/// extension_point!(
///     Calculator: CalculatorTrait;
///     fn add(&self, a: i32, b: i32) -> i32;
/// );
///
/// struct SimpleCalculator;
/// impl CalculatorTrait for SimpleCalculator {
///     fn add(&self, a: i32, b: i32) -> i32 {
///         a + b
///     }
/// }
///
/// let mut registry = HookRegistry::new();
///
/// // Register a hook
/// register_hook!(
///     registry,
///     "calculator_plugin", // a plugin id would be better
///     Calculator,
///     SimpleCalculator
/// );
///
/// // Use the registered hook
/// let hooks = registry.get_by_extension_point::<Calculator>();
/// assert_eq!(hooks.len(), 1);
/// assert_eq!(hooks[0].1.inner().add(2, 3), 5);
/// ```
#[macro_export]
macro_rules! register_hook {
    ($registry_mut:expr, $plugin_id:expr, $extension_point:ident, $hook:expr) => {
        $registry_mut
            .register(
                &$crate::hook::HookID::new(
                    $plugin_id,
                    <$extension_point as $crate::hook::ExtensionPoint>::id(),
                    None,
                ),
                $crate::hook::Hook::<$extension_point>::new(
                    Box::new($hook),
                    std::any::type_name_of_val(&$hook),
                ),
            )
            .expect("could not register hook")
    };
    ($registry_mut:expr, $plugin_id:expr, $extension_point:ident, $hook:expr, $discriminator:expr) => {
        $registry_mut
            .register(
                &$crate::hook::HookID::new(
                    $plugin_id,
                    <$extension_point as $crate::hook::ExtensionPoint>::id(),
                    Some($discriminator),
                ),
                $crate::hook::Hook::<$extension_point>::new(
                    Box::new($hook),
                    std::any::type_name_of_val(&$hook),
                ),
            )
            .expect("could not register hook")
    };
}
