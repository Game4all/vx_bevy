use std::f32::consts::PI;

use bevy::prelude::{
    Color, Commands, Deref, DirectionalLight, DirectionalLightBundle, Entity,
    OrthographicProjection, ParamSet, Plugin, Quat, Query, Res, Resource, Transform, With,
};

use super::player::PlayerController;

#[derive(Resource, Deref)]
struct SkyLightEntity(Entity);

fn setup_sky_lighting(mut cmds: Commands) {
    const SIZE: f32 = 200.0; //make this dynamic according to view distance???

    let sky_light_entity = cmds
        .spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
            directional_light: DirectionalLight {
                color: Color::WHITE,
                shadows_enabled: true,
                shadow_projection: OrthographicProjection {
                    left: -SIZE,
                    right: SIZE,
                    bottom: -SIZE,
                    top: SIZE,
                    near: -SIZE,
                    far: SIZE,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .id();

    cmds.insert_resource(SkyLightEntity(sky_light_entity));
}

fn update_light_position(
    sky_light_entity: Res<SkyLightEntity>,
    mut queries: ParamSet<(
        Query<&mut Transform>,
        Query<&Transform, With<PlayerController>>,
    )>,
) {
    let sky_light_entity = **sky_light_entity;
    let player_translation = if let Ok(ply) = queries.p1().get_single() {
        ply.translation
    } else {
        Default::default()
    };

    {
        let mut binding = queries.p0();
        let mut sky_light_transform = binding.get_mut(sky_light_entity).unwrap();
        sky_light_transform.translation = player_translation;
    }
}

pub struct InteractiveSkyboxPlugin;

impl Plugin for InteractiveSkyboxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup_sky_lighting)
            .add_system(update_light_position);
    }
}
