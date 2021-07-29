use bevy::{
    diagnostic::{DiagnosticId, Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use vx_core::world::{
    ChunkEntityMap, ChunkMeshingRequest, CHUNK_DATA_GEN_TIME, CHUNK_MESHING_TIME,
};

use crate::input::Action;

const LOGGED_DIAGS: &[DiagnosticId] = &[
    FrameTimeDiagnosticsPlugin::FPS,
    CHUNK_MESHING_TIME,
    CHUNK_DATA_GEN_TIME,
];

struct DebugCounter(DiagnosticId);

pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_startup_system(setup.system())
            .add_system(update_counters.system())
            .add_system(toggle_counters.system())
            .add_system(remesh_chunks.system());
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, diagnostics: Res<Diagnostics>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let mut val = Val::Px(15.0);

    for diag in LOGGED_DIAGS {
        if let Some(diagnostic) = diagnostics.get(*diag) {
            register_counter(
                *diag,
                diagnostic.name.to_string(),
                &mut commands,
                &asset_server,
                val,
            );
            val += 20.0;
        }
    }
}

fn register_counter(
    diag: DiagnosticId,
    name: String,
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Val,
) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: position,
                    left: Val::Px(15.),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: name,
                        style: TextStyle {
                            font: asset_server.load("fonts/dogica.ttf"),
                            font_size: 8.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: " ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/dogica.ttf"),
                            font_size: 8.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/dogica.ttf"),
                            font_size: 8.0,
                            color: Color::WHITE,
                        },
                    },
                ],
                alignment: TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..Default::default()
                },
            },
            visible: Visible {
                is_visible: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(DebugCounter(diag));
}

fn update_counters(
    diagnostics: ResMut<Diagnostics>,
    mut counter: Query<(&mut Text, &DebugCounter)>,
) {
    for (mut text, counter) in counter.iter_mut() {
        if let Some(diag) = diagnostics.get(counter.0) {
            if let Some(avg) = diag.average() {
                text.sections[2].value = format!("{:.4}", avg);
            }
        }
    }
}

fn toggle_counters(
    mut counters: Query<&mut Visible, With<DebugCounter>>,
    input: Res<Input<Action>>,
) {
    if input.just_pressed(Action::ToggleDebugUi) {
        for mut visible in counters.iter_mut() {
            visible.is_visible = !visible.is_visible;
        }
    }
}

fn remesh_chunks(
    actions: Res<Input<Action>>,
    chunk_map: Res<ChunkEntityMap>,
    mut meshing_events: EventWriter<ChunkMeshingRequest>,
) {
    if actions.just_pressed(Action::RemeshChunks) {
        meshing_events.send_batch(chunk_map.values().map(|k| ChunkMeshingRequest(*k)));
        info!("Queued remesh of all visible chunks.");
    }
}
