
// Shader code underneath this is extracted from the veloren project shader assets and as such is licensed under the project GPLv3 license
// See https://github.com/veloren/veloren

fn hash(p: vec4<f32>) -> f32 {
    var p: vec4<f32> = fract(p * 0.3183099 + 0.1) - fract(p + 23.22121);
    p = p * 17.0;
    return (fract(p.x * p.y * (1.0 - p.z) * p.w * (p.x + p.y + p.z + p.w)) - 0.5) * 2.0;
}