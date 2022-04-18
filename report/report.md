---
documentclass: scrartcl
title: Procedurally generated planets
course: TDT4230
subtitle: TDT4230 Final project
author: Andreas Klavenes Berg
date: \today
toc: false
---

# Project description

The project objective was to render a collection of procedurally generated planets, that is, a collection of high resolution spheres with distorted vertices following a noise function. While doing this, I wanted to explore the concept of Level of Detail, to allow rendering many planets at a high resolution while keeping a good performance. The project was inspired by Sebastian Lague's solar system project[^lague].

At the time of delivery, the project renders a scene with a collection of planets of varying size and colour scheme, arranged into some kind of a solar system. The starry background is as well generated from a noise function and projected on a skybox. Planets very far away from the camera are rendered just as a coloured dot on the skybox. This dot also functions as a halo representing an atmosphere when the mesh is rendered.

The player can move freely around in the scene, increasing and decreasing movement speed. When closing in on a planet, one can switch state into the "anchored" or the "landed" state. In both states, the player's up vector is set pointing up from the planet center. One will also notice that the quality of the terrain increases stepwise.

# Implementation

## Type of sphere

For the planets, I needed a sphere with vertices evenly distributed for a good continuity in the terrain, and which is easy to subdivide. There are multiple ways of connecting vertices into a sphere, where the UV-sphere and the icosphere are the most commonly used. The UV-sphere is maybe most intuitive to build, as it follows longitudes and latitudes, but its downside is that vertices are placed much tighter towards the poles. The icosphere is more evenly distributed but is more cumbersome to subdivide.

### Cubesphere

The mesh I selected was the cubesphere. It is created by making a subdivided cube, and projecting each vertex onto a sphere with a radius equal to the sides of the cube. The cube is quite intuitive to make as a combination of six planes, and it is easy to subdivide recursively by splitting a plane into four smaller ones. Lague abandoned this one as well, but I consider it a good choice for my application.

