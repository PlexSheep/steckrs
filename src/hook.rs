use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::error::{HookError, HookResult};
use crate::PluginID;

/// Hook ID type
pub type ExtensionPointID = std::any::TypeId;

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

impl HookID {
    pub fn new(
        plugin_id: PluginID,
        extension_point_id: ExtensionPointID,
        discriminator: Option<&'static str>,
    ) -> Self {
        HookID {
            plugin_id,
            extension_point_id,
            discriminator: discriminator.map(|s| s.into()),
        }
    }
}

/// Extension point trait - defines the interface for a specific hook type
pub trait ExtensionPoint: 'static {
    /// The trait that hooks implement for this extension point
    type HookTrait: ?Sized + Send + Sync + 'static;

    /// Unique identifier for this extension point
    fn id() -> ExtensionPointID {
        std::any::TypeId::of::<Self>()
    }

    /// Human readable name of this extension point
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A wrapper around a hook trait object
pub struct Hook<E: ExtensionPoint> {
    /// The actual hook trait object
    inner: Box<E::HookTrait>,
}

impl<E: ExtensionPoint> Hook<E> {
    /// Create a new hook with the given trait implementation
    pub fn new(hook: Box<E::HookTrait>) -> Self {
        Hook { inner: hook }
    }

    /// Access the inner trait implementation
    pub fn inner(&self) -> &E::HookTrait {
        &self.inner
    }
}

/// A type-erased hook that can be stored in a HashMap
pub struct BoxedHook {
    /// The actual hook trait object, type-erased
    hook: Box<dyn Any + Send + Sync>,
    eid: ExtensionPointID,
}

impl BoxedHook {
    pub fn new<E: ExtensionPoint>(hook: Hook<E>) -> Self {
        BoxedHook {
            hook: Box::new(hook),
            eid: E::id(),
        }
    }

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

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Register a hook with the given ID
    pub fn register<E: ExtensionPoint>(&mut self, id: &HookID, hook: Hook<E>) -> HookResult<()> {
        if self.exists::<E>(id) {
            return Err(HookError::AlreadyRegistered);
        }

        let boxed_hook = BoxedHook::new(hook);

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
        let Some(boxed_hooks) = self.hooks.get(&E::id()) else {
            return Vec::new();
        };
        boxed_hooks
            .iter()
            .filter_map(|(_k, v)| v.downcast())
            .collect()
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
