use crate::app::{AppState, EditorSystems};
use crate::editor::ui::UiState;
use crate::editor::EditorEntity;
use bevy::camera::Viewport;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::math::DMat4;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::bevy_egui::EguiContextSettings;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        set_camera_viewport.run_if(in_state(AppState::Editor)),
    );

    app.add_systems(Update, handle_camera_input.in_set(EditorSystems::Input));
}

// Marker component for the editor camera
#[derive(Component)]
pub struct EditorCamera;

#[derive(Resource)]
pub struct EditorCameraState {
    pub pivot: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for EditorCameraState {
    fn default() -> Self {
        Self {
            pivot: Vec3::ZERO,
            distance: 10.0,
            yaw: 0.0,
            pitch: -0.3,
        }
    }
}

pub fn camera_bundle() -> impl Bundle {
    (
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        EditorCamera,
        EditorEntity,
        Name::new("Editor Camera"),
    )
}

fn handle_camera_input(
    ui_state: Res<UiState>,
    button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    motion_input: Res<AccumulatedMouseMotion>,
    scroll_input: Res<AccumulatedMouseScroll>,
    mut state: ResMut<EditorCameraState>,
    mut camera: Single<&mut Transform, With<EditorCamera>>,
) {
    if !ui_state.pointer_in_viewport {
        return;
    }

    let middle = button_input.pressed(MouseButton::Middle);
    let shift = keyboard_input.pressed(KeyCode::ShiftLeft);
    let motion_delta = motion_input.delta;

    if middle && !shift && motion_delta != Vec2::ZERO {
        // Orbit
        let sensitivity = 0.005;
        state.yaw -= motion_delta.x * sensitivity;
        state.pitch -= motion_delta.y * sensitivity;
        state.pitch = state.pitch.clamp(-1.5, 1.5);
    }

    if middle && shift && motion_delta != Vec2::ZERO {
        // Pan
        let sensitivity = state.distance * 0.001;
        let right = camera.rotation * Vec3::X;
        let up = camera.rotation * Vec3::Y;
        state.pivot -= right * motion_delta.x * sensitivity;
        state.pivot += up * motion_delta.y * sensitivity;
    }

    if scroll_input.delta != Vec2::ZERO {
        let amount = match scroll_input.unit {
            MouseScrollUnit::Line => scroll_input.delta.y * 0.5,
            MouseScrollUnit::Pixel => scroll_input.delta.y * 0.01,
        };
        state.distance = (state.distance - amount).max(0.1);
    }

    // Reconstruct camera transform from spherical coordinates
    let rotation = Quat::from_rotation_y(state.yaw) * Quat::from_rotation_x(state.pitch);
    let offset = rotation * Vec3::new(0.0, 0.0, state.distance);
    camera.translation = state.pivot + offset;
    camera.rotation = rotation;
}

pub fn set_camera_viewport(
    ui_state: Res<UiState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam: Single<&mut Camera, With<EditorCamera>>,
    egui_settings: Single<&EguiContextSettings>,
) {
    let scale_factor = window.scale_factor() * egui_settings.scale_factor;

    let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position = UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size = UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    let rect = physical_position + physical_size;
    let window_size = window.physical_size();

    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    }
}

pub fn extract_camera_matrices(
    camera_transform: &GlobalTransform,
    camera: &Camera,
) -> (DMat4, DMat4) {
    let m = camera_transform.to_matrix().inverse();
    let view_matrix = DMat4::from_cols_array(&m.to_cols_array().map(|v| v as f64));

    let m2 = camera.clip_from_view();
    let projection_matrix = DMat4::from_cols_array(&m2.to_cols_array().map(|v| v as f64));

    (view_matrix, projection_matrix)
}
