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
uniform vec3 u_player_position;

uniform uint u_node_type;
uniform uint u_current_planet_id;   // Just in case multiple planets should be rendered
uniform bool u_has_texture;

#define N_LAYERS 5

// Array of planets
#define MAX_PLANETS 64
uniform uint u_planets_len;
// uniform uint u_closest_planet;
uniform struct Planet {
    uint planet_id;     // Unique ID for each planet
    // Geometry
    vec3 position;      // Planet's position
    // vec3 rotation;      // Planet's rotation (for mapping noise correctly)
    float radius;        // Planet's radius
    // Lighting
    bool lightsource;   // True if planet is a lightsource
    vec3 emission;      // Emission colour (most relevant for a star)
    vec3 reflection;    // Reflection colour or quotient? Just reuse emission?
    // Terrain colours
    vec3 color_scheme[N_LAYERS];        // Colours of height map
    float color_thresholds[N_LAYERS-1]; // Levels for changing colour
    float color_blending;               // Level of blending between colours
    vec3 ocean_dark_color;  // Colour of the ocean
    vec3 ocean_light_color; // Colour of the ocean
} u_planets[MAX_PLANETS];

uniform uint u_lightsources_len;
uniform uint u_lightsources[MAX_PLANETS];

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

float fractal_noise3d(vec3 pos, float size, float height);
//-noise.glsl header end-------------------------------------------------------/

float sphere_sdf(vec3 pos, vec3 origo, float r)
{
    return length(pos - origo) - r;
}
// sphere of size ra centered at point ce, as implemented at https://www.iquilezles.org/www/articles/intersectors/intersectors.htm
// (Cognite guest lecture)
vec2 sphIntersect( in vec3 ro, in vec3 rd, in vec3 ce, float ra )
{
    vec3 oc = ro - ce;
    float b = dot( oc, rd );
    float c = dot( oc, oc ) - ra*ra;
    float h = b*b - c;
    if( h<0.0 ) return vec2(-1.0); // no intersection
    h = sqrt( h );
    return vec2( -b-h, -b+h );
}

//-----------------------------------------------------------------------------/
// Shaders declarations
//-----------------------------------------------------------------------------/
vec4 phong_light(
    vec3 diffuse_color, 
    vec3 ambient_color, 
    vec3 position,
    vec3 normal,
    float alpha
);
vec4 planet_shader(vec3 position, vec3 normal, uint planet_id);
vec4 skybox_shader();
vec4 ocean_shader(vec3 v_position, vec3 v_normal, vec3 ocean_dark_color, vec3 ocean_light_color);

