# Changelog

## [Unreleased]

## [0.1.0]

### ⛰️ Features

- Add more metadata to hook and always sort in hook registry - ([208702c](https://github.com/PlexSheep/steckrs/commit/208702c3936afe43f27397dc5bde7a4cd18308ee))
- Add more metadata to Hook - ([56db107](https://github.com/PlexSheep/steckrs/commit/56db10733d58bdc6e00b7f979de0eb01d6e35633))
- Register hooks from the plugin, not necessarily the application - ([9cef8af](https://github.com/PlexSheep/steckrs/commit/9cef8afb86c4af2bc9b911cf6548ab640feb9072))
- Add register_hook macro - ([1f76f4a](https://github.com/PlexSheep/steckrs/commit/1f76f4a4915e25532e9e58b857bcb3d467903506))
- Simple_plugin macro - ([a9eabb3](https://github.com/PlexSheep/steckrs/commit/a9eabb35d4cda4f89681090629941c248e2984ca))
- Add macro to easily define extension points - ([30cc326](https://github.com/PlexSheep/steckrs/commit/30cc326f353341b8cf2ee829e0bfb81c65b722a6))
- Register access and deregister hooks by plugin - ([aa3f597](https://github.com/PlexSheep/steckrs/commit/aa3f59746aab296ae5e74f051b1533d15ce15602))
- Integrate the new hook system into the plugin manager - ([661c4af](https://github.com/PlexSheep/steckrs/commit/661c4af32bfc65910f09118b0d1fc52491b2a54c))
- ExtensionPoint added - ([5c5adc7](https://github.com/PlexSheep/steckrs/commit/5c5adc7a55d20967afe3ff21c99e3f3b27b24ffa))
- Generic hooks - ([014be60](https://github.com/PlexSheep/steckrs/commit/014be60402b69446134ede96b789b0692c957c2d))
- Basic structure - ([8ea480e](https://github.com/PlexSheep/steckrs/commit/8ea480e877b958dd842f53663ba22ac7f60eb0f9))

### 🚜 Refactor

- Remove unused trait - ([b06332a](https://github.com/PlexSheep/steckrs/commit/b06332a5b402caadb9b7e9ec96e33bd65e24a8b7))
- Minimally update the macros - ([bf98f93](https://github.com/PlexSheep/steckrs/commit/bf98f932d4633c6683a3deb5b7579e652dc6cb12))
- Rename Hook::hook to Hook::inner to be more clear - ([f38c8e6](https://github.com/PlexSheep/steckrs/commit/f38c8e62f8a0b8863119f4c9d182309ebef9afb6))
- Remove getting plugin as any - ([02fe452](https://github.com/PlexSheep/steckrs/commit/02fe45236c18bdcc69ffe3e42858b887e77bc28b))
- Make hooks with references safe - ([8fb47f5](https://github.com/PlexSheep/steckrs/commit/8fb47f554b701e9444502ddba328f3aff9d6b956))
- Somehow pass reference to extension points - ([9045c3f](https://github.com/PlexSheep/steckrs/commit/9045c3f93c95d2bb2992d4098dfd5b28690cbd4a))
- Rename Plugin.name to Plugin.id - ([e9cf4b9](https://github.com/PlexSheep/steckrs/commit/e9cf4b972812551ca08648806a8c0ee5a11686cb))
- Store hooks by extension and include plugin info - ([bdd112d](https://github.com/PlexSheep/steckrs/commit/bdd112ddbf70b7f51e33c8a10dd4a71ac547bf39))

### 📚 Documentation

- Fix rustdoc warnings - ([754ec44](https://github.com/PlexSheep/steckrs/commit/754ec443460c8e5544f4c71cee1dd8634dbd7321))
- Add fancy readme - ([efc4f32](https://github.com/PlexSheep/steckrs/commit/efc4f322a11b4a32e291f73422e1af90a0c4befd))
- Add hello example - ([ae19b6a](https://github.com/PlexSheep/steckrs/commit/ae19b6ae061e5d1e9e0e322b3680dbc8107dcb52))
- Fix the examples in hook - ([1e33d4e](https://github.com/PlexSheep/steckrs/commit/1e33d4ece3f9c33abc20d9bf00d1b4927ddb22ab))
- Fix many doctests - ([1dfe844](https://github.com/PlexSheep/steckrs/commit/1dfe8440df0c8838f754754cad246bc87784a720))
- Fix macro doctests - ([8e4d06e](https://github.com/PlexSheep/steckrs/commit/8e4d06e95b3c4b0bc71a57f747754e1eb53ba7b5))
- Document macros module - ([df4d6d2](https://github.com/PlexSheep/steckrs/commit/df4d6d251b123a60139ffdb98ffbb5ae5f034b20))
- Document error module and remove unused variants - ([515b90b](https://github.com/PlexSheep/steckrs/commit/515b90ba5d08553c482b9d9d53d587687567b444))
- Document and refactor the hook module - ([1d512b1](https://github.com/PlexSheep/steckrs/commit/1d512b158b3136b3992691a01638f97388dba9f0))
- Document lib.rs module and refactor a tiny bit - ([e7f5aa0](https://github.com/PlexSheep/steckrs/commit/e7f5aa0b4788991297ce8d7e174e81192bf2d78b))
- Refactor command_processor example - ([5ee1455](https://github.com/PlexSheep/steckrs/commit/5ee14552f775aef2b101322568677ba57274ae9f))
- Add command_processor example - ([d8485eb](https://github.com/PlexSheep/steckrs/commit/d8485eb0d48beb578e2ae0e888a37d300726b320))

### ⚙️ Miscellaneous Tasks

- Adjust cargo.toml for release - ([e1baccc](https://github.com/PlexSheep/steckrs/commit/e1baccc42ae2c50b03f36d8052a22b3d1b93c9c5))
- Automatic Rust CI changes - ([bd258dd](https://github.com/PlexSheep/steckrs/commit/bd258dd404087dbbbd34e91869d353262112d8fa))
- Rename example - ([1b80496](https://github.com/PlexSheep/steckrs/commit/1b80496b67b252489d25ab702b9f711eda0b0f48))
- Audit ci generate lockfile - ([861cebc](https://github.com/PlexSheep/steckrs/commit/861cebc7db7261ae876358450e889e6d4aa0b511))
- Dont run cargo ci on all pushes - ([b2cd4f2](https://github.com/PlexSheep/steckrs/commit/b2cd4f25567a10d823470efdc9fd02767aa63975))
- Project setup - ([17d4a94](https://github.com/PlexSheep/steckrs/commit/17d4a949f883a6914e84c0c33f3d36fd70206db1))

