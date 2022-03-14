use glium;
use glium::glutin;
use glium::{Surface};

use std::sync::{Arc, Mutex};

use glm::noise2;


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
