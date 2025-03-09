use std::any::Any;
use std::fmt::Debug;

use steckrs::{
    error::PluginResult,
    extension_point,
    hook::{ExtensionPoint, Hook, HookID},
    Plugin, PluginID, PluginManager,
};

extension_point!(TextFilter: TextFilterFunctions,
    fn filter(&self, text: &str) -> String,
    fn tester(&self, num: usize) -> String,
);

// Define implementations of the text filter trait
struct UppercaseFilter;
impl TextFilterFunctions for UppercaseFilter {
    fn filter(&self, text: &str) -> String {
        text.to_uppercase()
    }

    fn tester(&self, num: usize) -> String {
        format!("{num}")
    }
}

struct ReverseFilter;
impl TextFilterFunctions for ReverseFilter {
    fn filter(&self, text: &str) -> String {
        text.chars().rev().collect()
    }
    fn tester(&self, num: usize) -> String {
        format!("{}", num * 2)
    }
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

        // Apply any plugin filters that are registered
        let filtered_results: Vec<String> = self.apply_filters(text);

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

    fn apply_filters(&self, text: &str) -> Vec<String> {
        let registry = self.plugin_manager.hook_registry();

        // Get all text filter hooks and apply them
        let filter_hooks = registry.get_by_extension_point::<TextFilter>();

        // Apply each filter to the text - no unsafe needed!
        filter_hooks
            .iter()
            .map(|hook| hook.hook().filter(text))
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

        // Uppercase plugin - using trait implementation
        let uppercase_hook = Hook::<TextFilter>::new(Box::new(UppercaseFilter));
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new("uppercase_plugin", TextFilter::id(), None),
            uppercase_hook,
        )?;

        // Reverse plugin - using trait implementation
        let reverse_hook = Hook::<TextFilter>::new(Box::new(ReverseFilter));
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new("reverse_plugin", TextFilter::id(), None),
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
