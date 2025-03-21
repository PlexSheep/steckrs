//! # Hook System
//!
//! This module provides the core type-safe hook system for [`steckrs`](crate).
//!
//! The hook system enables plugins to provide implementations for extension points,
//! which are then stored in a registry and can be retrieved and executed by the
//! application. The key components are:
//!
//! - [`ExtensionPoint`]: A trait that defines an interface that plugins can implement
//! - [`Hook`]: A wrapper for a specific implementation of an extension point
//! - [`HookID`]: A unique identifier for a specific hook implementation
//! - [`HookRegistry`]: A registry that stores and manages hooks
//!
//! The hook system uses Rust's type system to provide compile-time safety for
//! extension points and hooks, while still allowing for dynamic dispatch at runtime.
//!
//! The hook system could be used entirely without [Plugins](crate::Plugin) if whished, except for
//! needing a [`PluginID`] for the [`HookID`]s.
//!
//! ## Example
//!
//! ```rust
//! use steckrs::{extension_point, hook::{ExtensionPoint, Hook, HookRegistry}};
//!
//! // Define an extension point
//! extension_point!(
//!     Greeter: GreeterTrait;
//!     fn greet(&self, name: &str) -> String;
//! );
//!
//! // Implement the extension point
//! struct EnglishGreeter;
//! impl GreeterTrait for EnglishGreeter {
//!     fn greet(&self, name: &str) -> String {
//!         format!("Hello, {}!", name)
//!     }
//! }
//!
//! // Create a hook
//! let hook = Hook::<Greeter>::new(Box::new(EnglishGreeter), "myhook");
//!
//! // Create a hook ID
//! let hook_id = steckrs::hook::HookID::new(
//!     "example_plugin",
//!     Greeter::id(),
//!     None,
//! );
//!
//! // Register the hook
//! let mut registry = HookRegistry::new();
//! registry.register(&hook_id, hook).unwrap();
//!
//! // Get hooks by extension point
//! let hooks = registry.get_by_extension_point::<Greeter>();
//! assert_eq!(hooks.len(), 1);
//! assert_eq!(hooks[0].1.inner().greet("World"), "Hello, World!");
//! ```

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::error::{HookError, HookResult};
use crate::PluginID;

/// Type identifier for extension points.
///
/// This is used to uniquely identify different extension point types
/// in the hook registry.
pub type ExtensionPointID = std::any::TypeId;

/// Unique identifier for a specific hook instance.
///
/// A `HookID` consists of:
/// - The [`PluginID`] of the [`Plugin`](crate::Plugin) that owns the hook
/// - The [`ExtensionPointID`] of the [`ExtensionPoint`] that the hook implements
/// - An optional discriminator string to allow multiple hooks for the same
///   extension point from the same plugin
///
/// # Examples
///
/// ```
/// use steckrs::hook::{HookID, ExtensionPoint};
///
/// // Define a trivial extension point
/// #[derive(Ord, Eq, PartialOrd, PartialEq)]
/// struct MyExtPoint;
/// impl ExtensionPoint for MyExtPoint {
///     type HookTrait = dyn Send + Sync; // a real point would have it's own trait
/// }
///
/// // Create a hook ID with no discriminator
/// let simple_id = HookID::new(
///     "my_plugin",
///     MyExtPoint::id(),
///     None,
/// );
///
/// // Create a hook ID with a discriminator
/// let with_discriminator = HookID::new(
///     "my_plugin",
///     MyExtPoint::id(),
///     Some("variant1"),
/// );
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HookID {
    /// Plugin that owns this hook
    pub plugin_id: PluginID,
    /// Extension point this hook implements
    pub extension_point_id: ExtensionPointID,
    /// Optional discriminator if a plugin registers multiple hooks for same extension point
    pub discriminator: Option<String>,
}

impl HookID {
    /// Creates a new hook ID.
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: The ID of the plugin that owns this hook
    /// - `extension_point_id`: The type ID of the extension point this hook implements
    /// - `discriminator`: An optional string to distinguish between multiple hooks of the same type
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::hook::{HookID, ExtensionPoint};
    ///
    /// // Define a trivial extension point
    /// #[derive(Ord, Eq, PartialOrd, PartialEq)]
    /// struct LoggerExtPoint;
    /// impl ExtensionPoint for LoggerExtPoint {
    ///     type HookTrait = dyn Send + Sync;
    /// }
    ///
    /// // Create a hook ID
    /// let hook_id = HookID::new(
    ///     "logging_plugin",
    ///     LoggerExtPoint::id(),
    ///     Some("file_logger"),
    /// );
    ///
    /// assert_eq!(hook_id.plugin_id, "logging_plugin");
    /// assert_eq!(hook_id.extension_point_id, LoggerExtPoint::id());
    /// assert_eq!(hook_id.discriminator, Some("file_logger".into()));
    /// ```
    #[must_use]
    pub fn new(
        plugin_id: PluginID,
        extension_point_id: ExtensionPointID,
        discriminator: Option<&'static str>,
    ) -> Self {
        HookID {
            plugin_id,
            extension_point_id,
            discriminator: discriminator.map(std::convert::Into::into),
        }
    }
}

