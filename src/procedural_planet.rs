use nalgebra_glm as glm;
use crate::scene_graph::{self, SceneNodeType};
use crate::{mesh, shader::Shader};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::globals::*;
use crate::util;

pub static PLANET_COUNTER: AtomicU64 = AtomicU64::new(0);
pub static IN_FLIGHT: AtomicU64 = AtomicU64::new(0);

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
    pub node        : usize,            // scene node kept separate
    pub planet_id   : usize,
    pub parts       : usize,            // Number of meshes
    pub position    : glm::Vec3,        // Handled by scene node
    pub rotation    : glm::Vec3,        // Handled by scene node
    pub radius      : f32,              // Radius to ocean level
    // Physics
    pub gravity     : f32,              // Gravitational pull, for physics
    pub trajectory  : f32,              // Radius of trajectory
    pub traj_speed  : f32,              // Angle speed of trajectory
    pub traj_init_angle : glm::Vec3,    // Inital trajectory position
    pub rot_speed   : f32,              // Angle speed of rotaion
    pub rot_axis    : glm::Vec3,        // Axis around which the planet rotates
    pub rot_init_angle : f32,           // Initial rotation
    pub parent_id   : usize,
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

    pub noise_fn    : u32,
    pub seed        : u32,
    // Some independent generators for increased variation
    pub noise       : NoiseParams,
    perlin0         : noise::Perlin,
    perlin1         : noise::Perlin,
    perlin2         : noise::Perlin,
}

