use bevy::core_pipeline::Transparent3d;
use bevy::pbr::{
    DrawMesh, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup,
};
use bevy::prelude::{
    Bundle, ComputedVisibility, Entity, GlobalTransform, Mesh, Msaa, Query, Res, ResMut, Transform,
    Visibility, With,
};
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline};
use bevy::render::render_resource::{
    RenderPipelineCache, SpecializedPipeline, SpecializedPipelines,
};
use bevy::render::view::ExtractedView;
use bevy::render::RenderStage;
use bevy::{
    pbr::MeshPipeline,
    prelude::{AssetServer, Component, FromWorld, Handle, Plugin, Shader},
    render::{
        render_component::{ExtractComponent, ExtractComponentPlugin},
        RenderApp,
    },
};

#[derive(Component, Clone, Default)]
/// A marker component for voxel meshes.
pub struct VoxelMesh;

impl ExtractComponent for VoxelMesh {
    type Query = &'static VoxelMesh;

    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

/// A render pipeline for rendering voxel meshes.
pub struct VoxelMeshRenderPipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
}

impl FromWorld for VoxelMeshRenderPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        VoxelMeshRenderPipeline {
            mesh_pipeline: world.get_resource::<MeshPipeline>().unwrap().clone(),
            shader: world
                .get_resource::<AssetServer>()
                .unwrap()
                .load("shaders/voxel_pipeline.wgsl") as Handle<Shader>,
        }
    }
}

impl SpecializedPipeline for VoxelMeshRenderPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let mut descriptor = self.mesh_pipeline.specialize(key);
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_voxel_meshes(
    t3d_draw_funcs: Res<DrawFunctions<Transparent3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    voxel_pipeline: Res<VoxelMeshRenderPipeline>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    mut specialized_pipelines: ResMut<SpecializedPipelines<VoxelMeshRenderPipeline>>,
    msaa: Res<Msaa>,
    material_meshes: Query<(Entity, &Handle<Mesh>, &MeshUniform), With<VoxelMesh>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = t3d_draw_funcs.read().get_id::<DrawVoxel>().unwrap();
    let key = MeshPipelineKey::from_msaa_samples(msaa.samples);
    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);

        let add_render_phase =
            |(entity, mesh_handle, mesh_uniform): (Entity, &Handle<Mesh>, &MeshUniform)| {
                if let Some(mesh) = render_meshes.get(mesh_handle) {
                    transparent_phase.add(Transparent3d {
                        entity,
                        pipeline: specialized_pipelines.specialize(
                            &mut pipeline_cache,
                            &voxel_pipeline,
                            key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology),
                        ),
                        draw_function: draw_custom,
                        distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                    });
                }
            };
        material_meshes.for_each(add_render_phase);
    }
}

type DrawVoxel = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMesh,
);

#[derive(Bundle, Default)]
pub struct VoxelMeshBundle {
    pub mesh: Handle<Mesh>,
    pub voxel: VoxelMesh,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub aabb: Aabb
}

pub struct VoxelMeshRenderPipelinePlugin;

impl Plugin for VoxelMeshRenderPipelinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<VoxelMesh>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawVoxel>()
            .init_resource::<VoxelMeshRenderPipeline>()
            .init_resource::<SpecializedPipelines<VoxelMeshRenderPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_voxel_meshes);
    }
}
