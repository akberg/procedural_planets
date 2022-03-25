extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

use std::collections::HashMap;

mod shader;
mod util;
mod mesh;
mod scene_graph;
mod player;
mod procedural_planet;
mod texture;

use procedural_planet as planet;
use texture::load_texture;

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

const POLYMODES: [u32;3] = [gl::FILL, gl::POINT, gl::LINE];




fn main() {
    //-------------------------------------------------------------------------/
    // Set up the necessary objects to deal with windows and event handling
    //-------------------------------------------------------------------------/
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Procedural planets")
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize::new(SCREEN_W, SCREEN_H))
        ;
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
        let setup_timer = std::time::SystemTime::now();
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
        let mut conf = util::Config::load();

        let mut player_state = player::PlayerState::Anchored(glm::vec3(0.0, 0.0, 0.0));

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
        let v = glm::vec3(1.0, 1.0, 1.0);
        let timer = std::time::SystemTime::now();
        print!("Compiling shader . . . ");
        let sh = unsafe {
            let sh = shader::ShaderBuilder::new()
                .attach_file("./resources/shaders/scene.vert")
                .attach_file("./resources/shaders/scene.frag")
                .link();

            sh.activate();
            sh
        };
        println!("took {:?}", timer.elapsed());

        //---------------------------------------------------------------------/
        // Load charmap texture
        //---------------------------------------------------------------------/
        // let mut charmap = ImageReader::open("resources/textures/charmap.png").unwrap()
        //     .decode().unwrap()
        //     .flipv()
        //     .into_rgba8();
        // let charmap_id = unsafe { get_texture_id(&charmap) };
        let charmap_id = load_texture("resources/textures/charmap.png");

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
        let cube_vao = unsafe { cube_mesh.mkvao() };
        let cube_node = SceneNode::from_vao(cube_vao);

        // Skybox, inverted cube that stays centered around the player
        let mut skybox_mesh = mesh::Mesh::cube(
            glm::vec3(conf.clip_far-0.1, conf.clip_far-0.1, conf.clip_far-0.1), // Defines visible distance of other objects
            glm::vec2(1.0, 1.0), true, true, 
            glm::vec3(1.0, 1.0, 1.0),
            glm::vec4(0.05, 0.01, 0.06, 0.2),
        );
        // TODO: Figure out
        // use noise::NoiseFn;
        // let noisefn = noise::Perlin::new();
        // for i in 0..skybox_mesh.vertices.len() / 3 {
        //     let v = (noisefn.get([
        //         skybox_mesh.vertices[i * 3 + 0] as f64 * 10.0,
        //         skybox_mesh.vertices[i * 3 + 1] as f64 * 10.0,
        //         skybox_mesh.vertices[i * 3 + 2] as f64 * 10.0,
        //     ]) + 1.0) as f32 / 2.0;
        //     skybox_mesh.colors[i * 4 + 0] = v;
        //     skybox_mesh.colors[i * 4 + 1] = v;
        //     skybox_mesh.colors[i * 4 + 2] = v;
        //     skybox_mesh.colors[i * 4 + 3] = 1.0;
        // }
        let skybox_vao = unsafe { skybox_mesh.mkvao() };
        let mut skybox_node = SceneNode::from_vao(skybox_vao);
        skybox_node.node_type = SceneNodeType::Skybox;


        // TODO: Make this more elegant:
        // FIXME
        let size = 10.0;
        let height = 0.05;
        let offset = 0.0;
        // let mut cubesphere = SceneNode::make_cubesphere(
        //     glm::vec3(10.0, 10.0, 10.0), 
        //     glm::vec3(0.0, 0.0, 0.0), 
        //     glm::vec3(0.0, 0.0, 0.0), 
        //     32,
        //     None
        // );
        let mut cubesphere = SceneNode::with_type(SceneNodeType::Empty);
        cubesphere.scale *= 10.0;
        // for &n in cubesphere.children.iter() {
        //     mesh::displace_vertices((&mut *n).mesh, size, height, offset);
        // }
        let subdivisions = 144;
        let color = glm::vec4(0.2, 0.8, 0.4, 1.0);

        // Top
        let mut plane0_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        mesh::displace_vertices(&mut plane0_mesh, size, height, offset);
        let plane0_vao = unsafe { plane0_mesh.mkvao() };
        let mut plane0_node = SceneNode::from_vao(plane0_vao);
        plane0_node.node_type = SceneNodeType::Planet;
        // Bottom
        let mut plane1_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(std::f32::consts::PI, 0.0, 0.0),
            glm::vec3(0.0, -1.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        mesh::displace_vertices(&mut plane1_mesh, size, height, offset);
        let plane1_vao = unsafe { plane1_mesh.mkvao() };
        let mut plane1_node = SceneNode::from_vao(plane1_vao);
        plane1_node.node_type = SceneNodeType::Planet;
        // Front
        let mut plane2_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
            subdivisions, true,
            Some(color)
        );
        mesh::displace_vertices(&mut plane2_mesh, size, height, offset);
        let plane2_vao = unsafe { plane2_mesh.mkvao() };
        let mut plane2_node = SceneNode::from_vao(plane2_vao);
        plane2_node.node_type = SceneNodeType::Planet;
        // Back
        let mut plane3_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(-std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            glm::vec3(0.0, 0.0, -1.0),
            subdivisions, true,
            Some(color)
        );
        mesh::displace_vertices(&mut plane3_mesh, size, height, offset);
        let plane3_vao = unsafe { plane3_mesh.mkvao() };
        let mut plane3_node = SceneNode::from_vao(plane3_vao);
        plane3_node.node_type = SceneNodeType::Planet;
        // Left
        let mut plane4_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(0.0, 0.0, -std::f32::consts::FRAC_PI_2),
            glm::vec3(1.0, 0.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        mesh::displace_vertices(&mut plane4_mesh, size, height, offset);
        let plane4_vao = unsafe { plane4_mesh.mkvao() };
        let mut plane4_node = SceneNode::from_vao(plane4_vao);
        plane4_node.node_type = SceneNodeType::Planet;
        // Right
        let mut plane5_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),
            glm::vec3(-1.0, 0.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        mesh::displace_vertices(&mut plane5_mesh, size, height, offset);
        let plane5_vao = unsafe { plane5_mesh.mkvao() };
        let mut plane5_node = SceneNode::from_vao(plane5_vao);
        plane5_node.node_type = SceneNodeType::Planet;
                
        cubesphere.add_child(&plane0_node);
        cubesphere.add_child(&plane1_node);
        cubesphere.add_child(&plane2_node);
        cubesphere.add_child(&plane3_node);
        cubesphere.add_child(&plane4_node);
        cubesphere.add_child(&plane5_node);
        // Cubesphere creation done
        

        // Create cubesphere
        let mut cs_ocean = SceneNode::with_type(SceneNodeType::Empty);
        cs_ocean.scale *= 10.001;
        let size = 10.0;
        let height = 0.05;
        let offset = 0.0;
        let subdivisions = 16;
        let color = glm::vec4(0.1, 0.3, 0.8, 0.8);

        // Top
        let mut plane0_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(0.0, 0.0, 0.0),
            glm::vec3(0.0, 1.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        // mesh::displace_vertices(&mut plane0_mesh, size, height, offset);
        let plane0_vao = unsafe { plane0_mesh.mkvao() };
        let mut plane0_node = SceneNode::from_vao(plane0_vao);
        plane0_node.node_type = SceneNodeType::Ocean;
        // Bottom
        let mut plane1_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(std::f32::consts::PI, 0.0, 0.0),
            glm::vec3(0.0, -1.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        // mesh::displace_vertices(&mut plane1_mesh, size, height, offset);
        let plane1_vao = unsafe { plane1_mesh.mkvao() };
        let mut plane1_node = SceneNode::from_vao(plane1_vao);
        plane1_node.node_type = SceneNodeType::Ocean;
        // Front
        let mut plane2_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            glm::vec3(0.0, 0.0, 1.0),
            subdivisions, true,
            Some(color)
        );
        // mesh::displace_vertices(&mut plane2_mesh, size, height, offset);
        let plane2_vao = unsafe { plane2_mesh.mkvao() };
        let mut plane2_node = SceneNode::from_vao(plane2_vao);
        plane2_node.node_type = SceneNodeType::Ocean;
        // Back
        let mut plane3_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(-std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            glm::vec3(0.0, 0.0, -1.0),
            subdivisions, true,
            Some(color)
        );
        // mesh::displace_vertices(&mut plane3_mesh, size, height, offset);
        let plane3_vao = unsafe { plane3_mesh.mkvao() };
        let mut plane3_node = SceneNode::from_vao(plane3_vao);
        plane3_node.node_type = SceneNodeType::Ocean;
        // Left
        let mut plane4_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(0.0, 0.0, -std::f32::consts::FRAC_PI_2),
            glm::vec3(1.0, 0.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        // mesh::displace_vertices(&mut plane4_mesh, size, height, offset);
        let plane4_vao = unsafe { plane4_mesh.mkvao() };
        let mut plane4_node = SceneNode::from_vao(plane4_vao);
        plane4_node.node_type = SceneNodeType::Ocean;
        // Right
        let mut plane5_mesh = mesh::Mesh::cs_plane(
            glm::vec3(1.0, 1.0, 1.0), 
            glm::vec3(0.0, 0.0, std::f32::consts::FRAC_PI_2),
            glm::vec3(-1.0, 0.0, 0.0),
            subdivisions, true,
            Some(color)
        );
        // mesh::displace_vertices(&mut plane5_mesh, size, height, offset);
        let plane5_vao = unsafe { plane5_mesh.mkvao() };
        let mut plane5_node = SceneNode::from_vao(plane5_vao);
        plane5_node.node_type = SceneNodeType::Ocean;


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


        cs_ocean.add_child(&plane0_node);
        cs_ocean.add_child(&plane1_node);
        cs_ocean.add_child(&plane2_node);
        cs_ocean.add_child(&plane3_node);
        cs_ocean.add_child(&plane4_node);
        cs_ocean.add_child(&plane5_node);
        // Cubesphere creation done

        // let part_plane = mesh::Mesh::cs_part_plane(glm::vec3(-1.0, 0.0, 1.0), glm::vec3(1.0, 0.0, -1.0), 64, true);
        // let pplane_vao = unsafe { mkvao(&part_plane) };
        // let mut pplane_node = SceneNode::from_vao(pplane_vao.vao, pplane_vao.n);

        let text_title = mesh::Mesh::text_buffer("THE GAME", 49.0 / 29.0, 1.0);
        let mut text_title_node = SceneNode::from_vao(unsafe { text_title.mkvao() });
        text_title_node.node_type = SceneNodeType::Geometry2d;
        text_title_node.texture_id = Some(charmap_id);
        text_title_node.position = glm::vec3(-0.5, 0.7, 0.0);
        text_title_node.scale = glm::vec3(1.0, 1.0, 1.0);

        let mut text_pos_mesh = mesh::Mesh::text_buffer(".", 49.0 / 29.0, 1.0);
        let mut text_pos_node = SceneNode::from_vao(unsafe { text_pos_mesh.mkvao() });
        text_pos_node.node_type = SceneNodeType::Geometry2d;
        text_pos_node.texture_id = Some(charmap_id);
        text_pos_node.position = glm::vec3(-1.0, -1.0, 0.0);
        text_pos_node.scale = glm::vec3(1.0, 1.0, 1.0) * 0.9;



        //---------------------------------------------------------------------/
        // Make Scene graph
        //---------------------------------------------------------------------/
        let mut scene_root = SceneNode::new();
        // scene_root.add_child(&cube_node);
        scene_root.add_child(&skybox_node);
        scene_root.add_child(&cubesphere);
        scene_root.add_child(&cs_ocean);
        // scene_root.add_child(&plane0_node);

        unsafe { scene_root.update_node_transformations(&glm::identity()); }

        scene_root.print();

        let mut gui_root = SceneNode::new();
        gui_root.add_child(&text_title_node);
        gui_root.add_child(&text_pos_node);

        //---------------------------------------------------------------------/
        // Uniform values
        //---------------------------------------------------------------------/
        let mut timestamp = std::time::SystemTime::now();


        //---------------------------------------------------------------------/        
        // Timing
        //---------------------------------------------------------------------/
        let first_frame_time = std::time::Instant::now();
        let mut last_frame_time = first_frame_time;

        let mut key_debounce: HashMap<VirtualKeyCode, u32> = HashMap::new();

        // The main rendering loop
        eprintln!("Setup done in {:?}. Starting rendering loop.", setup_timer.elapsed().unwrap());
        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(last_frame_time).as_secs_f32();
            last_frame_time = now;

            key_debounce.iter_mut().for_each(|(_, v)| if *v > 0 { *v -= 1; });

            //-----------------------------------------------------------------/
            // Handle keyboard input
            //-----------------------------------------------------------------/
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    use player::PlayerState::*;
                    let up = match player_state {
                        Anchored(a) => glm::normalize(&(position - a)),
                        FreeFloat => glm::cross(&direction, &right),
                    };
                    let flat_direction =  glm::normalize(&glm::vec3(direction.x, 0.0, direction.z));
                    // Set movement relative to helicopter rotation
                    // let heli_direction = util::vec_direction(heli_body_nodes[n_helis].rotation.y, 0.0);
                    // let flat_direction = -heli_direction; //glm::normalize(&glm::vec3(heli_direction.x, 0.0, heli_direction.z));
                    // right = glm::cross(&flat_direction, &glm::vec3(0.0, 1.0, 0.0));
                    // TODO: Handle inputs in a state machine
                    match key {
                        /* Move left/right */
                        VirtualKeyCode::A => {
                            // tilt_dir.1 = 1;
                            // heli_body_nodes[n_helis].position -= right * delta_time * movement_speed;
                            // position -= right * delta_time * movement_speed;
                            position -= match player_state {
                                FreeFloat => right * delta_time * movement_speed,
                                Anchored(a) => right * delta_time * movement_speed,
                            }
                        },
                        VirtualKeyCode::D => {
                            // heli_body_nodes[n_helis].position += right * delta_time * movement_speed;
                            // position += right * delta_time * movement_speed;
                            position += match player_state {
                                FreeFloat => right * delta_time * movement_speed,
                                Anchored(a) => right * delta_time * movement_speed,
                            }
                        },
                        /* Move forward (inward)/backward, in camera direction */
                        VirtualKeyCode::W => {
                            // heli_body_nodes[n_helis].position += flat_direction * delta_time * movement_speed;
                            // position += direction * delta_time * movement_speed;
                            position += match player_state {
                                FreeFloat => direction * delta_time * movement_speed,
                                Anchored(a) => direction * delta_time * movement_speed,
                            }
                        },
                        VirtualKeyCode::S => {
                            // heli_body_nodes[n_helis].position -= flat_direction * delta_time * movement_speed;
                            // position -= direction * delta_time * movement_speed;
                            position -= match player_state {
                                FreeFloat => direction * delta_time * movement_speed,
                                Anchored(a) => direction * delta_time * movement_speed,
                            }
                        },
                        /* Move up/down */
                        VirtualKeyCode::Space => {
                            position += up * delta_time * movement_speed
                        },
                        VirtualKeyCode::LShift => {
                            position -= up * delta_time * movement_speed
                        },
                        VirtualKeyCode::M => {
                            let v = key_debounce.entry(VirtualKeyCode::M).or_insert(0);
                            if *v == 0 {
                                conf.polymode = (conf.polymode + 1) % 3;
                                *v = 10;
                            }
                        },
                        VirtualKeyCode::F => {
                            let v = key_debounce.entry(VirtualKeyCode::F).or_insert(0);
                            if *v == 0 {
                                use player::PlayerState::*;
                                player_state = match player_state {
                                    FreeFloat => {
                                        let a = glm::vec3(0.0, 0.0, 0.0);
                                        let up = glm::normalize(&(position - a));
                                        right = glm::cross(&direction, &up);
                                        Anchored(a)  // Later: anchor to closest planet
                                    },
                                    Anchored(_) => FreeFloat
                                };
                                *v = 10;
                            }
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
                right = util::vec_right(h_angle);
                up = glm::cross(&right, &direction);

                *delta = (0.0, 0.0);
            }

            skybox_node.position = position;
            // Log position
            text_pos_mesh = mesh::Mesh::text_buffer(
                &format!("global position: {:.3},{:.3},{:.3}", position.x, position.y, position.z),
                49.0 / 29.0, 1.0
            );
            text_pos_node.update_buffers(&text_pos_mesh);

            let wsize = context.window().inner_size();

            let mut perspective_mat: glm::Mat4 = glm::perspective(
                //*aspect.read().unwrap(),         // aspect
                wsize.width as f32 / wsize.height as f32,
                conf.fov,       // field of view
                conf.clip_near, // near
                conf.clip_far   // far
            );

            //-------------------------------------------------------------/
            // Draw section
            //-------------------------------------------------------------/
            unsafe {
                // Global uniforms
                let u_time = sh.get_uniform_location("u_time");
                gl::Uniform1f(u_time, timestamp.elapsed().unwrap().as_secs_f32());

                // First person view
                let cam = glm::look_at(&position, &(position+direction), &up);
                let perspective_view = perspective_mat * cam;
                // let perspective_view = perspective_mat * glm::look_at(&position, &heli_body_nodes[n_helis].position, &up);

                // Clear background
                gl::ClearColor(conf.bg_color[0], conf.bg_color[1], conf.bg_color[2], conf.bg_color[3]);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                /* Draw scene graph */
                gl::PolygonMode(gl::FRONT_AND_BACK, POLYMODES[conf.polymode]);
                scene_root.update_node_transformations(&glm::identity());
                scene_root.draw_scene(&perspective_view, &sh);
                gui_root.update_node_transformations(&glm::identity());
                gui_root.draw_scene(&perspective_view, &sh);
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
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                println!("Resized window to {} x {}", size.width, size.height);
            }
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
