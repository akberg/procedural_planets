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

uniform float u_time;

uniform uint u_node_type;
uniform bool u_has_texture;

#define N_LAYERS 5

uniform struct Planet {
    uint planet_id;     // Unique ID for each planet
    // Geometry
    vec3 position;      // Planet's position
    vec3 rotation;      // Planet's rotation (for mapping noise correctly)
    uint radius;        // Planet's radius
    // Lighting
    vec3 emission;      // Emission colour (most relevant for a star)
    vec3 reflection;    // Reflection colour or quotient?
    // Terrain colours
    bool has_terrain;   // Does planet have terrain (false for stars, gas planets)
    vec3 color_scheme[N_LAYERS];        // Colours of height map
    float color_thresholds[N_LAYERS];   // Levels for changing colour
    float color_blending;               // Level of blending between colours
    // Ocean colours
    bool has_ocean;     // Does planet have an ocean
    vec3 ocean_dark_color;  // Colour of the ocean
    vec3 ocean_light_color; // Colour of the ocean
    // Other noise parameters
    float noise_size;       // Noise parameters
    float noise_height;
    float noise_seed;
} u_planets[];

// Array of planets
uniform int u_planets_len;
uniform uint u_closest_planet;

out vec4 color;

  const uint k = 1103515245U;  // GLIB C
//const uint k = 134775813U;   // Delphi and Turbo Pascal
//const uint k = 20170906U;    // Today's date (use three days ago's dateif you want a prime)
//const uint k = 1664525U;     // Numerical Recipes


//-----------------------------------------------------------------------------/
// noise.glsl header
//-----------------------------------------------------------------------------/
float rand1(in float x);
float rand2(vec2 co);
float dither(vec2 uv);

vec2 hash22(vec2 p);
float hash12(vec2 p);
vec3 hash32(vec2 p);
vec3 hash33(vec3 p3);
float hash13(vec3 p3);

float noise2d(vec2 p);
float noise3d(vec3 p);
//-noise.glsl header end-------------------------------------------------------/

float sphere_sdf(vec3 pos, vec3 origo, float r)
{
    return length(pos - origo) - r;
}

//-----------------------------------------------------------------------------/
// Shaders declarations
//-----------------------------------------------------------------------------/
vec4 planet_shader();
vec4 skybox_shader();
vec4 ocean_shader(vec3 ocean_dark_color, vec3 ocean_light_color);

void main()
{
    //color = vec4(1.0, 0.1, 0.9, 1.0);
    switch (u_node_type) {
    case NODE_TYPE_GEOMETRY:
    case NODE_TYPE_PLANET:
        color = planet_shader();
        break;
    case NODE_TYPE_OCEAN:
        color = ocean_shader(vec3(0.01, 0.2, 0.3), vec3(0.04, 0.3, 0.43));
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

vec4 ocean_shader(vec3 ocean_dark_color, vec3 ocean_light_color)
{
    vec3 normal = v_normal;
    vec3 diffuse_color = mix(ocean_dark_color, 1.3 * ocean_light_color, (
        0.5
        + 0.3 * noise3d(((vec3(u_time * 0.01, 0.0, 0.0) + v_position)) * 80.0) 
        + sin(u_time) * 0.1 * noise3d(v_position * 150.0)
    ));

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
    ), 0.93);
}

//-----------------------------------------------------------------------------/
// SKYBOX
// Renders a starry sky, and a texture of distant planets
//-----------------------------------------------------------------------------/
vec4 skybox_shader()
{
    vec3 pos = normalize(v_position);
    // One sun as proof of concept for implicit rendering of planets on skybox
    vec3 sun = vec3(1.0 + u_time * 1, 1.0 + u_time * 1, 1.0);
    vec3 sun_pos = (normalize(sun) + 1.0) / 2.0;
    float sun_rad = 1.0;

    float r = sin(atan(sun_rad / length(pos - sun)));

    // TODO: extend to apply for all but closest planet
    for (int i = 0; i < 1; i++) {
        if (sphere_sdf(pos, normalize(sun_pos), r) < 0) {
            return vec4(0.8118, 0.3922, 0.0, 1.0);
        }
    }
    // Starry sky if there's nothing else (not really optimized as this will have to be
    // computed anyway :/ )
    vec3 res = vec3(2.0, 2.0, 2.0);
    vec3 st = (normalize(pos) + 1.0) / 2.0;
    st *= 50.0;
    vec3 ipos = floor(st);
    vec3 fpos = fract(st);
    float n = noise3d(st);
    float sn = abs(n - 0.5);
    float radgrad = max(0.0, 1.0 - length(abs(fpos - 0.5)) / sn);
    radgrad *= radgrad;
    
    float v = rand2(vec2(rand2(st.xy), st.z));

    return vec4(vec3(radgrad), 1.0);

}