Projecting the vertices onto a sphere required some fancy math. One could simply normalize each vertex, but this would create shearing towards the corners of the plane. I found an [article](https://catlikecoding.com/unity/tutorials/cube-sphere/)[^cubesphere] providing a better approach of projecting in an even manner.

## Noise

For terrain generation, I used the `noise` crate[^noise-rs], which provides a powerfull toolbox for noise generation. I ended up only using its Perlin generator, but it has potensial for more sophisticated application.

## Level of Detail

The main technical goal of the project was to explore Level of Detail techniques, methods of focusing the detail level and the computational capacity where it matters. Initially, I tried an approach of rendering only the mesh of the closest planet, rendering the rest as implicit geometry[^cognite] on the skybox. However, this would require duplicating the noise function used on the CPU side in order to properly display other planets. I abandoned the pure idea, but kept the projection of faded, coloured dots as a lowest level of detail, when a planet is so far away one cannot distinguish any detail anyway. The fading also doubles as a glowing halo of an atmosphere for the rendered planets.

The main LoD technique used is the subdivision of the cubesphere sides, generating stepwise higher resolution terrains. Scene graphs nodes are arranged under the planet root in a quad tree for each side, where all rendered planets start with some 16 subdivisions on each plane. A LoD function traverses the tree and selects the requested level based on the player's distance from the planet and the angle to the segment's normal. If a node satisfies the conditions for increasing the LoD but the child nodes and higher resolution meshes don't exist yet, they are generated and added to the tree. In order to limit the tree depth and the number of draw calls, the number of subdivisions increase for each new level, in addition to there being four planes in place of one.

### Fearless concurrency – they say

The simple approach for generating detail would block the game loop when approaching a new planet, requiring lots of computation. To fix this, I did what the Rust language advertises it does so well, make it multithreaded. When reaching a node without a mesh, a thread is started to generate the mesh. When the thread exits, a ready flag is set, and the next time the node is visited it will have its vertex array object updated. The level above is used until all child nodes are ready. The number of threads allowed to run at once can be tuned according to the running computer.

## Adding some physics

Just free-floating and watching a static scene quickly becomes boring. To add some life to the player, I added a feature of "anchoring" to a planet. This *anchor* state sets the planet center as the center of gravity, levels the camera, and causes movement to stick to the sphere at a constant radius. Another state, *landed*, adds a gravitational pull, making the player fall and stick to the ground. It is now possible to walk (or rather hover) around any planet.

Another consideration when moving up close to a planet is not to fall through it. For this, I reused the noise function used to generate the mesh, by calling the `Planet::get_height` function, creating a continous hitbox matching the rendered mesh. This ability was a great consideration for keeping mesh generation on CPU instead of porting it to a compute shader.

We can now land on and walk around a planet, but the solar system is still completely static. This all changes with every planet getting a parent, a trajectory radius and a trajectory speed. Now, planet move in circular path around the sun in the middle, and moons move in circular paths around their planet, making a living scene for us to explore. 

## Skybox

The skybox displays a starry sky, which was shamelessly borrowed from this [blog post](https://www.overdraw.xyz/blog/2018/7/17/using-cellular-noise-to-generate-procedural-stars)[^stars]. Initially, the skybox was sized to limit of the clipping box and drawn last, but was a bit hacky. The chapter on skybox from LearnOpenLG[^skybox] teaches the method of setting the size to 1, but setting `w` after transform to 1 for far and drawing using the `GL_EQUAL` blending function.

## Scales and coordinates

Talking about clipping, the scene requires a wide range of z-buffering. When deciding to render all planets until a certain size, large planets far away would quickly get clipped. Likewise, with a clipping box adjusted for the large size, near clipping would cause issue when attempting to land on a planet. Trying to make a clipping box covering the entire range would create z-buffer conflicts. To remedy this, the scene is drawn over multiple passes, each using a different clipping box ranging from `(100.0, 165000.0)` down to `(0.0001, 0.5)`, and each time clearing the depth buffer. Some discussion on the [Khronos forum](https://community.khronos.org/t/near-and-far-clipping-ratios/32439/3)[^clipping] suggested something like this, but I mostly had to work it out myself.

### Floating point precision

A major issue when more planets were added and placed far apart was caused by floating point errors. When moving far away from the center, the entire scene and skybox would start shaking violently. Turns out requiring 3-4 decimal precision from floating points in the position `[15804.882, -3.0002, 48911.63]` was a bit much to ask for. The first step was to scale down the scene, although this merely moves the problem down the line.

Floating points will give the best precision close to zero, so a solution to the precision problem is to keep the positions that matter, those of the player and the anchored planet, as close to zero as possible. This is where I should add some funny, narcisitic comment. I'm keeping things as they are in the free-float player state, but when toggling the anchored state, two things happen:

1. The entire scene is scaled up by a constant factor.
2. The entire scene is moved so that the planet in focus is centered, and all other planets' movements are computed with this one as origin. This centers the planet and the player, and also removes some moving parts by having the ground being constant, further reducing the sources of error.

When returning to free-float state, the scene is scaled back down, and planet positions are computed from the sun as origin again. Though, the scene is not moved back to return the sun to the middle, this is a personacentric system ... or something. The floating point precision was perhaps the greatest challenge of the project.


[^lague]: https://www.youtube.com/watch?v=lctXaT9pxA0&t=512s
[^cubesphere]: https://catlikecoding.com/unity/tutorials/cube-sphere/
[^clipping]: https://community.khronos.org/t/near-and-far-clipping-ratios/32439/3
[^cognite]: Taking inspiration from the Cognite guest lecture about the topic.
[^noise-rs]: https://crates.io/crates/noise
[^stars]: https://www.overdraw.xyz/blog/2018/7/17/using-cellular-noise-to-generate-procedural-stars
[^skybox]: https://learnopengl.com/Advanced-OpenGL/Cubemaps