/// Defines an extension point where plugins can hook into the application.
///
/// An extension point is essentially a contract (trait) that plugins can implement.
/// It provides a type-safe way for plugins to extend the application's functionality.
///
/// # Type Parameters
///
/// - `HookTrait`: The trait that hooks will implement
///
/// # Examples
///
/// Using the [`extension_point!`](crate::extension_point) macro is the recommended way to define extension points:
///
/// ```
/// use steckrs::extension_point;
///
/// extension_point!(
///     ConfigLoader: ConfigLoaderTrait;
///     fn load_config(&self, path: &str) -> Result<String, String>;
///     fn supports_format(&self, format: &str) -> bool;
/// );
///
/// // Now `ConfigLoader` is an extension point that plugins can implement
/// ```
///
/// Manually implementing the trait:
///
/// ```
/// use steckrs::hook::ExtensionPoint;
///
/// // Define a trait for the extension point
/// pub trait FormatterFunctions: Send + Sync {
///     fn format(&self, text: &str) -> String;
/// }
///
/// // Define the extension point
/// #[derive(Ord, Eq, PartialOrd, PartialEq)]
/// pub struct TextFormatter;
///
/// impl ExtensionPoint for TextFormatter {
///     type HookTrait = dyn FormatterFunctions;
/// }
/// ```
pub trait ExtensionPoint: Eq + Ord + 'static {
    /// The trait that hooks implement for this extension point
    type HookTrait: ?Sized + Send + Sync + 'static;

    /// Returns a unique identifier for this extension point type.
    ///
    /// By default, this uses Rust's [`TypeId`](std::any::TypeId)
    /// system to generate a unique ID based on the extension point's type.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::hook::ExtensionPoint;
    ///
    /// #[derive(Ord, Eq, PartialOrd, PartialEq)]
    /// struct MyExtPoint;
    /// impl ExtensionPoint for MyExtPoint {
    ///     type HookTrait = dyn Send + Sync;
    /// }
    ///
    /// let id = MyExtPoint::id();
    /// ```
    #[must_use]
    fn id() -> ExtensionPointID {
        std::any::TypeId::of::<Self>()
    }

    /// Returns the human-readable name of this extension point.
    ///
    /// By default, this returns the type name of the extension point.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::hook::ExtensionPoint;
    ///
    /// #[derive(Ord, Eq, PartialOrd, PartialEq)]
    /// struct CustomValidator;
    /// impl ExtensionPoint for CustomValidator {
    ///     type HookTrait = dyn Send + Sync;
    /// }
    ///
    /// let name = CustomValidator::name();
    /// assert!(name.contains("CustomValidator"));
    /// ```
    #[must_use]
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A wrapper around a hook trait object for a specific extension point.
///
/// The [`Hook`] struct provides type safety for extension point implementations,
/// allowing the application to store and retrieve hook implementations with
/// the correct type information.
///
/// # Type Parameters
///
/// - `E`: The [`ExtensionPoint`] type
///
/// # Examples
///
/// ```
/// use steckrs::{extension_point, hook::Hook};
///
/// extension_point!(
///     Logger: LoggerTrait;
///     fn log(&self, message: &str);
/// );
///
/// struct ConsoleLogger;
/// impl LoggerTrait for ConsoleLogger {
///     fn log(&self, message: &str) {
///         println!("Log: {}", message);
///     }
/// }
///
/// // Create a hook with the logger implementation
/// let hook = Hook::<Logger>::new(Box::new(ConsoleLogger), "myhook");
///
/// // Use the hook
/// hook.inner().log("Hello from hook!");
/// ```
#[derive(Debug)]
pub struct Hook<E: ExtensionPoint> {
    /// The actual hook trait object
    inner: Box<E::HookTrait>,
    hook_t: PhantomData<E::HookTrait>,
    name: &'static str,
}

impl<E: ExtensionPoint> PartialEq for Hook<E> {
    fn eq(&self, other: &Self) -> bool {
        self.hook_t == other.hook_t && self.name == other.name
    }
}

impl<E: ExtensionPoint> Eq for Hook<E> {}

impl<E: ExtensionPoint> PartialOrd for Hook<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: ExtensionPoint> Ord for Hook<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(other.name)
    }
}

impl<E: ExtensionPoint> Hook<E> {
    /// Creates a new hook with the given trait implementation.
    ///
    /// # Parameters
    ///
    /// - `hook`: A boxed trait object implementing the extension point's trait
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::Hook};
    ///
    /// extension_point!(
    ///     Validator: ValidatorTrait;
    ///     fn validate(&self, input: &str) -> bool;
    /// );
    ///
    /// struct LengthValidator;
    /// impl ValidatorTrait for LengthValidator {
    ///     fn validate(&self, input: &str) -> bool {
    ///         input.len() > 5
    ///     }
    /// }
    ///
    /// let hook = Hook::<Validator>::new(Box::new(LengthValidator), "myhook");
    /// ```
    #[must_use]
    pub fn new(hook: Box<E::HookTrait>, name: PluginID) -> Self {
        Hook {
            inner: hook,
            hook_t: PhantomData,
            name,
        }
    }

    /// Returns a reference to the inner trait implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::Hook};
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
    /// let hook = Hook::<Calculator>::new(Box::new(SimpleCalculator), "myhook");
    /// assert_eq!(hook.inner().add(2, 3), 5);
    /// ```
    #[must_use]
    pub fn inner(&self) -> &E::HookTrait {
        &self.inner
    }

    /// Returns a mutable reference to the inner trait implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::Hook};
    ///
    /// extension_point!(
    ///     Calculator: CalculatorTrait;
    ///     fn add(&mut self, a: i32, b: i32) -> i32;
    /// );
    ///
    /// struct SimpleCalculator;
    /// impl CalculatorTrait for SimpleCalculator {
    ///     fn add(&mut self, a: i32, b: i32) -> i32 {
    ///         a + b
    ///     }
    /// }
    ///
    /// let mut hook = Hook::<Calculator>::new(Box::new(SimpleCalculator), "myhook");
    /// assert_eq!(hook.inner_mut().add(2, 3), 5);
    /// ```
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut E::HookTrait {
        &mut self.inner
    }

    /// Get the human readable name for this hook
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// A type-erased hook that can be stored in a [`HookRegistry`].
///
/// [`BoxedHook`] acts as a type-erased container for [Hooks](Hook) of any [`ExtensionPoint`] type,
/// allowing the [`HookRegistry`] to store hooks for different extension points in a
/// single collection.
///
/// # Examples
///
/// This type is mainly used internally by the [`HookRegistry`], but here's how it works:
///
/// ```
/// use steckrs::{extension_point, hook::{BoxedHook, Hook}};
///
/// extension_point!(
///     Formatter: FormatterTrait;
///     fn format(&self, text: &str) -> String;
/// );
///
/// struct UppercaseFormatter;
/// impl FormatterTrait for UppercaseFormatter {
///     fn format(&self, text: &str) -> String {
///         text.to_uppercase()
///     }
/// }
///
/// // Create a typed hook
/// let hook = Hook::<Formatter>::new(Box::new(UppercaseFormatter), "myhook");
///
/// // Create a type-erased hook
/// let boxed_hook = BoxedHook::new(hook);
///
/// // Downcast back to the original type
/// let hook_ref = boxed_hook.downcast::<Formatter>().unwrap();
/// assert_eq!(hook_ref.inner().format("hello"), "HELLO");
/// ```
pub struct BoxedHook {
    /// The actual hook trait object, type-erased
    hook: Box<dyn Any + Send + Sync>,
    hook_name: &'static str,
    eid: ExtensionPointID,
}

