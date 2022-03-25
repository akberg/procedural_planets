#version 430 core

layout (binding = 0) uniform sampler2D u_texture;

#define NODE_TYPE_GEOMETRY      0
#define NODE_TYPE_SKYBOX        1
#define NODE_TYPE_GEOMETRY2D    2
#define NODE_TYPE_PLANET        3
#define NODE_TYPE_OCEAN         4

in vec3 v_position;
in vec4 v_color;
in vec3 v_normal;
in vec2 v_uv;
in vec3 v_model_position;

uniform uint u_node_type;
uniform bool u_has_texture;
// uniform float u_time; // TODO add

out vec4 color;

  const uint k = 1103515245U;  // GLIB C
//const uint k = 134775813U;   // Delphi and Turbo Pascal
//const uint k = 20170906U;    // Today's date (use three days ago's dateif you want a prime)
//const uint k = 1664525U;     // Numerical Recipes

// Hash functions taken from demo by Dave Hoskins: https://www.shadertoy.com/view/4djSRW
// 2 in 2 out
vec2 hash22(vec2 p)
{
	vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}
// 2 in 1 out
float hash12(vec2 p)
{
	vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}
// 2 in 3 out
vec3 hash32(vec2 p)
{
	vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yxz+33.33);
    return fract((p3.xxy+p3.yzz)*p3.zyx);
}
// 3 in 3 out
vec3 hash33(vec3 p3)
{
	p3 = fract(p3 * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yxz+33.33);
    return fract((p3.xxy + p3.yxx)*p3.zyx);
}
// 3 in 1 out
float hash13(vec3 p3)
{
	p3  = fract(p3 * .1031);
    p3 += dot(p3, p3.zyx + 31.32);
    return fract((p3.x + p3.y) * p3.z);
}

float rand1(in float x) { return fract(sin(x)*1e4); }
float rand2(vec2 co) { return fract(sin(dot(co.xy, vec2(12.9898,78.233))) * 43758.5453); }
float dither(vec2 uv) { return (rand2(uv)*2.0-1.0) / 256.0; }

float noise2d(vec2 st)
{
    vec2 i = floor(st);
    vec2 f = fract(st);

    // Four corners in 2D of a tile
    float a = rand2(i);
    float b = rand2(i + vec2(1.0, 0.0));
    float c = rand2(i + vec2(0.0, 1.0));
    float d = rand2(i + vec2(1.0, 1.0));

    // Smooth interpolation

    // Cubic Hermine Curbe, same as smoothstep(0.0, 1.0, f);
    vec2 u = f*f*(3.0-2.0*f);

    // Mix 4 corners percentages
    return mix(a, b, u.x) +
        (c - a) * u.y * (1.0 - u.x) +
        (d - b) * u.x * u.y;
}

float noise3d(vec3 p, float size)
{
    const vec3 _step = vec3(110.0, 241.0, 171.0);
    p *= size;

    vec3 i = floor(p);
    vec3 f = fract(p);

    float n = dot(i, _step);

    // Cubic Hermine Curbe, same as smoothstep(0.0, 1.0, f);
    vec3 u = f*f*(3.0-2.0*f);

    return mix( mix(mix(rand1(n + dot(_step, vec3(0,0,0))),
                        rand1(n + dot(_step, vec3(1,0,0))),
                        u.x),
                    mix(rand1(n + dot(_step, vec3(0,1,0))),
                        rand1(n + dot(_step, vec3(1,1,0))),
                        u.x),
                u.y),
                mix(mix(rand1(n + dot(_step, vec3(0,0,1))),
                        rand1(n + dot(_step, vec3(1,0,1))),
                        u.x),
                    mix(rand1(n + dot(_step, vec3(0,1,1))),
                        rand1(n + dot(_step, vec3(1,1,1))),
                        u.x),
                u.y),
            u.z);
}

float sphere_sdf(vec3 pos, vec3 origo, float r)
{
    return length(pos - origo) - r;
}

