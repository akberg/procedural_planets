use nalgebra_glm as glm;
use crate::scene_graph::{self, SceneNodeType};
use crate::{util, mesh, shader::Shader};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

static PLANET_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Thresholds for level of detail
const MAX_LOD: usize = 6;
const THRESHOLD: [f32; MAX_LOD] = [128.0, 32.0, 16.0, 8.0, 4.0, 2.0];
const SUBDIVS_PER_LEVEL: usize = 128; // 256: 480+380=860ms, 128: 127+98=225ms
const N_LAYERS: usize = 5;


/// Procedurally generated planet. Will use a quad-tree form, each side
/// either drawing a plane or subdividing into nodes covering recursively
/// smaller planes. 
/// 
/// SceneNodeType::Empty used to mark a layer 
///
/// Expected usage:
/// - Create object containing parameters for generating planet
/// - Object controls generating subdivided meshes as needed, and how deep
/// to render, connecting with the scene graph
/// - Sun: Set low maximum LoD for terrain or completely avoid generating 
/// terrain, set high emission and get ocean as sun surface
/// ```
/// let planet0 = Planet::new(600.0)    // radius
///     .position(glm::vec3(0.0, 0.0, 0.0))
///     .height(1.0)
///     .noise_params({ size: 3.5, niter: 5, .. });
/// 
/// scene_root.add_child(planet0.node);
/// ```
/// Planet in the scene graph
/// ----planet_root : controls planet scale, rotation and position
///     |---ocean : (if has_ocean)
///     |   |---front
///     |   |---back
///     |   |---left
///     |   |---right
///     |   |---top
///     |   +---bottom
///     +---terrain : (if has_terrain) height texture colours
///         |---front : lod 0
///         |   |---q0 : lod 1
///         |   |---q1 : lod 1
///         |   |---q2 : skip
///         |   |   |---q0 : lod 2
///         |   |   |---q1 : lod 2
///         |   |   |---q2 : lod 2
///         |   |   +---q3 : lod 2
///         |   +---q3
///         |---back
///         |---left
///         |---right
///         |---top
///         +---bottom
#[derive(Default, Debug)]
pub struct Planet {
    //pub node        : scene_graph::Node,// scene node kept separate
    pub planet_id   : usize,
    //pub position    : glm::TVec3<f32>,// Handled by scene node
    //pub rotation    : glm::TVec3<f32>,// Handled by scene node
    pub radius      : f32,              // Radius to ocean level

    // Lighting
    pub emission    : glm::TVec3<f32>,  // Emission colour and intensity
    pub reflection  : glm::TVec3<f32>,  // Reflection colour and intensity
    // Terrain
    pub has_terrain : bool,
    pub color_scheme: [glm::TVec3<f32>; N_LAYERS],
    pub coloru_thresholds   : [f32; N_LAYERS],
    pub color_blending      : f32,
    // Ocean colours
    pub has_ocean   : bool,             // Set true to include ocean
    pub ocean_lvl   : f32,              // offset from radius
    pub ocean_dark_color    : glm::TVec3<f32>,
    pub ocean_light_color   : glm::TVec3<f32>,

    pub noise_fn    : noise::Perlin,
    pub noise_params    : PlanetParameters,
}

use noise::*;
impl Planet {
    pub fn new() -> Self {
        let planet_id = PLANET_COUNTER.fetch_add(1, Ordering::Relaxed) as usize;

        Planet {
            radius      : 1.0,
            planet_id,
            emission    : glm::vec3(0.0, 0.0, 0.0),
            has_ocean   : true,
            ocean_lvl   : 0.0,
            noise_fn    : noise::Perlin::new(),
            noise_params: PlanetParameters { seed: rand::random::<_>(), ..Default::default() },
            ..Default::default()
        }
    }
    pub fn with_seed(seed: u32) -> Self {
        let planet_id = PLANET_COUNTER.fetch_add(1, Ordering::Relaxed) as usize;

        Planet {
            radius      : 1.0,
            planet_id,
            emission    : glm::vec3(0.0, 0.0, 0.0),
            has_ocean   : true,
            ocean_lvl   : 0.0,
            noise_fn    : noise::Perlin::new().set_seed(seed),
            noise_params: PlanetParameters { ..Default::default() },
            ..Default::default()
        }
    }
    