impl PartialEq for BoxedHook {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for BoxedHook {}

impl PartialOrd for BoxedHook {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BoxedHook {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name().cmp(other.name())
    }
}

impl BoxedHook {
    /// Creates a new boxed hook from a typed hook.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type
    ///
    /// # Parameters
    ///
    /// - `hook`: The typed [`Hook`] to box
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{BoxedHook, Hook}};
    ///
    /// extension_point!(
    ///     Timer: TimerTrait;
    ///     fn get_time(&self) -> u64;
    /// );
    ///
    /// struct SystemTimer;
    /// impl TimerTrait for SystemTimer {
    ///     fn get_time(&self) -> u64 {
    ///         42 // Placeholder value
    ///     }
    /// }
    ///
    /// let hook = Hook::<Timer>::new(Box::new(SystemTimer), "myhook");
    /// let boxed_hook = BoxedHook::new(hook);
    /// ```
    #[must_use]
    pub fn new<E: ExtensionPoint>(hook: Hook<E>) -> Self {
        BoxedHook {
            hook_name: hook.name(),
            hook: Box::new(hook),
            eid: E::id(),
        }
    }

    /// Attempts to downcast the boxed hook to a specific [`Hook`] type.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type to downcast with
    ///
    /// # Returns
    ///
    /// - `Some(&Hook<E>)` if the hook is of the correct type
    /// - `None` if the hook is of a different type
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{BoxedHook, Hook, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Counter: CounterTrait;
    ///     fn count(&self) -> i32;
    /// );
    ///
    /// struct SimpleCounter;
    /// impl CounterTrait for SimpleCounter {
    ///     fn count(&self) -> i32 {
    ///         1
    ///     }
    /// }
    ///
    /// let hook = Hook::<Counter>::new(Box::new(SimpleCounter), "myhook");
    /// let boxed_hook = BoxedHook::new(hook);
    ///
    /// // Successful downcast
    /// let typed_hook = boxed_hook.downcast::<Counter>();
    /// assert!(typed_hook.is_some());
    /// assert_eq!(typed_hook.unwrap().inner().count(), 1);
    ///
    /// // Failed downcast (trying to downcast to wrong type)
    /// #[derive(Ord,Eq,PartialOrd,PartialEq)]
    /// struct OtherExtPoint;
    /// impl ExtensionPoint for OtherExtPoint {
    ///     type HookTrait = dyn Send + Sync;
    /// }
    /// let wrong_type = boxed_hook.downcast::<OtherExtPoint>();
    /// assert!(wrong_type.is_none());
    /// ```
    #[must_use]
    pub fn downcast<E: ExtensionPoint>(&self) -> Option<&Hook<E>> {
        self.hook.downcast_ref::<Hook<E>>()
    }

    /// Attempts to downcast the boxed hook to a specific mutable [`Hook`] type.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type to downcast with
    ///
    /// # Returns
    ///
    /// - `Some(&Hook<E>)` if the hook is of the correct type
    /// - `None` if the hook is of a different type
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{BoxedHook, Hook, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Counter: CounterTrait;
    ///     fn count(&mut self) -> i32;
    /// );
    ///
    /// struct SimpleCounter;
    /// impl CounterTrait for SimpleCounter {
    ///     fn count(&mut self) -> i32 {
    ///         1
    ///     }
    /// }
    ///
    /// let mut hook = Hook::<Counter>::new(Box::new(SimpleCounter), "myhook");
    /// let mut boxed_hook = BoxedHook::new(hook);
    ///
    /// // Successful downcast
    /// let mut typed_hook = boxed_hook.downcast_mut::<Counter>();
    /// assert!(typed_hook.is_some());
    /// assert_eq!(typed_hook.unwrap().inner_mut().count(), 1);
    ///
    /// // Failed downcast (trying to downcast to wrong type)
    /// #[derive(Ord,Eq,PartialOrd,PartialEq)]
    /// struct OtherExtPoint;
    /// impl ExtensionPoint for OtherExtPoint {
    ///     type HookTrait = dyn Send + Sync;
    /// }
    /// let wrong_type = boxed_hook.downcast::<OtherExtPoint>();
    /// assert!(wrong_type.is_none());
    /// ```
    #[must_use]
    pub fn downcast_mut<E: ExtensionPoint>(&mut self) -> Option<&mut Hook<E>> {
        self.hook.downcast_mut::<Hook<E>>()
    }

    /// Returns the [`ExtensionPointID`] for this hook.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{BoxedHook, Hook, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     MyExt: MyExtTrait;
    ///     fn do_something(&self) -> bool;
    /// );
    ///
    /// struct MyImpl;
    /// impl MyExtTrait for MyImpl {
    ///     fn do_something(&self) -> bool {
    ///         true
    ///     }
    /// }
    ///
    /// let hook = Hook::<MyExt>::new(Box::new(MyImpl), "myhook");
    /// let boxed_hook = BoxedHook::new(hook);
    ///
    /// assert_eq!(boxed_hook.eid(), MyExt::id());
    /// ```
    #[must_use]
    pub fn eid(&self) -> ExtensionPointID {
        self.eid
    }

    /// Get name of this hook
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.hook_name
    }
}

impl Debug for BoxedHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedHook").finish_non_exhaustive()
    }
}

