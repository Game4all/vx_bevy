
// A GPU-suited representation of a voxel material.
struct VoxelMaterial {
    base_color: vec4<f32>;
};

struct VoxelMaterials {
    materials: array<VoxelMaterial, 256u>;
};

[[group(2), binding(0)]]
var<uniform> VOXEL_MATERIALS: VoxelMaterials;