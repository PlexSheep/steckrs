<div align="center">
  <h1>🔌 steckrs 🧩</h1>
  <p>
    A lightweight, type-safe plugin system for Rust applications and libraries
  </p>
  <br/>
  <a href="https://github.com/PlexSheep/steckrs/actions/workflows/cargo.yaml">
    <img src="https://img.shields.io/github/actions/workflow/status/PlexSheep/steckrs/cargo.yaml?label=Rust%20CI" alt="Rust CI"/>
  </a>
  <a href="https://github.com/PlexSheep/steckrs/blob/master/LICENSE">
    <img src="https://img.shields.io/crates/l/steckrs" alt="License"/>
  </a>
  <a href="https://github.com/PlexSheep/steckrs/releases">
    <img src="https://img.shields.io/github/v/release/PlexSheep/steckrs" alt="Release"/>
  </a>
  <br/>
  <a href="https://rust-lang.org">
    <img src="https://img.shields.io/badge/language-Rust-blue.svg" alt="Rust"/>
  </a>
  <a href="https://crates.io/crates/steckrs">
    <img alt="Crates.io" src="https://img.shields.io/crates/v/steckrs">
    <img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/steckrs">
  </a>
  <a href="https://docs.rs/steckrs/latest/steckrs/">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/steckrs">
  </a>
</div>

# steckrs

* [GitHub](https://github.com/PlexSheep/steckrs)
* [crates.io](https://crates.io/crates/steckrs)
* [Documentation on docs.rs](https://docs.rs/steckrs/latest/steckrs/)

A lightweight, trait-based plugin system for Rust applications.
The name "steckrs" is a wordplay combining the German word "Stecker" (meaning "plug" or "connector") and the Rust file extension (.rs).

## Features

- **Type-Safe Extension Points**: Define clear interfaces where plugins can add functionality
- **Dynamic Plugin Lifecycle**: Load, enable, disable, and unload plugins at runtime (currently only statically compiled)
- **Hook Registration**: Register implementations for extension points with proper type safety
- **Low Boilerplate**: Convenient macros make plugin implementation concise and readable
- **Minimal Dependencies**: Built with a focus on being lightweight and efficient

## Installation

### From crates.io

```bash
# in your rust/cargo project
cargo add steckrs
```

### From source

```bash
git clone https://github.com/PlexSheep/steckrs.git
cd steckrs
cargo build --release
ls ./target/release/libsteckrs.rlib # here is the rlib if you want that
```

## Quick Start

Here's a simple example of how to use steckrs to create a plugin-enabled application:

```rust
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

    // Use the plugin
    let registry = plugin_manager.hook_registry();
    let hooks = registry.get_by_extension_point::<GreeterExtension>();

    // Execute all hooks relevant for this extension point
    for hook in hooks {
        println!("{}", hook.inner().greet("World"));
    }
}
```

## Core Concepts

### Extension Points

Extension points define interfaces where plugins can add functionality. Each extension point:
- Is defined as a trait that plugins implement
- Specifies the contract that plugins must fulfill
- Provides type-safe interaction between the core application and plugins

### Plugins

Plugins are self-contained data structures that implement functionality for extension points. Each plugin:
- Has a unique identifier
- Can be enabled or disabled at runtime
- Can register multiple hooks to different extension points
- Has lifecycle methods (`on_load`, `on_unload`)

### Hooks

Hooks are implementations of extension points that plugins register. They:
- Implement the trait defined by an extension point
- Are invoked when the application calls that extension point
- Can be uniquely identified by their plugin ID, extension point, and optional discriminator

## Macros

steckrs provides several convenience macros to reduce boilerplate:

- `extension_point!` - Defines an extension point and its associated trait
- `simple_plugin!` - Creates a simple plugin with minimal boilerplate
- `register_hook!` - Registers a hook with the hook registry

## Advanced Usage

For more complex scenarios, you can implement the `Plugin` trait directly, allowing for more customized plugin behavior and state management. A good starting point would be defining a plugin with `simple_plugin!` and then expanding the macro.

## Use Cases

- **Modular Applications**: Break monolithic applications into pluggable components
- **Extensible Libraries**: Allow users to extend your library with custom functionality
- **Framework Development**: Build frameworks that can be customized without modifying core code
- **Runtime Customization**: Add or modify functionality without having to hardcode small things

## Planned features

- **Dynamic Loading**: It would be cool if plugins could be loaded from dynamic
  libraries in the future.
- **Generics Support**: Currently, handling functions with type parameters is
  difficult in the traits that the extension points implement, because they cannot
  be made into trade objects with `dyn` then.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Distributed under the LGPL License. See `LICENSE` for more information.
