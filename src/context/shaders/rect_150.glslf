#version 150 core

uniform sampler2D sampler;

in vec2 out_uv;
out vec4 target;

layout (std140) uniform Properties {
    vec4 prop_src;
    vec4 prop_dest;
    vec4 prop_overlay_colour;
    float prop_rotation;
};

void main() {
    // Get the colour of the texel
    vec4 colour = texture(sampler, out_uv);

    // Get the amount to mix (interpolate) the colours by
    float mix_amount = prop_overlay_colour.a;
    // Mix the colours!
    vec3 mixed_colour = colour.rgb * (1 - mix_amount) + prop_overlay_colour.rgb * mix_amount;

    // Return the mixed colour (with the alpha unchanged)
    target = vec4(mixed_colour, colour.a);
}