/// Registry for storing and retrieving hooks.
///
/// The [`HookRegistry`] provides a central place to register, deregister, and
/// query hooks from different [Plugins](crate::Plugin), [Extension Points](ExtensionPoint) and
/// other qualities.
///
/// # Examples
///
/// ```
/// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
///
/// // Define extension points
/// extension_point!(
///     Validator: ValidatorTrait;
///     fn validate(&self, input: &str) -> bool;
/// );
///
/// // Implement extension points
/// struct MinLengthValidator;
/// impl ValidatorTrait for MinLengthValidator {
///     fn validate(&self, input: &str) -> bool {
///         input.len() >= 3
///     }
/// }
///
/// struct MaxLengthValidator;
/// impl ValidatorTrait for MaxLengthValidator {
///     fn validate(&self, input: &str) -> bool {
///         input.len() <= 10
///     }
/// }
///
/// // Create and register hooks
/// let mut registry = HookRegistry::new();
///
/// let min_hook = Hook::<Validator>::new(Box::new(MinLengthValidator), "minhook");
/// let min_id = HookID::new("plugin1", Validator::id(), Some("min_length"));
///
/// let max_hook = Hook::<Validator>::new(Box::new(MaxLengthValidator), "maxhook");
/// let max_id = HookID::new("plugin1", Validator::id(), Some("max_length"));
///
/// registry.register(&min_id, min_hook).unwrap();
/// registry.register(&max_id, max_hook).unwrap();
///
/// // Use the hooks
/// let hooks = registry.get_by_extension_point::<Validator>();
/// assert_eq!(hooks.len(), 2);
///
/// // Test both validators
/// let short = "ab";
/// let good = "jigglypuff";
/// let long = "this string is too long for the max validator";
///
/// dbg!(hooks[0].1.name()); // maxhook
/// dbg!(hooks[1].1.name()); // minhook
///
/// // Only the max len passes for the very short one
/// assert!(hooks[0].1.inner().validate(short));
/// assert!(!hooks[1].1.inner().validate(short));
///
/// // Both validators pass for good
/// assert!(hooks[0].1.inner().validate(good));
/// assert!(hooks[1].1.inner().validate(good));
///
/// // Min passes but max fails for long
/// assert!(!hooks[0].1.inner().validate(long));
/// assert!(hooks[1].1.inner().validate(long));
/// ```
#[derive(Debug, Default)]
pub struct HookRegistry {
    hooks: HashMap<ExtensionPointID, HashMap<HookID, BoxedHook>>,
}

impl HookRegistry {
    /// Creates a new empty hook registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::hook::HookRegistry;
    ///
    /// let registry = HookRegistry::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Registers a hook with the given [`HookID`].
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type
    ///
    /// # Parameters
    ///
    /// - `id`: The unique identifier for this hook
    /// - `hook`: The hook implementation to register
    ///
    /// # Errors
    ///
    /// Returns a [`HookError::AlreadyRegistered`] if a hook with the same ID is already registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Serializer: SerializerTrait;
    ///     fn serialize(&self, data: &str) -> Vec<u8>;
    /// );
    ///
    /// struct ByteSerializer;
    /// impl SerializerTrait for ByteSerializer {
    ///     fn serialize(&self, data: &str) -> Vec<u8> {
    ///         data.as_bytes().to_vec()
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let hook = Hook::<Serializer>::new(Box::new(ByteSerializer), "myhook");
    /// let id = HookID::new("byte_plugin", Serializer::id(), None);
    ///
    /// registry.register(&id, hook).unwrap();
    /// ```
    pub fn register<E: ExtensionPoint>(&mut self, id: &HookID, hook: Hook<E>) -> HookResult<()> {
        if self.exists(id) {
            return Err(HookError::AlreadyRegistered);
        }

        let boxed_hook = BoxedHook::new(hook);

        self.hooks
            .entry(E::id())
            .or_default()
            .insert(id.clone(), boxed_hook);

        Ok(())
    }

