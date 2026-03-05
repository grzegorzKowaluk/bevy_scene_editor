use avian3d::PhysicsPlugins;
use avian3d::prelude::PhysicsDebugPlugin;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_enhanced_input::EnhancedInputPlugin;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Scene Editor".to_string(),
                        mode: WindowMode::Windowed,
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                        .into(),
                    ..default()
                }),
            EnhancedInputPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin
        ));
    }
}