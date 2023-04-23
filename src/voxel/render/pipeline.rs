use bevy::core_pipeline::core_3d::AlphaMask3d;
use bevy::pbr::{DrawMesh, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup};
use bevy::prelude::{
    Bundle, ComputedVisibility, Entity, GlobalTransform, IntoSystemConfig, Mesh, Msaa, Query, Res,
    ResMut, Resource, Transform, Visibility, With,
};
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayout};

use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline};
use bevy::render::render_resource::{
    BindGroupLayout, PipelineCache, SpecializedMeshPipeline, SpecializedMeshPipelineError,
    SpecializedMeshPipelines, VertexFormat,
};

use bevy::render::view::ExtractedView;
use bevy::render::RenderSet;
use bevy::{
    pbr::MeshPipeline,
    prelude::{AssetServer, Component, FromWorld, Handle, Plugin, Shader},
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        RenderApp,
    },
};

use super::terrain_uniforms::{self, SetTerrainUniformsBindGroup, TerrainUniforms};

#[derive(Component, Clone, Default, ExtractComponent)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh;

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Data", 1, VertexFormat::Uint32);
}

/// A render pipeline for rendering voxel terrain meshes.
#[derive(Resource)]
pub struct VoxelTerrainRenderPipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
    material_array_layout: BindGroupLayout,
}

impl FromWorld for VoxelTerrainRenderPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        Self {
            mesh_pipeline: world.get_resource::<MeshPipeline>().unwrap().clone(),
            shader: world
                .get_resource::<AssetServer>()
                .unwrap()
                .load("shaders/terrain_pipeline.wgsl") as Handle<Shader>,
            material_array_layout: world
                .get_resource::<TerrainUniforms>()
                .unwrap()
                .bind_group_layout
                .clone(),
        }
    }
}

impl SpecializedMeshPipeline for VoxelTerrainRenderPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<bevy::render::render_resource::RenderPipelineDescriptor, SpecializedMeshPipelineError>
    {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor
            .fragment
            .as_mut()
            .expect("Failed to get fragment shader from mesh pipeline")
            .shader = self.shader.clone();
        descriptor.layout.push(self.material_array_layout.clone());
        Ok(descriptor)
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_voxel_meshes(
    oq_draw_funcs: Res<DrawFunctions<AlphaMask3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    voxel_pipeline: Res<VoxelTerrainRenderPipeline>,
    pipeline_cache: Res<PipelineCache>, // Was ResMut
    mut specialized_pipelines: ResMut<SpecializedMeshPipelines<VoxelTerrainRenderPipeline>>,
    msaa: Res<Msaa>,
    material_meshes: Query<(Entity, &Handle<Mesh>, &MeshUniform), With<VoxelTerrainMesh>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<AlphaMask3d>)>,
) {
    let draw_custom = oq_draw_funcs.read().get_id::<DrawVoxel>().unwrap();
    let key = MeshPipelineKey::from_msaa_samples(msaa.samples());
    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        material_meshes.for_each(|(entity, mesh_handle, mesh_uniform)| {
            if let Some(mesh) = render_meshes.get(mesh_handle) {
                transparent_phase.add(AlphaMask3d {
                    entity,
                    pipeline: specialized_pipelines
                        .specialize(
                            &pipeline_cache,
                            &voxel_pipeline,
                            key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology),
                            &mesh.layout,
                        )
                        .unwrap(),
                    draw_function: draw_custom,
                    distance: view_row_2.dot(mesh_uniform.transform.col(3)),
                });
            }
        })
    }
}

type DrawVoxel = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetTerrainUniformsBindGroup<2>,
    DrawMesh,
);

#[derive(Bundle, Default)]
pub struct VoxelTerrainMeshBundle {
    pub mesh: Handle<Mesh>,
    pub voxel: VoxelTerrainMesh,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub aabb: Aabb
}
pub struct VoxelMeshRenderPipelinePlugin;

impl Plugin for VoxelMeshRenderPipelinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<VoxelTerrainMesh>::default())
            .add_plugin(terrain_uniforms::VoxelTerrainUniformsPlugin);
        app.sub_app_mut(RenderApp)
            .add_render_command::<AlphaMask3d, DrawVoxel>()
            .init_resource::<VoxelTerrainRenderPipeline>()
            .init_resource::<SpecializedMeshPipelines<VoxelTerrainRenderPipeline>>()
            .add_system(queue_voxel_meshes.in_set(RenderSet::Queue));
    }
}
