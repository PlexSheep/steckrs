# Changelog

## [0.3.4](https://github.com/PlexSheep/steckrs/compare/v0.3.3...v0.3.4)

### ⛰️ Features

- Add plugin_is_enabled method for manager - ([3912944](https://github.com/PlexSheep/steckrs/commit/391294439185fa00aad80f8e6b59ce400936f4d9))
- Add PluginIDOwned and optional serde support - ([a622555](https://github.com/PlexSheep/steckrs/commit/a62255515f3201b4e60716848ef3fbfae816482d))

### 🚜 Refactor

- PluginIdOwned now only has a &'static str inside - ([ff54dd8](https://github.com/PlexSheep/steckrs/commit/ff54dd83a0074a7072057b9bee48a54542f3c045))

### 📚 Documentation

- Document PluginIDOwned - ([6c53d6d](https://github.com/PlexSheep/steckrs/commit/6c53d6d774d9402e78149fcaca0b2e612a8dfb82))


## [0.3.3](https://github.com/PlexSheep/steckrs/compare/v0.3.2...v0.3.3)

### ⛰️ Features

- Automatically document the new method with simple_plugin - ([f4f2c66](https://github.com/PlexSheep/steckrs/commit/f4f2c66cc32f4d4199c6d9434cb1c7644d597355))


## [0.3.2](https://github.com/PlexSheep/steckrs/compare/v0.3.1...v0.3.2)

### ⛰️ Features

- Introduce mutable versions of all get-hook-functions - ([3bb68d4](https://github.com/PlexSheep/steckrs/commit/3bb68d46cdef81d105d77390f8bb748d4cb80592))

### 🐛 Bug Fixes

- Macros now accept datatypes that are not unit structs - ([a25ba29](https://github.com/PlexSheep/steckrs/commit/a25ba29b741f06f368dabf703eaf2b08ebb7c130))

### 📚 Documentation

- Adjust return types for mutable functions - ([c409e82](https://github.com/PlexSheep/steckrs/commit/c409e829eb9edc1042efadba16b0e944a3916c2a))
- Example now shows a mutable hook too - ([e865dd7](https://github.com/PlexSheep/steckrs/commit/e865dd794628594f64fbef55a99204898414ab79))


## [0.3.1](https://github.com/PlexSheep/steckrs/compare/v0.3.0...v0.3.1)

### 📚 Documentation

- The readme example was outdated again - ([81bb6b2](https://github.com/PlexSheep/steckrs/commit/81bb6b2b59780afce8b2dac906076ea27d1a363c))


## [0.3.0](https://github.com/PlexSheep/steckrs/compare/v0.2.0...v0.3.0)

### 🚜 Refactor

- [**breaking**] Return both hook and hook id - ([a62768f](https://github.com/PlexSheep/steckrs/commit/a62768f9de3e89d8554dab4075802e1681bd9f6a))

### 📚 Documentation

- Fix docs examples as the id was added, docs for new function - ([fc23fa0](https://github.com/PlexSheep/steckrs/commit/fc23fa05e8445659460db42fe986f4cda537dead))
- Fix examples - ([f679e3f](https://github.com/PlexSheep/steckrs/commit/f679e3ffe873aadb3227f53a0d1ea60cd91ef802))
- Typo in readme - ([70a0dbb](https://github.com/PlexSheep/steckrs/commit/70a0dbbac0da00f00ccd3fceb59f5c14cf762d48))
- License note in readme had the wrong license - ([eaab58a](https://github.com/PlexSheep/steckrs/commit/eaab58acd63d33880d0ae3844d640e9150353dfc))
- Example in readme was outdated - ([6261d4c](https://github.com/PlexSheep/steckrs/commit/6261d4cce46fe7efa511cd13e24d16206cb9d699))

### ⚙️ Miscellaneous Tasks

- Automatic Rust CI changes - ([f34d9cd](https://github.com/PlexSheep/steckrs/commit/f34d9cde47ac0411c20992f24dd2b0c575eb566c))


## [0.2.0](https://github.com/PlexSheep/steckrs/compare/v0.1.1...v0.2.0)

### 🚀 Features

- [**breaking**] Update extension_point and simple_plugin macro to be more flexible and allow doc comments - ([124d44](https://github.com/PlexSheep/steckrs/commit/124d44ca1f3750c8866ee00834c217c56f3d99c3))

### 📚 Documentation

- Note that generics are difficult in the readme #7 - ([4a46a17](https://github.com/PlexSheep/steckrs/commit/4a46a177b4d08946b12bf94d12e600befe5d8090))
- Set license to LGPL-3.0-or-later - ([e834db6](https://github.com/PlexSheep/steckrs/commit/e834db6905b7db72f0a8676099409d997a41fb4f))
- Fix usages of macros in doc examples for changes to the macros - ([69cf19a](https://github.com/PlexSheep/steckrs/commit/69cf19a617477bcf94b3786fb2394a125da622f8))

### ⚙️ Miscellaneous Tasks

- Try to fix release-plz - ([566e2ed](https://github.com/PlexSheep/steckrs/commit/566e2ed1ce98711b76ca5525f114869addc02a4c))
- Super small change to trigger the ci/cd - ([5b61cc6](https://github.com/PlexSheep/steckrs/commit/5b61cc62d207957130a8bc4893ab6bf59dc96994))
- Remove commented out example code - ([775d868](https://github.com/PlexSheep/steckrs/commit/775d868c7b913c2cc461b38538e9c09768922227))

