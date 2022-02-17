use std::num::NonZeroU64;

use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::{Entity, FromWorld, Plugin, Res, ResMut},
    render::{
        render_phase::EntityRenderCommand,
        render_resource::{
            std140::AsStd140, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
            BufferBinding, BufferDescriptor, BufferSize, BufferUsages, ShaderStages,
        },
        renderer::RenderDevice,
        RenderApp, RenderStage,
    },
};

/// A GPU-suited representation of the PBR info for a voxel material type
#[derive(AsStd140)]
pub struct GpuVoxelMaterialData {
    pub base_color: [f32; 4],
}

#[derive(AsStd140)]
pub struct GpuVoxelMaterialArray {
    pub materials: [GpuVoxelMaterialData; 256],
}

/// A resource wrapping the GPU voxel material array buffer and bind group.
pub struct GpuVoxelMaterialArrayBindGroup {
    pub buffer: Option<Buffer>,
    pub bind_group: Option<BindGroup>,
    pub bind_group_layout: BindGroupLayout,
}

impl FromWorld for GpuVoxelMaterialArrayBindGroup {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        GpuVoxelMaterialArrayBindGroup {
            buffer: None,
            bind_group: None,
            bind_group_layout: {
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

/// Sets up the GPU buffer for storing the voxel material array and creates the associated bind group if they doesn't exist yet.
fn setup_voxel_material_array_bind_group(
    mut material_array_bind_group: ResMut<GpuVoxelMaterialArrayBindGroup>,
    render_device: Res<RenderDevice>,
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
                layout: &material_array_bind_group.bind_group_layout,
            }));
    }
}

/// Binds a GPU-suited representation of voxel materials at the specified bind group index.
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

/// Handles the management of voxel materials for rendering on GPU-side.
pub struct VoxelGpuMaterialPlugin;

impl Plugin for VoxelGpuMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<GpuVoxelMaterialArrayBindGroup>()
            .add_system_to_stage(RenderStage::Prepare, setup_voxel_material_array_bind_group);
    }
}
