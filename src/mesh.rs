use tobj;
use itertools::Itertools;
use std::fmt::Debug;
use glm::Scalar;

// internal helper
fn generate_color_vec(color: glm::TVec4<f32>, num: usize) -> Vec<f32> {
    glm::value_ptr(&color).iter().cloned().cycle().take(num*4).collect()
    //color.iter().cloned().cycle().take(num*4).collect()
}
/// Smooth min
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = 0.0f32.max(k - (a-b).abs()) / k;
    return a.min(b) - h.powi(3) * k / 6.0;
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

//-----------------------------------------------------------------------------/
// Mesh
//-----------------------------------------------------------------------------/

pub struct Mesh {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub texture_coordinates: Vec<f32>,
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
            texture_coordinates: if mesh.texcoords.len() > 0 { mesh.texcoords } else { vec![0.0; num_verts * 2] },
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
            texture_coordinates: from_array_of_vec2(texture_coordinates),
            colors: generate_color_vec(color, vertex_count),
            index_count: 36
        }
    }

    pub fn text_buffer(text: &str, char_height_over_width: f32, total_text_width: f32) -> Self {
        let char_w = total_text_width / text.len() as f32;
        let char_h = char_height_over_width * char_w;

        let vertex_count = 4 * text.len();
        let index_count = 6 * text.len() as i32;

        let mut vertices = vec![glm::vec3(0.0, 0.0, 0.0); vertex_count];
        let mut texture = vec![glm::vec2(0.0, 0.0); vertex_count];
        let mut normals = vec![glm::vec3(0.0, 0.0, 0.0); vertex_count];
        let mut indices = vec![0; index_count as usize];

        for (i, c) in text.chars().enumerate() {
            println!("{}", c as u8);
            let base_x = i as f32 * char_w;

            vertices[4 * i + 0] = glm::vec3(base_x, 0.0, 0.0);
            vertices[4 * i + 1] = glm::vec3(base_x + char_w, 0.0, 0.0);
            vertices[4 * i + 2] = glm::vec3(base_x + char_w, char_h, 0.0);
            vertices[4 * i + 3] = glm::vec3(base_x, char_h, 0.0);

            normals[4 * i + 0] = glm::vec3(0.0, 0.0, -1.0);
            normals[4 * i + 1] = glm::vec3(0.0, 0.0, -1.0);
            normals[4 * i + 2] = glm::vec3(0.0, 0.0, -1.0);
            normals[4 * i + 3] = glm::vec3(0.0, 0.0, -1.0);

            texture[4 * i + 0] = glm::vec2((c as u8) as f32 / 128.0, 0.0);
            texture[4 * i + 1] = glm::vec2((c as u8 + 1) as f32 / 128.0, 0.0);
            texture[4 * i + 2] = glm::vec2((c as u8 + 1) as f32 / 128.0, 1.0);
            texture[4 * i + 3] = glm::vec2((c as u8) as f32 / 128.0, 1.0);

            indices[6 * i + 0] = 4 * i as u32 + 0;
            indices[6 * i + 1] = 4 * i as u32 + 1;
            indices[6 * i + 2] = 4 * i as u32 + 2;
            indices[6 * i + 3] = 4 * i as u32 + 0;
            indices[6 * i + 4] = 4 * i as u32 + 2;
            indices[6 * i + 5] = 4 * i as u32 + 3;
        }

        Mesh {
            vertices: from_array_of_vec3(vertices),
            normals: from_array_of_vec3(normals),
            texture_coordinates: from_array_of_vec2(texture),
            colors: generate_color_vec(glm::vec4(1.0, 1.0, 1.0, 1.0), vertex_count),
            indices,
            index_count,
        }
    }

    pub fn plane(
        scale: glm::TVec3<f32>, 
        rotation: glm::TVec3<f32>,
        position: glm::TVec3<f32>,
        subdivisions: usize, tiling_textures: bool,
        color: Option<glm::TVec4<f32>>
    ) -> Self {
        let res = 1 + subdivisions;
        let vertex_count = res * res;
        let index_count = 6 * (res-1) * (res-1);
        let mut vertices = vec![glm::vec3(0.0, 0.0, 0.0); vertex_count];
        let normals = vec![glm::vec3(0.0, 1.0, 0.0); vertex_count];
        let mut texture = vec![glm::vec2(0.0, 0.0); vertex_count];
        let mut indices = vec![0; index_count];


        for z in 0..res {
            for x in 0..res {
                // Transform position
                let mut pos = glm::vec3(
                    2.0 * x as f32 / subdivisions as f32 - 1.0,
                    0.0f32,
                    2.0 * z as f32 / subdivisions as f32 - 1.0,
                );
                pos = glm::rotate_x_vec3(&pos, rotation.x);
                pos = glm::rotate_y_vec3(&pos, rotation.y);
                pos = glm::rotate_z_vec3(&pos, rotation.z);
                pos += position;
                // Side of cubesphere
                // vertices[z * res + x] = glm::vec3(
                //     posx * (1.0 - posy.powi(2) / 2.0 - posz.powi(2) / 2.0 + posy.powi(2) * posz.powi(2) / 3.0).sqrt(),
                //     posy * (1.0 - posx.powi(2) / 2.0 - posz.powi(2) / 2.0 + posx.powi(2) * posz.powi(2) / 3.0).sqrt(),
                //     posz * (1.0 - posx.powi(2) / 2.0 - posy.powi(2) / 2.0 + posx.powi(2) * posy.powi(2) / 3.0).sqrt(),
                // ).component_mul(&scale) * 0.5;
                // Flat plane
                vertices[z * res + x] = pos.component_mul(&scale) * 0.5;
                // vertices[z * res + x] = glm::vec3(
                //     2.0 * x as f32 / res as f32 - 1.0,
                //     0.0,
                //     2.0 * z as f32 / res as f32 - 1.0,
                // ).component_mul(&scale) * 0.5;
                // Waves
                // vertices[y * res + x] = glm::vec3(
                //     2.0 * x as f32 / res as f32 - 1.0,
                //     ((2.0 * x as f32 / res as f32 - 1.0)*10.0).sin() / 10.0 + ((2.0 * y as f32 / res as f32 - 1.0)*10.0).sin() / 10.0,
                //     2.0 * y as f32 / res as f32 - 1.0,
                // ).component_mul(&scale) * 0.5;
                texture[z * res + x] = glm::vec2(
                    x as f32 / res as f32,
                    z as f32 / res as f32,
                );
                if z < subdivisions && x < subdivisions {
                    let offset = 6 * (z * subdivisions + x);
                    indices[offset + 0] = (z * res + x + 1) as u32;
                    indices[offset + 1] = (z * res + x + 0) as u32;
                    indices[offset + 2] = ((z + 1) * res + x + 1) as u32;

                    indices[offset + 3] = (z * res + x) as u32;
                    indices[offset + 4] = ((z + 1) * res + x) as u32;
                    indices[offset + 5] = ((z + 1) * res + x + 1) as u32;
                }
            }
        }

        Mesh {
            vertices: from_array_of_vec3(vertices),
            normals: from_array_of_vec3(normals),
            texture_coordinates: from_array_of_vec2(texture),
            colors: generate_color_vec(color.unwrap_or(glm::vec4(1.0, 1.0, 1.0, 1.0)), vertex_count),
            indices,
            index_count: index_count as i32,
        }
    }

    pub fn cs_plane(
        scale: glm::TVec3<f32>,
        rotation: glm::TVec3<f32>,
        position: glm::TVec3<f32>, 
        subdivisions: usize, tiling_textures: bool,
        color: Option<glm::TVec4<f32>>
    ) -> Self {
        let res = 1 + subdivisions;
        let vertex_count = res * res;
        let index_count = 6 * (res-1) * (res-1);
        let mut vertices = vec![glm::vec3(0.0, 0.0, 0.0); vertex_count];
        let mut normals = vec![glm::vec3(0.0, 1.0, 0.0); vertex_count];
        let mut texture = vec![glm::vec2(0.0, 0.0); vertex_count];
        let mut indices = vec![0; index_count];

        for z in 0..res {
            for x in 0..res {
                // Transform position
                let mut pos = glm::vec3(
                    2.0 * x as f32 / subdivisions as f32 - 1.0,
                    0.0f32,
                    2.0 * z as f32 / subdivisions as f32 - 1.0,
                );
                pos = glm::rotate_x_vec3(&pos, rotation.x);
                pos = glm::rotate_y_vec3(&pos, rotation.y);
                pos = glm::rotate_z_vec3(&pos, rotation.z);
                pos += position;
                // Side of cubesphere
                pos = glm::vec3(
                    pos.x * (1.0 - pos.y.powi(2) / 2.0 - pos.z.powi(2) / 2.0 + pos.y.powi(2) * pos.z.powi(2) / 3.0).sqrt(),
                    pos.y * (1.0 - pos.x.powi(2) / 2.0 - pos.z.powi(2) / 2.0 + pos.x.powi(2) * pos.z.powi(2) / 3.0).sqrt(),
                    pos.z * (1.0 - pos.x.powi(2) / 2.0 - pos.y.powi(2) / 2.0 + pos.x.powi(2) * pos.y.powi(2) / 3.0).sqrt(),
                ).component_mul(&scale) * 0.5;
                vertices[z * res + x] = pos;
                // Waves
                // vertices[y * res + x] = glm::vec3(
                //     2.0 * x as f32 / res as f32 - 1.0,
                //     ((2.0 * x as f32 / res as f32 - 1.0)*10.0).sin() / 10.0 + ((2.0 * y as f32 / res as f32 - 1.0)*10.0).sin() / 10.0,
                //     2.0 * y as f32 / res as f32 - 1.0,
                // ).component_mul(&scale) * 0.5;
                texture[z * res + x] = glm::vec2(
                    ((pos.x + pos.z).atan() + 1.0) / 2.0,
                    ((pos.y / pos.x).atan() + 1.0) / 2.0,
                );
                // Normal is just the position normalized for now
                normals[z * res + x] = glm::normalize(&vertices[z * res + x]);

                if z < subdivisions && x < subdivisions {
                    let offset = 6 * (z * subdivisions + x);
                    indices[offset + 0] = (z * res + x + 1) as u32;
                    indices[offset + 1] = (z * res + x + 0) as u32;
                    indices[offset + 2] = ((z + 1) * res + x + 1) as u32;

                    indices[offset + 3] = (z * res + x) as u32;
                    indices[offset + 4] = ((z + 1) * res + x) as u32;
                    indices[offset + 5] = ((z + 1) * res + x + 1) as u32;
                }
            }
        }

        Mesh {
            vertices: from_array_of_vec3(vertices),
            normals: from_array_of_vec3(normals),
            texture_coordinates: from_array_of_vec2(texture),
            colors: generate_color_vec(color.unwrap_or(glm::vec4(1.0, 1.0, 1.0, 1.0)), vertex_count),
            indices,
            index_count: index_count as i32,
        }
    }

    /// (Not working) Generate a plane projected to a cubesphere from corner start to corner stop
    /// * start and stop are points on the surface of the surrounding cube, meaning
    /// one of their components should be equal
    pub fn cs_part_plane(
        //scale: glm::TVec3<f32>,
        start: glm::TVec3<f32>,
        stop: glm::TVec3<f32>,
        subdivisions: usize, 
        tiling_textures: bool
    ) -> Self {
        let res = 1 + subdivisions;
        let vertex_count = res * res;
        let index_count = 6 * (res-1) * (res-1);
        let mut vertices = vec![glm::vec3(0.0, 0.0, 0.0); vertex_count];
        let normals = vec![glm::vec3(0.0, 1.0, 0.0); vertex_count];
        let mut texture = vec![glm::vec2(0.0, 0.0); vertex_count];
        let mut indices = vec![0; index_count];

        let diff = stop - start;
        let step = diff / res as f32;
        let mut pos = start;

        for z in 0..res {
            for x in 0..res {
                // Flat plane
                vertices[z * res + x] = pos;
                // Side of cubesphere
                // vertices[z * res + x] = glm::vec3(
                //     pos.x * (1.0 - pos.y.powi(2) / 2.0 - pos.z.powi(2) / 2.0 + pos.y.powi(2) * pos.z.powi(2) / 3.0).sqrt(),
                //     pos.y * (1.0 - pos.x.powi(2) / 2.0 - pos.z.powi(2) / 2.0 + pos.x.powi(2) * pos.z.powi(2) / 3.0).sqrt(),
                //     pos.z * (1.0 - pos.x.powi(2) / 2.0 - pos.y.powi(2) / 2.0 + pos.x.powi(2) * pos.y.powi(2) / 3.0).sqrt(),
                // ).component_mul(&scale) * 0.5;
                if z == 0 && x == 0 {
                    println!("0,0: {:?}", vertices[z * res + x]);
                }
                texture[z * res + x] = glm::vec2(
                    x as f32 / res as f32,
                    z as f32 / res as f32,
                );
                if z < subdivisions && x < subdivisions {
                    let offset = 6 * (z * subdivisions + x);
                    indices[offset + 0] = (z * res + x + 1) as u32;
                    indices[offset + 1] = (z * res + x + 0) as u32;
                    indices[offset + 2] = ((z + 1) * res + x + 1) as u32;

                    indices[offset + 3] = (z * res + x) as u32;
                    indices[offset + 4] = ((z + 1) * res + x) as u32;
                    indices[offset + 5] = ((z + 1) * res + x + 1) as u32;
                }

            }
        }

        Mesh {
            vertices: from_array_of_vec3(vertices),
            normals: from_array_of_vec3(normals),
            texture_coordinates: from_array_of_vec2(texture),
            colors: generate_color_vec(glm::vec4(1.0, 1.0, 1.0, 1.0), vertex_count),
            indices,
            index_count: index_count as i32,
        }
    }
}

