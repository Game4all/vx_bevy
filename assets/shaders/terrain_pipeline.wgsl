#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

#import "shaders/voxel_data.wgsl"
#import "shaders/terrain_uniforms.wgsl"
#import "shaders/noise.wgsl"
#import "shaders/fog.wgsl"

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) voxel_data: u32,
};

@group(1) @binding(0)
var<uniform> mesh: Mesh;

#import bevy_pbr::mesh_functions

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) voxel_normal: vec3<f32>,
    @location(1) voxel_data: u32,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh_position_local_to_world(mesh.model, vec4<f32>(vertex.position, 1.0));

    var out: VertexOutput;
    out.clip_position = mesh_position_world_to_clip(world_position);
    out.voxel_normal = voxel_data_extract_normal(vertex.voxel_data);
    out.voxel_data = vertex.voxel_data;
    out.world_position = world_position.xyz;

    return out;
}

struct Fragment {
    /// The normalized normal of the voxel.
    @location(0) voxel_normal: vec3<f32>,
    /// The extracted color data from voxel data.
    @location(1) voxel_color: u32,
    /// The world position of the voxel vertex.
    @location(2) world_position: vec3<f32>,
};

@fragment
fn fragment(frag: Fragment) -> @location(0) vec4<f32> {
    let material = VOXEL_MATERIALS.materials[voxel_data_extract_material_index(frag.voxel_color)];

    var base_color: vec4<f32> = material.base_color;
    base_color = base_color + hash(vec4<f32>(floor(frag.world_position - frag.voxel_normal * 0.5), 1.0)) * 0.0226;

    // final voxel color with ambient lighting + normal face lighting
    var scolor = calc_voxel_lighting(base_color.xyz, frag.voxel_normal);

    // fragment distance from camera, used to determine amount of fog to apply.
    let fog_distance = distance(frag.world_position, view.world_position);
    return ffog_apply_fog(fog_distance, f32(terrain_settings.render_distance) * f32(TERRAIN_CHUNK_LENGTH), f32(TERRAIN_CHUNK_LENGTH), vec4<f32>(scolor, base_color.w));
}
