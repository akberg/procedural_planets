/* Main function, uniforms & utils */

#ifdef GL_ES
    precision mediump float;
#endif

float rand1(in float x) { return fract(sin(x)*1e4); }
float rand2(vec2 co) { return fract(sin(dot(co.xy, vec2(12.9898,78.233))) * 43758.5453); }
// 2 in 2 out
vec2 hash22(vec2 p)
{
    p *= 100.0;
	vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}
float hash12(vec2 p)
{
	vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

#define PI_TWO			1.570796326794897
#define PI				3.141592653589793
#define TWO_PI			6.283185307179586

/* Coordinate and unit utils */
vec2 coord(in vec2 p) {
    p = p / u_resolution.xy;
    // correct aspect ratio
    if (u_resolution.x > u_resolution.y) {
        p.x *= u_resolution.x / u_resolution.y;
        p.x += (u_resolution.y - u_resolution.x) / u_resolution.y / 2.0;
    } else {
        p.y *= u_resolution.y / u_resolution.x;
        p.y += (u_resolution.x - u_resolution.y) / u_resolution.x / 2.0;
    }
    // centering
    p -= 0.5;
    p *= vec2(-1.0, 1.0);
    return p;
}
#define rx 1.0 / min(u_resolution.x, u_resolution.y)
#define uv gl_FragCoord.xy / u_resolution.xy
#define st coord(gl_FragCoord.xy)
#define mx coord(u_mouse)
#define sti vec2(int(st.x * 512.0), int(st.y * 512.0))


void main() {
    // int idx = 0;
    // float dist = 10.0;
    // for (int i = 0; i < 9; i++) {
    //     float d = length(st - hash22(sti + vec2(pts[i%3], pts[i/3])));
    //     if (d < dist) {
    //         idx = i;
    //         dist = d;
    //     }
    // }
    // vec3 color = vec3(
    //     abs(cos(st.x + mx.x)), 
    //     abs(sin(st.y + mx.y)), 
    //     abs(sin(u_time))
    // );
    vec3 col = vec3((
        rand2(sti + vec2(-1.0,0.0)) +
        rand2(sti + vec2(-1.0,-1.0)) +
        rand2(sti + vec2(0.0,-1.0) +
        rand2(sti + vec2(0.0,0.0))) +
        rand2(sti + vec2(1.0,0.0)) +
        rand2(sti + vec2(1.0,1.0)) +
        rand2(sti + vec2(1.0,0.0)) +
        rand2(sti + vec2(1.0,-1.0)) +
        rand2(sti + vec2(-1.0,1.0))) / 9.0
       );
    vec3 color = vec3(hash22(sti).y, hash22(sti).y, hash22(sti).y);
    gl_FragColor = vec4(col, 1.0);
}