use std::mem::size_of;
use std::num::NonZeroUsize;

use bevy::core_pipeline::Transparent3d;

use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::pbr::{DrawMesh, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup};
use bevy::prelude::{
    Bundle, ComputedVisibility, Entity, GlobalTransform, Mesh, Msaa, Query, Res, ResMut, Transform,
    Visibility, With,
};
use bevy::render::mesh::GpuBufferInfo;
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
    SetItemPipeline,
};
use bevy::render::render_resource::{
    BindGroupLayout, RenderPipelineCache, SpecializedPipeline, SpecializedPipelines,
    VertexAttribute, VertexBufferLayout, VertexFormat,
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

use super::terrain_uniforms::{self, SetTerrainUniformsBindGroup, TerrainUniformsMeta};

#[derive(Component, Clone, Default)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh {
    pub transparent_phase_mesh_index: Option<NonZeroUsize>,
}

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_DATA: &'static str = "Vertex_Data";
}

impl ExtractComponent for VoxelTerrainMesh {
    type Query = &'static VoxelTerrainMesh;

    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

/// A render pipeline for rendering voxel terrain meshes.
pub struct VoxelTerrainRenderPipeline {
    mesh_pipeline: MeshPipeline,
    shader: Handle<Shader>,
    material_array_layout: BindGroupLayout,
}

impl FromWorld for VoxelTerrainRenderPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        VoxelTerrainRenderPipeline {
            mesh_pipeline: world.get_resource::<MeshPipeline>().unwrap().clone(),
            shader: world
                .get_resource::<AssetServer>()
                .unwrap()
                .load("shaders/terrain_pipeline.wgsl") as Handle<Shader>,
            material_array_layout: world
                .get_resource::<TerrainUniformsMeta>()
                .unwrap()
                .bind_group_layout
                .clone(),
        }
    }
}

impl SpecializedPipeline for VoxelTerrainRenderPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        let mut descriptor = self.mesh_pipeline.specialize(key);
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
            self.material_array_layout.clone(),
        ]);
        descriptor.vertex.buffers = vec![VertexBufferLayout {
            array_stride: 16,
            step_mode: bevy::render::render_resource::VertexStepMode::Vertex,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x3, //Vertex_Position
                    offset: size_of::<u32>() as u64,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32, //Vertex_Data
                    offset: 0,
                    shader_location: 1,
                },
            ],
        }];
        descriptor
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_voxel_meshes(
    t3d_draw_funcs: Res<DrawFunctions<Transparent3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    voxel_pipeline: Res<VoxelTerrainRenderPipeline>,
    mut pipeline_cache: ResMut<RenderPipelineCache>,
    mut specialized_pipelines: ResMut<SpecializedPipelines<VoxelTerrainRenderPipeline>>,
    msaa: Res<Msaa>,
    material_meshes: Query<(Entity, &Handle<Mesh>, &MeshUniform), With<VoxelTerrainMesh>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = t3d_draw_funcs.read().get_id::<DrawVoxel<true>>().unwrap();
    let key = MeshPipelineKey::from_msaa_samples(msaa.samples);
    for (view, mut transparent_phase) in views.iter_mut() {
        let view_matrix = view.transform.compute_matrix();
        let view_row_2 = view_matrix.row(2);
        material_meshes.for_each(|(entity, mesh_handle, mesh_uniform)| {
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
        })
    }
}

type DrawVoxel<const P: bool> = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetTerrainUniformsBindGroup<2>,
    //DrawVoxelMesh<P>,
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
    pub aabb: Aabb,
}

pub struct DrawVoxelMesh<const P: bool>;

impl<const P: bool> EntityRenderCommand for DrawVoxelMesh<P> {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<(Read<Handle<Mesh>>, Read<VoxelTerrainMesh>)>,
    );

    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, query): bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let (mesh_handle, voxel_mesh) = query.get(item).unwrap();
        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            if let GpuBufferInfo::Indexed {
                buffer,
                count,
                index_format,
            } = &gpu_mesh.buffer_info
            {
                pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                let max_indice = *count
                    - voxel_mesh
                        .transparent_phase_mesh_index
                        .map(|x| x.get())
                        .unwrap_or_default() as u32;
                pass.draw_indexed(0..max_indice, 0, 0..1);
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct VoxelMeshRenderPipelinePlugin;

impl Plugin for VoxelMeshRenderPipelinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<VoxelTerrainMesh>::default())
            .add_plugin(terrain_uniforms::VoxelTerrainUniformsPlugin);
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawVoxel<true>>()
            .init_resource::<VoxelTerrainRenderPipeline>()
            .init_resource::<SpecializedPipelines<VoxelTerrainRenderPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_voxel_meshes);
    }
}
