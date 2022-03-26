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

uniform float u_time; // TODO add

uniform uint u_node_type;
uniform bool u_has_texture;

uniform struct Planet {
    uint planet_id;
    vec3 position;
    vec3 rotation;
    uint radius;
    vec3 emission;
    bool has_ocean;
    vec3 ocean_color;
    float noise_size;
    float noise_height;
} u_planets[];
uniform int u_planets_len;
uniform uint u_closest_planet;

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

float noise3d(vec3 p)
{
    const vec3 _step = vec3(110.0, 241.0, 171.0);

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
vec4 ocean_shader(vec3 ocean_color);

void main()
{
    //color = vec4(1.0, 0.1, 0.9, 1.0);
    switch (u_node_type) {
    case NODE_TYPE_GEOMETRY:
    case NODE_TYPE_PLANET:
        color = planet_shader();
        break;
    case NODE_TYPE_OCEAN:
        color = ocean_shader(vec3(0.05, 0.4, 0.7));
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

vec3 phong_light(
    vec3 diffuse_color, 
    vec3 ambient_color, 
    vec3 specular_color, 
    vec3 normal, 
    vec3 light
) {
    // Lighting
    float diffuse = max(dot(normalize(normal), normalize(light)), 0.0);
    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(normal)), 0.0), 32.0);

    return vec3(
        ambient_color + diffuse * diffuse_color + specular * specular_color
    );
}


//-----------------------------------------------------------------------------/
// "Seascape" by Alexander Alekseev aka TDM - 2014
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.
// Contact: tdmaav@gmail.com
//-----------------------------------------------------------------------------/
// sea constants (to be made uniforms)
const float PI	 	= 3.141592;
const float EPSILON	= 1e-3;
const int ITER_GEOMETRY = 3;
const int ITER_FRAGMENT = 5 * 2;
const float SEA_HEIGHT = 0.3 / 8.0;
const float SEA_CHOPPY = 8.0;
const float SEA_SPEED = 0.8 / 64.0;
const float SEA_FREQ = 0.16 * 32.0;
const vec3 SEA_BASE = vec3(0.0,0.09,0.18);
const vec3 SEA_WATER_COLOR = vec3(0.8,0.9,0.6)*0.6;
#define SEA_TIME (1.0 + u_time * SEA_SPEED)
const mat2 octave_m = mat2(1.6,1.2,-1.2,1.6);
// sea
float sea_octave(vec2 uv, float choppy) {
    uv += noise2d(uv);        
    vec2 wv = 1.0-abs(sin(uv));
    vec2 swv = abs(cos(uv));    
    wv = mix(wv,swv,wv);
    return pow(1.0-pow(wv.x * wv.y,0.65),choppy);
}
float map_detailed(vec3 p) {
    float freq = SEA_FREQ;
    float amp = SEA_HEIGHT;
    float choppy = SEA_CHOPPY;
    vec2 uv = p.xz; uv.x *= 0.75;
    
    float d, h = 0.0;    
    for(int i = 0; i < ITER_FRAGMENT; i++) {        
    	d = sea_octave((uv+SEA_TIME)*freq,choppy);
    	d += sea_octave((uv-SEA_TIME)*freq,choppy);
        h += d * amp;        
    	uv *= octave_m; freq *= 1.9; amp *= 0.22;
        choppy = mix(choppy,1.0,0.2);
    }
    return p.y - h;
}
// vec3 getSkyColor(vec3 e) {
//     e.y = (max(e.y,0.0)*0.8+0.2)*0.8;
//     return vec3(pow(1.0-e.y,2.0), 1.0-e.y, 0.6+(1.0-e.y)*0.4) * 1.1;
// }
// vec3 getSeaColor(vec3 p, vec3 n, vec3 l, vec3 eye, vec3 dist) {  
//     float fresnel = clamp(1.0 - dot(n,-eye), 0.0, 1.0);
//     fresnel = pow(fresnel,3.0) * 0.5;
        
