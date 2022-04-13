use nalgebra_glm as glm;
use crate::scene_graph::{self, SceneNodeType};
use crate::{util, mesh, shader::Shader};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

static PLANET_COUNTER: AtomicU64 = AtomicU64::new(0);
/// Thresholds for level of detail
const MAX_LOD: usize = 4;
//const THRESHOLD: [f32; MAX_LOD] = [128.0, 32.0, 16.0, 8.0, 4.0, 2.0];
const SUBDIVS_PER_LEVEL: usize = 16; // 256: 480+380=860ms, 128: 127+98=225ms
const N_LAYERS: usize = 5;  // Must match with scene.frag:22


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
#[derive(Default, Debug, Copy, Clone)]
pub struct Planet {
    pub node        : usize,// scene node kept separate
    pub planet_id   : usize,
    pub parts       : usize,    // Number of meshes
    pub position    : glm::TVec3<f32>,  // Handled by scene node
    pub rotation    : glm::TVec3<f32>,  // Handled by scene node
    pub radius      : f32,              // Radius to ocean level
    // Physics
    pub gravity     : f32,              // Gravitational pull, for physics
    pub trajectory  : f32,              // Radius of trajectory
    pub traj_speed  : f32,              // Trajectory speed
    pub init_angle  : glm::TVec3<f32>,
    pub parent_id   : Option<usize>,
    // Lighting
    pub lightsource : bool,
    pub emission    : glm::TVec3<f32>,  // Emission colour and intensity
    pub reflection  : glm::TVec3<f32>,  // Reflection colour and intensity
    // Terrain
    pub has_terrain : bool,
    pub max_height  : f32,
    pub color_scheme: [glm::TVec3<f32>; N_LAYERS],
    pub color_thresholds: [f32; N_LAYERS-1],
    pub color_blending  : f32,
    pub max_lod     : usize,
    // Ocean colours
    pub has_ocean   : bool,             // Set true to include ocean
    pub ocean_lvl   : f32,              // offset from radius
    pub ocean_dark_color    : glm::TVec3<f32>,
    pub ocean_light_color   : glm::TVec3<f32>,

    pub noise_fn    : noise::Perlin,
    pub seed        : u32,
    pub noise_size  : f32,
}

use noise::*;
impl Planet {
    pub fn new() -> Self {
        let planet_id = PLANET_COUNTER.fetch_add(1, Ordering::Relaxed) as usize;
        let seed = rand::random::<_>();
        Planet {
            node        : std::usize::MAX,
            radius      : 1.0,
            gravity     : 0.5,
            traj_speed  : 0.01,
            planet_id,
            emission    : glm::vec3(1.0, 1.0, 0.0),
            lightsource : false,
            max_lod     : MAX_LOD,
            has_ocean   : true,
            ocean_lvl   : 0.0,
            ocean_dark_color    : glm::vec3(0.01, 0.2, 0.3),
            ocean_light_color   : glm::vec3(0.04, 0.3, 0.43),
            noise_fn    : noise::Perlin::new().set_seed(seed),
            seed,
            noise_size  : 10.0,
            ..Default::default()
        }
    }
    pub fn with_seed(seed: u32) -> Self {
        let mut planet = Self::new();
        planet.seed = seed;
        planet.noise_fn = noise::Perlin::new().set_seed(seed);
        planet
    }
    
