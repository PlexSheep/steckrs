use steckrs::{extension_point, simple_plugin, PluginManager};

// Define an extension point with documentation
extension_point!(
    GreeterExtension:
    GreeterTrait;
    /// foo
    fn greet(&self, name: &str) -> String;
);

// For a version with associated types, you could do:
// extension_point!(
//     /// Extension point for localized greeting functionality
//     LocalizedGreeterExtension:
//     /// Trait defining localized greeting operations
//     LocalizedGreeterTrait,
//     /// The language type for this greeter
//     type Language: Send + Sync;
//     /// Returns a greeting for the given name in the implementation's language
//     fn greet(&self, name: &str) -> String,
// );

/// English implementation of the greeter
struct EnglishGreeter;
impl GreeterTrait for EnglishGreeter {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}

// Create a plugin with documentation
simple_plugin!(
    /// Plugin providing English greeting functionality
    HelloPlugin,
    "hello_plugin",
    "A simple greeting plugin",
    hooks: [(GreeterExtension, EnglishGreeter)]
);

fn main() {
    // Create plugin manager
    let mut plugin_manager = PluginManager::new();

    // Load and enable the plugin
    plugin_manager
        .load_plugin(Box::new(HelloPlugin::new()))
        .unwrap();
    plugin_manager.enable_plugin(HelloPlugin::ID).unwrap();

    // Use the plugin
    let registry = plugin_manager.hook_registry();
    let hooks = registry.get_by_extension_point::<GreeterExtension>();

    // Execute all hooks relevant for this extension point
    for hook in hooks {
        println!("{}", hook.inner().greet("World"));
    }
}