    /// Deregisters a hook by [`HookID`].
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the hook to deregister
    ///
    /// # Returns
    ///
    /// - `Some(BoxedHook)` if the hook was found and removed
    /// - `None` if no hook with the given ID was found
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Parser: ParserTrait;
    ///     fn parse(&self, input: &str) -> bool;
    /// );
    ///
    /// struct SimpleParser;
    /// impl ParserTrait for SimpleParser {
    ///     fn parse(&self, _: &str) -> bool {
    ///         true
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let hook = Hook::<Parser>::new(Box::new(SimpleParser), "myhook");
    /// let id = HookID::new("parser_plugin", Parser::id(), None);
    ///
    /// registry.register(&id, hook).unwrap();
    /// assert!(registry.exists(&id));
    ///
    /// let removed = registry.deregister(&id);
    /// assert!(removed.is_some());
    /// assert!(!registry.exists(&id));
    /// ```
    pub fn deregister(&mut self, id: &HookID) -> Option<BoxedHook> {
        let id = self.get_by_id(id)?.0.clone();
        if let Some(h) = self.hooks.get_mut(&id.extension_point_id) {
            h.remove(&id)
        } else {
            None
        }
    }

    /// Checks if a hook with the given [`HookID`] exists.
    ///
    /// # Parameters
    ///
    /// - `id`: The ID to check
    ///
    /// # Returns
    ///
    /// - `true` if a hook with the given ID exists
    /// - `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Handler: HandlerTrait;
    ///     fn handle(&self, input: &str) -> String;
    /// );
    ///
    /// struct EchoHandler;
    /// impl HandlerTrait for EchoHandler {
    ///     fn handle(&self, input: &str) -> String {
    ///         input.to_string()
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let hook = Hook::<Handler>::new(Box::new(EchoHandler), "myhook");
    /// let id = HookID::new("echo_plugin", Handler::id(), None);
    ///
    /// assert!(!registry.exists(&id));
    /// registry.register(&id, hook).unwrap();
    /// assert!(registry.exists(&id));
    /// ```
    #[must_use]
    pub fn exists(&self, id: &HookID) -> bool {
        self.get_by_id(id).is_some()
    }

    /// Gets a hook by [`HookID`] and extension point type.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the hook to get
    ///
    /// # Returns
    ///
    /// - `Some(&Hook<E>)` if the hook was found
    /// - `None` if no hook with the given ID was found for the extension point
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Encoder: EncoderTrait;
    ///     fn encode(&self, input: &str) -> Vec<u8>;
    /// );
    ///
    /// struct Base64Encoder;
    /// impl EncoderTrait for Base64Encoder {
    ///     fn encode(&self, input: &str) -> Vec<u8> {
    ///         input.as_bytes().to_vec() // Simplified for example
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let hook = Hook::<Encoder>::new(Box::new(Base64Encoder), "myhook");
    /// let id = HookID::new("encoder_plugin", Encoder::id(), None);
    ///
    /// registry.register(&id, hook).unwrap();
    ///
    /// let retrieved = registry.get::<Encoder>(&id);
    /// assert!(retrieved.is_some());
    /// assert_eq!(retrieved.unwrap().inner().encode("test").len(), 4);
    /// ```
    #[must_use]
    pub fn get<E: ExtensionPoint>(&self, id: &HookID) -> Option<&Hook<E>> {
        match self.hooks.get(&E::id()) {
            Some(hooks) => {
                let boxed_hook = hooks.get(id)?;
                boxed_hook.downcast()
            }
            None => None,
        }
    }

    /// Gets a mutable hook by [`HookID`] and extension point type.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the hook to get
    ///
    /// # Returns
    ///
    /// - `Some(&mut Hook<E>)` if the hook was found
    /// - `None` if no hook with the given ID was found for the extension point
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Encoder: EncoderTrait;
    ///     fn encode(&mut self, input: &str) -> Vec<u8>;
    /// );
    ///
    /// struct Base64Encoder;
    /// impl EncoderTrait for Base64Encoder {
    ///     fn encode(&mut self, input: &str) -> Vec<u8> {
    ///         input.as_bytes().to_vec() // Simplified for example
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let mut hook = Hook::<Encoder>::new(Box::new(Base64Encoder), "myhook");
    /// let id = HookID::new("encoder_plugin", Encoder::id(), None);
    ///
    /// registry.register(&id, hook).unwrap();
    ///
    /// let mut retrieved = registry.get_mut::<Encoder>(&id);
    /// assert!(retrieved.is_some());
    /// assert_eq!(retrieved.unwrap().inner_mut().encode("test").len(), 4);
    /// ```
    #[must_use]
    pub fn get_mut<E: ExtensionPoint>(&mut self, id: &HookID) -> Option<&mut Hook<E>> {
        match self.hooks.get_mut(&E::id()) {
            Some(hooks) => {
                let boxed_hook = hooks.get_mut(id)?;
                boxed_hook.downcast_mut()
            }
            None => None,
        }
    }

    /// Gets a hook by [`HookID`].
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the hook to get
    ///
    /// # Returns
    ///
    /// - `Some(&BoxedHook)` if the hook was found
    /// - `None` if no hook with the given ID was found
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Hasher: HasherTrait;
    ///     fn hash(&self, input: &str) -> u64;
    /// );
    ///
    /// struct SimpleHasher;
    /// impl HasherTrait for SimpleHasher {
    ///     fn hash(&self, input: &str) -> u64 {
    ///         input.len() as u64 // Simplified hash function
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let hook = Hook::<Hasher>::new(Box::new(SimpleHasher), "myhook");
    /// let id = HookID::new("hasher_plugin", Hasher::id(), None);
    ///
    /// registry.register(&id, hook).unwrap();
    ///
    /// let retrieved = registry.get_by_id(&id);
    /// assert!(retrieved.is_some());
    /// ```
    #[must_use]
    pub fn get_by_id(&self, id: &HookID) -> Option<(&HookID, &BoxedHook)> {
        self.get_by_filter(|(hid, _)| *hid == id).first().copied()
    }

