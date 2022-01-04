use bevy::prelude::*;

#[derive(Debug, Default, Component)]
pub struct Visibility {
    pub visible: bool,
}

/// Update the render visibility ([`Visible`]) of entities according to their current ([`Visibility`]).
fn update_visibility(
    mut visible_entities: Query<(&mut Visible, &Visibility), Changed<Visibility>>,
) {
    visible_entities
        .for_each_mut(|(mut visible, visibility)| visible.is_visible = visibility.visible);
}

pub struct MeshCullingPlugin;

impl Plugin for MeshCullingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            update_visibility
                .system()
                .before(bevy::render::RenderSystem::VisibleEntities),
        );
    }
}
