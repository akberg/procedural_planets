#[allow(unused_imports)]
use std::thread;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

use nalgebra_glm as glm;
use glutin::event::{
    VirtualKeyCode
};

use crate::*;
use crate::procedural_planet as planet;
use crate::texture::load_texture;
use crate::scene_graph::{self, SceneNode, SceneNodeType};

const POLYMODES: [u32;3] = [gl::FILL, gl::POINT, gl::LINE];

/// Initializes game ad runs main game loop
pub fn render(
    mouse_delta: Arc<Mutex<(f32, f32)>>, 
    pressed_keys: Arc<Mutex<Vec<VirtualKeyCode>>>,
    context: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>
) {

    let setup_timer = std::time::SystemTime::now();
    

    //-------------------------------------------------------------------------/
    // Read config
    //-------------------------------------------------------------------------/
    let mut conf = util::Config::load();

    let mut player = player::Player { ..Default::default() };


    //-------------------------------------------------------------------------/
    // Shaders and locating uniforms
    //-------------------------------------------------------------------------/
    let timer = std::time::SystemTime::now();
    print!("Compiling shader . . . ");
    let sh = unsafe {
        let sh = shader::ShaderBuilder::new()
            .attach_file("./resources/shaders/scene.vert", None)
            .attach_file(
                "./resources/shaders/scene.frag", 
                Some(vec![
                    "./resources/shaders/noise.glsl",
                ])
            )
            .link();

        sh.activate();
        sh
    };
    println!("took {:?}", timer.elapsed());

    //-------------------------------------------------------------------------/
    // Load charmap texture
    //-------------------------------------------------------------------------/
    let charmap_id = load_texture("resources/textures/charmap.png");


    //-------------------------------------------------------------------------/
    // Camera setup (available for keypress handler)
    //-------------------------------------------------------------------------/
    player.position = glm::vec3(
        conf.init_position[0],
        conf.init_position[1],
        conf.init_position[2],
    );
    let mut h_angle = conf.init_h_angle;
    let mut v_angle = conf.init_v_angle;
    player.direction = util::vec_direction(h_angle, v_angle);
    let mut up = glm::vec3(0.0, 1.0, 0.0);
    player.right = util::vec_right(h_angle);
    

    //-------------------------------------------------------------------------/
    // GUI meshes
    //-------------------------------------------------------------------------/
    let text_title = mesh::Mesh::text_buffer("PROCEDURAL PLANETS", 49.0 / 29.0, 1.0);
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

    #[allow(unused_assignments)]
    let mut text_pstate_mesh = mesh::Mesh::text_buffer(".", 49.0 / 29.0, 1.0);
    let mut text_pstate_node = SceneNode::from_vao(unsafe { text_pos_mesh.mkvao() });
    text_pstate_node.node_type = SceneNodeType::Geometry2d;
    text_pstate_node.texture_id = Some(charmap_id);
    text_pstate_node.position = glm::vec3(-1.0, -0.95, 0.0);
    text_pstate_node.scale = glm::vec3(1.0, 1.0, 1.0) * 0.9;

    #[allow(unused_assignments)]
    let mut text_mspeed_mesh = mesh::Mesh::text_buffer(".", 49.0 / 29.0, 1.0);
    let mut text_mspeed_node = SceneNode::from_vao(unsafe { text_mspeed_mesh.mkvao() });
    text_mspeed_node.node_type = SceneNodeType::Geometry2d;
    text_mspeed_node.texture_id = Some(charmap_id);
    text_mspeed_node.position = glm::vec3(-1.0, -0.9, 0.0);
    text_mspeed_node.scale = glm::vec3(1.0, 1.0, 1.0) * 0.9;

    #[allow(unused_assignments)]
    let mut text_closest_mesh = mesh::Mesh::text_buffer(".", 49.0 / 29.0, 1.0);
    let mut text_closest_node = SceneNode::from_vao(unsafe { text_closest_mesh.mkvao() });
    text_closest_node.node_type = SceneNodeType::Geometry2d;
    text_closest_node.texture_id = Some(charmap_id);
    text_closest_node.position = glm::vec3(-1.0, -0.85, 0.0);
    text_closest_node.scale = glm::vec3(1.0, 1.0, 1.0) * 0.9;
    

    //-------------------------------------------------------------------------/
    // Vertex Array Objects, create vertices or load models
    //-------------------------------------------------------------------------/
    
    // Skybox, inverted cube that stays centered around the player
    let skybox_mesh = mesh::Mesh::cube(
        glm::vec3(conf.clip_far-0.1, conf.clip_far-0.1, conf.clip_far-0.1), // Defines visible distance of other objects
        glm::vec2(1.0, 1.0), true, true, 
        glm::vec3(1.0, 1.0, 1.0),
        glm::vec4(0.05, 0.01, 0.06, 0.2),
    );
    let mut skybox_node = SceneNode::from_vao(unsafe { skybox_mesh.mkvao() });
    skybox_node.node_type = SceneNodeType::Skybox;
    
    
    //-------------------------------------------------------------------------/
    // Generate planets
    //-------------------------------------------------------------------------/
    // Small earth-like planet
    let mut planet0 = planet::Planet::with_seed(4393);
    //planet0.radius = 5.0;
    planet0.max_height = 0.05;
    planet0.noise_size = 25.0;
    planet0.ocean_dark_color = glm::vec3(0.01, 0.2, 0.3);
    planet0.ocean_light_color = glm::vec3(0.04, 0.3, 0.43);
    planet0.color_scheme = [
        glm::vec3(0.4, 0.4, 0.3),
        glm::vec3(0.7, 0.55, 0.0),
        glm::vec3(0.2, 0.6, 0.4),
        glm::vec3(0.5, 0.4, 0.4),
        glm::vec3(0.91, 1.0, 1.0),
    ];
    let mut planet0_node = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
    planet0_node.scale *= 25.0;
    planet0_node.position = glm::vec3(250.0, 0.0, 0.0);
    planet0.node = planet0_node.node_id;
    unsafe { planet0.lod(&mut planet0_node, player.position) };


    // Other planet
    let mut planet1 = planet::Planet::with_seed(4393);
    //planet1.radius = 5.0; // must be 1/2 of scale
    planet1.max_height = 0.3;
    planet1.noise_size = 4.0;
    planet1.ocean_dark_color = glm::vec3(0.01, 0.2, 0.3);
    planet1.ocean_light_color = glm::vec3(0.04, 0.3, 0.43);
    planet1.color_scheme = [
        glm::vec3(0.6118, 0.3137, 0.1961),
        glm::vec3(0.6118, 0.3137, 0.1961),
        glm::vec3(0.1686, 0.3922, 0.3176),
        glm::vec3(0.4588, 0.4588, 0.4588),
        glm::vec3(0.91, 1.0, 1.0),
    ];
    let mut planet1_node = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
    planet1_node.scale *= 60.0;
    planet1_node.position = glm::vec3(-250.0, 0.0, 0.0);
    planet1.node = planet1_node.node_id;
    unsafe { planet1.lod(&mut planet1_node, player.position) };


    // Small mars-like planet
    let mut planet2 = planet::Planet::with_seed(4393);
    //planet2.radius = 5.0;
    planet2.max_height = 0.05;
    planet2.noise_size = 10.0;
    planet2.has_ocean = false;
    planet2.color_scheme = [
        glm::vec3(0.6118, 0.1255, 0.1255),
        glm::vec3(0.7, 0.55, 0.0),
        glm::vec3(0.7804, 0.2275, 0.0118),
        glm::vec3(0.8275, 0.302, 0.0),
        glm::vec3(0.91, 1.0, 1.0),
    ];
    let mut planet2_node = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
    planet2_node.scale *= 10.0;
    planet2_node.position = glm::vec3(00.0, 0.0, 150.0);
    planet2.node = planet2_node.node_id;
    unsafe { planet2.lod(&mut planet2_node, player.position) };


    // planet
    let mut planet3 = planet::Planet::with_seed(4393);
    //planet3.radius = 5.0;
    planet3.max_height = 0.045;   // relative to scale
    planet3.noise_size = 2.0;
    planet3.has_ocean = false;
    planet3.color_scheme = [
        glm::vec3(0.3, 0.2, 0.4),
        glm::vec3(0.7, 0.1, 0.1),
        glm::vec3(0.6, 0.4, 0.3),
        glm::vec3(0.4, 0.8, 0.3),
        glm::vec3(0.9, 1.0, 1.0),
    ];
    planet3.color_thresholds = [
        -0.0005, 0.001, 0.014, 0.024
    ];
    let mut planet3_node = scene_graph::SceneNode::with_type(SceneNodeType::Empty);
    planet3_node.scale *= 160.0;
    planet3_node.position = glm::vec3(00.0, 0.0, -150.0);
    planet3.node = planet3_node.node_id;
    unsafe { planet3.lod(&mut planet3_node, player.position) };



    // TODO: Automatically add to array
    let mut planets = vec![planet0, planet1, planet2, planet3];
    let mut planet_nodes = vec![planet0_node, planet1_node, planet2_node, planet3_node];
    let mut closest_planet_id  = 0;


    //-------------------------------------------------------------------------/
    // Make Scene graph
    //-------------------------------------------------------------------------/
    let mut scene_root = SceneNode::new();
    scene_root.add_child(&skybox_node);
    scene_root.add_child(&planet_nodes[0]);
    scene_root.add_child(&planet_nodes[1]);
    scene_root.add_child(&planet_nodes[2]);
    scene_root.add_child(&planet_nodes[3]);

    unsafe { scene_root.update_node_transformations(&glm::identity()); }



    //-------------------------------------------------------------------------/        
    // Build GUI
    //-------------------------------------------------------------------------/        
    let mut gui_root = SceneNode::new();
    //gui_root.add_child(&text_title_node);
    gui_root.add_child(&text_pos_node);
    gui_root.add_child(&text_pstate_node);
    gui_root.add_child(&text_mspeed_node);
    gui_root.add_child(&text_closest_node);


    //-------------------------------------------------------------------------/        
    // Timing
    //-------------------------------------------------------------------------/
    let first_frame_time = std::time::Instant::now();
    let mut last_frame_time = first_frame_time;
    
    let mut key_debounce: HashMap<VirtualKeyCode, u32> = HashMap::new();
    

    //-------------------------------------------------------------------------/
    //-------------------------------------------------------------------------/
    //
    // The main rendering loop
    //
    //-------------------------------------------------------------------------/
    //-------------------------------------------------------------------------/
    eprintln!("Setup done in {:?}. Starting rendering loop.", 
        setup_timer.elapsed().unwrap()
    );
    loop {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(first_frame_time).as_secs_f32();
        let delta_time = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;

        key_debounce.iter_mut().for_each(|(_, v)| if *v > 0 { *v -= 1; });

        //---------------------------------------------------------------------/
        // Handle keyboard and mouse input
        //---------------------------------------------------------------------/
        if let Ok(keys) = pressed_keys.lock() {
            keyboard_input(
                keys, 
                &mut key_debounce, 
                &mut player, 
                &planets[closest_planet_id],
                &mut conf, 
                delta_time,
            );
        }
        skybox_node.position = player.position;

        // Handle mouse movement. delta contains the x and y movement of 
        // the mouse since last frame in pixels
        if let Ok(mut delta) = mouse_delta.lock() {
            mouse_input(
                &delta,
                &mut player,
                &planets[closest_planet_id],
                &mut conf,
                delta_time,
            );
            // /* Look left/right (horizontal angle), rotate around y axis */
            // h_angle -= (*delta).0 * delta_time * conf.mouse_speed;
            // /* Look up/down (vertical angle), rotate around x axis */
            // v_angle -= (*delta).1 * delta_time * conf.mouse_speed;
            // player.direction = util::vec_direction(h_angle, v_angle);
            // player.right = util::vec_right(h_angle);
            // //up = glm::cross(&player.right, &player.direction);

            *delta = (0.0, 0.0);
        }


        //---------------------------------------------------------------------/
        // Update GUI
        //---------------------------------------------------------------------/
        // Log position
        let s = format!("global position: {:.3},{:.3},{:.3}", 
            player.position.x, player.position.y, player.position.z);
        text_pos_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_pos_node.update_buffers(&text_pos_mesh);
        // Log movement speed
        let s = format!("Speed: {:.3}", conf.movement_speed);
        text_mspeed_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_mspeed_node.update_buffers(&text_mspeed_mesh);
        // Log closest planet
        let s = format!("Closest planet: {:}", closest_planet_id);
        text_closest_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_closest_node.update_buffers(&text_closest_mesh);
        // Display player state
        let s = match player.state {
            player::PlayerState::FreeFloat => String::from("Free floating"),
            player::PlayerState::Anchored(a) => String::from(
                &format!("Anchored to: {:.3},{:.3},{:.3}", a.x, a.y, a.z)
            ),
        };
        text_pstate_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_pstate_node.update_buffers(&text_pstate_mesh);

        //---------------------------------------------------------------------/
        // Update perspective
        //---------------------------------------------------------------------/
        let wsize = context.window().inner_size();
        let perspective_mat: glm::Mat4 = glm::perspective(
            //*aspect.read().unwrap(),         // aspect
            wsize.width as f32 / wsize.height as f32,
            conf.fov,       // field of view
            conf.clip_near, // near
            conf.clip_far   // far
        );
        
        //---------------------------------------------------------------------/
        // First person view
        //---------------------------------------------------------------------/
        // TODO: Something's weird with camera and direction
        let up = player.up();
        let cam = glm::look_at(&player.position, &(player.position+player.direction), &up);
        let perspective_view = perspective_mat * cam;
        // let perspective_view = perspective_mat * glm::look_at(&position, &heli_body_nodes[n_helis].position, &up);


        //---------------------------------------------------------------------/
        // Draw section
        //---------------------------------------------------------------------/
        unsafe {
            //-----------------------------------------------------------------/
            // Global uniforms
            //-----------------------------------------------------------------/
            let u_time = sh.get_uniform_location("u_time");
            gl::Uniform1f(u_time, elapsed);
            
            
            //-----------------------------------------------------------------/
            // Clear background, set polygon mode
            //-----------------------------------------------------------------/
            gl::ClearColor(
                conf.bg_color[0], conf.bg_color[1], 
                conf.bg_color[2], conf.bg_color[3]
            );
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::PolygonMode(gl::FRONT_AND_BACK, POLYMODES[conf.polymode]);
            
            //-----------------------------------------------------------------/
            // Planet transforms and update uniforms
            // Compute closest planet
            //-----------------------------------------------------------------/
            scene_root.update_node_transformations(&glm::identity());
            
            let mut closest_planet = std::f32::MAX;
            let mut closest_planet_node_id = 0;
            for (node, mut planet) in planet_nodes.iter().zip(&mut planets) {
                planet.position = node.position;
                planet.rotation = node.rotation;
                planet.radius = node.scale.x / 2.0;
                planet.update_uniforms(&sh);
                let dist = glm::length(&(planet.position - player.position)) - planet.radius;
                if dist < closest_planet {
                    closest_planet = dist;
                    closest_planet_id = planet.planet_id;
                    closest_planet_node_id = planet.node;
                }
            }
            for node in &mut planet_nodes {
                if node.node_id == closest_planet_node_id {
                    node.node_type = SceneNodeType::Empty;
                }
                else {
                    node.node_type = SceneNodeType::PlanetSkip;
                }
            }
            gl::Uniform1ui(
                sh.get_uniform_location("u_planets_len"),
                planets.len() as u32
            );
            gl::Uniform1ui(
                sh.get_uniform_location("u_closest_planet"),
                closest_planet_id as u32
            );
            gl::Uniform3fv(
                sh.get_uniform_location("u_player_position"),
                1,
                player.position.as_ptr()
            );

            scene_root.draw_scene(&perspective_view, &sh);

            //-----------------------------------------------------------------/
            // Draw GUI if enabled
            //-----------------------------------------------------------------/
            if conf.draw_gui {
                gui_root.update_node_transformations(&glm::identity());
                gui_root.draw_scene(&perspective_view, &sh);
            }
        }

        context.swap_buffers().unwrap();
    }
}


