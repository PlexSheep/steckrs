use steckrs::{extension_point, simple_plugin, PluginManager};

// Define an extension point with documentation
extension_point!(
    GreeterExtension:
    GreeterTrait;
    /// foo
    fn greet(&self, name: &str) -> String;
);

// Implement a hook
/// English implementation of the greeter
struct EnglishGreeter;
impl GreeterTrait for EnglishGreeter {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}

// Create a plugin with documentation
simple_plugin!(
    /// A Plugin providing English greeting functionality
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

    // Get all enabled hooks (plugins could be disabled)
    let hooks = plugin_manager.get_enabled_hooks_by_ep::<GreeterExtension>();

    // Execute all hooks relevant for this extension point
    for (_id, hook) in hooks {
        println!("{}", hook.inner().greet("World"));
    }
}
