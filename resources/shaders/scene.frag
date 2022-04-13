#version 430 core

layout (binding = 0) uniform sampler2D u_texture;

#define NODE_TYPE_GEOMETRY      0
#define NODE_TYPE_SKYBOX        1
#define NODE_TYPE_GEOMETRY2D    2
#define NODE_TYPE_PLANET        3
#define NODE_TYPE_OCEAN         4
float specular_multiplier[] = {0.0, 0.0, 0.0, 0.2, 0.5};

in vec3 v_position;
in vec4 v_color;
in vec3 v_normal;
in vec2 v_uv;
in vec3 v_model_position;

uniform float u_time;
uniform vec3 u_player_position;
uniform mat4 u_model;
uniform mat4 u_mvp;
uniform mat4 u_perspective;
uniform mat4 u_view;

uniform uint u_node_type;
uniform uint u_current_planet_id;   // Just in case multiple planets should be rendered
uniform bool u_has_texture;

#define N_LAYERS 5

// Array of planets
#define MAX_PLANETS 64
uniform uint u_planets_len;
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
uniform uint u_planet_ids_sorted[MAX_PLANETS];

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

vec3 reject(vec3 from, vec3 onto)
{
    return from - onto * dot(from, onto) / dot(onto, onto);
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
vec4 ocean_shader(
    vec3 v_position, 
    vec3 v_normal, 
    vec3 ocean_dark_color, 
    vec3 ocean_light_color
);

void main()
{
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

//-----------------------------------------------------------------------------/
// Set colour according to colour scheme and height scheme, and apply lighting
//-----------------------------------------------------------------------------/
vec4 planet_shader(vec3 position, vec3 normal, uint planet_id)
{
    float radius = 10.0;
    float h = (length(position) - 0.5) * 2.0;
    vec3 diffuse_color;
    //-------------------------------------------------------------------------/
    // Simple height map sets diffuse colour
    //-------------------------------------------------------------------------/
    if (h < u_planets[planet_id].color_thresholds[0]) {
        diffuse_color = u_planets[planet_id].color_scheme[0];
        //diffuse_color = vec3(0.9137, 0.5176, 0.0);
    }
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

    //-------------------------------------------------------------------------/
    // Lighting
    //  Ambient colour is full diffuse colour for lightsources
    //  Specular colour is the lightsource's emission colour
    //-------------------------------------------------------------------------/
    vec3 ambient_color = diffuse_color.rgb * (
        u_planets[planet_id].lightsource ? 1.0 : 0.18
        );

    return phong_light(
        diffuse_color, 
        ambient_color, 
        position,
        normal, 
        1.0
    );
}

//-----------------------------------------------------------------------------/
// Apply Phong lighting for all lightsources in the scene
//-----------------------------------------------------------------------------/
// TODO: Add shadows and light intencity
vec4 phong_light(
    vec3 diffuse_color, 
    vec3 ambient_color, 
    vec3 position,      // vertex position from model, in model scale
    vec3 normal,
    float alpha
) {
    vec3 color = ambient_color;

    vec3 planet_center = u_planets[u_current_planet_id].position;
    mat3 normal_matrix = transpose(inverse(mat3(u_model)));
    position += planet_center;
    normal = normal_matrix * normal;

    // Lighting
    vec3 light;
    float diffuse;
    vec3 camera_dir = normalize(u_player_position-position);
    vec3 half_direction;
    vec3 specular_color;
    float specular = 0.0;// = pow(max(dot(half_direction, normalize(normal)), 0.0), 32.0);

    for (int i = 0; i < u_lightsources_len; i++) {
        uint light_id = u_lightsources[i];
        light = u_planets[light_id].position;
        vec3 light_dir = light - position;

        diffuse = max(dot(normalize(normal), normalize(light_dir)), 0.0) * 0.5;
        half_direction = normalize(normalize(light_dir) + camera_dir);
        specular = pow(max(dot(half_direction, normalize(normal)), 0.0), 4.0);
        specular *= specular_multiplier[u_node_type];
        specular_color = u_planets[light_id].emission;
        color += vec3(diffuse * diffuse_color + specular * specular_color);
    }
    return vec4(color, alpha);
}

//-----------------------------------------------------------------------------/
// Just blending dark and light ocean color with some simple time variant noise
//-----------------------------------------------------------------------------/
vec4 ocean_shader(
    vec3 v_position, 
    vec3 v_normal, 
    vec3 ocean_dark_color, 
    vec3 ocean_light_color
) {
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
        //---------------------------------------------------------------------/
        // Draw coloured, faded dots on planet positions, doubles as lowest LoD
        // and atmosphere (latter has some issues).
        // Might need to implement shadow here as well as in planet shader
        // (maybe extract to separate function)
        //---------------------------------------------------------------------/
        uint ii = u_planet_ids_sorted[i];
        vec3 ce = u_planets[ii].position;    // Center of planet

        vec3 dir = ce - ro;         // Direction from player to planet center
        vec3 dir_n = normalize(dir);

        float ra = u_planets[ii].radius;
        float r = ra / length(dir);    // Perceived radius from view

        float sdf_halo = sphere_sdf(rd, dir_n, r * 1.3); // atmosphere 30% of radius
        float sdf = sphere_sdf(rd, dir_n, r);

        if (sdf_halo < 0) {
            vec4 ci;
            ci.rgb = u_planets[ii].emission;
            ci.a = min(1.0, -sdf_halo / r * 2.0 - sdf_halo/5.0);
            if (closest_element == -1) {
                c = ci;
            } 
            else {
                c += ci * (1.0 - c.a);
            }
            closest_element = length(dir);
        }
    }

    //-------------------------------------------------------------------------/
    // Starry sky if there's nothing else (not really optimized as this will 
    // have to be computed anyway :/ )
    //-------------------------------------------------------------------------/
    vec3 res = vec3(2.0, 2.0, 2.0);
    vec3 st = (normalize(rd) + 1.0) / 2.0;
    st *= 50.0;
    vec3 ipos = floor(st);
    vec3 fpos = fract(st);
    float n = noise3d(st);
    float sn = abs(n - 0.5) * 0.9;
    float radgrad = max(0.0, 1.0 - length(abs(fpos - 0.5)) / sn);
    radgrad *= radgrad;
    
    float v = rand2(vec2(rand2(st.xy), st.z));

    return vec4(mix(vec3(radgrad), c.rgb, c.a), 1.0);
}