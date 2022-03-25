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