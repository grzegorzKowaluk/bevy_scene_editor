use avian3d::prelude::PhysicsDebugPlugin;
use avian3d::PhysicsPlugins;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_enhanced_input::EnhancedInputPlugin;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
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
        );
        app.add_plugins(MeshPickingPlugin);
        app.add_plugins(EnhancedInputPlugin);
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(PhysicsDebugPlugin);

        app.add_systems(Update, restore_primitive_meshes);
    }
}

/// Marker for entities that belong to the scene and should be serialized.
/// Every primitive placed from the palette gets this component.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SceneEntity;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub enum PrimitiveType {
    #[default]
    Cube,
    Sphere,
    Plane,
    Cylinder,
}

pub fn restore_primitive_meshes(
    query: Query<(Entity, &PrimitiveType), Without<Mesh3d>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, primitive_type) in query.iter() {
        let mesh = match primitive_type {
            PrimitiveType::Cube => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            PrimitiveType::Sphere => meshes.add(Sphere::new(0.5)),
            PrimitiveType::Plane => meshes.add(Plane3d::default().mesh().size(1.0, 1.0)),
            PrimitiveType::Cylinder => meshes.add(Cylinder::new(0.5, 1.0)),
        };
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.8, 0.8),
                ..default()
            })),
        ));
    }
}
