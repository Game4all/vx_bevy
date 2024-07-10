use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::{
        Color, EventReader, IntoSystemConfigs, IntoSystemSetConfigs, KeyCode, Plugin, Res, ResMut,
        Resource, SystemSet, Update,
    },
};

use bevy_egui::{
    egui::{self, Rgba, Slider},
    EguiContexts, EguiPlugin, EguiSet,
};

use crate::voxel::{
    material::VoxelMaterialRegistry, ChunkCommandQueue, ChunkEntities, ChunkLoadRadius,
    CurrentLocalPlayerChunk, DirtyChunks,
};

fn display_debug_stats(mut egui: EguiContexts, diagnostics: Res<DiagnosticsStore>) {
    egui::Window::new("performance stuff").show(egui.ctx_mut(), |ui| {
        ui.label(format!(
            "Avg. FPS: {:.02}",
            diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!(
            "Total Entity count: {}",
            diagnostics
                .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
    });
}

fn display_chunk_stats(
    mut egui: EguiContexts,
    dirty_chunks: Res<DirtyChunks>,
    player_pos: Res<CurrentLocalPlayerChunk>,
    mut chunk_loading_radius: ResMut<ChunkLoadRadius>,
    mut chunk_command_queue: ResMut<ChunkCommandQueue>,
    loaded_chunks: Res<ChunkEntities>,
) {
    egui::Window::new("voxel world stuff").show(egui.ctx_mut(), |ui| {
        ui.heading("Chunks");
        ui.label(format!(
            "Chunks invalidations (per frame):  {}",
            dirty_chunks.num_dirty()
        ));
        ui.label(format!("Loaded chunk count: {}", loaded_chunks.len()));
        ui.separator();
        ui.label("Horizontal chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.horizontal, 8..=32));
        ui.label("Vertical chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.vertical, 2..=10));
        ui.separator();

        if ui.button("Clear loaded chunks").clicked() {
            chunk_command_queue.queue_unload(loaded_chunks.iter_keys());
        }
        ui.separator();

        ui.heading("Current player position");
        ui.label(format!("Current position : {}", player_pos.world_pos));
        ui.label(format!("Current chunk : {:?}", player_pos.chunk_min));
    });
}

fn display_debug_ui_criteria(ui_state: Res<DebugUIState>) -> bool {
    ui_state.display_debug_info
}

fn display_mat_debug_ui_criteria(ui_state: Res<DebugUIState>) -> bool {
    ui_state.display_mat_debug
}

fn toggle_debug_ui_displays(
    mut inputs: EventReader<KeyboardInput>,
    mut ui_state: ResMut<DebugUIState>,
) {
    for input in inputs.read() {
        match input.key_code {
            KeyCode::F3 if input.state == ButtonState::Pressed => {
                ui_state.display_debug_info = !ui_state.display_debug_info;
            }
            KeyCode::F7 if input.state == ButtonState::Pressed => {
                ui_state.display_mat_debug = !ui_state.display_mat_debug;
            }
            _ => {}
        }
    }
}

fn display_material_editor(
    mut egui: EguiContexts,
    mut ui_state: ResMut<DebugUIState>,
    mut materials: ResMut<VoxelMaterialRegistry>,
) {
    egui::Window::new("material editor").show(egui.ctx_mut(), |ui| {
        ui.heading("Select material");
        egui::containers::ComboBox::from_label("Material")
            .selected_text(
                materials
                    .get_by_id(ui_state.selected_mat)
                    .unwrap()
                    .name
                    .to_string(),
            )
            .show_ui(ui, |content| {
                materials
                    .iter_mats()
                    .enumerate()
                    .for_each(|(mat_index, mat)| {
                        content.selectable_value(
                            &mut ui_state.selected_mat,
                            mat_index as u8,
                            mat.name,
                        );
                    })
            });

        ui.heading("Material properties");

        // base_color
        ui.label("Base color");

        let selected_mat = materials.get_mut_by_id(ui_state.selected_mat).unwrap();

        let mut editable_color = Rgba::from_rgba_unmultiplied(
            selected_mat.base_color.r(),
            selected_mat.base_color.g(),
            selected_mat.base_color.b(),
            selected_mat.base_color.a(),
        );
        egui::widgets::color_picker::color_edit_button_rgba(
            ui,
            &mut editable_color,
            egui::color_picker::Alpha::Opaque,
        );
        selected_mat.base_color = Color::rgba_from_array(editable_color.to_array());
        ui.label("Perceptual Roughness");
        ui.add(Slider::new(
            &mut selected_mat.perceptual_roughness,
            0.0..=1.0f32,
        ));
        ui.label("Metallic");
        ui.add(Slider::new(&mut selected_mat.metallic, 0.0..=1.0f32));
        ui.label("Reflectance");
        ui.add(Slider::new(&mut selected_mat.reflectance, 0.0..=1.0f32));
        ui.label("Emissive");

        let mut editable_emissive = Rgba::from_rgba_unmultiplied(
            selected_mat.emissive.r(),
            selected_mat.emissive.g(),
            selected_mat.emissive.b(),
            selected_mat.emissive.a(),
        );
        egui::widgets::color_picker::color_edit_button_rgba(
            ui,
            &mut editable_emissive,
            egui::color_picker::Alpha::Opaque,
        );
        selected_mat.emissive = Color::rgba_from_array(editable_emissive.to_array());
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
/// Systems related to the debug UIs.
pub enum DebugUISet {
    Toggle,
    Display,
}

pub struct DebugUIPlugins;

impl Plugin for DebugUIPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(EntityCountDiagnosticsPlugin)
            .add_systems(
                Update,
                (
                    toggle_debug_ui_displays.in_set(DebugUISet::Toggle),
                    display_material_editor
                        .in_set(DebugUISet::Display)
                        .run_if(display_mat_debug_ui_criteria),
                ),
            )
            .add_systems(
                Update,
                (display_debug_stats, display_chunk_stats)
                    .in_set(DebugUISet::Display)
                    .distributive_run_if(display_debug_ui_criteria),
            )
            .configure_sets(
                Update,
                (DebugUISet::Toggle, DebugUISet::Display)
                    .chain()
                    .after(EguiSet::ProcessInput),
            )
            .init_resource::<DebugUIState>();
    }
}

#[derive(Default, Resource)]
struct DebugUIState {
    display_debug_info: bool,
    display_mat_debug: bool,

    // DD
    pub selected_mat: u8,
}