    /// Gets a mutable hook by [`HookID`].
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the hook to get
    ///
    /// # Returns
    ///
    /// - `Some(&mut BoxedHook)` if the hook was found
    /// - `None` if no hook with the given ID was found
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Hasher: HasherTrait;
    ///     fn hash(&mut self, input: &str) -> u64;
    /// );
    ///
    /// struct SimpleHasher;
    /// impl HasherTrait for SimpleHasher {
    ///     fn hash(&mut self, input: &str) -> u64 {
    ///         input.len() as u64 // Simplified hash function
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let mut hook = Hook::<Hasher>::new(Box::new(SimpleHasher), "myhook");
    /// let id = HookID::new("hasher_plugin", Hasher::id(), None);
    ///
    /// registry.register(&id, hook).unwrap();
    ///
    /// let mut retrieved = registry.get_by_id_mut(&id);
    /// assert!(retrieved.is_some());
    /// ```
    #[must_use]
    pub fn get_by_id_mut(&mut self, id: &HookID) -> Option<(&HookID, &mut BoxedHook)> {
        self.get_by_filter_mut(|(hid, _)| *hid == id)
            .into_iter()
            .next()
    }

    /// Gets all hooks registered by a specific [Plugin](crate::Plugin).
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: The [`PluginID`] of the plugin
    ///
    /// # Returns
    ///
    /// A vector of references to all hooks registered by the plugin
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Formatter: FormatterTrait;
    ///     fn format(&self, input: &str) -> String;
    /// );
    ///
    /// extension_point!(
    ///     Parser: ParserTrait;
    ///     fn parse(&self, input: &str) -> bool;
    /// );
    ///
    /// struct JsonFormatter;
    /// impl FormatterTrait for JsonFormatter {
    ///     fn format(&self, input: &str) -> String {
    ///         format!("{{\"data\":\"{}\"}}", input)
    ///     }
    /// }
    ///
    /// struct JsonParser;
    /// impl ParserTrait for JsonParser {
    ///     fn parse(&self, _: &str) -> bool {
    ///         true
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let plugin_id = "json_plugin";
    ///
    /// let formatter_hook = Hook::<Formatter>::new(Box::new(JsonFormatter), "formathook");
    /// let formatter_id = HookID::new(plugin_id, Formatter::id(), None);
    ///
    /// let parser_hook = Hook::<Parser>::new(Box::new(JsonParser), "parsehook");
    /// let parser_id = HookID::new(plugin_id, Parser::id(), None);
    ///
    /// registry.register(&formatter_id, formatter_hook).unwrap();
    /// registry.register(&parser_id, parser_hook).unwrap();
    ///
    /// let plugin_hooks = registry.get_by_plugin(plugin_id);
    /// assert_eq!(plugin_hooks.len(), 2);
    /// ```
    #[must_use]
    pub fn get_by_plugin(&self, plugin_id: PluginID) -> Vec<(&HookID, &BoxedHook)> {
        self.get_by_filter(|(id, _)| id.plugin_id == plugin_id)
    }

    /// Gets all hooks registered by a specific [Plugin](crate::Plugin), mutable.
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: The [`PluginID`] of the plugin
    ///
    /// # Returns
    ///
    /// A vector of mutable references to all hooks registered by the plugin
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Formatter: FormatterTrait;
    ///     fn format(&mut self, input: &str) -> String;
    /// );
    ///
    /// extension_point!(
    ///     Parser: ParserTrait;
    ///     fn parse(&mut self, input: &str) -> bool;
    /// );
    ///
    /// struct JsonFormatter;
    /// impl FormatterTrait for JsonFormatter {
    ///     fn format(&mut self, input: &str) -> String {
    ///         format!("{{\"data\":\"{}\"}}", input)
    ///     }
    /// }
    ///
    /// struct JsonParser;
    /// impl ParserTrait for JsonParser {
    ///     fn parse(&mut self, _: &str) -> bool {
    ///         true
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let plugin_id = "json_plugin";
    ///
    /// let formatter_hook = Hook::<Formatter>::new(Box::new(JsonFormatter), "formathook");
    /// let formatter_id = HookID::new(plugin_id, Formatter::id(), None);
    ///
    /// let parser_hook = Hook::<Parser>::new(Box::new(JsonParser), "parsehook");
    /// let parser_id = HookID::new(plugin_id, Parser::id(), None);
    ///
    /// registry.register(&formatter_id, formatter_hook).unwrap();
    /// registry.register(&parser_id, parser_hook).unwrap();
    ///
    /// let mut plugin_hooks = registry.get_by_plugin_mut(plugin_id);
    /// assert_eq!(plugin_hooks.len(), 2);
    /// ```
    #[must_use]
    pub fn get_by_plugin_mut(&mut self, plugin_id: PluginID) -> Vec<(&HookID, &mut BoxedHook)> {
        self.get_by_filter_mut(|(id, _)| id.plugin_id == plugin_id)
    }

