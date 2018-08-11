#version 150 core

in vec2 in_pos;
in vec2 in_uv;
out vec2 out_uv;

/*layout (std140) uniform Properties {
    vec4 prop_src;
    vec4 prop_overlay_colour;
    vec2 prop_dest;
    float prop_rotation;
    float prop_scale;
};*/

uniform vec4 prop_src;
uniform vec4 prop_overlay_colour;
uniform vec2 prop_dest;
uniform float prop_rotation;
uniform float prop_scale;
uniform vec2 tileset_size;
uniform vec2 screen_resolution;

void main() {
    // Scale and translate the UV for the source, and then scale down for the tileset
    // TODO: figure out why the `vec2(0, 1)` needs to be here
    out_uv = vec2(0, 1) + (in_uv * prop_src.zw + prop_src.xy) / tileset_size;

    // Calculate the (clockwise) rotation matrix
    mat2 rotation = mat2(
        cos(prop_rotation), sin(prop_rotation),
        -sin(prop_rotation), cos(prop_rotation)
    );

    // Rotated and scale the position
    vec2 scaled = in_pos * rotation * prop_src.zw * prop_scale;
    // Get the output position
    vec2 pos = (scaled + prop_dest * 2) / screen_resolution;

    // Set the position
    gl_Position = vec4(pos, 0.0, 1.0);
}