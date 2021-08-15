use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics},
    prelude::*,
};

#[derive(Debug, Default)]
pub struct Visibility {
    pub visible: bool,
}

/// Update the render visibility ([`Visible`]) of entities according to their current ([`Visibility`]).
fn update_visibility(
    mut visible_entities: Query<(&mut Visible, &Visibility), Changed<Visibility>>,
) {
    for (mut visible, visibility) in visible_entities.iter_mut() {
        visible.is_visible = visibility.visible;
    }
}

pub struct MeshCullingPlugin;

impl Plugin for MeshCullingPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_system(
            update_visibility
                .system()
                .before(bevy::render::RenderSystem::VisibleEntities),
        );
    }
}