use noise::Fbm;
use noise::utils::SphereMapBuilder;
use noise::{NoiseFn, Perlin};

fn fractal_noise(generator: Perlin, point: &glm::TVec3<f32>, size: f64, height: f32, offset: f32) -> f32 {
    let mut noise_sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;

    for _ in 0..5 {
        let point = point * freq;
        noise_sum += generator.get([
            point.x as f64 * size,
            point.y as f64 * size,
            point.z as f64 * size,
        ]) as f32 * amp * height;
        freq *= 2.0;
        amp *= 0.5;
    }
    noise_sum
}

// TODO: Better integrate as a Planet struct with set parameters, function can 
// TODO  be reused as computed bounding box.

// TODO: Interpolated height colours (noise-rs probably has it already)
pub fn displace_vertices(mesh: &mut Mesh, size: f64, height: f32, offset: f32) {
    let mut vertices = to_array_of_vec3(mesh.vertices.clone());
    let perlin = Perlin::new();
    // let fbm = Fbm::new();
    // let b = SphereMapBuilder::new(&fbm).set
    for i in 0..vertices.len() {
        let val = 1.0 + fractal_noise(perlin, &vertices[i], size, height, offset);
        vertices[i] *= val;
        // let val = perlin.get([
        //     mesh.vertices[i*3 + 0] as f64 * size, 
        //     mesh.vertices[i*3 + 1] as f64 * size,
        //     mesh.vertices[i*3 + 2] as f64 * size,
        // ]) as f32 * height + offset;
        // mesh.vertices[i*3 + 0] *= 1.0 + val;
        // mesh.vertices[i*3 + 1] *= 1.0 + val;
        // mesh.vertices[i*3 + 2] *= 1.0 + val;
    }
    
    // TODO: Solve the seams, could reuse the noise generator and use polar coordinates
    let mut normals = to_array_of_vec3(mesh.normals.clone());
    for i in (0..mesh.index_count).step_by(3) {
        let i = i as usize;
        let v1 = vertices[mesh.indices[i + 1] as usize] - vertices[mesh.indices[i] as usize];
        let v2 = vertices[mesh.indices[i + 2] as usize] - vertices[mesh.indices[i] as usize];
        let norm = glm::normalize(&glm::cross(&v1, &v2));
        normals[mesh.indices[i] as usize] = norm;
        normals[mesh.indices[i + 1] as usize] = norm;
        normals[mesh.indices[i + 2] as usize] = norm;
    }
    mesh.normals = from_array_of_vec3(normals);
    mesh.vertices = from_array_of_vec3(vertices);
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

