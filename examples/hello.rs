use steckrs::{extension_point, simple_plugin, PluginManager};

// Define an extension point
extension_point!(
    GreeterExtension: GreeterTrait,
    fn greet(&self, name: &str) -> String,
);

// Implement a hook
struct EnglishGreeter;
impl GreeterTrait for EnglishGreeter {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}

// Create a plugin
simple_plugin!(
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
