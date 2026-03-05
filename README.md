# bevy_scene_editor

A runtime level editor built with [Bevy](https://bevyengine.org/), designed as a hobby
project to explore Bevy's reflection system, scene serialization, and ECS-driven tooling.

## What it is

An in-game level editor that lets you place, transform, and configure entities at runtime
without leaving your game. Built on top of Bevy's `DynamicScene` system and driven by
reflection, meaning editor UI is generated automatically for any registered component.

## Status

🚧 **Work in progress — hobby project, expect rough edges and breaking changes** 🚧

Tested against Bevy 0.18.

## Features (planned / in progress)

- [ ] Enter/exit editor mode at runtime via keyboard shortcut
- [ ] Free-fly editor camera
- [ ] Place and transform entities using gizmos
- [ ] Reflection-driven component inspector
- [ ] `EditorComponent` trait for registering editor-aware components
- [ ] Asset palette organized by level
- [ ] Scene persistence via Bevy `DynamicScene` and RON
- [ ] Grid with optional snapping

## Built with

- [bevy_egui](https://github.com/mvlabat/bevy_egui) — egui integration
- [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) — reflection-driven component inspector
- [transform_gizmo_bevy](https://github.com/urholaukkarinen/transform-gizmo) — transform gizmos
- [avian3d](https://github.com/Jondolf/avian) — physics
- [bevy_asset_loader](https://github.com/NiklasEi/bevy_asset_loader) — asset management
- [bevy_enhanced_input](https://github.com/projectharmonia/bevy_enhanced_input) — input handling
- [bevy_panorbit_camera](https://crates.io/crates/bevy_panorbit_camera) - one of planned cameras

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.