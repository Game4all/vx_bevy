use bevy::{
    core::Time,
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

/// A resource wrapping buffer references and bind groups for the different uniforms used for rendering terrains
pub struct TerrainUniformsMeta {
    pub bind_group_layout: BindGroupLayout,
    pub materials_buffer: Buffer,
    pub time_buffer: Buffer,
    pub bind_group: Option<BindGroup>,
}

impl FromWorld for TerrainUniformsMeta {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        TerrainUniformsMeta {
            bind_group_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("voxel_engine_material_array_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        ty: BindingType::Buffer {
                            has_dynamic_offset: false,
                            ty: bevy::render::render_resource::BufferBindingType::Uniform,
                            min_binding_size: BufferSize::new(
                                TerrainMaterialsUniform::std430_size_static() as u64,
                            ),
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        ty: BindingType::Buffer {
                            has_dynamic_offset: false,
                            ty: bevy::render::render_resource::BufferBindingType::Uniform,
                            min_binding_size: BufferSize::new(
                                TerrainTimeUniform::std430_size_static() as u64,
                            ),
                        },
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        count: None,
                    },
                ],
            }),
            materials_buffer: render_device.create_buffer(&BufferDescriptor {
                label: None,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                size: TerrainMaterialsUniform::std430_size_static() as u64,
                mapped_at_creation: false,
            }),
            time_buffer: render_device.create_buffer(&BufferDescriptor {
                label: None,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                size: TerrainTimeUniform::std430_size_static() as u64,
                mapped_at_creation: false,
            }),
            bind_group: None,
        }
    }
}

/// Prepares the the bind group
fn prepare_terrain_uniforms(
    mut terrain_uniforms: ResMut<TerrainUniformsMeta>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: terrain_uniforms.materials_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: terrain_uniforms.time_buffer.as_entire_binding(),
            },
        ],
        label: None,
        layout: &terrain_uniforms.bind_group_layout,
    });

    terrain_uniforms.bind_group = Some(bind_group);
}

// Materials uniform

#[derive(AsStd430, Clone, Copy)]
pub struct GpuVoxelMaterial {
    pub base_color: [f32; 4],
}

#[derive(AsStd430)]
struct TerrainMaterialsUniform {
    pub materials: [GpuVoxelMaterial; 256],
}

fn extract_voxel_materials(
    mut render_world: ResMut<RenderWorld>,
    materials: Res<VoxelMaterialRegistry>,
) {
    if materials.is_changed() {
        let mut gpu_mats = TerrainMaterialsUniform {
            materials: [GpuVoxelMaterial {
                base_color: [0f32; 4],
            }; 256],
        };

        materials
            .iter_mats()
            .enumerate()
            .for_each(|(index, material)| {
                gpu_mats.materials[index].base_color = material.base_color.as_rgba_f32();
            });

        render_world.insert_resource(gpu_mats);
    }
}

fn upload_voxel_materials(
    render_queue: Res<RenderQueue>,
    material_meta: Res<TerrainUniformsMeta>,
    materials: Res<TerrainMaterialsUniform>,
) {
    if materials.is_changed() {
        render_queue.write_buffer(
            &material_meta.materials_buffer,
            0,
            materials.as_std430().as_bytes(),
        );
    }
}

// time uniform
#[derive(AsStd430)]
pub struct TerrainTimeUniform {
    pub time: f32,
}

fn extract_time(mut render_world: ResMut<RenderWorld>, time: Res<Time>) {
    render_world.insert_resource(TerrainTimeUniform {
        time: time.seconds_since_startup() as f32,
    })
}

fn upload_time_uniform(
    render_queue: Res<RenderQueue>,
    material_meta: Res<TerrainUniformsMeta>,
    time: Res<TerrainTimeUniform>,
) {
    render_queue.write_buffer(&material_meta.time_buffer, 0, time.as_std430().as_bytes());
}

/// Binds the terrain uniforms for use in shaders.
pub struct SetTerrainUniformsBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTerrainUniformsBindGroup<I> {
    type Param = SRes<TerrainUniformsMeta>;

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

/// Handles the management of uniforms and bind groups for rendering terrain.
pub struct VoxelTerrainUniformsPlugin;

impl Plugin for VoxelTerrainUniformsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<TerrainUniformsMeta>()
            .add_system_to_stage(RenderStage::Extract, extract_voxel_materials)
            .add_system_to_stage(RenderStage::Extract, extract_time)
            .add_system_to_stage(RenderStage::Prepare, prepare_terrain_uniforms)
            .add_system_to_stage(RenderStage::Prepare, upload_voxel_materials)
            .add_system_to_stage(RenderStage::Prepare, upload_time_uniform);
    }
}