use noise::*;
impl Planet {
    pub fn new() -> Self {
        let seed = rand::random::<_>();
        Self::with_seed(seed)
    }
    pub fn with_seed(seed: u32) -> Self {
        let planet_id = PLANET_COUNTER.fetch_add(1, Ordering::Relaxed) as usize;
        Planet {
            node        : std::usize::MAX,
            radius      : 1.0,
            gravity     : 0.5,
            traj_speed  : 0.01,
            rot_axis    : glm::vec3(0.0, 1.0, 0.0),
            planet_id,
            emission    : glm::vec3(1.0, 1.0, 0.0),
            lightsource : false,
            max_lod     : MAX_LOD,
            has_ocean   : true,
            ocean_lvl   : 0.0,
            ocean_dark_color    : glm::vec3(0.01, 0.2, 0.3),
            ocean_light_color   : glm::vec3(0.04, 0.3, 0.43),
            noise_fn    : 0,
            perlin0     : noise::Perlin::new().set_seed(seed),
            perlin1     : noise::Perlin::new().set_seed(seed*seed),
            perlin2     : noise::Perlin::new().set_seed(seed*seed/2),
            seed,
            //noise_size  : 10.0,

            ..Default::default()
        }
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
        let center_position = planet_center 
            + glm::rotate_z_vec3(
                &glm::rotate_x_vec3(
                    &(position * self.radius), rotation.x
                ), rotation.z
            );
        let plane_normal = glm::normalize(&(center_position - planet_center));
        let player_normal = glm::normalize(&(player_position - planet_center));

        // cos of angle between player position and plane center
        let dot = glm::dot(&plane_normal, &glm::normalize(&player_normal));
        let angle = dot.acos();
        let angle_lim = std::f32::consts::FRAC_PI_2 * 1.5 / (level as f32 + 1.0).powf(1.5);
        
        // Use height to limit LoD when planet is further away
        let player_height = glm::length(&(player_position - self.position));
        let height_lim = self.radius * (1.0+self.max_height)
            + self.radius * 2.6 / (level as f32+1.0).powf(1.5);

        if angle < angle_lim && player_height < height_lim && level < self.max_lod {
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
                let in_flight = IN_FLIGHT.load(Ordering::Relaxed);
                if in_flight >= MAX_IN_FLIGHT { 
                    return false; 
                }
                IN_FLIGHT.fetch_add(1, Ordering::Relaxed);
                let planet = *self;
                *arc_vao_status.lock().unwrap() = (Generating, mesh::Mesh::default());
                std::thread::spawn(move || {
                    let mut planet_mesh = mesh::Mesh::cs_plane(scale, rotation, position,
                        (1+level) * SUBDIVS_PER_LEVEL, None, true
                    );
                    planet.displace_vertices(&mut planet_mesh);
                    *arc_vao_status.lock().unwrap() = (Ready, planet_mesh);
                    IN_FLIGHT.fetch_sub(1, Ordering::Relaxed);
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
        // self.radius * (
        //     1.0 + mesh::fractal_noise(
        //         self.noise_fn, &glm::normalize(&(pos - &position)), 
        //         self.noise_size.into(), self.max_height, 0.0
        //     )
        // )
        self.radius * (1.0 + self.noise(&glm::normalize(&(pos - &position))))
    }

    fn displace_vertices(&self, mesh: &mut mesh::Mesh) {
        let mut vertices = util::to_array_of_vec3(mesh.vertices.clone());
        for i in 0..vertices.len() {
            // let val = 1.0 + mesh::fractal_noise(
            //     self.noise_fn, 
            //     &glm::normalize(&vertices[i]), 
            //     self.noise_size.into(), 
            //     self.max_height, 
            //     0.0);
            let val = 1.0 + self.noise(&glm::normalize(&vertices[i]));
            vertices[i] *= val;
        }
        
        // TODO: Solve the seams, could reuse the noise generator and use polar coordinates
        let mut normals = util::to_array_of_vec3(mesh.normals.clone());
        for i in (0..mesh.index_count).step_by(3) {
            let i = i as usize;
            let v1 = vertices[mesh.indices[i + 1] as usize] - vertices[mesh.indices[i] as usize];
            let v2 = vertices[mesh.indices[i + 2] as usize] - vertices[mesh.indices[i] as usize];
            let norm = glm::normalize(&glm::cross(&v1, &v2));
            normals[mesh.indices[i] as usize] = norm;
            normals[mesh.indices[i + 1] as usize] = norm;
            normals[mesh.indices[i + 2] as usize] = norm;
        }
        mesh.normals = util::from_array_of_vec3(normals);
        mesh.vertices = util::from_array_of_vec3(vertices);
    }

    fn noise(&self, pos: &glm::Vec3) -> f32 {
        let params = self.noise;
        match self.noise_fn {
            _ => {
                // Simple fractal noise. This apparently is also called 
                // fractal Brownian Motion (https://thebookofshaders.com/13/)
                let mut noise_sum = 0.0;
                // Initial values
                let mut amp = params.amplitude;
                let mut freq = params.frequency;
                // Properties
                let gain_pos = pos * params.gain_frequency;
                let gain = params.gain 
                    + params.gain_amplitude * (
                        self.perlin2.get(
                            [pos.x.into(), pos.y.into(), pos.z.into()]
                        ) as f32 + params.gain_offset
                    );
                let lac_pos = pos * params.lac_frequency;
                let lacunarity = params.lacunarity 
                    + params.lac_amplitude * (
                        self.perlin1.get(
                            [lac_pos.x.into(), lac_pos.y.into(), lac_pos.z.into()]
                        ) as f32 + params.lac_offset
                    );

                // Iterations - or octaves
                for _ in 0..params.octaves {
                    let point = pos * freq;
                    noise_sum += self.perlin0.get([
                        (point.x * self.noise.size) as f64, // + seed as f64,
                        (point.y * self.noise.size) as f64, // + seed as f64,
                        (point.z * self.noise.size) as f64, // + seed as f64,
                    ]) as f32 * amp * self.max_height;
                    freq *= lacunarity;
                    amp *= gain;
                }
                noise_sum
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NoiseParams {
    // Initial values
    pub size        : f32,      // Constant multiplier on frequency
    pub amplitude   : f32,
    pub frequency   : f32,
    // Properties
    pub octaves     : usize,
    pub gain        : f32,      // Constant gain
    pub gain_frequency  : f32,
    pub gain_amplitude  : f32,
    pub gain_offset : f32,
    pub lacunarity  : f32,      // Constant lacunarity
    pub lac_frequency   : f32,
    pub lac_amplitude   : f32,
    pub lac_offset  : f32,
}

impl Default for NoiseParams {
    fn default() -> Self {
        NoiseParams {
            size        : 10.0,
            amplitude   : 1.0,
            frequency   : 0.5,
            octaves     : 6,
            gain        : 0.5,
            gain_frequency  : 0.0,
            gain_amplitude  : 0.0,
            gain_offset : 0.0,
            lacunarity  : 2.0,
            lac_frequency   : 2.0,
            lac_amplitude   : 0.05,
            lac_offset  : 1.0,

        }
    }
}