
float rand1(in float x) { return fract(sin(x)*1e4); }
float rand2(vec2 co) { return fract(sin(dot(co.xy, vec2(12.9898,78.233))) * 43758.5453); }
float dither(vec2 uv) { return (rand2(uv)*2.0-1.0) / 256.0; }

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
