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

vec2 screen_pos_to_opengl_pos(vec2 position) {
    return (vec2(0, 1) + vec2(1, -1) * position / screen_resolution - 0.5) * 2.0;
}

void main() {
    // Scale and translate the UV for the source, and then scale down for the tileset
    out_uv = vec2(0, 1) + (in_uv * prop_src.zw + prop_src.xy) / tileset_size;

    // Calculate the (clockwise) rotation matrix
    mat2 rotation = mat2(
        cos(prop_rotation), sin(prop_rotation),
        -sin(prop_rotation), cos(prop_rotation)
    );

    vec2 position = screen_pos_to_opengl_pos(prop_dest);

    vec2 image_size = in_pos * rotation / screen_resolution * prop_src.zw * prop_scale;

    gl_Position = vec4(position + image_size, 0.0, 1.0);
}