---
title: Final project
subtitle: Procedurally generated planet
course: TDT4230
author: Andreas Klavenes Berg
date: \today
toc: true
---

# Project description

# Theory

# Implementation

## Type of sphere

### Cubesphere

# Showcase

# Log
* 11.4:
    * Branching to remove all implicit shading without completely losing it. Keeping some of the code to possibly render some halo/atmosphere. Though clipping is still a thing as well, but passed that a planet may get constant colour.
    * Size of planets array is still an issue, looking into Shader Storage Buffer Object (https://www.khronos.org/opengl/wiki/Shader_Storage_Buffer_Object)
    * Another issue is clip near/far and zbuffer rounding problems, found suggestions (https://community.khronos.org/t/near-and-far-clipping-ratios/32439/3) to do one draw pass for close objects with one near/far and another for far away objects using another clipping plane. Should also consider size-distance ratio for when to clip, or just not draw.
        - Added lots (4) of clip plane separations, allowing proper rendering at very close, thus allowing small meshes to be walked on like giant planets
    * If so, skybox should be reworked (https://learnopengl.com/Advanced-OpenGL/Cubemaps). I have done as the initial solution here. Did this, totally worth it.
* 10.4:
    * This 2-ways-of-rendering-geometry thing is getting really cumbersome, maybe I should keep everything volumetric. Some 8-16 subdivs won't be very resource heavy for other planets.
    * Seams can look more correct if all relatively close planes are equally subdivided, and if all neighbours are at most one detail level apart
* 9.4:
    * Improve collision detection. Add a 'Landed' player state with ambition of
    implementing some very simple physics.
    * Implemented gravitational pull when in 'Landed' state. Realizing that I still haven't done the really graphics deep-dive technical part of my project description. Still missing
        - Lighting, both on volume and implicit geometry
        - Texturing of implicit geometry from noise
        - Generating planets in compute shader
        - Conditionally increasing detail level by subdividing planes of cubesphere when getting too close
    * Implemented initial LoD, design seems to work but needs massive tunings, as well as some bug fixes on distance measurement.
    * Lots of memory is spent on generated terrain also when leaving a planet. Might want to free it. Didn't seem to be too important
    * Looking into trigonometry for texturing and lighting implicit geometries. Aaargh 
    * Oh, and I've reached limit for my number of uniforms. F***. Resources are saying I should use textures instead (https://www.khronos.org/opengl/wiki/Sampler_(GLSL)). Handling of uniform is apparently much better on Intel Iris integrated graphics than on an NVIDIA GTX970. Might be because Iris does not have a dedicated VRAM but shares the physical DRAM with the CPU (Figured that when I tried to measure VRAM usage on the laptop, didn't work)
* 7.4:
    * Work on controls, fixing directions and movement for free float and anchored. Might want to add distinction between anchored flying and landed and implement simple gravitational pull
* 6.4:
    * Keep radius uniform consistent with actual scale
    * Refactor game logic to keep `main.rs` small
    * Add height check to make collision box for planets (needs work on accuracy)
* 5.4:
    * Work on skybox shader to render implicit shapes for planets
    * Switching of closest planet
    * Planet uniforms, allowing differently coloured planets:)
* 4.4:
    * Figured noise::Perlin can be seeded. Add seed and other parameters
    * Timed Perlin on CPU, ~220 ms for 128 subdivisions, might be okay to keep it for now and focus on LoD/quad-tree
    * Change far and near clipping to reduce artifacts
    * Work on deciding parameters of planet generation, need to fix uniform passing soon
* 1.4:
    * Extracted noise GLSL functions to separate file, hacking a multifile shader
* 26.3:
    * Might have found the way of doing planets. Put planet generation into the new lod function of the Planet struct.
    * Created a few planets
* 25.3:
    * Failed to support resizable window, will return to this later.
    * Fixed some bug in `SceneNode::update\_buffers`
    * Stress tested a planet with 2048 subdivisions per plane, ~30s each to generate, chokes the GPU. Adds motivation for implementing quad-tree in order to get adequate LoD close to the ground.
    * Begin thinking about planet implementation and switching player state between floating and anchored to a planet.
    * Procrastinated the above tasks by looking up some ocean waves and adding a skybox with better stars.
* 24.3:
    * Add some primitive height colouring in shader as proof of concept/testing
* 22.3:
    * Solved issue with texture loading, can now display text.
    * Some refactoring of vao handling mesh/scene_graph
* 21.3:
    * Calculate normals for displaced normals, but cubesphere now gets an annoying seam (as Lague experienced). Will need to decide whether to stay with cubesphere and find a solution for this, or follow Lague in using a spherified pyramid.
* 20.3: 
    * Subdivided cube consisting of a mesh per side, first step towards a successful cubesphere.
    * Trying a bit on texture coordinates, 2d UVs will be challenging, maybe stick with position as texture parameter, which seems to work with a basic perlin noise.
    * Found a promising noise library which includes utilities for world generation.
* 19.3: 
    * Add skybox and experiment with noise. Generate subdivided plane
    * Player is now surrounded by a dark skybox rendering a half-hearted attempt at stars as well as an implicitly rendered sun (orange dot) to demonstrate how planets can be cheaply rendered from far. Might set it so only the closest planet is rendered from a mesh while the rest are projected on to the skybox.
    * Implemented vertex placement evenly distributed on one side of a cubesphere. Thinking of building the sphere from 6 curved face meshes, that can be subdivided. In short, making each side a quad-tree.
    * Tried porting the charmap code from earlier, but failed for now. Would be great to figure it out and be able to display text.
* 18.3:
    * Settled on raw gl with Rust, building on the Gloom-rs project used in Visual Computing.
    * Generated a cube, stealing generation code from the earlier assignments.