use std::mem::size_of;
use std::num::NonZeroU64;

use bevy::core_pipeline::Transparent3d;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::pbr::{DrawMesh, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup};
use bevy::prelude::{
    Bundle, ComputedVisibility, Entity, GlobalTransform, Mesh, Msaa, Query, Res, ResMut, Transform,
    Visibility, With,
};

use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderPhase, SetItemPipeline,
};
use bevy::render::render_resource::std140::AsStd140;
use bevy::render::render_resource::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferDescriptor,
    BufferSize, BufferUsages, RenderPipelineCache, ShaderStages, SpecializedPipeline,
    SpecializedPipelines, VertexAttribute, VertexBufferLayout, VertexFormat,
};
use bevy::render::renderer::RenderDevice;
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

impl VoxelMesh {
    pub const ATTRIBUTE_DATA: &'static str = "Vertex_Data";
}

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
    pub material_array_layout: BindGroupLayout,
}

impl FromWorld for VoxelMeshRenderPipeline {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        VoxelMeshRenderPipeline {
            mesh_pipeline: world.get_resource::<MeshPipeline>().unwrap().clone(),
            shader: world
                .get_resource::<AssetServer>()
                .unwrap()
                .load("shaders/voxel_pipeline.wgsl") as Handle<Shader>,
            material_array_layout: {
                let render_device = world.get_resource::<RenderDevice>().unwrap();
                render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("voxel_engine_material_array_layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        ty: BindingType::Buffer {
                            has_dynamic_offset: false,
                            ty: bevy::render::render_resource::BufferBindingType::Uniform,
                            min_binding_size: BufferSize::new(
                                GpuVoxelMaterialArray::std140_size_static() as u64,
                            ),
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    }],
                })
            },
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
        material_meshes.for_each(
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
            },
        )
    }
}

/// A GPU representation of the PBR info for a voxel material type
#[derive(AsStd140)]
pub struct GpuVoxelMaterialData {
    pub base_color: [f32; 4],
}

#[derive(AsStd140)]
pub struct GpuVoxelMaterialArray {
    pub materials: [GpuVoxelMaterialData; 256],
}

/// A resource wrapping the GPU voxel material array representation for update by systems.
#[derive(Default)]
pub struct GpuVoxelMaterialArrayBindGroup {
    pub buffer: Option<Buffer>,
    pub bind_group: Option<BindGroup>,
}

/// Sets up the GPU buffer for storing the voxel material array and creates the associated bind group if they doesn't exist yet.
fn setup_voxel_material_array_bind_group(
    mut material_array_bind_group: ResMut<GpuVoxelMaterialArrayBindGroup>,
    render_device: Res<RenderDevice>,
    pipeline: Res<VoxelMeshRenderPipeline>,
) {
    // Create the material array buffer if isn't allocated yet.
    if material_array_bind_group.buffer.is_none() {
        material_array_bind_group.buffer = Some(render_device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_WRITE | BufferUsages::UNIFORM,
            size: GpuVoxelMaterialArray::std140_size_static() as u64,
            mapped_at_creation: false,
        }));
    }

    //and create the associated bing group as well if it doesn't exist yet.
    if material_array_bind_group.bind_group.is_none() {
        material_array_bind_group.bind_group =
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: material_array_bind_group.buffer.as_ref().unwrap(),
                        offset: 0,
                        size: NonZeroU64::new(GpuVoxelMaterialArray::std140_size_static() as u64),
                    }),
                }],
                label: None,
                layout: &pipeline.material_array_layout,
            }));
    }
}

pub struct SetVoxelMaterialArrayBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelMaterialArrayBindGroup<I> {
    type Param = SRes<GpuVoxelMaterialArrayBindGroup>;

    fn render<'w>(
        _view: Entity,
        _item: Entity,
        param: bevy::ecs::system::SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        pass.set_bind_group(I, param.into_inner().bind_group.as_ref().unwrap(), &[]);
        bevy::render::render_phase::RenderCommandResult::Success
    }
}

type DrawVoxel = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetVoxelMaterialArrayBindGroup<2>,
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
    pub aabb: Aabb,
}

pub struct VoxelMeshRenderPipelinePlugin;

impl Plugin for VoxelMeshRenderPipelinePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<VoxelMesh>::default());
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawVoxel>()
            .init_resource::<VoxelMeshRenderPipeline>()
            .init_resource::<SpecializedPipelines<VoxelMeshRenderPipeline>>()
            .init_resource::<GpuVoxelMaterialArrayBindGroup>()
            .add_system_to_stage(RenderStage::Queue, queue_voxel_meshes)
            .add_system_to_stage(RenderStage::Prepare, setup_voxel_material_array_bind_group);
    }
}