    /// Gets hooks that match a filter function.
    ///
    /// # Parameters
    ///
    /// - `f`: A function that takes a reference to a hook ID and hook, and returns a boolean
    ///
    /// # Returns
    ///
    /// A vector of references to hooks that match the filter
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Logger: LoggerTrait;
    ///     fn log(&self, level: &str, message: &str);
    /// );
    ///
    /// struct ConsoleLogger;
    /// impl LoggerTrait for ConsoleLogger {
    ///     fn log(&self, level: &str, message: &str) {
    ///         // In a real implementation, this would print to console
    ///     }
    /// }
    ///
    /// struct FileLogger;
    /// impl LoggerTrait for FileLogger {
    ///     fn log(&self, level: &str, message: &str) {
    ///         // In a real implementation, this would write to a file
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    ///
    /// let console_hook = Hook::<Logger>::new(Box::new(ConsoleLogger), "consoleh");
    /// let console_id = HookID::new("logger_plugin", Logger::id(), Some("console"));
    ///
    /// let file_hook = Hook::<Logger>::new(Box::new(FileLogger), "fileh");
    /// let file_id = HookID::new("logger_plugin", Logger::id(), Some("file"));
    ///
    /// registry.register(&console_id, console_hook).unwrap();
    /// registry.register(&file_id, file_hook).unwrap();
    ///
    /// // Get only the file logger
    /// let file_loggers = registry.get_by_filter(|(id, _)| {
    ///     id.discriminator.as_deref() == Some("file")
    /// });
    ///
    /// assert_eq!(file_loggers.len(), 1);
    /// ```
    #[must_use]
    pub fn get_by_filter<F>(&self, f: F) -> Vec<(&HookID, &BoxedHook)>
    where
        F: FnMut(&(&HookID, &BoxedHook)) -> bool,
    {
        let mut v = self.hooks.values().flatten().filter(f).collect::<Vec<_>>();
        v.sort();
        v
    }

    /// Gets mutable hooks that match a filter function.
    ///
    /// # Parameters
    ///
    /// - `f`: A function that takes a reference to a hook ID and hook, and returns a boolean
    ///
    /// # Returns
    ///
    /// A vector of mutable references to hooks that match the filter
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Logger: LoggerTrait;
    ///     fn log(&mut self, level: &str, message: &str);
    /// );
    ///
    /// struct ConsoleLogger;
    /// impl LoggerTrait for ConsoleLogger {
    ///     fn log(&mut self, level: &str, message: &str) {
    ///         // In a real implementation, this would print to console
    ///     }
    /// }
    ///
    /// struct FileLogger;
    /// impl LoggerTrait for FileLogger {
    ///     fn log(&mut self, level: &str, message: &str) {
    ///         // In a real implementation, this would write to a file
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    ///
    /// let console_hook = Hook::<Logger>::new(Box::new(ConsoleLogger), "consoleh");
    /// let console_id = HookID::new("logger_plugin", Logger::id(), Some("console"));
    ///
    /// let file_hook = Hook::<Logger>::new(Box::new(FileLogger), "fileh");
    /// let file_id = HookID::new("logger_plugin", Logger::id(), Some("file"));
    ///
    /// registry.register(&console_id, console_hook).unwrap();
    /// registry.register(&file_id, file_hook).unwrap();
    ///
    /// // Get only the file logger
    /// let file_loggers = registry.get_by_filter_mut(|(id, _)| {
    ///     id.discriminator.as_deref() == Some("file")
    /// });
    ///
    /// assert_eq!(file_loggers.len(), 1);
    /// ```
    #[must_use]
    pub fn get_by_filter_mut<F>(&mut self, f: F) -> Vec<(&HookID, &mut BoxedHook)>
    where
        F: FnMut(&(&HookID, &mut BoxedHook)) -> bool,
    {
        let mut v = self
            .hooks
            .values_mut()
            .flatten()
            .filter(f)
            .collect::<Vec<_>>();
        v.sort();
        v
    }

    /// Gets all hooks for a specific [`ExtensionPoint`] type.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type
    ///
    /// # Returns
    ///
    /// A vector of references to all hooks registered for the [`ExtensionPoint`]
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Validator: ValidatorTrait;
    ///     fn validate(&self, input: &str) -> bool;
    /// );
    ///
    /// struct LengthValidator;
    /// impl ValidatorTrait for LengthValidator {
    ///     fn validate(&self, input: &str) -> bool {
    ///         input.len() > 0
    ///     }
    /// }
    ///
    /// struct NumberValidator;
    /// impl ValidatorTrait for NumberValidator {
    ///     fn validate(&self, input: &str) -> bool {
    ///         input.parse::<i32>().is_ok()
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    ///
    /// let length_hook = Hook::<Validator>::new(Box::new(LengthValidator), "lenh");
    /// let length_id = HookID::new("validator_plugin", Validator::id(), Some("length"));
    ///
    /// let number_hook = Hook::<Validator>::new(Box::new(NumberValidator), "numh");
    /// let number_id = HookID::new("validator_plugin", Validator::id(), Some("number"));
    ///
    /// registry.register(&length_id, length_hook).unwrap();
    /// registry.register(&number_id, number_hook).unwrap();
    ///
    /// dbg!(&registry);
    /// let validators: Vec<(&HookID, &Hook<_>)> = registry.get_by_extension_point::<Validator>();
    /// assert_eq!(validators.len(), 2);
    /// // if we want to actually know which is
    /// //which, we can use the name method of the hook to get metadata
    /// validators.iter().for_each(|(_id,v)|{dbg!(v.name());});
    ///
    /// // Test the validators
    /// // 0 is len, 2 is num
    /// assert!(validators[0].1.inner().validate("123"));
    /// assert!(validators[1].1.inner().validate("123"));
    /// assert!(validators[0].1.inner().validate("abc"));
    /// assert!(!validators[1].1.inner().validate("abc"));
    /// ```
    #[must_use]
    pub fn get_by_extension_point<E: ExtensionPoint>(&self) -> Vec<(&HookID, &Hook<E>)> {
        let Some(boxed_hooks) = self.hooks.get(&E::id()) else {
            return Vec::new();
        };
        let mut v: Vec<(&HookID, &Hook<E>)> = boxed_hooks
            .iter()
            .filter_map(|(k, v)| v.downcast().map(|hook| (k, hook)))
            .collect();
        v.sort();
        v
    }

