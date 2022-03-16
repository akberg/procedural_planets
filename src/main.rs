use glium;
use glium::glutin;
use glium::{Surface};

use std::sync::{Arc, Mutex};

use nalgebra_glm as glm;

mod mesh;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
glium::implement_vertex!(Vertex, position);

enum GameState { Loading, Ready }

fn main() {
    // Setup
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb,cb, &event_loop).unwrap();

    let (window_height, window_width) = display.get_framebuffer_dimensions();

    // Charmap
    let charmap = image::io::Reader::open("resources/textures/charmap.png").unwrap().decode().unwrap();
    // image::load(std::io::Cursor::new(&include_bytes!("resources/textures/charmap.png")), image::ImageFormat::Png).unwrap().to_rgba8();
    let text_mesh = generate_text_geometry_buffer("Hello, World!", 49.0 / 29.0, 2.0);

    // Load shaders
    let vertex_shader_src = std::fs::read_to_string("resources/shaders/planet.vert").unwrap();
    let fragment_shader_src = std::fs::read_to_string("resources/shaders/planet.frag").unwrap();
    // If tessellation shaders are used:
    let program = glium::program::SourceCode {
        vertex_shader: &vertex_shader_src,
        fragment_shader: &fragment_shader_src,
        tessellation_control_shader: None,
        tessellation_evaluation_shader: None,
        geometry_shader: None,
    };
    let program = glium::Program::new(&display, program).unwrap();
    // Simple:
    // let program = glium::Program::from_source(&display, 
    //     &vertex_shader_src, 
    //     &fragment_shader_src, 
    //     None
    // ).unwrap();



    // Initialize game
    let shape = vec![
        Vertex { position: [-0.5, -0.5] },
        Vertex { position: [ 0.5, -0.25] },
        Vertex { position: [ 0.5,  0.5] },
    ];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    let perspective = glm::perspective(
        window_height as f32 / window_width as f32, 
        1.3, 
        0.01, 
        1000.0
    );
    

    // Event loop
    event_loop.run(move |ev, _, control_flow| {
        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent {event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return
                },
                // glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                //     match input.virtual_keycode {
                //         Some(glutin::event::VirtualKeyCode::Q) => {
                //             *control_flow = glutin::event_loop::ControlFlow::Exit;
                //             return

                //         }
                //     }
                // },
                _ => return,
            },
            _ => (),
        }

        // Draw parameters
        use glium::draw_parameters;
        let params = glium::DrawParameters {
            // depth: glium::Depth {
            //     test: draw_parameters::DepthTest::IfLess,
            //     write: true,
            //     .. Default::default()
            // },
            backface_culling: draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            polygon_mode: draw_parameters::PolygonMode::Line,
            .. Default::default()
        };

        // Clear screen
        let mut target = display.draw();
        
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        
        // Draw
        target.draw(
            &vertex_buffer, 
            &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), 
            &program, 
            &glium::uniforms::EmptyUniforms, 
            &params
        ).unwrap();

        // Finish drawing
        target.finish().unwrap();
    });
}

use obj;
fn generate_text_geometry_buffer(text: &str, char_h_over_w: f32, total_text_w: f32) -> mesh::Mesh {
    let char_w = total_text_w / text.len() as f32;
    let char_h = char_h_over_w * char_w;

    let vertex_count = 4 * text.len();
    let index_count = 6 * text.len();

    let mut vertices = vec![glm::vec3(0.0f32, 0.0, 0.0); vertex_count];
    let mut uv_coords = vec![glm::vec2(0.0f32, 0.0); vertex_count];
    let mut indices = vec![0u32; index_count];
    let mut normals = vec![glm::vec3(0.0f32, 0.0, 0.0); vertex_count];

    for (i, c) in text.chars().map(|c|c as u8).enumerate() {
        let base_coord = i as f32 * char_w;
        vertices[4 * i + 0] = glm::vec3(base_coord as f32, 0.0, 0.0);
        vertices[4 * i + 1] = glm::vec3(base_coord as f32 + char_w, 0.0, 0.0);
        vertices[4 * i + 2] = glm::vec3(base_coord as f32 + char_w, char_h, 0.0);
        
        vertices[4 * i + 3] = glm::vec3(base_coord as f32, char_h, 0.0);

        uv_coords[4 * i + 0] = glm::vec2(c as f32 / 128.0, 0.0);
        uv_coords[4 * i + 1] = glm::vec2((c+1) as f32 / 128.0, 0.0);
        uv_coords[4 * i + 2] = glm::vec2((c+1) as f32 / 128.0, 1.0);

        uv_coords[4 * i + 3] = glm::vec2(c as f32 / 128.0, 1.0);

        normals[4 * i + 0] = glm::vec3(0.0, 0.0, -1.0);
        normals[4 * i + 1] = glm::vec3(0.0, 0.0, -1.0);
        normals[4 * i + 2] = glm::vec3(0.0, 0.0, -1.0);
        normals[4 * i + 3] = glm::vec3(0.0, 0.0, -1.0);
        
        indices[4 * i + 0] = (4 * i + 0) as u32;
        indices[4 * i + 1] = (4 * i + 1) as u32;
        indices[4 * i + 2] = (4 * i + 2) as u32;
        indices[4 * i + 3] = (4 * i + 3) as u32;
        indices[4 * i + 4] = (4 * i + 4) as u32;
        indices[4 * i + 5] = (4 * i + 5) as u32;
    }


    mesh::Mesh {
        vertices: vertices,
        indices: indices,
        colors: None,
        index_count: index_count as i32,
        normals: normals,
        uv_texture: Some(uv_coords),
    }
}


fn generate_terrain_texture() {
    let cell_w = 16;
    let cell_h = 16;
}

fn generate_cubesphere() -> mesh::Mesh {
    use std::f32::consts::PI;
    let subdivisions = 1;
    let radius = 1.0;

    let res = subdivisions + 2;
    let steps = res * 2 - 1;
    let step_size = PI / steps as f32;
    // Start with one face
    let vertex_count = res * res;
    let face_count = (res - 1) * (res - 1);
    // let vertex_count = res * res * 2 + subdivisions * 4 * (res - 1);
    // let face_count = (res - 1) * (res - 1) * 2 * 6;
    let index_count = face_count * 3;

    let mut vertices = vec![glm::vec3(0.0f32, 0.0, 0.0); vertex_count];
    let mut normals = vec![glm::vec3(0.0f32, 0.0, 0.0); vertex_count];
    let mut indices = vec![0u32; index_count];

    let mut i = 0;
    // Front and back face
    for y in 0..res {
        for x in 0..res {
            let vx = glm::vec3(x, y, 0);
            let vx: glm::TVec3<f32> = glm::convert(vx);
            // let pos = glm::vec3(
            //     f32::cos(PI / 4.0 + x as f32 * step_size),
            //     f32::sin(PI / 4.0 + x as f32 * step_size),

            // )

            // let vx = glm::vec3(x, y, res-1);

        }
    }
    // for z in 0..subdivisions {
    //     // Top and bottom
    //     for x in 0..res {
    //         let vx = glm::vec3(x, 0, z+1);

    //         let vx = glm::vec3(x, res-1, z+1);
    //     }
    //     // Left and right
    //     for y in 0..subdivisions {
    //         let vx = glm::vec3(0, y+1, z+1);

    //         let vx = glm::vec3(res-1, y+1, z+1);
    //     }
    // }

    mesh::Mesh {
        vertices: vertices,
        indices: indices,
        normals: normals,
        colors: None,
        index_count: index_count as i32,
        uv_texture: None,
    }
}


struct Mesh {

}