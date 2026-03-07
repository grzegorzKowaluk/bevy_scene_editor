use crate::app::{AppState, EditorSystems};
use crate::editor::camera::{extract_camera_matrices, EditorCamera};
use crate::editor::{PrimitiveKind, SpawnPrimitive};
use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::math::{DQuat, DVec3};
use bevy::picking::pointer::{PointerAction, PointerInput, PointerInteraction};
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext};
use bevy_inspector_egui::bevy_inspector::hierarchy::{hierarchy_ui, SelectedEntities};
use bevy_inspector_egui::bevy_inspector::{
    ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::egui::LayerId;
use bevy_inspector_egui::{bevy_inspector, egui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::any::TypeId;
use transform_gizmo_egui::math::Transform as GizmoTransform;
use transform_gizmo_egui::{EnumSet, GizmoExt, GizmoMode, GizmoOrientation};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiPrimaryContextPass,
        show_ui_system.run_if(in_state(AppState::Editor)),
    );
    app.add_systems(Update, (handle_pick_events.in_set(EditorSystems::Picking),));
}

fn handle_pick_events(
    mut ui_state: ResMut<UiState>,
    mut click_events: MessageReader<PointerInput>,
    pointers: Query<&PointerInteraction>,
    button: Res<ButtonInput<KeyCode>>,
) {
    if !ui_state.pointer_in_viewport || ui_state.gizmo_active {
        return;
    }
    for event in click_events.read() {
        if let PointerAction::Press(PointerButton::Primary) = event.action {
            let any_hits = pointers
                .iter()
                .any(|interaction| !interaction.as_slice().is_empty());

            if any_hits {
                for interaction in pointers.iter() {
                    for (entity, _hit) in interaction.as_slice() {
                        let add = button.any_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft]);
                        ui_state.selected_entities.select_maybe_add(*entity, add);
                    }
                }
            } else {
                ui_state.selected_entities.clear();
            }
        }
    }
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource)]
pub struct UiState {
    state: DockState<EguiWindow>,
    pub viewport_rect: egui::Rect,
    pub selected_entities: SelectedEntities,
    selection: InspectorSelection,
    pub pointer_in_viewport: bool,
    gizmo: transform_gizmo_egui::Gizmo,
    gizmo_modes: EnumSet<GizmoMode>,
    gizmo_orientation: GizmoOrientation,
    gizmo_active: bool,
}

impl UiState {
    pub fn new() -> Self {
        let mut state = DockState::new(vec![EguiWindow::GameView]);
        let tree = state.main_surface_mut();
        let [game, _inspector] = tree.split_right(
            NodeIndex::root(),
            0.75,
            vec![EguiWindow::Inspector, EguiWindow::Palette],
        );
        let [game, _hierarchy] = tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
        let [_game, _bottom] =
            tree.split_below(game, 0.8, vec![EguiWindow::Resources, EguiWindow::Assets]);

        Self {
            state,
            selected_entities: SelectedEntities::default(),
            selection: InspectorSelection::Entities,
            viewport_rect: egui::Rect::NOTHING,
            pointer_in_viewport: false,
            gizmo: transform_gizmo_egui::Gizmo::default(),
            gizmo_modes: GizmoMode::all(),
            gizmo_orientation: GizmoOrientation::Local,
            gizmo_active: false,
        }
    }

    fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
            pointer_in_viewport: &mut self.pointer_in_viewport,
            gizmo: &mut self.gizmo,
            gizmo_modes: &mut self.gizmo_modes,
            gizmo_orientation: &mut self.gizmo_orientation,
            gizmo_active: &mut self.gizmo_active,
        };
        DockArea::new(&mut self.state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

#[derive(Debug)]
enum EguiWindow {
    GameView,
    Hierarchy,
    Resources,
    Assets,
    Inspector,
    Palette,
}

struct TabViewer<'a> {
    world: &'a mut World,
    selected_entities: &'a mut SelectedEntities,
    selection: &'a mut InspectorSelection,
    viewport_rect: &'a mut egui::Rect,
    pointer_in_viewport: &'a mut bool,
    gizmo: &'a mut transform_gizmo_egui::Gizmo,
    gizmo_modes: &'a mut EnumSet<GizmoMode>,
    gizmo_orientation: &'a mut GizmoOrientation,
    gizmo_active: &'a mut bool,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = EguiWindow;

    fn title(&mut self, window: &mut Self::Tab) -> egui::WidgetText {
        format!("{window:?}").into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, window: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();

        match window {
            EguiWindow::GameView => {
                *self.viewport_rect = ui.clip_rect();
                self.draw_gizmo(ui);
            }
            EguiWindow::Hierarchy => {
                let selected = hierarchy_ui(self.world, ui, self.selected_entities);
                if selected {
                    *self.selection = InspectorSelection::Entities;
                }
            }
            EguiWindow::Resources => select_resource(ui, &type_registry, self.selection),
            EguiWindow::Assets => select_asset(ui, &type_registry, self.world, self.selection),
            EguiWindow::Inspector => match *self.selection {
                InspectorSelection::Entities => match self.selected_entities.as_slice() {
                    &[entity] => ui_for_entity_with_children(self.world, entity, ui),
                    entities => ui_for_entities_shared_components(self.world, entities, ui),
                },
                InspectorSelection::Resource(type_id, ref name) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_resource(
                        self.world,
                        type_id,
                        ui,
                        name,
                        &type_registry,
                    )
                }
                InspectorSelection::Asset(type_id, ref name, handle) => {
                    ui.label(name);
                    bevy_inspector::by_type_id::ui_for_asset(
                        self.world,
                        type_id,
                        handle,
                        ui,
                        &type_registry,
                    );
                }
            },
            EguiWindow::Palette => self.draw_palette(ui),
        }

        *self.pointer_in_viewport = ui
            .ctx()
            .rect_contains_pointer(LayerId::background(), self.viewport_rect.shrink(16.));
    }

    fn clear_background(&self, window: &Self::Tab) -> bool {
        !matches!(window, EguiWindow::GameView)
    }
}

