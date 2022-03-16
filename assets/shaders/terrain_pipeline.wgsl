#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

#import "shaders/voxel_data.wgsl"
#import "shaders/terrain_uniforms.wgsl"
#import "shaders/noise.wgsl"

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] data: u32;
};

[[group(1), binding(0)]]
var<uniform> mesh: Mesh;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] normal_pt: vec3<f32>;
    [[location(1)]] data: u32;
    [[location(2)]] world_pos: vec3<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.clip_position = view.view_proj * world_position;
    out.normal_pt = voxel_data_extract_normal(vertex.data);
    out.data = vertex.data;
    out.world_pos = world_position.xyz;

    return out;
}

struct Fragment {
    [[location(0)]] normal: vec3<f32>;
    [[location(1)]] data: u32;
    [[location(2)]] world_pos: vec3<f32>;
};

[[stage(fragment)]]
fn fragment(frag: Fragment) -> [[location(0)]] vec4<f32> {
    let material = VOXEL_MATERIALS.materials[voxel_data_extract_material_index(frag.data)];

    var base_color: vec4<f32> = material.base_color;
    base_color = base_color + hash(vec4<f32>(floor(frag.world_pos - frag.normal * 0.5), 1.0)) * 0.0226;

    // final voxel color with ambient lighting + normal face lighting
    let scolor = calc_voxel_lighting(base_color.xyz, frag.normal);

    return vec4<f32>(scolor, base_color.w);
}
