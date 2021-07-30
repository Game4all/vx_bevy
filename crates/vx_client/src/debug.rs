use bevy::{
    diagnostic::{DiagnosticId, Diagnostics, FrameTimeDiagnosticsPlugin},
    ecs::system::EntityCommands,
    prelude::*,
};
use vx_core::world::{
    ChunkEntityMap, ChunkMeshingRequest, CHUNK_DATA_GEN_TIME, CHUNK_MESHING_TIME,
};

use crate::{input::Action, player::PlayerController};

const TRACKED_DIAGS: &[DiagnosticId] = &[
    FrameTimeDiagnosticsPlugin::FPS,
    CHUNK_MESHING_TIME,
    CHUNK_DATA_GEN_TIME,
];

const TRACKED_VALUES: &[&'static str] = &["Pos"];

struct DiagnosticCounter(DiagnosticId);

struct DebugValue(&'static str);

struct DebugUIComponent;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, diagnostics: Res<Diagnostics>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let mut val = Val::Px(15.0);

    for diag in TRACKED_DIAGS {
        if let Some(diagnostic) = diagnostics.get(*diag) {
            create_counter(
                &mut commands,
                &asset_server,
                Rect {
                    top: val,
                    left: Val::Px(15.),
                    ..Default::default()
                },
                HorizontalAlign::Left,
                diagnostic.name.to_string(),
                |cmds: &mut EntityCommands| {
                    cmds.insert(DiagnosticCounter(*diag));
                },
            );
            val += 20.0;
        }
    }

    let mut val = Val::Px(15.0);

    for value in TRACKED_VALUES {
        create_counter(
            &mut commands,
            &asset_server,
            Rect {
                top: val,
                right: Val::Px(15.),
                ..Default::default()
            },
            HorizontalAlign::Right,
            value.to_string(),
            |cmds| {
                cmds.insert(DebugValue(*value));
            },
        );
        val += 20.0;
    }
}

//

fn create_counter(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Rect<Val>,
    alignment: HorizontalAlign,
    name: String,
    mut config: impl FnMut(&mut EntityCommands),
) {
    let mut cmds = commands.spawn_bundle(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: position,
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
                horizontal: alignment,
                ..Default::default()
            },
        },
        visible: Visible {
            is_visible: false,
            ..Default::default()
        },
        ..Default::default()
    });
    cmds.insert(DebugUIComponent);

    config(&mut cmds);
}

fn update_diagnostic_counters(
    diagnostics: ResMut<Diagnostics>,
    mut counter: Query<(&mut Text, &DiagnosticCounter)>,
) {
    for (mut text, counter) in counter.iter_mut() {
        if let Some(diag) = diagnostics.get(counter.0) {
            if let Some(avg) = diag.average() {
                text.sections[2].value = format!("{:.4}", avg);
            }
        }
    }
}

fn update_debug_values(
    mut counters: Query<(&mut Text, &DebugValue)>,
    player: Query<(&PlayerController, &Transform)>,
) {
    for (mut text, debug_cnt) in counters.iter_mut() {
        for (_, transform) in player.single() {
            if debug_cnt.0 == TRACKED_VALUES[0] {
                text.sections[2].value = format!("{}", &transform.translation.round());
            }
        }
    }
}

// debug input handling

fn toggle_debug_ui(
    mut counters: Query<&mut Visible, With<DebugUIComponent>>,
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

// plugin

pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_startup_system(setup.system())
            .add_system(update_diagnostic_counters.system())
            .add_system(update_debug_values.system())
            .add_system(toggle_debug_ui.system())
            .add_system(remesh_chunks.system());
    }
}
