extern crate nalgebra_glm as glm;

use std::mem::ManuallyDrop;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{mesh, util};

static NODE_COUNTER: AtomicU64 = AtomicU64::new(0);

// Used to create an unholy abomination upon which you should not cast your gaze. This ended up
// being a necessity due to wanting to keep the code written by students as "straight forward" as
// possible. It is very very double plus ungood Rust, and intentionally leaks memory like a sieve.
// But it works, and you're more than welcome to pretend it doesn't exist! In case you're curious
// about how it works: It allocates memory on the heap (Box), promises to prevent it from being
// moved or deallocated until dropped (Pin) and finally prevents the compiler from dropping it
// automatically at all (ManuallyDrop).
// ...
// If that sounds like a janky solution, it's because it is!
// Prettier, Rustier and better solutions were tried numerous times, but were all found wanting of
// having what I arbitrarily decided to be the required level of "simplicity of use".
pub type Node = ManuallyDrop<Pin<Box<SceneNode>>>;

pub enum LightSourceType {
    Point,
    Spot,
    Directional
}

pub struct LightSource {
    pub color: glm::TVec3<f32>,
    pub node: Node,
    pub light_type: LightSourceType,
}
impl LightSource {
    pub fn new(light_type: LightSourceType, r: f32, g: f32, b: f32) -> Self {
        LightSource {
            color: glm::vec3(r, g, b),
            light_type,
            node: SceneNode::with_type(SceneNodeType::LightSource)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SceneNodeType {
    Geometry = 0,   // Unused   
    Skybox = 1,     // Skybox shader
    Geometry2d = 2, // For gui
    Planet = 3,     // Planet terrain shader
    Ocean = 4,      // Planet ocean shader
    LightSource,    // Unused
    Empty,          // Empty nodes with other functions
    PlanetSkip,     // Planet, but skip this one, return
}

pub struct SceneNode {
    pub node_id         : usize,
    pub position        : glm::Vec3,   // Where I am in relation to my parent
    pub rotation        : glm::Vec3,   // How I should be rotated
    pub scale           : glm::Vec3,   // How I should be scaled
    pub reference_point : glm::Vec3,   // About which point I shall rotate about

    pub node_type   : SceneNodeType,
    pub name        : String,
    pub current_transformation_matrix: glm::Mat4, // The fruits of my labor

    pub vao         : mesh::VAOobj,             // What I should draw
    pub index_count : i32,             // How much of it I shall draw

    // IDs of maps
    pub texture_id  : Option<u32>,

    pub children: Vec<*mut SceneNode>, // Those I command
}

impl SceneNode {

    pub fn new() -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            node_id: NODE_COUNTER.fetch_add(1, Ordering::Relaxed) as usize,
            position        : glm::zero(),
            rotation        : glm::zero(),
            scale           : glm::vec3(1.0, 1.0, 1.0),
            reference_point : glm::zero(),
            node_type       : SceneNodeType::Empty,
            name            : String::new(),
            current_transformation_matrix: glm::identity(),
            vao             : Default::default(),
            index_count     : -1,
            texture_id      : None,
            children        : vec![],
        })))
    }

    pub fn with_type(node_type: SceneNodeType) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            node_id: NODE_COUNTER.fetch_add(1, Ordering::Relaxed) as usize,
            position        : glm::zero(),
            rotation        : glm::zero(),
            scale           : glm::vec3(1.0, 1.0, 1.0),
            reference_point : glm::zero(),
            node_type,
            name            : String::new(),
            current_transformation_matrix: glm::identity(),
            vao             : Default::default(),
            index_count     : -1,
            texture_id      : None,
            children        : vec![],
        })))
    }

    pub fn from_vao(vao: mesh::VAOobj) -> Node {
        ManuallyDrop::new(Pin::new(Box::new(SceneNode {
            node_id: NODE_COUNTER.fetch_add(1, Ordering::Relaxed) as usize,
            position        : glm::zero(),
            rotation        : glm::zero(),
            scale           : glm::vec3(1.0, 1.0, 1.0),
            reference_point : glm::zero(),
            node_type       : SceneNodeType::Geometry,
            name            : String::new(),
            current_transformation_matrix: glm::identity(),
            vao             : vao,
            index_count     : vao.n,
            texture_id      : None,
            children: vec![],
        })))
    }

    pub fn update_vao(&mut self, vao: mesh::VAOobj) {
        self.vao = vao;
        self.index_count = vao.n;
    }

    pub fn add_child(&mut self, child: &SceneNode) {
        self.children.push(child as *const SceneNode as *mut SceneNode)
    }

    #[allow(dead_code)]
    pub fn get_child(&mut self, index: usize) -> &mut SceneNode {
        unsafe {
            &mut (*self.children[index])
        }
    }

    #[allow(dead_code)]
    pub fn get_n_children(&self) -> usize {
        self.children.len()
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        let m = self.current_transformation_matrix;
        println!(
            "SceneNode {{
                VAO:       {:?}
                Indices:   {}
                Children:  {}
                Position:  [{:.2}, {:.2}, {:.2}]
                Rotation:  [{:.2}, {:.2}, {:.2}]
                Reference: [{:.2}, {:.2}, {:.2}]
                Current Transformation Matrix:
                    {:.2}  {:.2}  {:.2}  {:.2}
                    {:.2}  {:.2}  {:.2}  {:.2}
                    {:.2}  {:.2}  {:.2}  {:.2}
                    {:.2}  {:.2}  {:.2}  {:.2}
            }}",
            self.vao,
            self.index_count,
            self.children.len(),
            self.position.x,
            self.position.y,
            self.position.z,
            self.rotation.x,
            self.rotation.y,
            self.rotation.z,
            self.reference_point.x,
            self.reference_point.y,
            self.reference_point.z,
            m[0], m[4], m[8],  m[12],
            m[1], m[5], m[9],  m[13],
            m[2], m[6], m[10], m[14],
            m[3], m[7], m[11], m[15],
        );
    }

    /// Update node transformations and accumulate global uniforms
    pub unsafe fn update_node_transformations(
        &mut self,
        transformation_so_far: &glm::Mat4,
        player_position: &glm::TVec3<f32>,
    ) {
        // Construct the correct transformation matrix
        let mut transform = glm::identity();
        // Translate
        transform = glm::translate(&transform, &self.position);
        // Rotate around reference point
        transform = glm::translate(&transform, &(self.reference_point));
        transform = glm::rotate_y(&transform, self.rotation[1]);
        transform = glm::rotate_z(&transform, self.rotation[2]);
        transform = glm::rotate_x(&transform, self.rotation[0]);
        // Move back from reference point
        transform = glm::translate(&transform, &(-self.reference_point));
        // Scale
        transform = glm::scale(&transform, &self.scale);
    
    
        // Update the node's transformation matrix
        self.current_transformation_matrix = transformation_so_far * transform;
        // Recurse
        for &child in &self.children {
            (&mut *child).update_node_transformations(
                &self.current_transformation_matrix,
                player_position
            );
        }
    }

    /// Draw scene from scene graph
    /// * `node` - Current node
    /// * `view_projection_matrix` - Precalculated view and perspective matrix
    /// * `sh` - Active shader
    pub unsafe fn draw_scene(
        &self,
        view_projection_matrix: &glm::Mat4, 
        sh: &crate::shader::Shader
    ) {
        // Check if node is drawable, set model specific uniforms, draw
        match self.node_type {
        SceneNodeType::PlanetSkip => return,
        SceneNodeType::Geometry | 
        SceneNodeType::Geometry2d | 
        SceneNodeType::Planet | 
        SceneNodeType::Ocean |
        SceneNodeType::Skybox => {
            gl::BindVertexArray(self.vao.vao);
        
            let u_node_type = sh.get_uniform_location("u_node_type");
            gl::Uniform1ui(u_node_type, self.node_type as u32);
            
            let u_mvp = sh.get_uniform_location("u_mvp");
            let mvp = match self.node_type {
                SceneNodeType::Geometry2d => self.current_transformation_matrix,
                _ => view_projection_matrix * self.current_transformation_matrix
            };
            gl::UniformMatrix4fv(u_mvp, 1, gl::FALSE, mvp.as_ptr());
            
            let u_model = sh.get_uniform_location("u_model");
            gl::UniformMatrix4fv(u_model, 1, gl::FALSE, self.current_transformation_matrix.as_ptr());

            // Bind textures, or signal that none exist
            let u_has_texture = sh.get_uniform_location("u_has_texture");
            if let Some(texture_id) = self.texture_id {
                gl::BindTextureUnit(0, texture_id);
                gl::Uniform1i(u_has_texture, 1);
            } else {
                gl::Uniform1i(u_has_texture, 1);
            }
        
            gl::DrawElements(gl::TRIANGLES, self.index_count, gl::UNSIGNED_INT, std::ptr::null());
        },
        _ => ()
        }

        // Recurse
        for &child in &self.children {
            (&*child).draw_scene(view_projection_matrix, sh);
        }
    }

    pub fn update_buffers(&mut self, mesh: &mesh::Mesh) {
        unsafe { self.update_vertex_buffer(mesh) };
        unsafe { self.update_normal_buffer(mesh) };
        unsafe { self.update_texture_buffer(mesh) };
        unsafe { self.update_index_buffer(mesh) };
    }
    pub unsafe fn update_vertex_buffer(&self, mesh: &mesh::Mesh) {
        gl::BindVertexArray(self.vao.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vao.vbo);
        
        let vbuf_size = util::byte_size_of_array(&mesh.vertices);
        let vbuf_data = util::pointer_to_array(&mesh.vertices);

        gl::BufferData(gl::ARRAY_BUFFER, 
                        vbuf_size,
                        vbuf_data as *const _,
                        gl::STATIC_DRAW); 
    }
    pub unsafe fn update_index_buffer(&mut self, mesh: &mesh::Mesh) {
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.vao.ibo);

        let ibuf_size = util::byte_size_of_array(&mesh.indices);
        let ibuf_data = util::pointer_to_array(&mesh.indices);

        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                    ibuf_size,
                    ibuf_data as *const _,
                    gl::STATIC_DRAW);
        self.index_count = mesh.index_count;
    }
    pub unsafe fn update_normal_buffer(&self, mesh: &mesh::Mesh) {
        gl::BindVertexArray(self.vao.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vao.nbo);
        
        let nbuf_size = util::byte_size_of_array(&mesh.normals);
        let nbuf_data = util::pointer_to_array(&mesh.normals);

        gl::BufferData(gl::ARRAY_BUFFER, 
                        nbuf_size,
                        nbuf_data as *const _,
                        gl::STATIC_DRAW); 
    }
    pub unsafe fn update_texture_buffer(&self, mesh: &mesh::Mesh) {
        gl::BindVertexArray(self.vao.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vao.texbo);
        
        let tbuf_size = util::byte_size_of_array(&mesh.texture_coordinates);
        let tbuf_data = util::pointer_to_array(&mesh.texture_coordinates);

        gl::BufferData(gl::ARRAY_BUFFER, 
                        tbuf_size,
                        tbuf_data as *const _,
                        gl::STATIC_DRAW); 
    }
}


// You can also use square brackets to access the children of a SceneNode
use std::ops::{Index, IndexMut};
impl Index<usize> for SceneNode {
    type Output = SceneNode;
    fn index(&self, index: usize) -> &SceneNode {
        unsafe {
            & *(self.children[index] as *const SceneNode)
        }
    }
}
impl IndexMut<usize> for SceneNode {
    fn index_mut(&mut self, index: usize) -> &mut SceneNode {
        unsafe {
            &mut (*self.children[index])
        }
    }
}
