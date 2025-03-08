use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::error::{HookError, HookResult};
use crate::PluginID;

/// Hook ID type
pub type ExtensionPointID = std::any::TypeId;
/// A hook function that can be registered
pub type HookFunction<I, O> = Box<dyn Fn(I) -> O + Send + Sync>;

/// Hook identifier type - uniquely identifies a specific hook instance
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct HookID {
    /// Plugin that owns this hook
    pub plugin_id: PluginID,
    /// Extension point this hook implements
    pub extension_point_id: ExtensionPointID,
    /// Optional discriminator if a plugin registers multiple hooks for same extension point
    pub discriminator: Option<String>,
}

/// Extension point trait - defines the interface for a specific hook type
pub trait ExtensionPoint: 'static {
    /// Unique identifier for this extension point
    fn id() -> ExtensionPointID {
        std::any::TypeId::of::<Self>()
    }
    /// Human readable name of this extension point
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Input type for this extension point
    type Input;
    /// Output type for this extension point
    type Output;
}

/// Container for a hook function with proper type information
pub struct Hook<E: ExtensionPoint> {
    /// Marker for the extension point type
    extension_point: PhantomData<E>,
    /// The actual hook function
    func: HookFunction<E::Input, E::Output>,
}

impl HookID {
    pub fn new(
        plugin_id: PluginID,
        extension_point_id: ExtensionPointID,
        discriminator: Option<String>,
    ) -> Self {
        HookID {
            plugin_id,
            extension_point_id,
            discriminator,
        }
    }
}

impl<E: ExtensionPoint> Hook<E> {
    /// Create a new hook with the given function
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(E::Input) -> E::Output + Send + Sync + 'static,
    {
        Hook {
            extension_point: PhantomData,
            func: Box::new(func),
        }
    }

    /// Execute the hook with the given input
    pub fn execute(&self, input: E::Input) -> E::Output {
        (self.func)(input)
    }

    pub fn eid(&self) -> ExtensionPointID {
        E::id()
    }
}

/// A type-erased hook that can be stored in a HashMap
pub struct BoxedHook {
    /// The actual hook function, type-erased
    hook: Box<dyn Any + Send + Sync>,
    eid: ExtensionPointID,
}

impl BoxedHook {
    pub fn downcast<E: ExtensionPoint>(&self) -> Option<&Hook<E>> {
        self.hook.downcast_ref::<Hook<E>>()
    }
    pub fn eid(&self) -> ExtensionPointID {
        self.eid
    }
}

impl Debug for BoxedHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedHook").finish_non_exhaustive()
    }
}

/// Registry for hooks
#[derive(Debug, Default)]
pub struct HookRegistry {
    hooks: HashMap<ExtensionPointID, HashMap<HookID, BoxedHook>>,
}

impl<E: ExtensionPoint + Send + Sync + 'static> From<Hook<E>> for BoxedHook {
    fn from(value: Hook<E>) -> Self {
        BoxedHook {
            eid: value.eid(),
            hook: Box::new(value),
        }
    }
}

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Register a hook with the given ID
    pub fn register<E: ExtensionPoint + Send + Sync + 'static>(
        &mut self,
        id: &HookID,
        hook: Hook<E>,
    ) -> HookResult<()> {
        if self.exists::<E>(id) {
            return Err(HookError::AlreadyRegistered);
        }

        let boxed_hook = BoxedHook::from(hook);

        let old = self
            .hooks
            .entry(E::id())
            .or_default()
            .insert(id.clone(), boxed_hook);

        assert!(old.is_none());

        Ok(())
    }

    pub fn deregister(&mut self, id: &HookID) -> Option<BoxedHook> {
        let hook = self.get_by_id(id)?;
        if let Some(h) = self.hooks.get_mut(&hook.eid()) {
            h.remove(id)
        } else {
            todo!()
        }
    }

    #[must_use]
    pub fn exists<E: ExtensionPoint>(&self, id: &HookID) -> bool {
        self.get::<E>(id).is_some()
    }

    /// Get the hooks with the given ID
    pub fn get<E: ExtensionPoint>(&self, id: &HookID) -> Option<&Hook<E>> {
        match self.hooks.get(&E::id()) {
            Some(hooks) => {
                let boxed_hook = hooks.get(id)?;
                assert!(id.extension_point_id == E::id());
                boxed_hook.downcast()
            }
            None => None,
        }
    }

    pub fn get_by_id(&self, id: &HookID) -> Option<&BoxedHook> {
        self.get_by_filter(|(hid, _)| *hid == id).first().copied()
    }

    pub fn get_by_plugin(&self, plugin_id: PluginID) -> Vec<&BoxedHook> {
        self.get_by_filter(|(id, _)| id.plugin_id == plugin_id)
    }

    pub fn get_by_filter<F>(&self, f: F) -> Vec<&BoxedHook>
    where
        F: FnMut(&(&HookID, &BoxedHook)) -> bool,
    {
        self.hooks
            .values()
            .flatten()
            .filter(f)
            .map(|(_id, hook)| hook)
            .collect()
    }

    pub fn get_by_filter_mut<F>(&mut self, f: F) -> Vec<&mut BoxedHook>
    where
        F: FnMut(&(&HookID, &mut BoxedHook)) -> bool,
    {
        self.hooks
            .values_mut()
            .flatten()
            .filter(f)
            .map(|(_id, hook)| hook)
            .collect()
    }

    pub fn get_by_extension_point<E: ExtensionPoint>(&self) -> Vec<&Hook<E>> {
        let Some(boxed_hooks) = self.hooks.get(&E::id()) else {return Vec::new()};
        boxed_hooks
            .iter()
            .map(|(_k, v)| v.downcast().expect("could not downcast BoxedHook to Hook"))
            .collect()
    }

    /// Execute the hooks with the given ID
    pub fn execute<E: ExtensionPoint>(
        &self,
        id: &HookID,
        input: E::Input,
    ) -> HookResult<E::Output> {
        if let Some(hook) = self.get::<E>(id) {
            Ok(hook.execute(input))
        } else {
            Err(HookError::HookNotFound(E::id()))
        }
    }

    pub fn deregister_hooks_for_plugin(&mut self, plugin_id: PluginID) {
        let to_del: Vec<HookID> = self
            .hooks
            .values()
            .flatten()
            .filter(|(id, _hook)| id.plugin_id == plugin_id)
            .map(|(id, _)| id)
            .map(|a| a.to_owned())
            .collect();

        for id in to_del {
            self.deregister(&id)
                .expect("could not deregister a hook that we know exists");
        }
    }
}