fn mouse_input(
    delta: &std::sync::MutexGuard<'_, (f32, f32)>,
    player: &mut player::Player,
    closest_planet: &planet::Planet,
    conf: &mut util::Config,
    delta_time: f32
) {
    /* Look left/right (horizontal angle), rotate around y axis */
    let delta_h = (*delta).0 * delta_time * conf.mouse_speed;
    /* Look up/down (vertical angle), rotate around x axis */
    let delta_v = (*delta).1 * delta_time * conf.mouse_speed;
    let up = player.up();
    match player.state {
        player::PlayerState::Anchored(a) => {
            // vertical angle rotates around right -> modifies only direction
            player.direction = glm::rotate_vec3(
                &player.direction, -delta_v, &player.right
            );
            // horizontal angle rotates around up -> modifies right and direction
            player.direction = glm::rotate_vec3(
                &player.direction, -delta_h, &up
            );
        },
        player::PlayerState::FreeFloat => {
            // vertical angle rotates around right -> modifies up and direction
            player.direction = glm::rotate_vec3(
                &player.direction, -delta_v, &player.right
            );
            // horizontal angle rotates around up -> modifies right and direction
            player.direction = glm::rotate_vec3(
                &player.direction, -delta_h, &glm::normalize(&up)
            );

        }
    }
    // player.direction = util::vec_direction(h_angle, v_angle);
    // player.right = util::vec_right(h_angle);
    //up = glm::cross(&player.right, &player.direction);
}


