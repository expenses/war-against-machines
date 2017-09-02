#version 150 core

uniform sampler2D sampler;

in vec2 out_uv;
out vec4 target;

layout (std140) uniform Properties {
    vec4 prop_src;
    vec4 prop_overlay_colour;
    vec2 prop_dest;
    float prop_rotation;
    float prop_scale;
};

void main() {
    // Get the colour from the texture
    vec4 colour = texture(sampler, out_uv);

    // Mix it with the overlay colour
    vec3 mixed_colour = mix(colour.rgb, prop_overlay_colour.rgb, prop_overlay_colour.a);

    // Return the mixed colour (with the alpha unchanged)
    target = vec4(mixed_colour, colour.a);
}