use std::fmt::Debug;

use steckrs::{error::PluginResult, extension_point, simple_plugin, PluginManager};

// Main command processor application
struct CommandProcessor {
    plugin_manager: PluginManager,
}

// Define the points where plugins can be called
extension_point!(
    /// The extension point
    CommandHandler:
    /// The trait implemented by the extension point
    CommandHandlerFunctions;
    /// document like this
    ///
    /// self is always needed
    fn can_handle(&mut self , command: &str) -> bool;
    fn handle(&mut self, command: &str, args: &[&str]) -> String;
);
extension_point!(
    ByeExtPoint: ByeExtPointF;
    fn say_bye(&self) -> String;
);

impl CommandProcessor {
    fn new() -> Self {
        Self {
            plugin_manager: PluginManager::new(),
        }
    }

    fn process_command(&mut self, input: &str) -> String {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return "Please enter a command".to_string();
        }

        let command = parts[0];
        let args = &parts[1..];

        // Get all enabled hooks (plugins could be disabled)
        let hooks = self
            .plugin_manager
            .get_enabled_hooks_by_ep_mut::<CommandHandler>();

        for (_id, hook) in hooks.into_iter() {
            // NOTE: first come first serve
            if hook.inner_mut().can_handle(command) {
                return hook.inner_mut().handle(command, args);
            }
        }

        format!("Unknown command: {}", command)
    }

    fn load_plugins(&mut self) -> PluginResult<()> {
        // Load core plugins
        self.plugin_manager
            .load_plugin(Box::new(CorePlugin::new()))?;
        self.plugin_manager
            .load_plugin(Box::new(EchoPlugin::new()))?;

        // Enable plugins
        self.plugin_manager.enable_plugin(CorePlugin::ID)?;
        self.plugin_manager.enable_plugin(EchoPlugin::ID)?;

        Ok(())
    }

    fn end(&mut self) {
        let hooks = self.plugin_manager.get_enabled_hooks_by_ep::<ByeExtPoint>();

        for (_id, hook) in hooks {
            println!("{}", hook.inner().say_bye())
        }
    }
}

simple_plugin!(
    /// Create a core plugin with basic functionality
    ///
    /// If you need a more complex struct for your plugin, please implement the Plugin trait
    /// yourself. With some clever design, this might even allow you to use the plugin datatype to
    /// run hooks directly, giving you access to your data.
    ///
    /// BTW, this is how you can document the plugin
    CorePlugin,                                // Datatype Identifier in source code
    "core_plugin",                             // PluginID
    "Core commands for the command processor", // Description
    // register hooks for your plugin
    hooks: [
        (CommandHandler, {HelpHook::default()}, "help"),         // if you register multiple hooks for an extension point
        (CommandHandler, VersionHook, "version"),   // you need to add a discriminant
        (ByeExtPoint, ByeHook)
    ]
);

// Another even simpler plugin
simple_plugin!(
    EchoPlugin,
    "echo_plugin",
    "Echoes input back to the user and is a very simple plugin",
    hooks: [(CommandHandler, EchoHook)]
);

// Define hoooks into the extension point
#[derive(Default)]
struct HelpHook {
    already_helped: bool, // you can even have local context
}
impl CommandHandlerFunctions for HelpHook {
    fn can_handle(&mut self, command: &str) -> bool {
        command == "help"
    }

    fn handle(&mut self, _command: &str, _args: &[&str]) -> String {
        if self.already_helped {
            eprintln!("okaaaaaaaay... helping agaaaaaaaaain....");
        }
        self.already_helped |= true;
        "Available commands: help, version, echo".to_string()
    }
}

// Version command handler
struct VersionHook;
impl CommandHandlerFunctions for VersionHook {
    fn can_handle(&mut self, command: &str) -> bool {
        command == "version"
    }

    fn handle(&mut self, _command: &str, _args: &[&str]) -> String {
        "Command Processor v1.0.0".to_string()
    }
}

// Echo command handler
struct EchoHook;
impl CommandHandlerFunctions for EchoHook {
    fn can_handle(&mut self, command: &str) -> bool {
        command == "echo"
    }

    fn handle(&mut self, _command: &str, args: &[&str]) -> String {
        args.join(" ")
    }
}

struct ByeHook;
impl ByeExtPointF for ByeHook {
    fn say_bye(&self) -> String {
        "さようなら".to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and initialize command processor
    let mut processor = CommandProcessor::new();
    processor.load_plugins()?;

    // Test the command processor with various commands
    let commands = [
        "help",
        "version",
        "echo Hello, Plugin System!",
        "thiscommanddoesnotexist",
        "help", // helps again, uses context of help hook
    ];

    for cmd in &commands {
        println!("Command: {}", cmd);
        println!("Response: {}", processor.process_command(cmd));
        println!("---");
    }

    // Unload a plugin to demonstrate lifecycle
    println!("Unloading the echo plugin...");
    processor.plugin_manager.unload_plugin("echo_plugin")?;

    println!("\nAfter unloading echo plugin:");
    println!("Command: echo This won't work anymore");
    println!(
        "Response: {}",
        processor.process_command("echo This won't work anymore")
    );

    println!("---");

    processor.end();

    Ok(())
}
