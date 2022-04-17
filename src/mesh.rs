use tobj;
use crate::util;

// internal helper
fn generate_color_vec(color: glm::TVec4<f32>, num: usize) -> Vec<f32> {
    glm::value_ptr(&color).iter().cloned().cycle().take(num*4).collect()
    //color.iter().cloned().cycle().take(num*4).collect()
}
/// Smooth min
// fn smin(a: f32, b: f32, k: f32) -> f32 {
//     let h = 0.0f32.max(k - (a-b).abs()) / k;
//     return a.min(b) - h.powi(3) * k / 6.0;
// }


// GL util VAO object
#[derive(Copy, Clone, Default, Debug)]
pub struct VAOobj {
    pub vao: u32,   // Vertex Array Object 
    pub vbo: u32,   // Vertex Buffer Object
    pub ibo: u32,   // Index Buffer Object
    pub cbo: u32,   // Color Buffer Object
    pub nbo: u32,   // Normal Buffer Object
    pub texbo: u32, // Texture Buffer Object
    pub n: i32,     // Index Count
}

//-----------------------------------------------------------------------------/
// Mesh
//-----------------------------------------------------------------------------/
#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub normals: Vec<f32>,
    pub texture_coordinates: Vec<f32>,
    pub colors: Vec<f32>,
    pub indices: Vec<u32>,
    pub index_count: i32,
}

impl Mesh {
    #[allow(unused)]
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

    /// Extended mkvao_simple_color to associate colors to vertices
    pub unsafe fn mkvao(&self) -> VAOobj {
        let mut id = VAOobj { n: self.index_count, ..Default::default() };

        /* Create and bind vertex array */
        gl::GenVertexArrays(1, &mut id.vao);
        gl::BindVertexArray(id.vao);

        /* Create and bind index buffer, add data */
        //let mut ibo = 0;
        gl::GenBuffers(1, &mut id.ibo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id.ibo);

        let ibuf_size = util::byte_size_of_array(&self.indices);
        let ibuf_data = util::pointer_to_array(&self.indices);

        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                    ibuf_size,
                    ibuf_data as *const _,
                    gl::STATIC_DRAW);

        // Next sections are vertex attributes

        /* Create and bind vertex buffer, add data */
        gl::GenBuffers(1, &mut id.vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, id.vbo);

        let vbuf_size = util::byte_size_of_array(&self.vertices);
        let vbuf_data = util::pointer_to_array(&self.vertices);

        gl::BufferData(gl::ARRAY_BUFFER, 
                        vbuf_size,
                        vbuf_data as *const _,
                        gl::STATIC_DRAW); 

