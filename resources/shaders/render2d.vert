#version 460 core

in vec3 position;
in vec3 normals;
in vec2 uv_texture;

uniform mat4 u_mvp;

out vec2 v_tex_uv;

void main()
{
    v_tex_uv = uv_texture;
    gl_Position = u_mvp * vec4(position, 1.0f);
}