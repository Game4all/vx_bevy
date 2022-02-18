use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::{Entity, FromWorld, Plugin, Res, ResMut},
    render::{
        render_phase::EntityRenderCommand,
        render_resource::{
            std140::AsStd140, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferDescriptor,
            BufferSize, BufferUsages, ShaderStages,
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
pub struct GpuVoxelMaterialArrayMeta {
    pub bind_group_layout: BindGroupLayout,
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

impl FromWorld for GpuVoxelMaterialArrayMeta {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        GpuVoxelMaterialArrayMeta {
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            }),
            buffer: render_device.create_buffer(&BufferDescriptor {
                label: None,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                size: GpuVoxelMaterialArray::std140_size_static() as u64,
                mapped_at_creation: false,
            }),
            bind_group: None,
        }
    }
}

/// Prepares the the bind group
fn prepare_material_array_bind_group(
    mut material_array_bind_group: ResMut<GpuVoxelMaterialArrayMeta>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
            binding: 0,
            resource: material_array_bind_group.buffer.as_entire_binding(),
        }],
        label: None,
        layout: &material_array_bind_group.bind_group_layout,
    });

    material_array_bind_group.bind_group = Some(bind_group);
}

/// Binds a GPU-suited representation of voxel materials at the specified bind group index.
pub struct SetVoxelMaterialArrayBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelMaterialArrayBindGroup<I> {
    type Param = SRes<GpuVoxelMaterialArrayMeta>;

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
            .init_resource::<GpuVoxelMaterialArrayMeta>()
            .add_system_to_stage(RenderStage::Prepare, prepare_material_array_bind_group);
    }
}