//     vec3 reflected = getSkyColor(reflect(eye,n));    
//     vec3 refracted = SEA_BASE + diffuse(n,l,80.0) * SEA_WATER_COLOR * 0.12; 
    
//     vec3 color = mix(refracted,reflected,fresnel);
    
//     float atten = max(1.0 - dot(dist,dist) * 0.001, 0.0);
//     color += SEA_WATER_COLOR * (p.y - SEA_HEIGHT) * 0.18 * atten;
    
//     color += vec3(specular(n,l,eye,60.0));
    
//     return color;
// }
vec3 get_normal(vec3 p, float eps) {
    vec3 n;
    n.y = map_detailed(p);    
    n.x = map_detailed(vec3(p.x+eps,p.y,p.z)) - n.y;
    n.z = map_detailed(vec3(p.x,p.y,p.z+eps)) - n.y;
    n.y = eps;
    return normalize(n);
}

vec4 ocean_shader(vec3 ocean_color)
{
    vec3 normal = get_normal(v_position, EPSILON);
    vec3 diffuse_color = ocean_color;

    vec3 light = -normalize(vec3(0.8, 1.0, 0.6));
    vec3 dir = -v_position;

    // return mix(
    //     getSkyColor(dir),
    //     getSeaColor(p,n,light,dir,dir),
    // 	pow(smoothstep(0.0,-0.02,dir.y),0.2));

    return vec4(phong_light(
        diffuse_color, 
        diffuse_color * 0.2, 
        vec3(1.0, 1.0, 1.0), 
        normal, 
        normalize(vec3(0.8, 1.0, 0.6))
    ), 1.0);
}

//-----------------------------------------------------------------------------/
// SKYBOX
// Renders a starry sky, and a texture of distant planets
//-----------------------------------------------------------------------------/
vec4 skybox_shader()
{
    vec3 pos = normalize(v_position);
    // pos = normalize(vec3(
    //     pos.x * sqrt(1.0 - pow(pos.y, 2) / 2.0 - pow(pos.z, 2) / 2.0 + pow(pos.y, 2) * pow(pos.z, 2) / 3.0),
    //     pos.y * sqrt(1.0 - pow(pos.x, 2) / 2.0 - pow(pos.z, 2) / 2.0 + pow(pos.x, 2) * pow(pos.z, 2) / 3.0),
    //     pos.z * sqrt(1.0 - pow(pos.x, 2) / 2.0 - pow(pos.y, 2) / 2.0 + pow(pos.x, 2) * pow(pos.y, 2) / 3.0)
    // ));
    vec3 sun = vec3(1.0, 1.0, 1.0);
    vec3 sun_pos = (normalize(sun) + 1.0) / 2.0;
    vec4 c;
    vec3 res = vec3(2.0, 2.0, 2.0);
    vec3 st = (normalize(pos) + 1.0) / 2.0;
    st *= 50.0;
    vec3 ipos = floor(st);
    vec3 fpos = fract(st);
    float n = noise3d(st); // + noise3d(st * 2) * 0.5;// + noise3d(st * 4) * 0.25;
    float sn = abs(n - 0.5);
    float radgrad = max(0.0, 1.0 - length(abs(fpos - 0.5)) / sn);
    radgrad *= radgrad;

    // n = n > 0.95 ? n : 0.0;
    // vec3 color = vec3(n);
    if (sphere_sdf(st, sun_pos, 0.02) < 0) {
        c = vec4(0.8118, 0.3922, 0.0, 1.0);
    }
    else {
        float v = rand2(vec2(rand2(st.xy), st.z));//(noise3d(st, 50.0) -0.2) * noise3d(st, 1000.0);

        //c = vec4(vec3(v > 0.9985 ? v : 0.0), 1.0);
        c = vec4(vec3(radgrad), 1.0);
        //c = vec4(pos, 1.0);
    }

    return c;
}