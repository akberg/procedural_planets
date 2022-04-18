# Log
* 
* 17.4
    * A example from noise-rs shows how the crate can be used to create more advanced planet terrain (https://github.com/Razaekel/noise-rs/blob/develop/examples/complexplanet.rs)
* 14-16.4
    * Work on those damned trajectories. Finally modified so closest planet becomes center of scene when switching to Landed/Anchored mode, removing lots of moving parts from player-planet dynamics and removing some sources of floating point errors.
    * Extracted planet creation to file scene.rs. Shortens the gamelogic function and more easily allows creating different scenes.
    * Still no handling of planet rotation, shouldn't be too hard to add now.
    * Thinking of scaling the entire scene up and down instead of actual movement speed in order to keep the illusion of toggling "space speed" and "landed speed" but staying at 1 as the actual movement speed. Could further help on floating point precision.
* 13.4
    * Tried adding trajectories. Work quite well until landing on a planet. Floating point precision again... To make this work, I would need to change so that camera is always at zero and scene is moved.
* 12.4
    * Fixed lighting by completely destroying all good practices in coordinate spaces.
    * Last pressing tasks are multithreading terrain generation to hide work and avoid stopping the render thread, and fixing LoD conditions.
    * A simple extra would be adding more planets and giving them some movement in trajectories
    * Framerate drops fast as more terrain is generated. Reduced max LoD depth and increase subdivs for every level to improve quality faster with many fewer nodes.
    * Trying to extract mesh generation to another thread. Learnt the hard way that GL contexts are thread specific – no VAOs for me:( Trying to pack entire mesh in a mutex instead, unsure how that affects performance. Seems to work quite nicely, got at least rid of the full stops when generating a lot of new terrain.
    * Add another planet. Scene starts looking good
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

## Initial project description




For the final project I would like to make an attempt at creating a procedurally generated planet. I want to be able to render it from far away and have it become more detailed as the player closes in, which means having to apply some LoD techniques, e.g. partition the planet into chunks, resampling in higher resolution as a chunk comes closer, or adapting vertex count to local shape complexity. If this turns out too easy, I'd like to add more planets and place them in some solar system.

I wanted to explore cube marching, but not knowing whether it would be suitable for this task, I will look into it as well as optional methods for generating a terrain on a sphere, like noise textures or tessellation, or signed distance functions. Marching cubes is probably the best suited for allowing caves. If there's more time, I'd like to try adding bounding boxes so a player controlled camera can land on the planet.

Some of these ideas have been thoroughly explored by YouTube content creator Sebastian Lague, but he uses a full-fledged game engine and a different language, giving him things like bounding boxes for free. I hope to make use of Lague's videos as resources to save research time and spend more time implementing fun stuff.

I have a few spaceship 3D models from the last time I took the effort to try learning some modelling in Blender. It would be fun to add one of them to the scene, like a third person player-controlled ship or a star destroyer hovering over the atmosphere of the planet.

This brainstorming has left me with more ideas than I believe I'll be able to implement, but I really have no clue how difficult some of these things will be. So here's a somewhat prioritized checklist of subgoals. A realistic aim given the current workload in other courses and activities is probably just to reach the main goal.

- [x] Main goal: Procedurally generate a planet with a satisfying level of detail on its terrain, with a corresponding texture and light from a sunlight source, and have a controllable camera to fly around and explore it.
- [x] Allow the planet to have an ocean. Additional sphere with lower poly count.
- [x] Generate several planets and place them in a solar system.
- [x] Detect collision with the ground, so the player can land.
- [ ] Adding an atmosphere that modifies the sunlight.
- [ ] Loading some spaceship meshes to populate the vast space.
- [ ] Add particles, allowing the planet(s) to grow some trees, have some stones, or some active volcanos.

Keywords:
* Level of Detail techniques
* Types of sphere: UV, normalized, spherified cube, spherified pyramid, fibonacci, icosphere
* Merging VAOs to save draw calls? Subdividing a cube potentially creates many meshes to draw
* Terrain: Cube marching, or noise texture mapped to sphere
    * Noise texture applied to vertices. Try porting cubesphere generation and noise generation to compute shader
* Optionally extend to a little solar system, let planets have fixed orbits
* Collision detection to allow for a player to land on the planet (use `length(player_pos - planet_pos) > length(noise(vertex))`)
* Lighting: A sun, could try to add an atmosphere
* Particles: stones, trees
* Controller: A spaceship mesh
* Skybox: stars and ambient light. Implicit geometry: draw closest planet from vertex buffer, and draw all other objects on skybox from noise parameters and SDF.

Online resources

https://www.researchgate.net/publication/226410924_Spherical_Parameterization_of_Marching_Cubes_IsoSurfaces_Based_upon_Nearest_Neighbor_Coordinates

https://graphics.stanford.edu/~mdfisher/MarchingCubes.html

https://www.researchgate.net/publication/275971902_A_standardized_procedure_for_the_derivation_of_smooth_and_partially_overset_grids_on_the_sphere_associated_with_polyhedra_that_admit_regular_griddings_of_their_surfaces_Part_I_Mathematical_principles_

Multi-dimensional https://www.sciencedirect.com/science/article/pii/S002199911300538X


Icosphere: http://blog.andreaskahler.com/2009/06/creating-icosphere-mesh-in-code.html

Cubesphere, source for math: https://catlikecoding.com/unity/tutorials/cube-sphere/

Skybox: https://www.overdraw.xyz/blog/2018/7/17/using-cellular-noise-to-generate-procedural-stars

Ocean: https://www.shadertoy.com/view/Ms2SD1

Hash functions: https://www.shadertoy.com/view/4djSRW

