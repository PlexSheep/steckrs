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
        impl $crate::hook::ExtensionPoint for $name {
            type HookTrait = dyn $trait_name;
        }
    };
}

#[macro_export]
macro_rules! simple_plugin {
        ($plugin_name:ident, $plugin_id:expr, $description:expr,
     hooks: [$(($extension_point:ident, $hook_impl:ident $(, $discrim:expr)?)),* $(,)?]) => {
        #[derive(Debug)]
        pub struct $plugin_name {
            enabled: bool,
        }

        impl $plugin_name {
            pub const ID: $crate::PluginID = $plugin_id;
            pub const DESCRIPTION: &'static str = $description;

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


            fn register_hooks(&self, registry: &mut $crate::hook::HookRegistry) -> PluginResult<()> {
                $(
                    $crate::register_hook!(registry, Self::ID, $extension_point, $hook_impl $(, $discrim)?);
                )*

                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! register_hook {
    ($registry_mut:expr, $plugin_id:expr, $extension_point:ident, $hook:ident) => {
        $registry_mut.register(
            &$crate::hook::HookID::new(
                $plugin_id,
                <$extension_point as $crate::hook::ExtensionPoint>::id(),
                None,
            ),
            $crate::hook::Hook::<$extension_point>::new(Box::new($hook)),
        )?;
    };
    ($registry_mut:expr, $plugin_id:expr, $extension_point:ident, $hook:ident, $discriminator:expr) => {
        $registry_mut.register(
            &$crate::hook::HookID::new(
                $plugin_id,
                <$extension_point as $crate::hook::ExtensionPoint>::id(),
                Some($discriminator),
            ),
            $crate::hook::Hook::<$extension_point>::new(Box::new($hook)),
        )?;
    };
}
