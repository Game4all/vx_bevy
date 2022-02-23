use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::{Entity, FromWorld, Plugin, Res, ResMut},
    render::{
        render_phase::EntityRenderCommand,
        render_resource::{
            std430::{AsStd430, Std430},
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferDescriptor,
            BufferSize, BufferUsages, ShaderStages,
        },
        renderer::{RenderDevice, RenderQueue},
        RenderApp, RenderStage, RenderWorld,
    },
};

use crate::voxel::material::VoxelMaterialRegistry;

#[derive(AsStd430)]
pub struct GpuVoxelMaterials {
    pub materials: [[f32; 4]; 256],
}

/// A resource wrapping the GPU voxel material array buffer and bind group.
pub struct GpuVoxelMaterialsMeta {
    pub bind_group_layout: BindGroupLayout,
    pub buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

impl FromWorld for GpuVoxelMaterialsMeta {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        GpuVoxelMaterialsMeta {
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("voxel_engine_material_array_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    ty: BindingType::Buffer {
                        has_dynamic_offset: false,
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        min_binding_size: BufferSize::new(
                            GpuVoxelMaterials::std430_size_static() as u64
                        ),
                    },
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                }],
            }),
            buffer: render_device.create_buffer(&BufferDescriptor {
                label: None,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                size: GpuVoxelMaterials::std430_size_static() as u64,
                mapped_at_creation: false,
            }),
            bind_group: None,
        }
    }
}

/// Prepares the the bind group
fn prepare_material_array_bind_group(
    mut material_array_bind_group: ResMut<GpuVoxelMaterialsMeta>,
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

fn extract_voxel_materials(
    mut render_world: ResMut<RenderWorld>,
    materials: Res<VoxelMaterialRegistry>,
) {
    if materials.is_changed() {
        let mut gpu_mats = GpuVoxelMaterials {
            materials: [[0.; 4]; 256],
        };

        materials
            .iter_mats()
            .enumerate()
            .for_each(|(index, material)| {
                gpu_mats.materials[index] = material.base_color.as_rgba_f32();
            });

        render_world.insert_resource(gpu_mats);
    }
}

fn upload_voxel_materials(
    render_queue: Res<RenderQueue>,
    material_meta: Res<GpuVoxelMaterialsMeta>,
    materials: Res<GpuVoxelMaterials>,
) {
    if materials.is_changed() {
        render_queue.write_buffer(&material_meta.buffer, 0, materials.as_std430().as_bytes());
    }
}

/// Binds a GPU-suited representation of voxel materials at the specified bind group index.
pub struct SetVoxelMaterialArrayBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelMaterialArrayBindGroup<I> {
    type Param = SRes<GpuVoxelMaterialsMeta>;

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
            .init_resource::<GpuVoxelMaterialsMeta>()
            .add_system_to_stage(RenderStage::Extract, extract_voxel_materials)
            .add_system_to_stage(RenderStage::Prepare, prepare_material_array_bind_group)
            .add_system_to_stage(RenderStage::Prepare, upload_voxel_materials);
    }
}