    /// Gets all mutable hooks for a specific [`ExtensionPoint`] type.
    ///
    /// # Type Parameters
    ///
    /// - `E`: The extension point type
    ///
    /// # Returns
    ///
    /// A vector of mutable references to all hooks registered for the [`ExtensionPoint`]
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Validator: ValidatorTrait;
    ///     fn validate(&mut self, input: &str) -> bool;
    /// );
    ///
    /// struct LengthValidator;
    /// impl ValidatorTrait for LengthValidator {
    ///     fn validate(&mut self, input: &str) -> bool {
    ///         input.len() > 0
    ///     }
    /// }
    ///
    /// struct NumberValidator;
    /// impl ValidatorTrait for NumberValidator {
    ///     fn validate(&mut self, input: &str) -> bool {
    ///         input.parse::<i32>().is_ok()
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    ///
    /// let length_hook = Hook::<Validator>::new(Box::new(LengthValidator), "lenh");
    /// let length_id = HookID::new("validator_plugin", Validator::id(), Some("length"));
    ///
    /// let number_hook = Hook::<Validator>::new(Box::new(NumberValidator), "numh");
    /// let number_id = HookID::new("validator_plugin", Validator::id(), Some("number"));
    ///
    /// registry.register(&length_id, length_hook).unwrap();
    /// registry.register(&number_id, number_hook).unwrap();
    ///
    /// dbg!(&registry);
    /// let mut validators: Vec<(&HookID, &mut Hook<_>)> = registry.get_by_extension_point_mut::<Validator>();
    /// assert_eq!(validators.len(), 2);
    /// // if we want to actually know which is
    /// //which, we can use the name method of the hook to get metadata
    /// validators.iter().for_each(|(_id,v)|{dbg!(v.name());});
    ///
    /// // Test the validators
    /// // 0 is len, 2 is num
    /// assert!(validators[0].1.inner_mut().validate("123"));
    /// assert!(validators[1].1.inner_mut().validate("123"));
    /// assert!(validators[0].1.inner_mut().validate("abc"));
    /// assert!(!validators[1].1.inner_mut().validate("abc"));
    /// ```
    #[must_use]
    pub fn get_by_extension_point_mut<E: ExtensionPoint>(
        &mut self,
    ) -> Vec<(&HookID, &mut Hook<E>)> {
        let Some(boxed_hooks) = self.hooks.get_mut(&E::id()) else {
            return Vec::new();
        };
        let mut v: Vec<(&HookID, &mut Hook<E>)> = boxed_hooks
            .iter_mut()
            .filter_map(|(k, v)| v.downcast_mut().map(|hook| (k, hook)))
            .collect();
        v.sort();
        v
    }

    /// Deregisters all hooks for a specific [Plugin](crate::Plugin).
    ///
    /// # Parameters
    ///
    /// - `plugin_id`: The [`PluginID`] of the plugin
    ///
    /// # Panics
    ///
    /// If deregistering fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use steckrs::{extension_point, hook::{HookRegistry, Hook, HookID, ExtensionPoint}};
    ///
    /// extension_point!(
    ///     Formatter: FormatterTrait;
    ///     fn format(&self, input: &str) -> String;
    /// );
    ///
    /// struct HtmlFormatter;
    /// impl FormatterTrait for HtmlFormatter {
    ///     fn format(&self, input: &str) -> String {
    ///         format!("<p>{}</p>", input)
    ///     }
    /// }
    ///
    /// struct XmlFormatter;
    /// impl FormatterTrait for XmlFormatter {
    ///     fn format(&self, input: &str) -> String {
    ///         format!("<text>{}</text>", input)
    ///     }
    /// }
    ///
    /// let mut registry = HookRegistry::new();
    /// let plugin_id = "formatter_plugin";
    ///
    /// let html_hook = Hook::<Formatter>::new(Box::new(HtmlFormatter), "htmlhook");
    /// let html_id = HookID::new(plugin_id, Formatter::id(), Some("html"));
    ///
    /// let xml_hook = Hook::<Formatter>::new(Box::new(XmlFormatter), "xmlhook");
    /// let xml_id = HookID::new(plugin_id, Formatter::id(), Some("xml"));
    ///
    /// registry.register(&html_id, html_hook).unwrap();
    /// registry.register(&xml_id, xml_hook).unwrap();
    ///
    /// // Before deregistration
    /// assert_eq!(registry.get_by_extension_point::<Formatter>().len(), 2);
    ///
    /// // Deregister all hooks for the plugin
    /// registry.deregister_hooks_for_plugin(plugin_id);
    ///
    /// // After deregistration
    /// assert_eq!(registry.get_by_extension_point::<Formatter>().len(), 0);
    /// ```
    pub fn deregister_hooks_for_plugin(&mut self, plugin_id: PluginID) {
        let to_del: Vec<HookID> = self
            .hooks
            .values()
            .flatten()
            .filter(|(id, _hook)| id.plugin_id == plugin_id)
            .map(|(id, _)| id)
            .map(std::borrow::ToOwned::to_owned)
            .collect();

        for id in to_del {
            self.deregister(&id)
                .expect("could not deregister a hook that we know exists");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_hook_with_owned_plugin_id() {
        extension_point!(
            Validator: ValidatorTrait;
            fn validate(&self, input: &str) -> bool;
        );

        struct LengthValidator;
        impl ValidatorTrait for LengthValidator {
            fn validate(&self, input: &str) -> bool {
                input.len() > 5
            }
        }

        let id = PluginIDOwned::from("foo");
        let hook = Hook::<Validator>::new(Box::new(LengthValidator), id.into());
        assert!(hook.inner().validate("this is long enough"));
    }
}
