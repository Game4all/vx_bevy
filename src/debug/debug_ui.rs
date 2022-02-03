use bevy::{
    diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    ecs::schedule::ShouldRun,
    input::{keyboard::KeyboardInput, ElementState},
    prelude::{CoreStage, EventReader, KeyCode, Plugin, Res, ResMut, SystemSet, SystemStage},
};
use bevy_egui::{
    egui::{self, Slider},
    EguiContext, EguiPlugin,
};

use crate::voxel::{storage::VoxelMap, ChunkLoadingRadius, ChunkShape, Voxel};

fn display_debug_stats(egui: ResMut<EguiContext>, diagnostics: Res<Diagnostics>) {
    egui::Window::new("performance stuff").show(egui.ctx(), |ui| {
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
    egui: ResMut<EguiContext>,
    chunk_map: Res<VoxelMap<Voxel, ChunkShape>>,
    mut chunk_loading_radius: ResMut<ChunkLoadingRadius>,
) {
    egui::Window::new("voxel world stuff").show(egui.ctx(), |ui| {
        ui.heading("Chunks");
        ui.label(format!("Loaded chunk count:  {}", chunk_map.chunks.len()));
        ui.separator();
        ui.label("Chunk loading radius");
        ui.add(Slider::new(&mut chunk_loading_radius.0, 16..=32));
        ui.separator();
    });
}

fn display_debug_ui_criteria(ui_state: Res<DebugUIState>) -> ShouldRun {
    if ui_state.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn toggle_ui_display(mut inputs: EventReader<KeyboardInput>, mut ui_state: ResMut<DebugUIState>) {
    for input in inputs.iter() {
        match input.key_code {
            Some(key_code) if key_code == KeyCode::F3 && input.state == ElementState::Pressed => {
                ui_state.0 = !ui_state.0;
            }
            _ => {}
        }
    }
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
                    .with_system(toggle_ui_display)
                    .with_system_set(
                        SystemSet::new()
                            .with_system(display_debug_stats)
                            .with_system(display_chunk_stats)
                            .with_run_criteria(display_debug_ui_criteria),
                    ),
            )
            .init_resource::<DebugUIState>();
    }
}

#[derive(Default)]
struct DebugUIState(bool);