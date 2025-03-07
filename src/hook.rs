use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::error::{HookError, HookResult};

/// Hook ID type
pub type HookID = &'static str;

/// A hook function that can be registered
pub type HookFunction<I, O> = Box<dyn Fn(I) -> O + Send + Sync>;

/// Container for a hook function with proper type information
pub struct Hook<I, O> {
    _param_type: PhantomData<I>,
    _return_type: PhantomData<O>,
    func: HookFunction<I, O>,
}

impl<I, O> Hook<I, O> {
    /// Create a new hook with the given function
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(I) -> O + Send + Sync + 'static,
    {
        Hook {
            _param_type: PhantomData,
            _return_type: PhantomData,
            func: Box::new(func),
        }
    }

    /// Execute the hook with the given input
    pub fn execute(&self, input: I) -> O {
        (self.func)(input)
    }
}

/// A type-erased hook that can be stored in a HashMap
struct BoxedHook {
    /// The actual hook function, type-erased
    hook: Box<dyn Any + Send + Sync>,
    /// Type information for downcasting
    type_id: (std::any::TypeId, std::any::TypeId),
}

impl Debug for BoxedHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxedHook")
            .field("type_id", &format!("{:?}", self.type_id))
            .finish()
    }
}

/// Registry for hooks
#[derive(Debug, Default)]
pub struct HookRegistry {
    hooks: HashMap<HookID, BoxedHook>,
}

impl HookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Register a hook with the given ID
    pub fn register<I: Send + Sync + 'static, O: Send + Sync + 'static>(
        &mut self,
        id: HookID,
        hook: Hook<I, O>,
    ) -> HookResult<()> {
        let boxed_hook = BoxedHook {
            hook: Box::new(hook),
            type_id: (std::any::TypeId::of::<I>(), std::any::TypeId::of::<O>()),
        };

        self.hooks.insert(id, boxed_hook);

        Ok(())
    }

    /// Deregister all hooks with the given ID
    pub fn deregister(&mut self, id: HookID) -> HookResult<()> {
        self.hooks.remove(id);
        Ok(())
    }

    /// Get all hooks with the given ID and types
    pub fn get<I: 'static, O: 'static>(&self, id: HookID) -> Option<&Hook<I, O>> {
        match self.hooks.get(id) {
            Some(h) => {
                if h.type_id == (std::any::TypeId::of::<I>(), std::any::TypeId::of::<O>()) {
                    Some(h.hook.downcast_ref::<Hook<I, O>>()?)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Execute all hooks with the given ID and types
    pub fn execute<I: 'static, O: 'static>(&self, id: HookID, input: I) -> HookResult<O> {
        if let Some(hook) = self.get(id) {
            hook.execute(input)
        } else {
            Err(HookError::HookNotFound(id))
        }
    }
}
