use std::any::Any;
use std::fmt::Debug;

use steckrs::{
    error::PluginResult,
    hook::{ExtensionPoint, Hook, HookID, RefAny},
    Plugin, PluginID, PluginManager,
};

// Define our extension point for text filtering
pub struct TextFilter;
impl ExtensionPoint for TextFilter {
    type Input = RefAny<str>; // Use RefAny to handle references
    type Output = String;
}

// Define a simple text processor app
struct TextProcessor {
    plugin_manager: PluginManager,
}

impl TextProcessor {
    fn new() -> Self {
        Self {
            plugin_manager: PluginManager::new(),
        }
    }

    // Core functionality - convert text to lowercase
    fn process_text(&self, text: &str) -> String {
        // Basic functionality: convert to lowercase
        let result = text.to_lowercase();

        // Create a reference wrapper
        let text_ref = RefAny::new(text);

        // Apply any plugin filters that are registered
        let filtered_results: Vec<String> = self.apply_filters(text_ref);

        // Combine the results
        if filtered_results.is_empty() {
            result
        } else {
            format!(
                "{}\nPlugin results:\n{}",
                result,
                filtered_results.join("\n")
            )
        }
    }

    fn apply_filters(&self, text_ref: RefAny<str>) -> Vec<String> {
        let registry = self.plugin_manager.hook_registry();

        // Get all text filter hooks and apply them
        let filter_hooks = registry.get_by_extension_point::<TextFilter>();

        // Create a new RefAny for each hook from the original string
        let original_text = unsafe { text_ref.get() };
        filter_hooks
            .iter()
            .map(|hook| {
                // Create a fresh RefAny for each hook
                let new_ref = RefAny::new(original_text);
                hook.execute(new_ref)
            })
            .collect()
    }

    fn load_plugins(&mut self) -> PluginResult<()> {
        // Load plugins
        self.plugin_manager
            .load_plugin(Box::new(UppercasePlugin::new()))?;
        self.plugin_manager
            .load_plugin(Box::new(ReversePlugin::new()))?;

        // Enable plugins
        self.plugin_manager.enable_plugin("uppercase_plugin")?;
        self.plugin_manager.enable_plugin("reverse_plugin")?;

        // Register hooks for each plugin

        // Uppercase plugin - safely using the RefAny wrapper
        let uppercase_hook = Hook::<TextFilter>::new(|text_ref: RefAny<str>| {
            // Safely get the reference back with the right lifetime
            let text = unsafe { text_ref.get() };
            text.to_uppercase()
        });
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new("uppercase_plugin".into(), TextFilter::id(), None),
            uppercase_hook,
        )?;

        // Reverse plugin - also safely using RefAny
        let reverse_hook = Hook::<TextFilter>::new(|text_ref: RefAny<str>| {
            let text = unsafe { text_ref.get() };
            text.chars().rev().collect()
        });
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new("reverse_plugin".into(), TextFilter::id(), None),
            reverse_hook,
        )?;

        Ok(())
    }
}

// A simple plugin that converts text to uppercase
#[derive(Debug)]
struct UppercasePlugin {
    enabled: bool,
}

impl UppercasePlugin {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl Plugin for UppercasePlugin {
    fn id(&self) -> PluginID {
        "uppercase_plugin"
    }

    fn description(&self) -> &str {
        "Converts text to uppercase"
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

    fn on_load(&mut self) -> PluginResult<()> {
        println!("Uppercase plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> PluginResult<()> {
        println!("Uppercase plugin unloaded!");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// A simple plugin that reverses text
#[derive(Debug)]
struct ReversePlugin {
    enabled: bool,
}

impl ReversePlugin {
    fn new() -> Self {
        Self { enabled: false }
    }
}

impl Plugin for ReversePlugin {
    fn id(&self) -> PluginID {
        "reverse_plugin"
    }

    fn description(&self) -> &str {
        "Reverses text"
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

    fn on_load(&mut self) -> PluginResult<()> {
        println!("Reverse plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> PluginResult<()> {
        println!("Reverse plugin unloaded!");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and initialize our text processor
    let mut processor = TextProcessor::new();
    processor.load_plugins()?;

    // Test the basic app with plugins
    let sample_texts = [
        "Hello, Plugin System!",
        "TESTING Lowercase Conversion",
        "This will be filtered by plugins",
    ];

    for text in &sample_texts {
        println!("Original: {}", text);
        println!("Processed: {}", processor.process_text(text));
        println!("---");
    }

    // Unload a plugin to demonstrate lifecycle
    println!("Unloading the uppercase plugin...");
    processor.plugin_manager.unload_plugin("uppercase_plugin")?;

    println!("\nAfter unloading uppercase plugin:");
    println!("Original: {}", sample_texts[0]);
    println!("Processed: {}", processor.process_text(sample_texts[0]));

    Ok(())
}
