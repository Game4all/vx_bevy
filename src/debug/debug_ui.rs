use bevy::{
    diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    ecs::schedule::ShouldRun,
    input::{keyboard::KeyboardInput, ElementState},
    prelude::{
        Color, CoreStage, EventReader, KeyCode, ParallelSystemDescriptorCoercion, Plugin, Res,
        ResMut, SystemSet, SystemStage,
    },
};
use bevy_egui::{
    egui::{self, Rgba, Slider},
    EguiContext, EguiPlugin,
};

use crate::voxel::{
    material::VoxelMaterialRegistry, ChunkLoadRadius, CurrentLocalPlayerChunk, DirtyChunks,
};

fn display_debug_stats(mut egui: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    egui::Window::new("performance stuff").show(egui.ctx_mut(), |ui| {
        ui.label(format!(
            "Avg. FPS: {:.02}",
            diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
        ui.label(format!(
            "Total Entity count: {}",
            diagnostics
                .get(EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                .unwrap()
                .average()
                .unwrap_or_default()
        ));
    });
}

fn display_chunk_stats(
    mut egui: ResMut<EguiContext>,
    dirty_chunks: Res<DirtyChunks>,
    player_pos: Res<CurrentLocalPlayerChunk>,
    mut chunk_loading_radius: ResMut<ChunkLoadRadius>,
) {
    egui::Window::new("voxel world stuff").show(egui.ctx_mut(), |ui| {
        ui.heading("Chunks");
        ui.label(format!(
            "Chunks invalidations (per frame):  {}",
            dirty_chunks.num_dirty()
        ));
        ui.separator();
        ui.label(" Horizontal chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.horizontal, 8..=32));
        ui.separator();
        ui.heading("Current player position");
        ui.label(format!("Current position : {}", player_pos.world_pos));
        ui.label(format!(
            "Current chunk : {}",
            player_pos.chunk_pos.location()
        ));
    });
}

fn display_debug_ui_criteria(ui_state: Res<DebugUIState>) -> ShouldRun {
    if ui_state.display_debug_info {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn display_mat_debug_ui_criteria(ui_state: Res<DebugUIState>) -> ShouldRun {
    if ui_state.display_mat_debug {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn toggle_debug_ui_displays(
    mut inputs: EventReader<KeyboardInput>,
    mut ui_state: ResMut<DebugUIState>,
) {
    for input in inputs.iter() {
        match input.key_code {
            Some(key_code) if key_code == KeyCode::F3 && input.state == ElementState::Pressed => {
                ui_state.display_debug_info = !ui_state.display_debug_info;
            }
            Some(key_code) if key_code == KeyCode::F7 && input.state == ElementState::Pressed => {
                ui_state.display_mat_debug = !ui_state.display_mat_debug;
            }
            _ => {}
        }
    }
}

fn display_material_editor(
    mut egui: ResMut<EguiContext>,
    mut ui_state: ResMut<DebugUIState>,
    mut materials: ResMut<VoxelMaterialRegistry>,
) {
    egui::Window::new("material editor").show(egui.ctx_mut(), |ui| {
        ui.heading("Select material");
        egui::containers::ComboBox::from_label("Material")
            .selected_text(format!(
                "{}",
                materials.get_by_id(ui_state.selected_mat).unwrap().name
            ))
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

        ui.heading("Edit material");
        let selected_color = &mut materials
            .get_mut_by_id(ui_state.selected_mat)
            .unwrap()
            .base_color;
        let mut editable_color = Rgba::from_rgba_premultiplied(
            selected_color.r(),
            selected_color.g(),
            selected_color.b(),
            selected_color.a(),
        );
        egui::widgets::color_picker::color_edit_button_rgba(
            ui,
            &mut editable_color,
            egui::color_picker::Alpha::Opaque,
        );

        *selected_color = Color::from(editable_color.to_array());
    });
}

pub struct DebugUIPlugins;

impl Plugin for DebugUIPlugins {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_plugin(EntityCountDiagnosticsPlugin)
            .add_stage_after(
                CoreStage::PostUpdate,
                "debug_ui_stage",
                SystemStage::parallel()
                    .with_system(toggle_debug_ui_displays)
                    .with_system_set(
                        SystemSet::new()
                            .with_system(display_debug_stats)
                            .with_system(display_chunk_stats)
                            .with_run_criteria(display_debug_ui_criteria),
                    )
                    .with_system(
                        display_material_editor.with_run_criteria(display_mat_debug_ui_criteria),
                    ),
            )
            .init_resource::<DebugUIState>();
    }
}

#[derive(Default)]
struct DebugUIState {
    display_debug_info: bool,
    display_mat_debug: bool,

    // DD
    pub selected_mat: u8,
}
