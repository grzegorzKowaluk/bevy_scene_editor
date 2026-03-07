pub mod camera;
pub mod ui;

use crate::app::{AppState, EditorSystems};
use crate::core::{PrimitiveType, SceneEntity};
use crate::editor::camera::{camera_bundle, EditorCamera, EditorCameraState};
use crate::editor::ui::UiState;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::scene::serde::SceneDeserializer;
use bevy_inspector_egui::bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default());
        app.add_plugins(DefaultInspectorConfigPlugin);

        app.add_plugins((camera::plugin, ui::plugin));

        app.add_observer(spawn_primitive_observer);
        app.add_observer(save_editor_scene_observer);
        app.add_observer(save_game_scene_observer);

        app.add_systems(OnEnter(AppState::Editor), (setup_editor, load_editor_scene));
        app.add_systems(OnExit(AppState::Editor), teardown_editor);

        app.add_systems(Update, handle_editor_inputs.in_set(EditorSystems::Input));
    }
}

// Marker component for the editor entities
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct EditorEntity;

#[derive(Event, Clone, Copy)]
struct SaveEditorScene;

#[derive(Event, Clone, Copy)]
struct SaveGameScene;

/// Event sent by the palette UI and consumed by `spawn_primitive_system`.
/// Keeping this as an event (rather than calling spawn directly from the UI)
/// means the UI code stays inside `show_ui_system` (exclusive world access)
/// while the actual spawn happens in a normal system with asset resources.
#[derive(Event, Clone, Copy)]
struct SpawnPrimitive {
    primitive_kind: PrimitiveKind,
}

#[derive(Clone, Copy)]
enum PrimitiveKind {
    Cube,
    Sphere,
    Plane,
    Cylinder,
}

fn save_editor_scene_observer(
    _trigger: On<SaveEditorScene>,
    world: &World,
    scene_entities: Query<Entity, With<SceneEntity>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let entities: Vec<Entity> = scene_entities.iter().collect();

    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all()
        .allow_component::<Transform>()
        .allow_component::<Name>()
        .allow_component::<PrimitiveType>()
        .allow_component::<SceneEntity>()
        .extract_entities(entities.iter().copied())
        .build();

    let type_registry = type_registry.read();
    match scene.serialize(&type_registry) {
        Ok(ron) => {
            let path = "scenes/editor_scene.ron";
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, ron).unwrap();
            info!("Scene saved");
        }
        Err(e) => error!("Failed to serialize scene: {}", e),
    }
}

fn save_game_scene_observer(
    _trigger: On<SaveGameScene>,
    world: &World,
    scene_entities: Query<Entity, With<SceneEntity>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let entities: Vec<Entity> = scene_entities.iter().collect();

    let scene = DynamicSceneBuilder::from_world(world)
        .deny_all()
        .allow_component::<Transform>()
        .allow_component::<Name>()
        .allow_component::<PrimitiveType>()
        .allow_component::<SceneEntity>()
        .extract_entities(entities.iter().copied())
        .build();

    let type_registry = type_registry.read();
    match scene.serialize(&type_registry) {
        Ok(ron) => {
            let path = "scenes/game_scene.ron";
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, ron).unwrap();
            info!("Game Scene saved");
        }
        Err(e) => error!("Failed to serialize scene: {}", e),
    }
}

fn load_editor_scene(world: &mut World) {
    let path = std::path::Path::new("scenes/editor_scene.ron");
    if !path.exists() {
        return;
    }

    let ron = std::fs::read_to_string(path).unwrap();
    let type_registry = world.resource::<AppTypeRegistry>().0.clone();

    let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
    let scene_deserializer = SceneDeserializer {
        type_registry: &type_registry.read(),
    };

    match scene_deserializer.deserialize(&mut deserializer) {
        Ok(scene) => {
            let mut entity_map = EntityHashMap::default();
            scene.write_to_world(world, &mut entity_map).unwrap();
            info!("Editor Scene loaded");
        }
        Err(e) => error!("Failed to deserialize scene: {}", e),
    }
}

fn setup_editor(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    egui_global_settings.auto_create_primary_context = false;

    commands.spawn(camera_bundle());

    commands.spawn((
        Camera2d,
        Name::new("Egui Camera"),
        PrimaryEguiContext,
        EditorEntity,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        ..default()
    });
    commands.insert_resource(UiState::new());
    commands.insert_resource(EditorCameraState::default());
}

fn teardown_editor(
    mut commands: Commands,
    editor_entities: Query<Entity, With<EditorEntity>>,
    scene_entities: Query<Entity, With<SceneEntity>>,
) {
    for entity in editor_entities.iter().chain(scene_entities.iter()) {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<UiState>();
    commands.remove_resource::<EditorCameraState>();
    info!("Editor torn down");
}

fn handle_editor_inputs(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft);
    let shift = keyboard.pressed(KeyCode::ShiftLeft);

    if ctrl && !shift && keyboard.just_pressed(KeyCode::KeyS) {
        commands.trigger(SaveEditorScene);
    }
    if ctrl && shift && keyboard.just_pressed(KeyCode::KeyS) {
        commands.trigger(SaveGameScene);
    }
}

fn spawn_primitive_observer(
    spawn_event: On<SpawnPrimitive>,
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
) {
    let event = spawn_event.event();

    let name = match event.primitive_kind {
        PrimitiveKind::Cube => "Cube",
        PrimitiveKind::Sphere => "Sphere",
        PrimitiveKind::Plane => "Plane",
        PrimitiveKind::Cylinder => "Cylinder",
    };

    let primitive_type = match event.primitive_kind {
        PrimitiveKind::Cube => PrimitiveType::Cube,
        PrimitiveKind::Sphere => PrimitiveType::Sphere,
        PrimitiveKind::Plane => PrimitiveType::Plane,
        PrimitiveKind::Cylinder => PrimitiveType::Cylinder,
    };

    let entity = commands
        .spawn((
            primitive_type,
            Transform::default(),
            SceneEntity,
            Name::new(name),
        ))
        .id();

    ui_state.selected_entities.select_replace(entity);

    info!("Spawned {} at world origin", name);
}
