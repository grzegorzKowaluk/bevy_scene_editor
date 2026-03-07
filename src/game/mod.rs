use crate::app::AppState;
use crate::core::SceneEntity;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::scene::serde::SceneDeserializer;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), (setup_game, load_game_scene));
        app.add_systems(OnExit(AppState::Game), teardown_game);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct GameEntity;

fn load_game_scene(world: &mut World) {
    let path = std::path::Path::new("scenes/game_scene.ron");
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

fn setup_game(mut commands: Commands) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameEntity,
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        GameEntity,
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 80.0,
        ..default()
    });

    info!("Game started");
}

fn teardown_game(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    scene_entities: Query<Entity, With<SceneEntity>>,
) {
    for entity in game_entities.iter().chain(scene_entities.iter()) {
        commands.entity(entity).despawn();
    }
    info!("Game torn down");
}