        let mut attrib_idx = 0;
        /* Define attrib ptr for vertex buffer */
        gl::EnableVertexAttribArray(attrib_idx);
        gl::VertexAttribPointer(attrib_idx, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

        /* Create and bind color buffer, add data */
        gl::GenBuffers(1, &mut id.cbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, id.cbo);

        let cbuf_size = util::byte_size_of_array(&self.colors);
        let cbuf_data = util::pointer_to_array(&self.colors);

        gl::BufferData( gl::ARRAY_BUFFER,
                        cbuf_size,
                        cbuf_data as *const _,
                        gl::STATIC_DRAW);

        attrib_idx += 1;
        /* Define attrib ptr for color buffer */
        gl::EnableVertexAttribArray(attrib_idx);
        gl::VertexAttribPointer(attrib_idx, 4, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

        /* Add normals */
        gl::GenBuffers(1, &mut id.nbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, id.nbo);
        let nbo_size = util::byte_size_of_array(&self.normals);
        let nbo_data = util::pointer_to_array(&self.normals);

        gl::BufferData( gl::ARRAY_BUFFER,
                        nbo_size,
                        nbo_data as *const _,
                        gl::STATIC_DRAW);
        
        attrib_idx += 1;
        /* Define attrib ptr for normals buffer */
        gl::EnableVertexAttribArray(attrib_idx);
        gl::VertexAttribPointer(attrib_idx, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

        /* Add texture coordinates */
        gl::GenBuffers(1, &mut id.texbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, id.texbo);
        let texbo_size = util::byte_size_of_array(&self.texture_coordinates);
        let texbo_data = util::pointer_to_array(&self.texture_coordinates);

        gl::BufferData( gl::ARRAY_BUFFER,
                        texbo_size,
                        texbo_data as *const _,
                        gl::STATIC_DRAW);
        
        attrib_idx += 1;
        /* Define attrib ptr for normals buffer */
        gl::EnableVertexAttribArray(attrib_idx);
        gl::VertexAttribPointer(attrib_idx, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

        id
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
            vertices: util::from_array_of_vec3(vertices),
            indices: mindices,
            normals: util::from_array_of_vec3(mnormals),
            texture_coordinates: util::from_array_of_vec2(texture_coordinates),
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
            vertices: util::from_array_of_vec3(vertices),
            normals: util::from_array_of_vec3(normals),
            texture_coordinates: util::from_array_of_vec2(texture),
            colors: generate_color_vec(glm::vec4(1.0, 1.0, 1.0, 1.0), vertex_count),
            indices,
            index_count,
        }
    }

    pub fn cs_plane(
        scale: glm::TVec3<f32>,
        rotation: glm::TVec3<f32>,
        position: glm::TVec3<f32>, 
        subdivisions: usize,
        color: Option<glm::TVec4<f32>>,
        cubesphere: bool,
    ) -> Self {
        let res = 1 + subdivisions;
        let vertex_count = res * res;
        let index_count = 6 * (res-1) * (res-1);
        let step = scale / subdivisions as f32 * 2.0;
        util::MEMORY_USAGE.fetch_add(
            vertex_count as u64 * 4 * 8
            + index_count as u64* 4, 
            std::sync::atomic::Ordering::Relaxed
        );
        // let timer = std::time::SystemTime::now();
        // eprint!("Constructing CS plane with {} vertices . . . ", vertex_count);
        let mut vertices = vec![glm::vec3(0.0, 0.0, 0.0); vertex_count];
        let mut normals = vec![glm::vec3(0.0, 1.0, 0.0); vertex_count];
        let mut texture = vec![glm::vec2(0.0, 0.0); vertex_count];
        let mut indices = vec![0; index_count];


        for z in 0..res {
            for x in 0..res {
                // Transform position
                let mut pos = glm::vec3(
                    position.x - scale.x + step.x * x as f32,
                    //2.0 * x as f32 / subdivisions as f32 - 1.0,
                    1.0,
                    position.z - scale.z + step.z * z as f32,
                    //2.0 * z as f32 / subdivisions as f32 - 1.0,
                );
                // Convert to side of cubesphere
                if cubesphere {
                    pos = glm::vec3(
                        pos.x * (1.0 - pos.y.powi(2) / 2.0 - pos.z.powi(2) / 2.0 + pos.y.powi(2) * pos.z.powi(2) / 3.0).sqrt(),
                        pos.y * (1.0 - pos.x.powi(2) / 2.0 - pos.z.powi(2) / 2.0 + pos.x.powi(2) * pos.z.powi(2) / 3.0).sqrt(),
                        pos.z * (1.0 - pos.x.powi(2) / 2.0 - pos.y.powi(2) / 2.0 + pos.x.powi(2) * pos.y.powi(2) / 3.0).sqrt(),
                    ) * 0.5; // removed: .component_mul(&scale)

                }
                pos = glm::rotate_x_vec3(&pos, rotation.x);
                pos = glm::rotate_y_vec3(&pos, rotation.y);
                pos = glm::rotate_z_vec3(&pos, rotation.z);
                vertices[z * res + x] = pos;

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

        // eprintln!("took {:?}", timer.elapsed().unwrap());
        Mesh {
            vertices: util::from_array_of_vec3(vertices),
            normals: util::from_array_of_vec3(normals),
            texture_coordinates: util::from_array_of_vec2(texture),
            colors: generate_color_vec(color.unwrap_or(glm::vec4(1.0, 1.0, 1.0, 1.0)), vertex_count),
            indices,
            index_count: index_count as i32,
        }
    }

}

use noise::{NoiseFn, Perlin};

/// Some iterations of noise function to create a fractal noise
/// - `offset` deprecated
pub fn fractal_noise(generator: Perlin, point: &glm::TVec3<f32>, size: f64, height: f32, _offset: f32) -> f32 {
    let mut noise_sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;

    for _ in 0..5 {
        let point = point * freq;
        noise_sum += generator.get([
            point.x as f64 * size, // + seed as f64,
            point.y as f64 * size, // + seed as f64,
            point.z as f64 * size, // + seed as f64,
        ]) as f32 * amp * height;
        freq *= 2.0;
        amp *= 0.5;
    }
    noise_sum
}
