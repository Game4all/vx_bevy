
let VOXEL_MAT_FLAG_LIQUID: u32 = 2u; // 1 << 1
let TERRAIN_CHUNK_LENGTH: u32 = 32u;

struct VoxelMat {
    base_color: vec4<f32>,
    flags: u32,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
};

// A GPU-suited representation of voxel materials.
struct VoxelMaterials {
    materials: array<VoxelMat>
};

struct TerrainRenderSettings {
    render_distance: u32,
};

@group(2) @binding(0)
var<storage> VOXEL_MATERIALS: VoxelMaterials;

@group(2)  @binding(1)
var<storage> terrain_settings: TerrainRenderSettings;

// Returns computed fragment color from the current ambient light + diffuse per face lighting
fn calc_voxel_lighting(col: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let per_face_light = vec3<f32>(0.8, 1.0, 0.6);
    let normal = abs(dot(n, vec3<f32>(1., 0., 0.)) * per_face_light.x) 
               + abs(dot(n, vec3<f32>(0., 1., 0.)) * per_face_light.y) 
               + abs(dot(n, vec3<f32>(0., 0., 1.)) * per_face_light.z);

    return normal * col + lights.ambient_color.xyz * 0.21;
}