

// A GPU-suited representation of voxel materials.
struct VoxelMaterials {
    materials: array<vec4<f32>, 256u>;
};

struct Time {
    time: f32;
};

[[group(2), binding(0)]]
var<uniform> VOXEL_MATERIALS: VoxelMaterials;

[[group(2), binding(1)]]
var<uniform> time: Time;

// Returns computed fragment color from the current ambient light + diffuse per face lighting
fn calc_voxel_lighting(col: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let per_face_light = vec3<f32>(0.8, 1.0, 0.6);
    let normal = abs(dot(n, vec3<f32>(1., 0., 0.)) * per_face_light.x) 
               + abs(dot(n, vec3<f32>(0., 1., 0.)) * per_face_light.y) 
               + abs(dot(n, vec3<f32>(0., 0., 1.)) * per_face_light.z);

    return normal * col + lights.ambient_color.xyz * 0.21;
}