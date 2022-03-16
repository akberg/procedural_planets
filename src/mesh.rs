use obj;
use nalgebra_glm as glm;

// internal helper
fn generate_color_vec(color: [f32; 4], num: usize) -> Vec<f32> {
    color.iter().cloned().cycle().take(num*4).collect()
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<glm::TVec3<f32>>,
    pub normals: Vec<glm::TVec3<f32>>,
    pub colors: Option<Vec<f32>>,
    pub uv_texture: Option<Vec<glm::TVec2<f32>>>,
    pub indices: Vec<u32>,
    pub index_count: i32,
}

impl Mesh {
    // pub fn from(mesh: tobj::Mesh, color: [f32; 4]) -> Self {
    //     let num_verts = mesh.positions.len() / 3;
    //     let index_count = mesh.indices.len() as i32;
    //     Mesh {
    //         vertices: mesh.positions,
    //         normals: mesh.normals,
    //         indices: mesh.indices,
    //         colors: generate_color_vec(color, num_verts),
    //         index_count,
    //     }
    // }
}

pub enum SceneNodeType {
    Geometry2D
}

pub struct SceneNode {
    mesh: Mesh,

}