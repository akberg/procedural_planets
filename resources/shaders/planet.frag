#version 460 core

in vec3 v_position;
in vec4 v_color;
in vec3 v_normal;

out vec4 color;

float rand(vec2 co){
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

void main()
{
    vec3 light = normalize(vec3(0.8, 1.0, 0.6));
    vec3 ambient_color = v_color.rgb * 0.1; //vec3(0.1, 0.05, 0.1);
    vec3 specular_color = vec3(0.1, 0.1, 0.1);
    //vec3 diffuse_color = vec3(0.7, 0.7, 0.7);
    vec3 diffuse_color = 0.7 * v_color.rgb;// * max(0, dot(v_normal, -light));

    float diffuse = max(dot(normalize(v_normal), normalize(light)), 0.0);
    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

    //color = vec4(1.0, 0.1, 0.9, 1.0);
    //color = vec4(v_color.rgb * max(0, dot(v_normal, -light)), v_color.a);
    color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}