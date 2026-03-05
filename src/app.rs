use bevy::prelude::*;
use bevy_enhanced_input::prelude::InputAction;
use crate::core::CorePlugin;
use crate::editor::EditorPlugin;
use crate::game::GamePlugin;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AppState>()
            .add_plugins(CorePlugin)    // window, DefaultPlugins, shared resources
            .add_plugins(GamePlugin)    // game systems, run conditions on AppState::Game
            .add_plugins(EditorPlugin);     // editor systems, run conditions on AppState::Editor

    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Game,
    Editor,
}

#[derive(InputAction)]
#[action_output(bool)]
struct ChangeMode;
