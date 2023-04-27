use bevy::core_pipeline::core_3d::AlphaMask3d;
use bevy::pbr::{
    DrawMesh, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup,
    MAX_CASCADES_PER_LIGHT, MAX_DIRECTIONAL_LIGHTS,
};
use bevy::prelude::{
    Bundle, ComputedVisibility, Entity, GlobalTransform, IntoSystemConfig, Mesh, Msaa, Query, Res,
    ResMut, Resource, Transform, Visibility, With,
};
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayout};
use bevy::render::RenderSet;

use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline};
use bevy::render::render_resource::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PipelineCache,
    PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderDefVal, SpecializedMeshPipeline,
    SpecializedMeshPipelineError, SpecializedMeshPipelines, StencilFaceState, StencilState,
    TextureFormat, VertexFormat, VertexState,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::ExtractedView;
use bevy::{
    pbr::MeshPipeline,
    prelude::{AssetServer, Component, FromWorld, Handle, Plugin, Shader},
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        RenderApp,
    },
};
use bevy_egui::egui::PlatformOutput;

use super::terrain_uniforms::{self, SetTerrainUniformsBindGroup, TerrainUniforms};

#[derive(Component, Clone, Default)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh;

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Data", 1, VertexFormat::Uint32);
}

impl ExtractComponent for VoxelTerrainMesh {
    type Query = &'static Self;
    type Filter = ();
    type Out = Self;

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Option<Self::Out> {
        Some(item.clone())
    }
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
        descriptor.layout.push(self.material_array_layout.clone());
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers = vec![layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            VoxelTerrainMesh::ATTRIBUTE_DATA.at_shader_location(1),
        ])?];
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        Ok(descriptor)
        /*
        Ok(RenderPipelineDescriptor {
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: defs.clone(),
                buffers: vec![layout.get_layout(&[
                    Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                    VoxelTerrainMesh::ATTRIBUTE_DATA.at_shader_location(1),
                ])?],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: defs,
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            layout: vec![
                self.mesh_pipeline.view_layout.clone(),
                self.mesh_pipeline.mesh_layout.clone(),

            ],
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("voxel pipeline".into()),
            push_constant_ranges: vec![],
        })
         */
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_voxel_meshes(
    oq_draw_funcs: Res<DrawFunctions<AlphaMask3d>>,
    render_meshes: Res<RenderAssets<Mesh>>,
    voxel_pipeline: Res<VoxelTerrainRenderPipeline>,
    mut pipeline_cache: ResMut<PipelineCache>,
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
                            &mut pipeline_cache,
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
    pub aabb: Aabb,
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
