use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use vx_core::world::CHUNK_MESHING_TIME;

struct DebugCounter(DiagnosticId);

pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_startup_system(setup.system())
            .add_system(update_counters.system());
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut diagnostics: ResMut<Diagnostics>,
) {
    diagnostics.add(Diagnostic::new(CHUNK_MESHING_TIME, "Chunk meshing time", 3));
    commands.spawn_bundle(UiCameraBundle::default());

    register_counter(
        CHUNK_MESHING_TIME,
        "Chunk meshing time ".to_string(),
        &mut commands,
        &asset_server,
        Val::Px(15.0),
    );
    register_counter(
        FrameTimeDiagnosticsPlugin::FPS,
        "FPS ".to_string(),
        &mut commands,
        &asset_server,
        Val::Px(32.0),
    );
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
                text.sections[1].value = format!("{:.4}", avg);
            }
        }
    }
}
