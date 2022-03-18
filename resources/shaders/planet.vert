#version 460 core

in vec3 position;
in vec3 normal;
in vec2 v_texture;

out vec3 v_position;
out vec3 v_normal;
out vec3 v_texture;


void main()
{
    v_position = position;
    gl_Position = vec4(position, 0.0, 1.0);

}