void main()
{
    //color = vec4(1.0, 0.1, 0.9, 1.0);
    switch (u_node_type) {
    case NODE_TYPE_GEOMETRY:
    case NODE_TYPE_PLANET:
        color = planet_shader(v_position, v_normal, u_current_planet_id);
        break;
    case NODE_TYPE_OCEAN:
        color = ocean_shader(
            v_position, v_normal,
            u_planets[u_current_planet_id].ocean_dark_color, 
            u_planets[u_current_planet_id].ocean_dark_color
        );
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

vec4 planet_shader(vec3 position, vec3 normal, uint planet_id)
{
    // Simple height map
    float radius = 10.0;
    float h = (length(position) - 0.5) * 2.0;
    vec3 diffuse_color;// = vec3(0.0, 1.0, 0.0);//v_color.rgb;
    //if (u_node_type == NODE_TYPE_PLANET) {
        if (h < u_planets[planet_id].color_thresholds[0]) {
            diffuse_color = u_planets[planet_id].color_scheme[0];
            //diffuse_color = vec3(0.9137, 0.5176, 0.0);
        }
        // else if (h > -0.001 && h < 0.001) {
        //     diffuse_color = vec3(0.2, 0.2, 0.7);
        // }
        else if (h < u_planets[planet_id].color_thresholds[1]) {
            diffuse_color = u_planets[planet_id].color_scheme[1];
            //diffuse_color = vec3(0.4588, 0.4588, 0.4588);
        }
        else if (h < u_planets[planet_id].color_thresholds[2]) {
            diffuse_color = u_planets[planet_id].color_scheme[2];
            //diffuse_color = vec3(0.2, 0.6, 0.4);
        }
        else if (h < u_planets[planet_id].color_thresholds[3]) {
            diffuse_color = u_planets[planet_id].color_scheme[3];
            //diffuse_color = vec3(0.5, 0.4, 0.4);
        }
        else {
            diffuse_color = u_planets[planet_id].color_scheme[4];
            //diffuse_color = vec3(1.0, 1.0, 1.0);
        }
    //}
    // Lighting
    vec3 ambient_color = diffuse_color.rgb * (u_planets[planet_id].lightsource ? 1.0 : 0.2); //vec3(0.1, 0.05, 0.1);
    vec3 specular_color; // = vec3(0.5, 0.2, 0.1);

    vec3 camera_dir = normalize(-position);
    vec3 half_direction; // = normalize(normalize(light) + camera_dir);
    float specular; // = pow(max(dot(half_direction, normalize(normal)), 0.0), 16.0);

    // Use texture colours 
    //color = vec4(v_color.rgb * max(0, dot(v_normal, -light)), v_color.a);
    color = phong_light(
        diffuse_color, 
        ambient_color, 
        position,
        normal, 
        1.0//normalize(v_position - u_planets[light_id].position)
    );

    return color; //vec4(color, v_color.a);
}

// Apply Phong lighting for all lightsources in the scene
// TODO: Add shadows and light intencity
vec4 phong_light(
    vec3 diffuse_color, 
    vec3 ambient_color, 
    vec3 position,
    vec3 normal,
    float alpha
) {
    vec3 color = ambient_color;
    // Lighting
    vec3 light;
    float diffuse;// = max(dot(normalize(normal), normalize(light)), 0.0);
    vec3 camera_dir = normalize(-position);
    vec3 half_direction;// = normalize(normalize(light) + camera_dir);
    vec3 specular_color;
    float specular;// = pow(max(dot(half_direction, normalize(normal)), 0.0), 32.0);

    for (int i = 0; i < u_lightsources_len; i++) {
        uint light_id = u_lightsources[i];
        light = normalize(position - u_planets[light_id].position);
        diffuse = max(dot(normalize(normal), normalize(light)), 0.0) * 0.5;
        half_direction = normalize(normalize(light) + camera_dir);
        specular = pow(max(dot(half_direction, normalize(normal)), 0.0), 16.0);
        specular_color = u_planets[light_id].emission;
        color += vec3(diffuse * diffuse_color + specular * specular_color);
    }
    return vec4(color, alpha);
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
/*
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
*/
vec3 get_normal(vec3 p, float eps) {
    vec3 n;
    n.y = map_detailed(p);    
    n.x = map_detailed(vec3(p.x+eps,p.y,p.z)) - n.y;
    n.z = map_detailed(vec3(p.x,p.y,p.z+eps)) - n.y;
    n.y = eps;
    return normalize(n);
}

vec4 ocean_shader(vec3 v_position, vec3 v_normal, vec3 ocean_dark_color, vec3 ocean_light_color)
{
    vec3 normal = v_normal;
    vec3 diffuse_color = mix(ocean_dark_color, 1.3 * ocean_light_color, (
        0.5
        + 0.3 * noise3d(((vec3(u_time * 0.01, 0.0, 0.0) + v_position)) * 80.0) 
        + sin(u_time) * 0.1 * noise3d(v_position * 150.0)
    ));

    vec4 color = phong_light(
        diffuse_color, 
        diffuse_color * 0.2,
        v_position,
        normal, 
        0.93
    );
    // return mix(
    //     getSkyColor(dir),
    //     getSeaColor(p,n,light,dir,dir),
    // 	pow(smoothstep(0.0,-0.02,dir.y),0.2));

    return color;
}

//-----------------------------------------------------------------------------/
// SKYBOX
// Renders a starry sky, and a texture of distant planets
//-----------------------------------------------------------------------------/
vec4 skybox_shader()
{
    vec4 c = vec4(0.0);
    float closest_element = -1.0;

    vec3 rd = normalize(v_position);    // Texture position on skybox -> ray direction
    vec3 ro = u_player_position;        // Ray origin

    for (int i = 0; i < u_planets_len; i++) {
        vec3 ce = u_planets[i].position;    // Center of planet

        vec3 dir = ce - ro;         // Direction from player to planet center
        vec3 dir_n = normalize(dir);//(normalize(p_pos) + 1.0) / 2.0;

        float ra = u_planets[i].radius;
        float r = ra / length(dir);    // Perceived radius from view

        float sdf_halo = sphere_sdf(rd, dir_n, r * 1.3); // atmosphere 10% of radius
        float sdf = sphere_sdf(rd, dir_n, r);

        if ((length(dir) < closest_element || closest_element == -1) // Oclusion culling
            //&& u_closest_planet != i // Comment out to show when volume is clipped
            && sdf < 0 
        ) {
            c.rgb = u_planets[i].emission;
            c.a = -sdf / r;//min(1.0, -sdf * r); //0.5; //sdf < 0 ? 1.0 : -sdf_halo / length(dir);
            closest_element = length(dir);
        }
    }

    //if (closest_element != -1) return c;

    // Starry sky if there's nothing else (not really optimized as this will have to be
    // computed anyway :/ )
    vec3 res = vec3(2.0, 2.0, 2.0);
    vec3 st = (normalize(rd) + 1.0) / 2.0;
    st *= 50.0;
    vec3 ipos = floor(st);
    vec3 fpos = fract(st);
    float n = noise3d(st);
    float sn = abs(n - 0.5);
    float radgrad = max(0.0, 1.0 - length(abs(fpos - 0.5)) / sn);
    radgrad *= radgrad;
    
    float v = rand2(vec2(rand2(st.xy), st.z));

    return vec4(mix(vec3(radgrad), c.rgb, c.a), 1.0);
}