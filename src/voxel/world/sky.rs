use std::f32::consts::PI;

use bevy::prelude::{
    Color, Commands, CoreSet, Deref, DirectionalLight, DirectionalLightBundle, Entity,
    IntoSystemConfig, IntoSystemSetConfig, ParamSet, Plugin, Quat, Query, Res, Resource, SystemSet,
    Transform, With,
};

use super::player::PlayerController;

#[derive(Resource, Deref)]
struct SkyLightEntity(Entity);

fn setup_sky_lighting(mut cmds: Commands) {
    let sky_light_entity = cmds
        .spawn(DirectionalLightBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 4.0)),
            directional_light: DirectionalLight {
                color: Color::WHITE,
                shadows_enabled: true,
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
    let player_translation = queries
        .p1()
        .get_single()
        .map_or_else(|_| Default::default(), |ply| ply.translation);

    {
        let mut binding = queries.p0();
        let mut sky_light_transform = binding.get_mut(sky_light_entity).unwrap();
        sky_light_transform.translation = player_translation;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct InteractiveSkyboxSet;

pub struct InteractiveSkyboxPlugin;

impl Plugin for InteractiveSkyboxPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup_sky_lighting)
            .add_system(
                update_light_position
                    .in_set(InteractiveSkyboxSet)
                    .ambiguous_with_all(),
                // @todo: update atmosphere library to add a set so that this is only ambiguous with that set
            )
            .configure_set(InteractiveSkyboxSet.in_base_set(CoreSet::Update));
    }
}
