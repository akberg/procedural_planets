extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;

use scene_graph::{SceneNode, SceneNodeType, LightSource, LightSourceType};
use util::CameraPosition::*;

use glutin::event::{
    Event,
    WindowEvent,
    DeviceEvent,
    KeyboardInput,
    ElementState::{Pressed, Released},
    VirtualKeyCode::{self, *}
};

use glutin::event_loop::ControlFlow;

const SCREEN_W: u32 = 800;
const SCREEN_H: u32 = 600;

const polymodes: [u32;3] = [gl::FILL, gl::POINT, gl::LINE];


struct VAOobj {
    vao: u32,   /* Vertex Array Object */
    n: i32,     /* Number of triangles */
}


/// Extended mkvao_simple_color to associate colors to vertices
unsafe fn mkvao(obj: &mesh::Mesh) -> VAOobj {

    /* Create and bind vertex array */
    let mut vao = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    /* Create and bind index buffer, add data */
    let mut ibo = 0;
    gl::GenBuffers(1, &mut ibo);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

    let ibuf_size = util::byte_size_of_array(&obj.indices);
    let ibuf_data = util::pointer_to_array(&obj.indices);

    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                   ibuf_size,
                   ibuf_data as *const _,
                   gl::STATIC_DRAW);

    // Next sections are vertex attributes

    /* Create and bind vertex buffer, add data */
    let mut vbo = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

    let vbuf_size = util::byte_size_of_array(&obj.vertices);
    let vbuf_data = util::pointer_to_array(&obj.vertices);

    gl::BufferData(gl::ARRAY_BUFFER, 
                    vbuf_size,
                    vbuf_data as *const _,
                    gl::STATIC_DRAW); 

    let mut attrib_idx = 0;
    /* Define attrib ptr for vertex buffer */
    gl::EnableVertexAttribArray(attrib_idx);
    gl::VertexAttribPointer(attrib_idx, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

    /* Create and bind color buffer, add data */
    let mut cbo = 0;
    gl::GenBuffers(1, &mut cbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, cbo);

    let cbuf_size = util::byte_size_of_array(&obj.colors);
    let cbuf_data = util::pointer_to_array(&obj.colors);

    gl::BufferData( gl::ARRAY_BUFFER,
                    cbuf_size,
                    cbuf_data as *const _,
                    gl::STATIC_DRAW);

    attrib_idx += 1;
    /* Define attrib ptr for color buffer */
    gl::EnableVertexAttribArray(attrib_idx);
    gl::VertexAttribPointer(attrib_idx, 4, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

    /* Add normals */
    let mut nbo = 0;
    gl::GenBuffers(1, &mut nbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, nbo);
    let nbo_size = util::byte_size_of_array(&obj.normals);
    let nbo_data = util::pointer_to_array(&obj.normals);

    gl::BufferData( gl::ARRAY_BUFFER,
                    nbo_size,
                    nbo_data as *const _,
                    gl::STATIC_DRAW);
    
    attrib_idx += 1;
    /* Define attrib ptr for normals buffer */
    gl::EnableVertexAttribArray(attrib_idx);
    gl::VertexAttribPointer(attrib_idx, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

    /* Add texture coordinates */
    let mut texbo = 0;
    gl::GenBuffers(1, &mut texbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, texbo);
    let texbo_size = util::byte_size_of_array(&obj.texture_coordinates);
    let texbo_data = util::pointer_to_array(&obj.texture_coordinates);

    gl::BufferData( gl::ARRAY_BUFFER,
                    texbo_size,
                    texbo_data as *const _,
                    gl::STATIC_DRAW);
    
    attrib_idx += 1;
    /* Define attrib ptr for normals buffer */
    gl::EnableVertexAttribArray(attrib_idx);
    gl::VertexAttribPointer(attrib_idx, 2, gl::FLOAT, gl::FALSE, 0, std::ptr::null());

    println!("Create vao={}, ibo={}, vbo={}, cbo={}, texbo={}", vao, ibo, vbo, cbo, texbo);

    VAOobj { vao, n: obj.index_count }
}
unsafe fn get_texture_id(img: &image::DynamicImage) -> u32 {
    use image::GenericImageView;

    println!("Generating texture id for image with dimensions {:?}", img.dimensions());

    let mut tex_id = 0;
    gl::GenTextures(1, &mut tex_id);

    gl::BindTexture(gl::TEXTURE_2D, tex_id);

    // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        img.dimensions().0 as i32,
        img.dimensions().1 as i32,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        util::pointer_to_array(&img.as_bytes())
    );

    gl::GenerateMipmap(gl::TEXTURE_2D);

    tex_id
}


fn main() {
    //-------------------------------------------------------------------------/
    // Set up the necessary objects to deal with windows and event handling
    //-------------------------------------------------------------------------/
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it 
    // to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("failed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    //-------------------------------------------------------------------------/
    // Spawn a separate thread for rendering, so event handling doesn't 
    // block rendering
    //-------------------------------------------------------------------------/
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers. This has 
        // to be done inside of the rendering thread, because an active OpenGL 
        // context cannot safely traverse a thread boundary.

        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        //---------------------------------------------------------------------/
        // Set up openGL
        //---------------------------------------------------------------------/
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);                                  // Enable transparency
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);  //
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        //---------------------------------------------------------------------/
        // Read config
        //---------------------------------------------------------------------/
        let conf = util::Config::load();
        // println!("{:?}", conf);
        let mut polymode = 0;

        //---------------------------------------------------------------------/
        // Camera setup (available for keypress handler)
        //---------------------------------------------------------------------/
        let mut position = glm::vec3(
            conf.init_position[0],
            conf.init_position[1],
            conf.init_position[2],
        );
        let mut h_angle = conf.init_h_angle;
        let mut v_angle = conf.init_v_angle;
        let mut direction = util::vec_direction(h_angle, v_angle);
        let mut up = glm::vec3(0.0, 1.0, 0.0);
        let mut right = util::vec_right(h_angle);

        // Controls multipliers
        let mouse_speed = conf.mouse_speed;
        let movement_speed = conf.movement_speed;
        let tilt_speed = conf.tilt_speed;

        let camera_position = match conf.camera_position {
            0 => ThirdPerson,
            1 => FirstPerson,
            2 => unimplemented!(),
            _ => unreachable!()
        };

        //---------------------------------------------------------------------/
        // Lighting
        //---------------------------------------------------------------------/
        let diffuse_light = vec![1.0, -1.0, 0.0];

        let v = glm::vec3(1.0, 1.0, 1.0);
        
        //---------------------------------------------------------------------/
        // Vertex Array Objects, create vertices or load models
        //---------------------------------------------------------------------/

        // Placeholder cube, will become a cubesphere at some point
        let cube_mesh = mesh::Mesh::cube(
            glm::vec3(0.1, 0.1, 0.1), 
            glm::vec2(1.0, 1.0), true, false, 
            glm::vec3(1.0, 1.0, 1.0),
            glm::vec4(1.0, 0.0, 0.0, 1.0),
        );
        let cube_vao = unsafe { mkvao(&cube_mesh) };
        let cube_node = SceneNode::from_vao(cube_vao.vao, cube_vao.n);

        // Skybox, inverted cube that stays centered around the player
        let skybox_mesh = mesh::Mesh::cube(
            glm::vec3(conf.clip_far-0.1, conf.clip_far-0.1, conf.clip_far-0.1), // Defines visible distance of other objects
            glm::vec2(1.0, 1.0), true, true, 
            glm::vec3(1.0, 1.0, 1.0),
            glm::vec4(0.01, 0.01, 0.06, 0.2),
        );
        let skybox_vao = unsafe { mkvao(&skybox_mesh) };
        let mut skybox_node = SceneNode::from_vao(skybox_vao.vao, skybox_vao.n);
        skybox_node.node_type = SceneNodeType::Skybox;


        // TODO: Make this more elegant:

        let mut cubesphere = SceneNode::with_type(SceneNodeType::Empty);

        // Top
        let plane0_mesh = mesh::Mesh::cs_plane(
            glm::vec3(0.5, 0.5, 0.5), 
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
            64, true,
            Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        );
        let plane0_vao = unsafe { mkvao(&plane0_mesh) };
        let mut plane0_node = SceneNode::from_vao(plane0_vao.vao, plane0_vao.n);
        // Bottom
        let plane1_mesh = mesh::Mesh::cs_plane(
            glm::vec3(0.5, 0.5, 0.5), 
            glm::vec3(std::f32::consts::PI, 0.0, 0.0),
            glm::vec3(0.0, -1.0, 0.0),
            64, true,
            Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        );
        let plane1_vao = unsafe { mkvao(&plane1_mesh) };
        let mut plane1_node = SceneNode::from_vao(plane1_vao.vao, plane1_vao.n);
        // Front
        let plane2_mesh = mesh::Mesh::cs_plane(
            glm::vec3(0.5, 0.5, 0.5), 
            glm::vec3(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
            64, true,
            Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        );
        let plane2_vao = unsafe { mkvao(&plane2_mesh) };
        let mut plane2_node = SceneNode::from_vao(plane2_vao.vao, plane2_vao.n);
        // Back
        let plane3_mesh = mesh::Mesh::cs_plane(
            glm::vec3(0.5, 0.5, 0.5), 
            glm::vec3(-std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            glm::vec3(0.0, 0.0, -1.0),
            64, true,
            Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        );
        let plane3_vao = unsafe { mkvao(&plane3_mesh) };
        let mut plane3_node = SceneNode::from_vao(plane3_vao.vao, plane3_vao.n);
        // Left
        let plane4_mesh = mesh::Mesh::cs_plane(
            glm::vec3(0.5, 0.5, 0.5), 
            glm::vec3(0.0, 0.0, -std::f32::consts::FRAC_PI_2),
            glm::vec3(1.0, 0.0, 0.0),
            64, true,
            Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        );
        let plane4_vao = unsafe { mkvao(&plane4_mesh) };
        let mut plane4_node = SceneNode::from_vao(plane4_vao.vao, plane4_vao.n);
        // Right
        let plane5_mesh = mesh::Mesh::cs_plane(
            glm::vec3(0.5, 0.5, 0.5), 
            glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),
            glm::vec3(-1.0, 0.0, 0.0),
            64, true,
            Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        );
        let plane5_vao = unsafe { mkvao(&plane5_mesh) };
        let mut plane5_node = SceneNode::from_vao(plane5_vao.vao, plane5_vao.n);
        // let mut plane5_node = SceneNode::with_type(SceneNodeType::Empty);
        // let plane50_mesh = mesh::Mesh::cs_plane(
        //     glm::vec3(0.25, 0.25, 0.25), 
        //     glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),
        //     glm::vec3(-1.0, -0.5, -0.5),
        //     16, true,
        //     Some(glm::vec4(0.8, 0.2, 0.4, 1.0))
        // );
        // let plane50_vao = unsafe { mkvao(&plane50_mesh) };
        // let mut plane50_node = SceneNode::from_vao(plane50_vao.vao, plane50_vao.n);
        // plane5_node.add_child(&plane50_node);
        
        cubesphere.add_child(&plane0_node);
        cubesphere.add_child(&plane1_node);
        cubesphere.add_child(&plane2_node);
        cubesphere.add_child(&plane3_node);
        cubesphere.add_child(&plane4_node);
        cubesphere.add_child(&plane5_node);

        // let part_plane = mesh::Mesh::cs_part_plane(glm::vec3(-1.0, 0.0, 1.0), glm::vec3(1.0, 0.0, -1.0), 64, true);
        // let pplane_vao = unsafe { mkvao(&part_plane) };
        // let mut pplane_node = SceneNode::from_vao(pplane_vao.vao, pplane_vao.n);

        // let my_text = mesh::Mesh::text_buffer("0123456789", 49.0 / 29.0, 2.0);
        // println!("text texture coordinates: {:?}", my_text.texture_coordinates);
        // let text_vao = unsafe {mkvao(&my_text)};
        // let mut text_node = SceneNode::from_vao(text_vao.vao, text_vao.n);
        // text_node.node_type = SceneNodeType::Geometry2d;

        // let charmap = image::open("resources/textures/charmap.png").unwrap().flipv();
        // use std::io::Cursor;
        // use image::io::Reader as ImageReader;
        // let charmap = ImageReader::open("resources/textures/charmap.png").unwrap().decode().unwrap().flipv();

        // let charmap_id = unsafe { get_texture_id(&charmap) };
        // text_node.texture_id = Some(charmap_id);
        // text_node.position = glm::vec3(-0.5, 0.0, 0.0);
        // text_node.scale = glm::vec3(0.2, 0.2, 0.2);
        
        /* Load terrain */
        // let terrain_obj = mesh::Terrain::load("resources/lunarsurface.obj");
        // let terrain_vao = unsafe { mkvao(&terrain_obj) };

        // /* Load Helicopter */
        // let helicopter = mesh::Helicopter::load("resources/helicopter.obj");
        // let heli_door_vao = unsafe { mkvao(&helicopter.door) };
        // let heli_body_vao = unsafe { mkvao(&helicopter.body) };
        // let heli_main_rotor_vao = unsafe { mkvao(&helicopter.main_rotor) };
        // let heli_tail_rotor_vao = unsafe { mkvao(&helicopter.tail_rotor) };

        // let mut doors = false;
        // let mut doors_start = 0.0;

        //---------------------------------------------------------------------/
        // Make Scene graph
        //---------------------------------------------------------------------/
        let mut scene_root = SceneNode::new();
        // scene_root.add_child(&cube_node);
        //scene_root.add_child(&skybox_node);
        // scene_root.add_child(&text_node);
        scene_root.add_child(&cubesphere);
        // scene_root.add_child(&plane0_node);

        unsafe { scene_root.update_node_transformations(&glm::identity()); }

        scene_root.print();

        // Basic usage of shader helper:
        // The example code below returns a shader object, which contains the field `.program_id`.
        // The snippet is not enough to do the assignment, and will need to be modified (outside of
        // just using the correct path), but it only needs to be called once
        //
        //     shader::ShaderBuilder::new()
        //        .attach_file("./path/to/shader.file")
        //        .link();
        //---------------------------------------------------------------------/
        // Shaders and locating uniforms
        //---------------------------------------------------------------------/
        let sh = unsafe {
            let sh = shader::ShaderBuilder::new()
                .attach_file("./resources/shaders/scene.vert")
                .attach_file("./resources/shaders/scene.frag")
                .link();

            sh.activate();
            sh
        };

        //---------------------------------------------------------------------/
        // Uniform values
        //---------------------------------------------------------------------/
        let aspect: f32 = SCREEN_W as f32 / SCREEN_H as f32;
        let fovy = conf.fov;
        let perspective_mat: glm::Mat4 = 
            glm::perspective(
                aspect,         // aspect
                fovy,           // fovy
                conf.clip_near, // near
                conf.clip_far   // far
            );

        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;
        // The main rendering loop
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;


            //-----------------------------------------------------------------/
            // Handle keyboard input
            //-----------------------------------------------------------------/
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    let flat_direction =  glm::normalize(&glm::vec3(direction.x, 0.0, direction.z));
                    // Set movement relative to helicopter rotation
                    // let heli_direction = util::vec_direction(heli_body_nodes[n_helis].rotation.y, 0.0);
                    // let flat_direction = -heli_direction; //glm::normalize(&glm::vec3(heli_direction.x, 0.0, heli_direction.z));
                    // right = glm::cross(&flat_direction, &glm::vec3(0.0, 1.0, 0.0));
                    
                    match key {
                        /* Move left/right */
                        VirtualKeyCode::A => {
                            // //heli_body_nodes[n_helis].rotation.z = 0.2;
                            // tilt_dir.1 = 1;
                            // heli_body_nodes[n_helis].position -= right * delta_time * movement_speed;
                            position -= right * delta_time * movement_speed;
                        },
                        VirtualKeyCode::D => {
                            // heli_body_nodes[n_helis].rotation.z = -0.2;
                            // tilt_dir.1 = -1;
                            // heli_body_nodes[n_helis].position += right * delta_time * movement_speed;
                            position += right * delta_time * movement_speed;
                        },
                        /* Move forward (inward)/backward, in camera direction */
                        VirtualKeyCode::W => {
                            // heli_body_nodes[n_helis].rotation.x = -0.2;
                            // tilt_dir.0 = -1;
                            // heli_body_nodes[n_helis].position += flat_direction * delta_time * movement_speed;
                            position += flat_direction * delta_time * movement_speed;
                        },
                        VirtualKeyCode::S => {
                            // heli_body_nodes[n_helis].rotation.x = 0.2;
                            // tilt_dir.0 = 1;
                            // heli_body_nodes[n_helis].position -= flat_direction * delta_time * movement_speed;
                            position -= flat_direction * delta_time * movement_speed;
                        },
                        /* Move up/down */
                        VirtualKeyCode::Space => {
                            // heli_body_nodes[n_helis].position += glm::vec3(0.0, 1.0, 0.0) * delta_time * movement_speed;
                            position += glm::vec3(0.0, 1.0, 0.0) * delta_time * movement_speed;
                        },
                        VirtualKeyCode::LShift => {
                            // heli_body_nodes[n_helis].position -= glm::vec3(0.0, 1.0, 0.0) * delta_time * movement_speed;
                            position -= glm::vec3(0.0, 1.0, 0.0) * delta_time * movement_speed;
                        },
                        VirtualKeyCode::M => {
                            polymode = (polymode + 1) % 3;
                        }
                        _ => { }
                    }
                }
            }

            // Handle mouse movement. delta contains the x and y movement of 
            // the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {
                /* Look left/right (horizontal angle), rotate around y axis */
                h_angle -= (*delta).0 * delta_time * mouse_speed;
                /* Look up/down (vertical angle), rotate around x axis */
                v_angle -= (*delta).1 * delta_time * mouse_speed;
                direction = util::vec_direction(h_angle, v_angle);
                //heli_body_nodes[n_helis].rotation = glm::vec3(-direction.x, -direction.z, -direction.y);
                right = util::vec_right(h_angle);
                up = glm::cross(&right, &direction);

                *delta = (0.0, 0.0);
            }

            skybox_node.position = position;

            unsafe {
                //-------------------------------------------------------------/
                // Draw section
                //-------------------------------------------------------------/
                // First person view
                let cam = glm::look_at(&position, &(position+direction), &up);
                let perspective_view = perspective_mat * cam;
                // let perspective_view = perspective_mat * glm::look_at(&position, &heli_body_nodes[n_helis].position, &up);

                gl::ClearColor(conf.bg_color[0], conf.bg_color[1], conf.bg_color[2], conf.bg_color[3]);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                /* Draw scene graph */
                gl::PolygonMode(gl::FRONT_AND_BACK, polymodes[polymode]);
                scene_root.update_node_transformations(&glm::identity());
                scene_root.draw_scene(&perspective_view, &sh);
            }

            context.swap_buffers().unwrap();
        }
    });

    //-------------------------------------------------------------------------/
    // Keep track of the health of the rendering thread
    //-------------------------------------------------------------------------/
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });
    
    //-------------------------------------------------------------------------/
    // Start the event loop -- This is where window events get handled
    //-------------------------------------------------------------------------/
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }
                // Handle escape separately
                match keycode {
                    Escape => {
                        *control_flow = ControlFlow::Exit;
                    },
                    Q => {
                        /////*control_flow = ControlFlow::Exit;
                    },
                    _ => { }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            },
            _ => { }
        }
    });
}
