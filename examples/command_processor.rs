use std::fmt::Debug;

use steckrs::{
    error::PluginResult,
    extension_point,
    hook::{ExtensionPoint, Hook, HookID},
    simple_plugin, Plugin, PluginID, PluginManager,
};

// Define a command processor extension point
extension_point!(CommandHandler: CommandHandlerFunctions,
    fn can_handle(&self, command: &str) -> bool,
    fn handle(&self, command: &str, args: &[&str]) -> String,
);

// Simple help command handler
struct HelpCommandHandler;
impl CommandHandlerFunctions for HelpCommandHandler {
    fn can_handle(&self, command: &str) -> bool {
        command == "help"
    }

    fn handle(&self, _command: &str, _args: &[&str]) -> String {
        "Available commands: help, version, echo".to_string()
    }
}

// Version command handler
struct VersionCommandHandler;
impl CommandHandlerFunctions for VersionCommandHandler {
    fn can_handle(&self, command: &str) -> bool {
        command == "version"
    }

    fn handle(&self, _command: &str, _args: &[&str]) -> String {
        "Command Processor v1.0.0".to_string()
    }
}

// Echo command handler
struct EchoCommandHandler;
impl CommandHandlerFunctions for EchoCommandHandler {
    fn can_handle(&self, command: &str) -> bool {
        command == "echo"
    }

    fn handle(&self, _command: &str, args: &[&str]) -> String {
        args.join(" ")
    }
}

// Main command processor application
struct CommandProcessor {
    plugin_manager: PluginManager,
}

impl CommandProcessor {
    fn new() -> Self {
        Self {
            plugin_manager: PluginManager::new(),
        }
    }

    fn process_command(&self, input: &str) -> String {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return "Please enter a command".to_string();
        }

        let command = parts[0];
        let args = &parts[1..];

        // Find a handler that can process this command
        let registry = self.plugin_manager.hook_registry();
        let command_handlers = registry.get_by_extension_point::<CommandHandler>();

        for handler in command_handlers {
            // NOTE: first come first serve
            if handler.hook().can_handle(command) {
                return handler.hook().handle(command, args);
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
        self.plugin_manager.enable_plugin("core_plugin")?;
        self.plugin_manager.enable_plugin("echo_plugin")?;

        // Register hooks
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new(CorePlugin::ID, CommandHandler::id(), Some("help")),
            Hook::<CommandHandler>::new(Box::new(HelpCommandHandler)),
        )?;
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new(CorePlugin::ID, CommandHandler::id(), Some("version")),
            Hook::<CommandHandler>::new(Box::new(VersionCommandHandler)),
        )?;
        self.plugin_manager.hook_registry_mut().register(
            &HookID::new(EchoPlugin::ID, CommandHandler::id(), None),
            Hook::<CommandHandler>::new(Box::new(EchoCommandHandler)),
        )?;

        Ok(())
    }
}

// Create a core plugin with basic functionality
simple_plugin!(
    CorePlugin,
    "core_plugin",
    "Core commands for the command processor"
);

// Create a core plugin with basic functionality
simple_plugin!(
    EchoPlugin,
    "echo_plugin",
    "Echoes input back to the user and is a very simple plugin"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and initialize command processor
    let mut processor = CommandProcessor::new();
    processor.load_plugins()?;

    // Test the command processor with various commands
    let commands = [
        "help",
        "version",
        "echo Hello, Plugin System!",
        "unknown command",
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

    Ok(())
}