    /// Update uniforms for planet in shader
    pub unsafe fn update_uniforms(&self, sh: &Shader) {
        gl::Uniform1ui(
            sh.get_uniform_location(&format!("u_planets[{}].planet_id", self.planet_id)), 
            self.planet_id as u32
        ); // u_planets[id].planet_id
        gl::Uniform3fv(
            sh.get_uniform_location(&format!("u_planets[{}].position", self.planet_id)),
            1,
            self.position.as_ptr()
        ); // u_planets[id].position
        gl::Uniform1f(
            sh.get_uniform_location(&format!("u_planets[{}].radius", self.planet_id)),
            self.radius
        ); // u_planets[id].radius
        //-Lighting------------------------------------------------------------/
        gl::Uniform1ui(
            sh.get_uniform_location(&format!("u_planets[{}].lightsource", self.planet_id)),
            self.lightsource as u32,
        ); // u_planets[id].lightsource
        gl::Uniform3fv(
            sh.get_uniform_location(&format!("u_planets[{}].emission", self.planet_id)),
            1,
            self.emission.as_ptr()
        ); // u_planets[id].emission
        gl::Uniform3fv(
            sh.get_uniform_location(&format!("u_planets[{}].reflection", self.planet_id)),
            1,
            self.emission.as_ptr()
        ); // u_planets[id].reflection
        //-Terrain-------------------------------------------------------------/
        gl::Uniform3fv(
            sh.get_uniform_location(&format!("u_planets[{}].color_scheme[{}]", self.planet_id, 0)),
            1,
            self.color_scheme[0].as_ptr()
        ); // u_planets[id].color_scheme[]
        for i in 0..N_LAYERS-1 {
            gl::Uniform3fv(
                sh.get_uniform_location(&format!("u_planets[{}].color_scheme[{}]", self.planet_id, i+1)),
                1,
                self.color_scheme[i+1].as_ptr()
            ); // u_planets[id].color_scheme[1..N_LAYERS]
            gl::Uniform1f(
                sh.get_uniform_location(&format!("u_planets[{}].color_thresholds[{}]", self.planet_id, i)),
                self.color_thresholds[i]
            ); // u_planets[id].color_thresholds[0..N_LAYERS-1]
        }
        gl::Uniform1f(
            sh.get_uniform_location(&format!("u_planets[{}].color_blending", self.planet_id)),
            self.color_blending
        ); // u_planets[id].color_blending
        //-Ocean---------------------------------------------------------------/
        gl::Uniform3fv(
            sh.get_uniform_location(&format!("u_planets[{}].ocean_dark_color", self.planet_id)),
            1,
            self.ocean_dark_color.as_ptr()
        ); // u_planets[id].ocean_dark_color
        gl::Uniform3fv(
            sh.get_uniform_location(&format!("u_planets[{}].ocean_dark_color", self.planet_id)),
            1,
            self.ocean_light_color.as_ptr()
        ); // u_planets[id].ocean_light_color
    }
    /// Set level of detail to be drawn, generate new if needed
    pub unsafe fn lod(&mut self, 
        node: &mut scene_graph::SceneNode, 
        player_position: glm::TVec3<f32>
    ) {
        self.parts = 0;
        self.position = glm::vec4_to_vec3(&(
            node.current_transformation_matrix * glm::vec4(0.0, 0.0, 0.0, 1.0)
        ));
        let rotations: [glm::TVec3<f32>; 6] = [
            glm::vec3(0.0, 0.0, 0.0),                           // Top
            glm::vec3(std::f32::consts::PI, 0.0, 0.0),          // Bottom
            glm::vec3(std::f32::consts::FRAC_PI_2, 0.0, 0.0),   // Front
            glm::vec3(-std::f32::consts::FRAC_PI_2, 0.0, 0.0),  // Back
            glm::vec3(0.0, 0.0, -std::f32::consts::FRAC_PI_2),  // Left
            glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),   // Right
        ];
        // Handle top of tree and call lod_terrain for terrain sides
        // let mut planet_root;
        if node.get_n_children() < 1 {
            let mut planet_root = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
            for i in 0..6 {
                // Generate nodes for sides if they don't exist yet
                planet_root.add_child(&scene_graph::SceneNode::with_type(SceneNodeType::Planet));
                planet_root.get_child(i).planet_id = self.planet_id;
            }
            node.add_child(&planet_root);
        }
        let planet_root = node.get_child(0);

        for (i, &child) in (&planet_root.children).iter().enumerate() {
            self.lod_terrain(
                &mut *child, 
                glm::vec3(1.0, 1.0, 1.0), 
                rotations[i], 
                glm::vec3(0.0, 1.0, 0.0),//positions[i], 
                0, 
                player_position
            );
        }

