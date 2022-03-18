#version 460 core

layout (binding = 0) uniform sampler2D charmap;

in vec2 v_tex_uv;

out vec4 color;

void main()
{
    color = texture(charmap, v_tex_uv);
    //color = vec4(v_texUV.x, 0.0, 0.0, 1.0);
}