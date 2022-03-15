use glium;
use glium::glutin;
use glium::{Surface};

use std::sync::{Arc, Mutex};

use nalgebra_glm as glm;


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

    // Charmap
    let charmap = image::io::Reader::open("resources/textures/charmap.png").unwrap().decode().unwrap();
    // image::load(std::io::Cursor::new(&include_bytes!("resources/textures/charmap.png")), image::ImageFormat::Png).unwrap().to_rgba8();

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
        Vertex { position: [ 0.5,  0.5] },
        Vertex { position: [ 0.5, -0.25] },
    ];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    

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

        // Clear screen
        let mut target = display.draw();
        
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        
        // Draw
        target.draw(
            &vertex_buffer, 
            &glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList), 
            &program, 
            &glium::uniforms::EmptyUniforms, 
            &Default::default()
        ).unwrap();

        // Finish drawing
        target.finish().unwrap();
    });
}


fn generate_text_geometry_buffer(text: &str, char_h_over_w: f32, total_text_w: f32) {
    let char_w = total_text_w / text.len() as f32;
    let char_h = char_h_over_w * char_w;

    let vertex_count = 4 * text.len();
    let index_count = 6 * text.len();

    let mut vertices = vec![glm::vec3(0.0f32, 0.0, 0.0); vertex_count];
    let mut uv_coords = vec![glm::vec2(0.0f32, 0.0); vertex_count];
    let mut indices = vec![glm::vec3(0u16, 0, 0); index_count];
    let mut normals = vec![glm::vec3(0.0f32, 0.0, 0.0); vertex_count];

    for (i, c) in text.chars().enumerate() {
        let base_coord = i as f32 * char_w;
        vertices[4 * i + 0] = glm::vec3(base_coord as f32, 0.0, 0.0);
        vertices[4 * i + 1] = glm::vec3(base_coord as f32 + char_w, 0.0, 0.0);
        vertices[4 * i + 2] = glm::vec3(base_coord as f32 + char_w, char_h, 0.0);
        
        vertices[4 * i + 3] = glm::vec3(base_coord as f32, char_h, 0.0);
    }
}