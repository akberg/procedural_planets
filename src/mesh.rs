use tobj;
use itertools::Itertools;
use num::Num;
use std::fmt::Debug;
use glm::Scalar;

// internal helper
fn generate_color_vec(color: glm::TVec4<f32>, num: usize) -> Vec<f32> {
    glm::value_ptr(&color).iter().cloned().cycle().take(num*4).collect()
    //color.iter().cloned().cycle().take(num*4).collect()
}
/// Convert an array of Vec2 into an array of numbers
pub fn from_array_of_vec2<T: Scalar + Copy>(arr: Vec<glm::TVec2<T>>) -> Vec<T> {
    arr.iter()
    .map(|v| vec![v[0], v[1]])
    .flatten()
    .collect::<_>()
}
/// Convert an array of Vec3 into an array of numbers
pub fn from_array_of_vec3<T: Scalar + Copy>(arr: Vec<glm::TVec3<T>>) -> Vec<T> {
    arr.iter()
    .map(|v| vec![v[0], v[1], v[2]])
    .flatten()
    .collect::<_>()
}
/// Convert an array of Vec4 into an array of numbers
pub fn from_array_of_vec4<T: Scalar + Copy>(arr: Vec<glm::TVec4<T>>) -> Vec<T> {
    arr.iter()
        .map(|v| vec![v[0], v[1], v[2], v[3]])
        .flatten()
        .collect::<_>()
}
/// Convert an array of numbers representing 2-tuples to array of vec2
pub fn to_array_of_vec2<T: Scalar + Copy>(arr: Vec<T>) -> Vec<glm::TVec2<T>> {
    arr.iter()
    .chunks(2)
    .into_iter()
    .map(|mut step| glm::vec2(
        *step.next().unwrap(), 
        *step.next().unwrap()
    ))
    .collect::<_>()
}
/// Convert an array of numbers representing 3-tuples to array of vec3
pub fn to_array_of_vec3<T: Scalar + Copy>(arr: Vec<T>) -> Vec<glm::TVec3<T>> {
    arr.iter()
    .chunks(3)
    .into_iter()
    .map(|mut step| glm::vec3(
        *step.next().unwrap(), 
        *step.next().unwrap(),
        *step.next().unwrap(),
    ))
    .collect::<_>()
}
/// Convert an array of numbers representing 4-tuples to array of vec4
pub fn to_array_of_vec4<T: Scalar + Copy>(arr: Vec<T>) -> Vec<glm::TVec4<T>> {
    arr.iter()
        .chunks(4)
        .into_iter()
        .map(|mut step| glm::vec4(
            *step.next().unwrap(), 
            *step.next().unwrap(),
            *step.next().unwrap(),
            *step.next().unwrap(),
        ))
        .collect::<_>()
}

pub struct Mesh {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
    pub index_count: i32,
}

impl Mesh {
    pub fn from(mesh: tobj::Mesh, color: glm::TVec4<f32>) -> Self {
        let num_verts = mesh.positions.len() / 3;
        let index_count = mesh.indices.len() as i32;
        Mesh {
            vertices: mesh.positions,
            normals: mesh.normals,
            indices: mesh.indices,
            colors: generate_color_vec(color, num_verts),
            index_count,
        }
    }

