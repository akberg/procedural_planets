use glium;
use glium::glutin;
use glium::{Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
glium::implement_vertex!(Vertex, position);

fn main() {
    // Setup
    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb,cb, &event_loop).unwrap();

    // Simple shader for now
    let vertex_shader_src = r#"

        in vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;
    let fragment_shader_src = r#"

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;
    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    // Initialize game
    let shape = vec![
        Vertex { position: [-0.5, -0.5] },
        Vertex { position: [ 0.5,  0.5] },
        Vertex { position: [ 0.5, -0.25] },
    ];
    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    

    // Render thread
    // std::thread::spawn(move || {
    //     // Clear screen
    //     let mut target = display.draw();
    //     target.clear_color(0.0, 0.0, 1.0, 1.0);
    //     target.finish().unwrap();
    
    //     // Draw
    
        

    // });

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
    });
}
