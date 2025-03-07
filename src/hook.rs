use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::error::{HookError, HookResult};

/// Hook ID type
pub type ExtensionPointID = &'static str;
pub type HookID = &'static str;

/// A hook function that can be registered
pub type HookFunction<I, O> = Box<dyn Fn(I) -> O + Send + Sync>;

pub trait ExtensionPoint {
    fn id() -> ExtensionPointID;
    type Input;
    type Output;
}

/// Container for a hook function with proper type information
pub struct Hook<E: ExtensionPoint> {
    extension_point: PhantomData<E>,
    func: HookFunction<E::Input, E::Output>,
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
}

/// A type-erased hook that can be stored in a HashMap
struct BoxedHook {
    /// The actual hook function, type-erased
    hook: Box<dyn Any + Send + Sync>,
    /// Type information for downcasting
    extension_point_type_id: std::any::TypeId,
}

impl Debug for BoxedHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedHook")
            .field(
                "extension_point_type_id",
                &format!("{:?}", self.extension_point_type_id),
            )
            .finish_non_exhaustive()
    }
}

/// Registry for hooks
#[derive(Debug, Default)]
pub struct HookRegistry {
    hooks: HashMap<HookID, Vec<BoxedHook>>,
}

impl<E: ExtensionPoint + Send + Sync + 'static> From<Hook<E>> for BoxedHook {
    fn from(value: Hook<E>) -> Self {
        BoxedHook {
            hook: Box::new(value),
            extension_point_type_id: std::any::TypeId::of::<E>(),
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
        id: HookID,
        hook: Hook<E>,
    ) -> HookResult<()> {
        let boxed_hook = BoxedHook::from(hook);

        self.hooks.entry(id).or_default().push(boxed_hook);

        Ok(())
    }

    /// Deregister all hooks with the given ID
    pub fn deregister(&mut self, id: ExtensionPointID) -> HookResult<()> {
        self.hooks.remove(id);
        Ok(())
    }

    /// Get all hooks with the given ID and types
    pub fn get<E: ExtensionPoint + 'static>(&self, id: HookID) -> Option<&Hook<E>> {
        match self.hooks.get(id) {
            Some(hooks) => {
                for h in hooks {
                    if h.extension_point_type_id == std::any::TypeId::of::<E>() {
                        return h.hook.downcast_ref::<Hook<E>>();
                    }
                }
                None
            }
            None => None,
        }
    }

    /// Execute all hooks with the given ID and types
    pub fn execute<E: ExtensionPoint + 'static>(
        &self,
        id: HookID,
        input: E::Input,
    ) -> HookResult<E::Output> {
        if let Some(hook) = self.get::<E>(id) {
            Ok(hook.execute(input))
        } else {
            Err(HookError::HookNotFound(id))
        }
    }
}