/// Handle keyboard input
fn keyboard_input(
    keys: std::sync::MutexGuard<'_, std::vec::Vec<glutin::event::VirtualKeyCode>>, 
    key_debounce: &mut std::collections::HashMap<glutin::event::VirtualKeyCode, u32>,
    player: &mut player::Player,
    closest_planet: &planet::Planet,
    conf: &mut util::Config,
    delta_time: f32
) {
    for key in keys.iter() {
        use player::PlayerState::*;
        let up = player.up();
        let _flat_direction = glm::cross(&up, &player.right);

        // TODO: Handle inputs in a state machine
        let mut position = player.position;
        match key {
            /* Move left/right */
            VirtualKeyCode::A => {
                // tilt_dir.1 = 1;
                // heli_body_nodes[n_helis].position -= right * delta_time * movement_speed;
                // position -= right * delta_time * movement_speed;
                position -= match player.state {
                    FreeFloat => player.right * delta_time * conf.movement_speed,
                    Anchored(_a) => player.right * delta_time * conf.movement_speed,
                }
            },
            VirtualKeyCode::D => {
                // heli_body_nodes[n_helis].position += right * delta_time * movement_speed;
                // position += right * delta_time * movement_speed;
                position += match player.state {
                    FreeFloat => player.right * delta_time * conf.movement_speed,
                    Anchored(_a) => player.right * delta_time * conf.movement_speed,
                }
            },
            /* Move forward (inward)/backward, in camera direction */
            VirtualKeyCode::W => {
                position += match player.state {
                    FreeFloat => player.direction * delta_time * conf.movement_speed,
                    Anchored(_a) => _flat_direction * delta_time * conf.movement_speed,
                }
            },
            VirtualKeyCode::S => {
                position -= match player.state {
                    FreeFloat => player.direction * delta_time * conf.movement_speed,
                    Anchored(_a) => _flat_direction * delta_time * conf.movement_speed,
                }
            },
            /* Move up/down */
            VirtualKeyCode::Space => {
                position += up * delta_time * conf.movement_speed
            },
            VirtualKeyCode::LShift => {
                position -= up * delta_time * conf.movement_speed
            },
            VirtualKeyCode::M => {
                let v = key_debounce.entry(VirtualKeyCode::M).or_insert(0);
                if *v == 0 {
                    conf.polymode = (conf.polymode + 1) % 3;
                    *v = 10;
                }
            },
            VirtualKeyCode::Up => {
                let v = key_debounce.entry(VirtualKeyCode::Up).or_insert(0);
                if *v == 0 {
                    conf.movement_speed = conf.movement_speed * 1.1;
                    *v = 10;
                }
            },
            VirtualKeyCode::Down => {
                let v = key_debounce.entry(VirtualKeyCode::Down).or_insert(0);
                if *v == 0 {
                    conf.movement_speed = conf.movement_speed / 1.1;
                    *v = 10;
                }
            },
            VirtualKeyCode::I => {
                let v = key_debounce.entry(VirtualKeyCode::I).or_insert(0);
                if *v == 0 {
                    conf.draw_gui = !conf.draw_gui;
                    *v = 10;
                }
            },
            VirtualKeyCode::F => {
                let v = key_debounce.entry(VirtualKeyCode::F).or_insert(0);
                if *v == 0 {
                    // TODO: Not entirely correct
                    use player::PlayerState::*;
                    player.state = match player.state {
                        FreeFloat => {
                            let a = closest_planet.position;
                            let up = glm::normalize(&(player.position - a));
                            player.right = glm::cross(&player.direction, &up);
                            Anchored(a)  // Later: anchor to closest planet
                        },
                        Anchored(_) => FreeFloat
                    };
                    *v = 10;
                }
            }
            _ => { }
        }

        let height = closest_planet.get_height(&position);
        let go_to = glm::length(&(position - closest_planet.position));
        if go_to > height {
            player.position = position;
        }
        else {
            let dir = glm::rotate_vec3(
                &(position - player.position), // requested movement direction
                ((height - go_to) / glm::length(&player.direction)).atan(), 
                &player.right
            );
            player.position += dir;
        }
    }
}