    pub fn cube(
        scale: glm::TVec3<f32>,
        texture_scale: glm::TVec2<f32>,
        tiling_textures: bool,
        inverted: bool,
        texture_scale3d: glm::TVec3<f32>,
        color: glm::TVec4<f32>
    ) -> Self {
        let mut points = [glm::vec3(0.0, 0.0, 0.0); 8];
        let mut indices = vec![0; 36];

        for y in 0..2 {
            for z in 0..2 {
                for x in 0..2 {
                    points[x+y*4+z*2] = glm::vec3(
                        x as f32 * 2.0 - 1.0, 
                        y as f32 * 2.0 - 1.0, 
                        z as f32 * 2.0 - 1.0,
                    ).component_mul(&scale) * 0.5;
                }
            }
        }

        let faces = [
            [2,3,0,1], // Bottom 
            [4,5,6,7], // Top 
            [7,5,3,1], // Right 
            [4,6,0,2], // Left 
            [5,4,1,0], // Back 
            [6,7,2,3], // Front 
        ];

        let scale = scale.component_mul(&texture_scale3d);
        let face_scale = [
            glm::vec2(-scale.x,-scale.z), // Bottom
            glm::vec2(-scale.x,-scale.z), // Top
            glm::vec2( scale.z, scale.y), // Right
            glm::vec2( scale.z, scale.y), // Left
            glm::vec2( scale.x, scale.y), // Back
            glm::vec2( scale.x, scale.y), // Front
        ];

        let normals = [
            glm::vec3( 0.0,-1.0, 0.0), // Bottom 
            glm::vec3( 0.0, 1.0, 0.0), // Top 
            glm::vec3( 1.0, 0.0, 0.0), // Right 
            glm::vec3(-1.0, 0.0, 0.0), // Left 
            glm::vec3( 0.0, 0.0,-1.0), // Back 
            glm::vec3( 0.0, 0.0, 1.0), // Front 
        ];

        let uvs = [
            glm::vec2(0.0, 0.0),
            glm::vec2(0.0, 1.0),
            glm::vec2(1.0, 0.0),
            glm::vec2(1.0, 1.0),
        ];
        let mut vertices = Vec::new();
        let mut mindices = Vec::new();
        let mut mnormals = Vec::new();
        let mut texture_coordinates = Vec::new();
        for face in 0..6 {
            let offset = face * 6;
            indices[offset + 0] = faces[face][0] as u32;
            indices[offset + 3] = faces[face][0] as u32;

            if !inverted {
                indices[offset + 1] = faces[face][3] as u32;
                indices[offset + 2] = faces[face][1] as u32;
                indices[offset + 4] = faces[face][2] as u32;
                indices[offset + 5] = faces[face][3] as u32;
            } else {
                indices[offset + 1] = faces[face][1] as u32;
                indices[offset + 2] = faces[face][3] as u32;
                indices[offset + 4] = faces[face][3] as u32;
                indices[offset + 5] = faces[face][2] as u32;
            }

            for i in 0..6 {
                vertices.push(points[indices[offset + i] as usize]);
                mindices.push((offset + i) as u32);
                mnormals.push(normals[face] * (if inverted{-1.0}else{1.0}));
            }

            let texture_scale_factor =  if tiling_textures {
                face_scale[face].component_div(&texture_scale)
            } else {
                glm::vec2(1.0, 1.0)
            };

            if inverted {
                for &i in [1,2,3,1,0,2].iter() {
                    texture_coordinates.push(uvs[i].component_mul(&texture_scale_factor));
                }
            } else {
                for &i in [3,1,0,3,0,2].iter() {
                    texture_coordinates.push(uvs[i].component_mul(&texture_scale_factor));
                }
            }
        }
        let vertex_count = vertices.len();
        Mesh {
            vertices: from_array_of_vec3(vertices),
            indices: mindices,
            normals: from_array_of_vec3(mnormals),
            colors: generate_color_vec(color, vertex_count),
            index_count: 36
        }
    }
}

pub fn generate_cube(scale: glm::TVec3<f32>, color: glm::TVec4<f32>) -> Mesh {
    let vertex_count = 4*6;
    let mut vertices = vec![glm::vec3(0.0f32, 0.0, 0.0);3*vertex_count];
    let mut normals = vec![glm::vec3(0.0f32, 0.0, 0.0);3*vertex_count];
    let index_count = 3*6*6;
    let mut indices = vec![0u32;index_count];

    let v_normals = [
        glm::vec3(0.0, 0.0, -1.0),
        glm::vec3(1.0, 0.0, 0.0),
        glm::vec3(0.0, 0.0, 1.0),
        glm::vec3(0.0, 0.0, -1.0),
        glm::vec3(0.0, 1.0, 0.0),
        glm::vec3(0.0, -1.0, 0.0),
    ];

    for i in 0..6 {
        indices[3*6*i + 0] = 3*4*i as u32 + 0;
        indices[3*6*i + 1] = 3*4*i as u32 + 3;
        indices[3*6*i + 2] = 3*4*i as u32 + 1;

        indices[3*6*i + 3] = 3*4*i as u32 + 0;
        indices[3*6*i + 4] = 3*4*i as u32 + 2;
        indices[3*6*i + 5] = 3*4*i as u32 + 3;

        normals[4*i + 0] = v_normals[i];
        normals[4*i + 1] = v_normals[i];
        normals[4*i + 2] = v_normals[i];
        normals[4*i + 3] = v_normals[i];

        normals[4*i + 3] = v_normals[i];
    }

    // vertices[0] = glm::vec3(-1.0, 1.0, -1.0).component_mul(&scale);
    // vertices[13] = glm::vec3(-1.0, 1.0, -1.0).component_mul(&scale);
    // vertices[18] = glm::vec3(-1.0, 1.0, -1.0).component_mul(&scale);

    // vertices[1] = glm::vec3(1.0, 1.0, -1.0).component_mul(&scale);
    // vertices[4] = glm::vec3(1.0, 1.0, -1.0).component_mul(&scale);
    // vertices[19] = glm::vec3(1.0, 1.0, -1.0).component_mul(&scale);

    // vertices[2] = glm::vec3(-1.0, -1.0, -1.0).component_mul(&scale);
    // vertices[15] = glm::vec3(-1.0, -1.0, -1.0).component_mul(&scale);
    // vertices[20] = glm::vec3(-1.0, -1.0, -1.0).component_mul(&scale);

    // vertices[3] = glm::vec3(1.0, -1.0, -1.0).component_mul(&scale);
    // vertices[6] = glm::vec3(1.0, -1.0, -1.0).component_mul(&scale);
    // vertices[21] = glm::vec3(1.0, -1.0, -1.0).component_mul(&scale);

    vertices[8] = glm::vec3(1.0, 1.0, 1.0).component_mul(&scale);
    // vertices[5] = glm::vec3(1.0, 1.0, 1.0).component_mul(&scale);
    // vertices[17] = glm::vec3(1.0, 1.0, 1.0).component_mul(&scale);

    vertices[9] = glm::vec3(-1.0, 1.0, 1.0).component_mul(&scale);
    // vertices[12] = glm::vec3(-1.0, 1.0, 1.0).component_mul(&scale);
    // vertices[16] = glm::vec3(-1.0, 1.0, 1.0).component_mul(&scale);

    vertices[10] = glm::vec3(1.0, -1.0, 1.0).component_mul(&scale);
    // vertices[17] = glm::vec3(1.0, -1.0, 1.0).component_mul(&scale);
    // vertices[23] = glm::vec3(1.0, -1.0, 1.0).component_mul(&scale);

    vertices[11] = glm::vec3(-1.0, -1.0, 1.0).component_mul(&scale);
    // vertices[14] = glm::vec3(-1.0, -1.0, 1.0).component_mul(&scale);
    // vertices[22] = glm::vec3(-1.0, -1.0, 1.0).component_mul(&scale);

    Mesh {
        vertices: from_array_of_vec3(vertices),
        normals: from_array_of_vec3(normals),
        indices,
        index_count: index_count as i32,
        colors: generate_color_vec(color, 36)
    }
}

