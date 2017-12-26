#version 150 core

in vec2 in_pos;
in vec2 in_uv;
out vec2 out_uv;

layout (std140) uniform Properties {
    vec4 prop_src;
    vec4 prop_overlay_colour;
    vec2 prop_dest;
    float prop_rotation;
    float prop_scale;
};

layout (std140) uniform Global {
    vec2 global_resolution;
};

layout (std140) uniform Constants {
    vec2 constant_tileset;
    float constant_dpi_ratio;
};

void main() {
    // Scale and translate the UV for the source, and then scale down for the tileset
    out_uv = (in_uv * prop_src.zw + prop_src.xy) / constant_tileset;

    // Calculate the (clockwise) rotation matrix
    mat2 rotation = mat2(
        cos(prop_rotation), sin(prop_rotation),
        -sin(prop_rotation), cos(prop_rotation)
    );

    // Rotated and scale the position
    vec2 scaled = in_pos * rotation * prop_src.zw * prop_scale * constant_dpi_ratio;
    // Get the output position
    vec2 pos = (scaled + prop_dest * 2 * constant_dpi_ratio) / global_resolution;

    // Set the position
    gl_Position = vec4(pos, 0.0, 1.0);
}