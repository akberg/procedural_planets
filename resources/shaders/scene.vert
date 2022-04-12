#version 430 core

in vec3 position;
in vec4 color;
in vec3 normal;
in vec2 uv;

out vec3 v_position;
out vec4 v_color;
out vec3 v_normal;
out vec2 v_uv;
out vec3 v_model_position;

uniform uint u_node_type;
uniform mat4 u_model;       // Transforms model into world coordinates
uniform mat4 u_mvp;         // Model-view-perspective matrix

void main()
{
    v_position = position;
    v_normal = normal;
    v_model_position = position;
    v_color = color;
    v_uv = uv;
    vec4 pos = u_mvp * vec4(v_position, 1.0f);
    gl_Position = (u_node_type == 1) ? pos.xyww : pos;

}