// pub struct Terrain;
// impl Terrain {
//     pub fn load(path: &str) -> Mesh {
//         println!("Loading terrain model...");
//         let before = std::time::Instant::now();
//         let (models, _materials)
//             = tobj::load_obj(path,
//                 &tobj::LoadOptions{
//                     triangulate: true,
//                     single_index: true,
//                     ..Default::default()
//                 }
//             ).expect("Failed to load terrain model");
//         let after = std::time::Instant::now();
//         println!("Done in {:.3}ms.", after.duration_since(before).as_micros() as f32 / 1e3);

//         if models.len() > 1 || models.len() == 0 {
//             panic!("Please use a model with a single mesh!")
//             // You could try merging the vertices and indices
//             // of the separate meshes into a single mesh.
//             // I'll leave that as an optional exercise. ;)
//         }

//         let terrain = models[0].to_owned();
//         println!("Loaded {} with {} points and {} triangles.",
//             terrain.name,
//             terrain.mesh.positions.len() /3,
//             terrain.mesh.indices.len() / 3,
//         );

//         Mesh::from(terrain.mesh, [1.0, 1.0, 1.0, 1.0])
//     }
// }

use std::ops::Index;
pub struct Helicopter {
    pub body       : Mesh,
    pub door       : Mesh,
    pub main_rotor : Mesh,
    pub tail_rotor : Mesh,
}

// You can use square brackets to access the components of the helicopter, if you want to use loops!
impl Index<usize> for Helicopter {
    type Output = Mesh;
    fn index<'a>(&'a self, i: usize) -> &'a Mesh {
        match i {
            0 => &self.body,
            1 => &self.main_rotor,
            2 => &self.tail_rotor,
            3 => &self.door,
            _ => panic!("Invalid index, try [0,3]"),
        }
    }
}

impl Helicopter {
    pub fn load(path: &str) -> Self {
        println!("Loading helicopter model...");
        let before = std::time::Instant::now();
        let (models, _materials)
            = tobj::load_obj(path,
                &tobj::LoadOptions{
                    triangulate: true,
                    single_index: true,
                    ..Default::default()
                }
            ).expect("Failed to load helicopter model");
        let after = std::time::Instant::now();
        println!("Done in {:.3}ms!", after.duration_since(before).as_micros() as f32 / 1e3);

        for model in &models {
            println!("Loaded {} with {} points and {} triangles.", model.name, model.mesh.positions.len() / 3, model.mesh.indices.len() / 3);
        }

        let body_model = models.iter().find(|m| m.name == "Body_body").expect("Incorrect model file!").to_owned();
        let door_model = models.iter().find(|m| m.name == "Door_door").expect("Incorrect model file!").to_owned();
        let main_rotor_model = models.iter().find(|m| m.name == "Main_Rotor_main_rotor").expect("Incorrect model file!").to_owned();
        let tail_rotor_model = models.iter().find(|m| m.name == "Tail_Rotor_tail_rotor").expect("Incorrect model file!").to_owned();

        Helicopter {
            body:       Mesh::from(body_model.mesh,         glm::vec4(0.3, 0.3, 0.3, 1.0)),
            door:       Mesh::from(door_model.mesh,         glm::vec4(0.1, 0.1, 0.3, 1.0)),
            main_rotor: Mesh::from(main_rotor_model.mesh,   glm::vec4(0.3, 0.1, 0.1, 1.0)),
            tail_rotor: Mesh::from(tail_rotor_model.mesh,   glm::vec4(0.1, 0.3, 0.1, 1.0)),
        }
    }
}