vec4 planet_shader();
vec4 skybox_shader();

void main()
{
    //color = vec4(1.0, 0.1, 0.9, 1.0);
    switch (u_node_type) {
    case NODE_TYPE_GEOMETRY:
    case NODE_TYPE_PLANET:
    case NODE_TYPE_OCEAN:
        color = planet_shader();
        break;
    case NODE_TYPE_SKYBOX:
        color = skybox_shader();
        break;
    case NODE_TYPE_GEOMETRY2D:
        color = texture(u_texture, v_uv);
        //color = vec4(v_uv.x, v_uv.y, 0.0, 1.0);
        break;
    default:
        color = vec4(0.0, 1.0, 0.0, 1.0);
        break;
    }
}

vec4 planet_shader()
{
    // Normal noise
    vec3 normal = v_normal;

    // Simple height map
    float radius = 10.0;
    float h = (length(v_position) - 0.5) * 2.0;
    vec3 diffuse_color = v_color.rgb;
    if (u_node_type == NODE_TYPE_PLANET) {
        if (h < -0.0001) {
            diffuse_color = vec3(0.4, 0.4, 0.3);
        }
        // else if (h > -0.001 && h < 0.001) {
        //     diffuse_color = vec3(0.2, 0.2, 0.7);
        // }
        else if (h < 0.001) {
            diffuse_color = vec3(0.7, 0.55, 0.0);
        }
        else if (h < 0.014) {
            diffuse_color = vec3(0.2, 0.6, 0.4);
        }
        else if (h < 0.024) {
            diffuse_color = vec3(0.5, 0.4, 0.4);
        }
        else {
            diffuse_color = vec3(1.0, 1.0, 1.0);
        }
    }
    // Lighting
    vec4 color;
    vec3 light = normalize(vec3(0.8, 1.0, 0.6));
    vec3 ambient_color = diffuse_color.rgb * 0.1; //vec3(0.1, 0.05, 0.1);
    vec3 specular_color = vec3(0.5, 0.2, 0.1);
    //vec3 diffuse_color = vec3(0.7, 0.7, 0.7);
    //vec3 diffuse_color = 0.7 * v_color.rgb;//(normal.rgb + 1.0) / 2.0;// * max(0, dot(v_normal, -light));

    float diffuse = max(dot(normalize(normal), normalize(light)), 0.0);
    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(normal)), 0.0), 16.0);

    // Use texture colours 
    //color = vec4(v_color.rgb * max(0, dot(v_normal, -light)), v_color.a);
    color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, v_color.a);
    // Colour using normals
    // float minr = 0.6;
    // float radius = 0.9-minr;
    //color = vec4(vec3((normalize(v_position) + 1.0) / 2.0) * (length(v_position)-minr) / radius, 1.0);
    //color = vec4(vec2(noise3d(v_normal, 10.0)), 1.0, 1.0);
    return color;
}


vec4 skybox_shader()
{
    vec3 sun = vec3(1.0, 1.0, 1.0);
    vec3 sun_pos = (normalize(sun) + 1.0) / 2.0;
    vec4 c;
    vec3 res = vec3(2.0, 2.0, 2.0);
    vec3 st = (normalize(v_model_position) + 1.0) / 2.0;
    //st *= 10.0;
    // vec2 ipos = floor(st);
    // vec2 fpos = fract(st);
    // float n = noise(st);
    // n = n > 0.95 ? n : 0.0;
    // vec3 color = vec3(n);
    if (sphere_sdf(st, sun_pos, 0.02) < 0) {
        c = vec4(0.8118, 0.3922, 0.0, 1.0);
    }
    else {
        // float v = rand2(vec2(rand2(st.xy), st.z));//(noise3d(st, 50.0) -0.2) * noise3d(st, 1000.0);

        // c = vec4(vec3(v > 0.9985 ? v : 0.0), 1.0);
        //c = vec4(st, 1.0);
        c = v_color;
    }

    return c;
}