impl TabViewer<'_> {
    fn draw_palette(&mut self, ui: &mut egui::Ui) {
        ui.heading("Primitives");
        ui.separator();

        let primitives: &[(&str, PrimitiveKind)] = &[
            ("⬛  Cube", PrimitiveKind::Cube),
            ("⬤  Sphere", PrimitiveKind::Sphere),
            ("▬  Plane", PrimitiveKind::Plane),
            ("⬤  Cylinder", PrimitiveKind::Cylinder),
        ];

        for (label, primitive_type) in primitives {
            if ui
                .add_sized([ui.available_width(), 32.0], egui::Button::new(*label))
                .clicked()
            {
                self.world.commands().trigger(SpawnPrimitive {
                    primitive_kind: *primitive_type,
                });
            }
        }
    }

    fn draw_gizmo(&mut self, ui: &mut egui::Ui) {
        // Get the selected entity — only show gizmo for a single selection
        let selected_entity = match self.selected_entities.as_slice() {
            &[entity] => entity,
            _ => return,
        };

        // Extract camera matrices from the world
        let camera_data = self
            .world
            .query_filtered::<(&GlobalTransform, &Camera), With<EditorCamera>>()
            .single(self.world);

        let Ok((camera_global_transform, camera)) = camera_data else {
            return;
        };

        let (view_matrix, projection_matrix) =
            extract_camera_matrices(camera_global_transform, camera);

        let viewport = ui.clip_rect();
        let snapping = ui.input(|input| input.modifiers.ctrl);

        self.gizmo.update_config(transform_gizmo_egui::GizmoConfig {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            viewport,
            modes: *self.gizmo_modes,
            orientation: *self.gizmo_orientation,
            snapping,
            ..Default::default()
        });

        // Read the entity's current transform
        let Ok(bevy_transform) = self
            .world
            .query::<&Transform>()
            .get(self.world, selected_entity)
        else {
            return;
        };
        let bevy_transform = *bevy_transform;

        let gizmo_transform = GizmoTransform::from_scale_rotation_translation(
            DVec3::new(
                bevy_transform.scale.x as f64,
                bevy_transform.scale.y as f64,
                bevy_transform.scale.z as f64,
            ),
            DQuat::from_xyzw(
                bevy_transform.rotation.x as f64,
                bevy_transform.rotation.y as f64,
                bevy_transform.rotation.z as f64,
                bevy_transform.rotation.w as f64,
            ),
            DVec3::new(
                bevy_transform.translation.x as f64,
                bevy_transform.translation.y as f64,
                bevy_transform.translation.z as f64,
            ),
        );

        *self.gizmo_active = self
            .gizmo
            .interact(ui, &[gizmo_transform])
            .map(|(_, new_transforms)| {
                if let Some(new_t) = new_transforms.first() {
                    if let Ok(mut transform) = self
                        .world
                        .query::<&mut Transform>()
                        .get_mut(self.world, selected_entity)
                    {
                        transform.translation = Vec3::new(
                            new_t.translation.x as f32,
                            new_t.translation.y as f32,
                            new_t.translation.z as f32,
                        );
                        transform.rotation = Quat::from_xyzw(
                            new_t.rotation.v.x as f32,
                            new_t.rotation.v.y as f32,
                            new_t.rotation.v.z as f32,
                            new_t.rotation.s as f32,
                        );
                        transform.scale = Vec3::new(
                            new_t.scale.x as f32,
                            new_t.scale.y as f32,
                            new_t.scale.z as f32,
                        );
                    }
                }
                true
            })
            .unwrap_or(false)
            || self.gizmo.is_focused();
    }
}

fn select_resource(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    selection: &mut InspectorSelection,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    for (resource_name, type_id) in resources {
        let selected = match *selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            *selection = InspectorSelection::Resource(type_id, resource_name.to_string());
        }
    }
}

fn select_asset(
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
    world: &World,
    selection: &mut InspectorSelection,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match *selection {
                    InspectorSelection::Asset(_, _, selected_id) => selected_id == handle,
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{handle:?}"))
                    .clicked()
                {
                    *selection =
                        InspectorSelection::Asset(asset_type_id, asset_name.to_string(), handle);
                }
            }
        });
    }
}
