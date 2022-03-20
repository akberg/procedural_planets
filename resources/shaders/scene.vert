#version 430 core

in vec3 position;
in vec4 color;
in vec3 normal;
in vec2 uv;

out vec3 v_position;
out vec4 v_color;
out vec3 v_normal;
out vec2 v_uv;

uniform mat4 u_model;       // Transforms model into world coordinates
uniform mat4 u_mvp;         // Model-view-perspective matrix

void main()
{
    v_position = position;
    v_color = color;
    v_uv = uv;
    v_normal = normalize(mat3(u_model) * normal);
    gl_Position = u_mvp * vec4(v_position, 1.0f);
}