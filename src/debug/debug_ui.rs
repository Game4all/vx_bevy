use bevy::{
    diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::{CoreStage, Plugin, Res, ResMut, SystemStage},
};
use bevy_egui::{
    egui::{self, Slider},
    EguiContext, EguiPlugin,
};

use crate::voxel::{storage::VoxelMap, ChunkLoadingRadius, ChunkShape, Voxel};

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
                    .with_system(display_debug_stats)
                    .with_system(display_chunk_stats),
            );
    }
}

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
