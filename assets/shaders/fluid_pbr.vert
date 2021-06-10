#version 450

/*
    This is a modified version of bevy's PBR pipeline fragment shader modified
    to support per-vertex coloring.
*/

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;
layout(location = 3) in vec4 Vertex_Color;

layout(location = 0) out vec3 v_WorldPosition;
layout(location = 1) out vec3 v_WorldNormal;
layout(location = 2) out vec2 v_Uv;
layout(location = 3) out vec4 v_Color;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

const float FLUID_HEIGHT_DELTA = -0.27;

void main() {
    vec3 fluid_vertex_pos = vec3(Vertex_Position.x, max(Vertex_Position.y + FLUID_HEIGHT_DELTA, 0), Vertex_Position.z);
    vec4 world_position = Model * vec4(fluid_vertex_pos, 1.0);
    v_WorldPosition = world_position.xyz;
    v_WorldNormal = mat3(Model) * Vertex_Normal;
    v_Uv = Vertex_Uv;
    gl_Position = ViewProj * world_position;
    v_Color = Vertex_Color * vec4(1.0, 1.0, 1.0, 0.5);
}