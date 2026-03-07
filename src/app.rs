use crate::core::CorePlugin;
use crate::editor::EditorPlugin;
use crate::game::GamePlugin;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::InputAction;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CorePlugin); // window, DefaultPlugins, shared resources
        app.add_plugins(GamePlugin); // game systems, run conditions on AppState::Game
        app.add_plugins(EditorPlugin); // editor systems, run conditions on AppState::Editor
        app.init_state::<AppState>();
        app.add_systems(Update, toggle_editor);

        app.configure_sets(
            Update,
            (
                GameSystems::Input,
                GameSystems::TickTimers,
                GameSystems::Update,
                GameSystems::PostUpdate,
            )
                .chain()
                .run_if(in_state(AppState::Game)),
        );

        app.configure_sets(
            Update,
            (
                EditorSystems::Input,
                EditorSystems::Camera,
                EditorSystems::Picking,
                EditorSystems::Update,
                EditorSystems::SceneSync,
                EditorSystems::Ui,
            )
                .chain()
                .run_if(in_state(AppState::Editor)),
        );
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Game,
    Editor,
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum GameSystems {
    Input,      // read player input
    TickTimers, // advance timers before logic reads them
    Update,     // main game logic
    PostUpdate, // reactions to state changes, camera follow, etc
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum EditorSystems {
    Input,     // process editor shortcuts, camera movement input
    Camera,    // apply camera movement based on input
    Picking,   // raycasting, click to select
    Update,    // general editor logic, gizmo target management
    SceneSync, // serialize/deserialize working copy
    Ui,        // egui, always last
}

fn toggle_editor(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    let e = keyboard.just_pressed(KeyCode::KeyE);

    if ctrl && e {
        match state.get() {
            AppState::Game => next_state.set(AppState::Editor),
            AppState::Editor => next_state.set(AppState::Game),
        }
    }
}