        if !self.has_ocean { return; }
        // Handle ocean
        if node.get_n_children() < 2 {
            let mut ocean_root = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
            for i in 0..6 {
                // Generate sides if they don't exist yet
                let ocean_mesh = mesh::Mesh::cs_plane(
                    glm::vec3(1.0, 1.0, 1.0),
                    rotations[i], 
                    glm::vec3(0.0, 1.0, 0.0),//positions[i], 
                    32, 
                    None, 
                    true
                );
                let mut ocean_node = scene_graph::SceneNode::from_vao(ocean_mesh.mkvao());
                ocean_node.node_type = SceneNodeType::Ocean;
                ocean_node.planet_id = self.planet_id;
                ocean_root.add_child(&ocean_node);
            }
            node.add_child(&ocean_root);
        }
    }

    pub unsafe fn lod_terrain(&self, 
        node: &mut scene_graph::SceneNode,  // Either gets the mesh (leaf) or becomes a parent to four subdivisions
        scale: glm::TVec3<f32>,     // 2D scale. Modify x and z components
        rotation: glm::TVec3<f32>,  // Won't be modified, same for all subdivs of a side
        position: glm::TVec3<f32>,  // 2D position. Modify x and z components
        level: usize, 
        player_position: glm::TVec3<f32>
    ) -> bool {
        let displacements: [glm::TVec3<f32>; 4] = [
            glm::vec3(1.0, 0.0, 1.0),
            glm::vec3(-1.0, 0.0, 1.0),
            glm::vec3(1.0, 0.0, -1.0),
            glm::vec3(-1.0, 0.0, -1.0),
        ];

        let planet_center = self.position;
        let center_position = planet_center + glm::rotate_z_vec3(&glm::rotate_x_vec3(&(position * self.radius), rotation.x), rotation.z); //.component_mul(&node.scale);
        // eprintln!("Planet center: {:?}, radius: {}, plane center: {:?}", planet_center, self.radius, center_position);
        let plane_normal = glm::normalize(&(center_position - planet_center));
        let player_normal = glm::normalize(&(player_position - planet_center));

        let dot = glm::dot(&plane_normal, &glm::normalize(&player_normal)); // cos of angle between player position and plane center
        let height = self.get_height(&player_position);
        let dist = glm::length(&(center_position - player_position));

        // TODO: LoD needs tuning, not sure what's best
        if dist < glm::length(&scale) * self.radius * 2.0 && dot >= 0.0 && level < self.max_lod {
            // Generate next level
            if node.children.len() == 0 {
                for i in 0..4 {
                    node.add_child(&scene_graph::SceneNode::with_type(SceneNodeType::Planet));
                    node.get_child(i).planet_id = self.planet_id;
                }
            }
            node.node_type = SceneNodeType::Empty;
            let mut ready = true;
            for i in 0..4 {
                ready &= self.lod_terrain(&mut node.get_child(i), scale / 2.0, rotation, 
                position + displacements[i] * scale.x / 2.0, level + 1, player_position);
            }
            if !ready {
                node.node_type = SceneNodeType::Planet;
            }
            return true
        }
        // Use this detail level
        node.node_type = SceneNodeType::Planet;
        if node.index_count != -1 { return true }
        //---------------------------------------------------------------------/
        // Generate terrain
        //---------------------------------------------------------------------/
        // Access vao status mutex
        use scene_graph::VAOStatus::*;
        let arc_vao_status = node.vao_generate.clone();
        let status = { arc_vao_status.lock().unwrap().0 };

        return match status {
            NotStarted => {
                // Start thread generating terrain
                let planet = *self;
                *arc_vao_status.lock().unwrap() = (Generating, mesh::Mesh::default());
                std::thread::spawn(move || {
                    let mut planet_mesh = mesh::Mesh::cs_plane(scale, rotation, position,
                        (1+level) * SUBDIVS_PER_LEVEL, None, true
                    );
                    planet.displace_vertices(&mut planet_mesh);
                    *arc_vao_status.lock().unwrap() = (Ready, planet_mesh);
                });
                false
            },
            Ready => {
                // Finish creating scene node
                let vao = arc_vao_status.lock().unwrap().1.mkvao();
                node.update_vao(vao);
                true
            },
            Generating => {
                // Just return while thread is still working
                false
            }
        }
    }

    pub fn get_height(&self, pos: &glm::TVec3<f32>) -> f32 {
        let pos = glm::vec3(
            (100.0*pos.x).round()/100.0,
            (100.0*pos.y).round()/100.0,
            (100.0*pos.z).round()/100.0,
        );
        let position = glm::vec3(
            (100.0*self.position.x).round()/100.0,
            (100.0*self.position.y).round()/100.0,
            (100.0*self.position.z).round()/100.0,
        );
        self.radius * (
            1.0 + mesh::fractal_noise(
                self.noise_fn, &glm::normalize(&(pos - &position)), 
                self.noise_size.into(), self.max_height, 0.0
            )
        )
    }

    fn displace_vertices(&self, mesh: &mut mesh::Mesh) {
        let timer = std::time::SystemTime::now();
        // eprint!("Generating noise . . . ");
        let mut vertices = mesh::to_array_of_vec3(mesh.vertices.clone());
        for i in 0..vertices.len() {
            let val = 1.0 + mesh::fractal_noise(
                self.noise_fn, 
                &glm::normalize(&vertices[i]), 
                self.noise_size.into(), 
                self.max_height, 
                0.0);
            vertices[i] *= val;
        }
        
        // TODO: Solve the seams, could reuse the noise generator and use polar coordinates
        let mut normals = mesh::to_array_of_vec3(mesh.normals.clone());
        for i in (0..mesh.index_count).step_by(3) {
            let i = i as usize;
            // let mut v0 = glm::normalize(&vertices[mesh.indices[i] as usize]);
            // let mut v1 = glm::rotate_x_vec3(&v0, std::f32::consts::PI / (4.0 * 4096.0));
            // let mut v2 = glm::rotate_z_vec3(&v0, -std::f32::consts::PI / (4.0 * 4096.0));
            // v0 *= 1.0 + fractal_noise(perlin, &v0, size, height, offset);
            // v1 *= 1.0 + fractal_noise(perlin, &v1, size, height, offset);
            // v2 *= 1.0 + fractal_noise(perlin, &v2, size, height, offset);
            let v1 = vertices[mesh.indices[i + 1] as usize] - vertices[mesh.indices[i] as usize];
            let v2 = vertices[mesh.indices[i + 2] as usize] - vertices[mesh.indices[i] as usize];
            // v1 = v1 - v0;
            // v2 = v2 - v0;
            let norm = glm::normalize(&glm::cross(&v1, &v2));
            normals[mesh.indices[i] as usize] = norm;
            normals[mesh.indices[i + 1] as usize] = norm;
            normals[mesh.indices[i + 2] as usize] = norm;
        }
        mesh.normals = mesh::from_array_of_vec3(normals);
        mesh.vertices = mesh::from_array_of_vec3(vertices);
        // eprintln!("took {:?}", timer.elapsed().unwrap());

    }
}