    /// Update uniforms for planet in shader
    pub unsafe fn update_uniforms(&self, sh: &Shader) {
        let u_planet_id = sh.get_uniform_location(&format!("u_planets[{}].planet_id", self.planet_id));
        gl::Uniform1ui(u_planet_id, self.planet_id as u32);
        let u_radius = sh.get_uniform_location(&format!("u_planets[{}].radius", self.planet_id));
        // TODO: Get absolute position and other attributes
    }
    /// Set level of detail to be drawn, generate new if needed
    pub unsafe fn lod(&self, 
        node: &mut scene_graph::SceneNode, 
        player_pos: glm::TVec3<f32>
    ) {
        let rotations: [glm::TVec3<f32>; 6] = [
            glm::vec3(0.0, 0.0, 0.0),                           // Top
            glm::vec3(std::f32::consts::PI, 0.0, 0.0),          // Bottom
            glm::vec3(std::f32::consts::FRAC_PI_2, 0.0, 0.0),   // Front
            glm::vec3(-std::f32::consts::FRAC_PI_2, 0.0, 0.0),  // Back
            glm::vec3(0.0, 0.0, -std::f32::consts::FRAC_PI_2),  // Left
            glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),   // Right
        ];
        let positions: [glm::TVec3<f32>; 6] = [
            glm::vec3(0.0, 1.0, 0.0),                           // Top
            glm::vec3(0.0, -1.0, 0.0),                          // Bottom
            glm::vec3(0.0, 0.0, 1.0),                           // Front
            glm::vec3(0.0, 0.0, -1.0),                          // Back
            glm::vec3(1.0, 0.0, 0.0),                           // Left
            glm::vec3(-1.0, 0.0, 0.0),                          // Right
        ];
        // Handle top of tree and call lod_terrain for terrain sides
        // let mut planet_root;
        if node.get_n_children() < 1 {
            let mut planet_root = scene_graph::SceneNode::with_type(SceneNodeType::PlanetSkip);
            for _ in 0..6 {
                // Generate nodes for sides if they don't exist yet
                planet_root.add_child(&scene_graph::SceneNode::with_type(SceneNodeType::Planet));
            }
            node.add_child(&planet_root);
        }
        let mut planet_root = node.get_child(0);

        for (i, &child) in (&planet_root.children).iter().enumerate() {
            self.lod_terrain(&mut *child, glm::vec3(1.0, 1.0, 1.0), rotations[i], 
                positions[i], 0, player_pos
            );
        }

        if !self.has_ocean { return; }
        // Handle ocean
        if node.get_n_children() < 2 {
            let mut ocean_root = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
            for i in 0..6 {
                let ocean_mesh = mesh::Mesh::cs_plane(glm::vec3(0.5, 0.5, 0.5),
                    rotations[i], positions[i], 32, true, None
                );
                let mut ocean_node = scene_graph::SceneNode::from_vao(ocean_mesh.mkvao());
                ocean_node.node_type = SceneNodeType::Ocean;
                ocean_root.add_child(&ocean_node);
            }
            node.add_child(&ocean_root);
        }
        let ocean_root = node.get_child(1);
        // Generate sides if needed
    }
    pub unsafe fn lod_terrain(&self, 
        node: &mut scene_graph::SceneNode,
        scale: glm::TVec3<f32>,
        rotation: glm::TVec3<f32>, 
        position: glm::TVec3<f32>, 

        level: usize, 
        player_pos: glm::TVec3<f32>
    ) {
        let rotations: [glm::TVec3<f32>; 6] = [
            glm::vec3(0.0, 0.0, 0.0),                           // Top
            glm::vec3(std::f32::consts::PI, 0.0, 0.0),          // Bottom
            glm::vec3(std::f32::consts::FRAC_PI_2, 0.0, 0.0),   // Front
            glm::vec3(-std::f32::consts::FRAC_PI_2, 0.0, 0.0),  // Back
            glm::vec3(0.0, 0.0, -std::f32::consts::FRAC_PI_2),  // Left
            glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),   // Right
        ];
        let positions: [glm::TVec3<f32>; 6] = [
            glm::vec3(0.0, 1.0, 0.0),                           // Top
            glm::vec3(0.0, -1.0, 0.0),                          // Bottom
            glm::vec3(0.0, 0.0, 1.0),                           // Front
            glm::vec3(0.0, 0.0, -1.0),                          // Back
            glm::vec3(1.0, 0.0, 0.0),                           // Left
            glm::vec3(-1.0, 0.0, 0.0),                          // Right
        ];

        let dist = glm::length(&(node.position - player_pos));

        let mut planet_mesh = mesh::Mesh::cs_plane(scale, rotation, position, SUBDIVS_PER_LEVEL, true, None);
        mesh::displace_vertices(&mut planet_mesh, 
            self.noise_params.size.into(), 
            self.noise_params.height, 
            0.0,
            self.noise_params.seed
        );
        node.update_vao(planet_mesh.mkvao());
        node.node_type = SceneNodeType::Planet;

        // TODO: actual LoD layers
        // if dist > THRESHOLD[level] {
        //     for (i, & child) in (&node.children).iter().enumerate() {
        //         // TODO Use i to offset position
        //         node.node_type = SceneNodeType::PlanetSkip;
        //         self.lod_terrain(&mut *child,
        //             scale, rotation, position, level+1, player_pos);
        
        //     }
        // }
        // else {
        //     node.node_type = SceneNodeType::Planet;
        //     if node.index_count != -1 { return; }
        //     // Else, generate terrain
        //     // TODO: Make it work. Next: Defer to separate thread, draw lower
        //     // TODO  LoD while mesh is being generated
        //     let m = mesh::Mesh::cs_plane(
        //         scale, 
        //         rotation, 
        //         position, 
        //         SUBDIVS_PER_LEVEL, 
        //         true, None
        //     );
        //     // TODO: Displace vertices according to noise func
        // }
    }
}


/// Noise parameters to unambiguously generate a planet terrain. Should be able
/// to generate both terrain and texture (?)
#[derive(Debug, Default)]
pub struct PlanetParameters {
    pub size: f32,
    pub seed: u32,
    pub niter: usize,
    pub height: f32,                // Distance from radius to highest point
}