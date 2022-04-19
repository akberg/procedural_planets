#[allow(unused_imports)]
use std::thread;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;

use nalgebra_glm as glm;
use glutin::event::{
    VirtualKeyCode
};

use crate::*;
use crate::player::PlayerState;
use crate::procedural_planet as planet;
use crate::texture::load_texture;
use crate::scene_graph::{SceneNode, SceneNodeType};
use crate::globals::*;

const POLYMODES: [u32;3] = [gl::FILL, gl::POINT, gl::LINE];


/// Initializes game ad runs main game loop
pub fn game(
    mouse_delta: Arc<Mutex<(f32, f32)>>, 
    pressed_keys: Arc<Mutex<Vec<VirtualKeyCode>>>,
    context: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>
) {

    let setup_timer = std::time::SystemTime::now();
    

    //-------------------------------------------------------------------------/
    // Read config
    //-------------------------------------------------------------------------/
    let mut conf = util::Config::load();

    let mut player = player::Player {
        height: conf.player_height,
        ..Default::default() 
    };


    //-------------------------------------------------------------------------/
    // Shaders and locating uniforms
    //-------------------------------------------------------------------------/
    let timer = std::time::SystemTime::now();
    eprint!("Compiling shader . . . ");
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
    eprintln!("took {:?}", timer.elapsed());

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
    let h_angle = conf.init_h_angle;
    let v_angle = conf.init_v_angle;
    player.direction = util::vec_direction(h_angle, v_angle);
    player.right = util::vec_right(h_angle);
    

    //-------------------------------------------------------------------------/
    // GUI meshes
    //-------------------------------------------------------------------------/
    let text_scale = 0.6;
    let text_title = mesh::Mesh::text_buffer("PROCEDURAL PLANETS", 49.0 / 29.0, 1.0);
    let mut text_title_node = SceneNode::from_vao(unsafe { text_title.mkvao() });
    text_title_node.node_type = SceneNodeType::Geometry2d;
    text_title_node.texture_id = Some(charmap_id);
    text_title_node.position = glm::vec3(-0.5, 0.7, 0.0);
    text_title_node.scale = glm::vec3(1.0, 1.0, 1.0);

    let mut text_pos_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_pos_node = SceneNode::from_vao(unsafe { text_pos_mesh.mkvao() });
    text_pos_node.node_type = SceneNodeType::Geometry2d;
    text_pos_node.texture_id = Some(charmap_id);
    text_pos_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 0.0, 0.0);
    text_pos_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    #[allow(unused_assignments)]
    let mut text_pstate_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_pstate_node = SceneNode::from_vao(unsafe { text_pos_mesh.mkvao() });
    text_pstate_node.node_type = SceneNodeType::Geometry2d;
    text_pstate_node.texture_id = Some(charmap_id);
    text_pstate_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 1.0, 0.0);
    text_pstate_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    #[allow(unused_assignments)]
    let mut text_mspeed_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_mspeed_node = SceneNode::from_vao(unsafe { text_mspeed_mesh.mkvao() });
    text_mspeed_node.node_type = SceneNodeType::Geometry2d;
    text_mspeed_node.texture_id = Some(charmap_id);
    text_mspeed_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 2.0, 0.0);
    text_mspeed_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    #[allow(unused_assignments)]
    let mut text_closest_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_closest_node = SceneNode::from_vao(unsafe { text_closest_mesh.mkvao() });
    text_closest_node.node_type = SceneNodeType::Geometry2d;
    text_closest_node.texture_id = Some(charmap_id);
    text_closest_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 3.0, 0.0);
    text_closest_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    #[allow(unused_assignments)]
    let mut text_height_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_height_node = SceneNode::from_vao(unsafe { text_height_mesh.mkvao() });
    text_height_node.node_type = SceneNodeType::Geometry2d;
    text_height_node.texture_id = Some(charmap_id);
    text_height_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 4.0, 0.0);
    text_height_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    #[allow(unused_assignments)]
    let mut text_mouse_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_mouse_node = SceneNode::from_vao(unsafe { text_mouse_mesh.mkvao() });
    text_mouse_node.node_type = SceneNodeType::Geometry2d;
    text_mouse_node.texture_id = Some(charmap_id);
    text_mouse_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 5.0, 0.0);
    text_mouse_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    #[allow(unused_assignments)]
    let mut text_gfxmem_mesh = mesh::Mesh::text_buffer("N/A", 49.0 / 29.0, 1.0);
    let mut text_gfxmem_node = SceneNode::from_vao(unsafe { text_gfxmem_mesh.mkvao() });
    text_gfxmem_node.node_type = SceneNodeType::Geometry2d;
    text_gfxmem_node.texture_id = Some(charmap_id);
    text_gfxmem_node.position = glm::vec3(-1.0, -1.0 + text_scale * 0.05 * 6.0, 0.0);
    text_gfxmem_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;

    let controls_text = [
        "WSAD/SHIFT/SPACE : movement",
        "UP/DOWN : increase and decrease movement speed",
        "F : cycle player state (free/anchored/landed)",
        "I : toggle text interface",
        "M : cycle polygon modes",
    ].iter().enumerate().map(|(i, s)| {
        let text_mesh = mesh::Mesh::text_buffer(s, 49.0 / 29.0, 1.0 * s.len() as f32 / 28.0);
        let mut text_node = SceneNode::from_vao(unsafe { text_mesh.mkvao() });
        text_node.node_type = SceneNodeType::Geometry2d;
        text_node.texture_id = Some(charmap_id);
        text_node.position = glm::vec3(-1.0, 1.0 - text_scale * 0.05 * (i+1) as f32, 0.0);
        text_node.scale = glm::vec3(1.0, 1.0, 1.0) * text_scale;
        text_node
    });
    

    //-------------------------------------------------------------------------/
    // Vertex Array Objects, create vertices or load models
    //-------------------------------------------------------------------------/
    
    // Skybox, inverted cube that stays centered around the player
    let skybox_mesh = mesh::Mesh::cube(
        glm::vec3(1.0, 1.0, 1.0), // Defines visible distance of other objects
        glm::vec2(1.0, 1.0), true, true, 
        glm::vec3(1.0, 1.0, 1.0),
        glm::vec4(0.05, 0.01, 0.06, 0.2),
    );
    let mut skybox_node = SceneNode::from_vao(unsafe { skybox_mesh.mkvao() });
    skybox_node.node_type = SceneNodeType::Skybox;
    
    
    //-------------------------------------------------------------------------/
    // Scene setup, build planets
    //-------------------------------------------------------------------------/
    let (mut planets, mut planet_nodes, lightsources) = scene::create_scene();
    //-------------------------------------------------------------------------/
    // Organize planets and nodes
    //-------------------------------------------------------------------------/

    player.closest_planet_id  = 0;


    //-------------------------------------------------------------------------/
    // Make Scene graph
    //-------------------------------------------------------------------------/
    let mut scene_root = SceneNode::new();
    // scene_root.add_child(&skybox_node);
    for planet in &planet_nodes {
        scene_root.add_child(planet);
    }


    //-------------------------------------------------------------------------/        
    // Build GUI
    //-------------------------------------------------------------------------/        
    let mut gui_root = SceneNode::new();
    //gui_root.add_child(&text_title_node);
    gui_root.add_child(&text_pos_node);
    gui_root.add_child(&text_pstate_node);
    gui_root.add_child(&text_mspeed_node);
    gui_root.add_child(&text_closest_node);
    gui_root.add_child(&text_height_node);
    gui_root.add_child(&text_mouse_node);
    gui_root.add_child(&text_gfxmem_node);
    controls_text.for_each(|nd| gui_root.add_child(&nd));


    //-------------------------------------------------------------------------/        
    // Timing
    //-------------------------------------------------------------------------/
    let first_frame_time = std::time::Instant::now();
    let mut last_frame_time = first_frame_time;
    
    let mut key_debounce: HashMap<VirtualKeyCode, u32> = HashMap::new();
    let mut frame_counter: u64 = 0;
    

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
    let mut scaled = true;
    
    loop {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(first_frame_time).as_secs_f32();
        let delta_time = now.duration_since(last_frame_time).as_secs_f32();
        last_frame_time = now;

        key_debounce.iter_mut().for_each(|(_, v)| if *v > 0 { *v -= 1; });

        let mut computed = vec![];
        if matches!(player.state, PlayerState::Anchored(_) | PlayerState::Landed(_)) {
            if scaled {
                // Scale up
                player.position *= SCALING_FACTOR;
                for i in 0..planets.len() {
                    planet_nodes[i].scale *= SCALING_FACTOR;
                    planets[i].trajectory *= SCALING_FACTOR;
                }
                scaled = false;
            }
            // Reverse origin if player is anchored, origin of scene at center of closest planet
            let mut idx = player.closest_planet_id as usize;
            computed.push(idx);
            planet_nodes[idx].position = glm::zero();
            planets[idx].position = glm::zero();
            planet_nodes[idx].rotation = glm::zero();

            while planets[idx].planet_id != planets[idx].parent_id {
                let idx_next = planets[idx].parent_id;
                computed.push(idx_next);
                let angle = planets[idx].traj_speed * WORLD_SPEED * elapsed + planets[idx].traj_init_angle.x;
                let traj_position = glm::vec3(
                    (angle).sin() * planets[idx].trajectory,
                    planets[idx].traj_init_angle.y, 
                    (angle).cos() * planets[idx].trajectory, 
                );
                // let rotation = planets[idx].rot_speed * WORLD_SPEED * elapsed + planets[idx].rot_init_angle;
                // let rotation_vec = planet_nodes[idx].rotation + rotation * planets[idx].rot_axis;
                planet_nodes[idx_next].position = planets[idx].position - traj_position;
                    // - glm::rotate_vec3(
                    //     &traj_position,
                    //     -rotation,
                    //     &planets[idx].rot_axis
                    // );
                planets[idx_next].position = planet_nodes[idx_next].position;
                // planet_nodes[idx_next].rotation = -rotation_vec;
                idx = idx_next;
            }
        }
        else if !scaled {
            // Scale down
            player.position /= SCALING_FACTOR;
            planet_nodes[0].position /= SCALING_FACTOR;

            for i in 0..planets.len() {
                planet_nodes[i].scale /= SCALING_FACTOR;
                planets[i].trajectory /= SCALING_FACTOR;
            }
            scaled = true;
        }

        // Planet trajectories, skip any that have already been computed
        // Skip 0 because sun is either origin or computed beforehand
        for i in (1..planets.len()).filter(|i| !computed.contains(i)) {
            // Origin of trajectory
            let origin = planet_nodes[planets[i].parent_id].position;
            // Angle of position to inital position
            let angle = planets[i].traj_speed * WORLD_SPEED * elapsed
                + planets[i].traj_init_angle.x;
            // planet_nodes[i].rotation: angle rotation around each axis
            // // Parent's rotation
            // let parent_rotation = planets[planets[i].parent_id].rot_speed * WORLD_SPEED * elapsed
            //     + planets[planets[i].parent_id].rot_init_angle;

            // Trajectory position relative to parent
            let traj_position = glm::vec3(
                (angle).sin() * planets[i].trajectory,
                planets[i].traj_init_angle.y, 
                (angle).cos() * planets[i].trajectory, 
            );

            // Rotate back and add origin to get global position
            // - or keep relative rotation as a feature?
            planet_nodes[i].position = origin + traj_position;
                // + glm::rotate_vec3(
                //     &traj_position, 
                //     parent_rotation, 
                //     &-planets[planets[i].parent_id].rot_axis
                // );
            planets[i].position = planet_nodes[i].position;

            // // Planet rotation
            // let rotation = planets[i].rot_speed * WORLD_SPEED * elapsed
            //     + planets[i].rot_init_angle;
            // let rotation_vec = planet_nodes[planets[i].parent_id].rotation + rotation * planets[i].rot_axis;
            // planet_nodes[i].rotation = rotation_vec;
        }

        //---------------------------------------------------------------------/
        // Handle keyboard and mouse input
        //---------------------------------------------------------------------/
        // Mouse input modifies direction
        //---------------------------------------------------------------------/
        // Handle mouse movement. delta contains the x and y movement of 
        // the mouse since last frame in pixels
        if let Ok(mut delta) = mouse_delta.lock() {
            let cpid = player.closest_planet_id;
            mouse_input(
                &delta,
                &mut player,
                &planets[cpid],
                &mut conf,
                delta_time,
            );
            *delta = (0.0, 0.0);
        }

        // Add active movement
        if let Ok(keys) = pressed_keys.lock() {
            let cpid = player.closest_planet_id;
            keyboard_input(
                keys, 
                &mut key_debounce, 
                &mut player, 
                &planets[cpid],
                &mut conf, 
                delta_time,
            );
        }

        // Lastly, center skybox around player
        skybox_node.position = player.position;


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
        // Log gpu memory
        let buf_mem = util::MEMORY_USAGE.load(std::sync::atomic::Ordering::Relaxed);
        let s = format!("GPU mem {}KiB used for planet buffers", 
            buf_mem / 1024);
        text_gfxmem_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_gfxmem_node.update_buffers(&text_gfxmem_mesh);
        // Log movement speed
        let s = format!("Speed: {:.3}", conf.movement_speed);
        text_mspeed_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_mspeed_node.update_buffers(&text_mspeed_mesh);
        // Log fps
        let s = format!("FPS: {:}", 1.0 / delta_time);
        text_closest_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_closest_node.update_buffers(&text_closest_mesh);
        // Log mouse directional vectors
        let up = player.up();
        let s = format!("dir: {:.3},{:.3},{:.3} right: {:.3},{:.3},{:.3}, up: {:.3},{:.3},{:.3}", 
            player.direction.x, player.direction.y, player.direction.z, 
            player.right.x, player.right.y, player.right.z, 
            up.x, up.y, up.z,
        );
        text_mouse_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_mouse_node.update_buffers(&text_mouse_mesh);
        // Display player state
        let s = match player.state {
            player::PlayerState::FreeFloat => String::from("Free floating"),
            player::PlayerState::Anchored(a) => String::from(
                &format!("Anchored to: {:.3},{:.3},{:.3}", a.x, a.y, a.z)
            ),
            player::PlayerState::Landed(a) => String::from(
                &format!("Landed on: {:.3},{:.3},{:.3}", a.x, a.y, a.z)
            ),
        };
        text_pstate_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_pstate_node.update_buffers(&text_pstate_mesh);
        // Display height over planet and planet's terrain heights
        let s = match player.state {
            player::PlayerState::FreeFloat => String::from("Free floating"),
            player::PlayerState::Landed(_)   |
            player::PlayerState::Anchored(_) => String::from(
                &format!("Player h: {:.3}, Terrain h: {:.3}, norm pos: {:.3},{:.3},{:.3}", 
                    glm::length(&(player.feet() - planets[player.closest_planet_id].position)),
                    planets[player.closest_planet_id].get_height(&player.position),
                    glm::normalize(&(player.feet() - planets[player.closest_planet_id].position)).x,
                    glm::normalize(&(player.feet() - planets[player.closest_planet_id].position)).y,
                    glm::normalize(&(player.feet() - planets[player.closest_planet_id].position)).z,
                )
            ),
        };
        text_height_mesh = mesh::Mesh::text_buffer(
            &s,
            49.0 / 29.0, 1.0 * s.len() as f32 / 28.0
        );
        text_height_node.update_buffers(&text_height_mesh);



        //---------------------------------------------------------------------/
        // Update perspective
        //---------------------------------------------------------------------/
        let wsize = context.window().inner_size();
        let aspect = wsize.width as f32 / wsize.height as f32;
        let perspective_mat: glm::Mat4 = glm::perspective(
            aspect,
            conf.fov,       // field of view
            conf.clip_near, // near
            conf.clip_far   // far
        );
        
        //---------------------------------------------------------------------/
        // First person view
        //---------------------------------------------------------------------/
        let up = player.up();
        // let cam = glm::look_at(&player.position, &(player.position+player.direction), &up);
        let cam = glm::look_at(&player.position, &(player.position+player.direction), &up);
        let perspective_view = perspective_mat * cam;


        //---------------------------------------------------------------------/
        // Draw section
        //---------------------------------------------------------------------/
        unsafe {
            // Fixed both viewport and movement direction??
            gl::Viewport(0, 0, wsize.width as i32, wsize.height as i32);
            //-----------------------------------------------------------------/
            // Global uniforms
            //-----------------------------------------------------------------/
            let u_time = sh.get_uniform_location("u_time");
            gl::Uniform1f(u_time, elapsed);
            let u_view = sh.get_uniform_location("u_view");
            let u_perspective = sh.get_uniform_location("u_perspective");
            gl::UniformMatrix4fv(
                u_view,
                1,
                gl::FALSE,
                cam.as_ptr(),
            );
            gl::UniformMatrix4fv(
                u_perspective,
                1,
                gl::FALSE,
                perspective_mat.as_ptr(),
            );
            
            
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
            scene_root.update_node_transformations(&glm::identity(), &player.position);
            
            let mut planets_sorted = vec![];
            for (node, mut planet) in planet_nodes.iter().zip(&mut planets) {
                planet.position = node.position;
                planet.rotation = node.rotation;
                planet.radius = node.scale.x / 2.0;
                planet.update_uniforms(&sh);
                let dist = glm::length(&(planet.position - player.position)) - planet.radius;
                planets_sorted.push((dist, planet.planet_id));
            }
            planets_sorted.sort_by(|&a,&b| a.0.partial_cmp(&b.0).unwrap());
            planets_sorted.iter().enumerate().for_each(|(i, &(_dist, id))| {
                gl::Uniform1ui(
                    sh.get_uniform_location(&format!("u_planet_ids_sorted[{}]", i)),
                    id as u32,
                )
            });
            if let PlayerState::FreeFloat = player.state {
                // Only update closest planet if position is not depending on it
                player.closest_planet_id = planets_sorted[0].1;
            }
            // Stop rendering passed render_limit
            (0..planets.len()).for_each(|i| {
                planets[i].lod(&mut (*planet_nodes[i]), player.position);
                let depth_test = planets[i].radius / glm::length(&(planets[i].position - player.position));
                planet_nodes[i].node_type = if depth_test.atan() < conf.render_limit {
                    SceneNodeType::PlanetSkip
                } else {
                    SceneNodeType::Empty
                };
            });

            gl::Uniform1ui(
                sh.get_uniform_location("u_planets_len"),
                planets.len() as u32
            );
            lightsources.iter().enumerate().for_each(|(i, &id)| {
                gl::Uniform1ui(
                    sh.get_uniform_location(&format!("u_lightsources[{}]", i)),
                    id as u32,
                )
            });
            gl::Uniform1ui(
                sh.get_uniform_location("u_lightsources_len"),
                lightsources.len() as u32
            );
            gl::Uniform3fv(
                sh.get_uniform_location("u_player_position"),
                1,
                player.position.as_ptr()
            );

            //-----------------------------------------------------------------/
            // Draw skybox
            //-----------------------------------------------------------------/
            gl::DepthFunc(gl::LEQUAL);
            skybox_node.update_node_transformations(&glm::identity(), &player.position);
            skybox_node.draw_scene(&perspective_view, &sh, (0.1, 10.0));
            gl::DepthFunc(gl::LESS);

            //-----------------------------------------------------------------/
            // Draw elements in multiple passes using different clipping planes
            //-----------------------------------------------------------------/
            // Draw objects very far away 
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            let clipping = (125.0, 162500.0);
            let perspective_mat: glm::Mat4 = glm::perspective(
                aspect,
                conf.fov,       // field of view
                clipping.0, // near
                clipping.1   // far
            );
            gl::UniformMatrix4fv(
                u_perspective,
                1,
                gl::FALSE,
                perspective_mat.as_ptr(),
            );
            let perspective_view = perspective_mat * cam;
            scene_root.draw_scene(&perspective_view, &sh, clipping);
            // Draw objects pretty far away 
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            let clipping = (2.5, 1250.0);
            let perspective_mat: glm::Mat4 = glm::perspective(
                aspect,
                conf.fov,       // field of view
                clipping.0, // near
                clipping.1   // far
            );
            gl::UniformMatrix4fv(
                u_perspective,
                1,
                gl::FALSE,
                perspective_mat.as_ptr(),
            );
            let perspective_view = perspective_mat * cam;
            scene_root.draw_scene(&perspective_view, &sh, clipping);
            // Draw objects far away (close planets)
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            let clipping = (0.005, 25.0);
            let perspective_mat: glm::Mat4 = glm::perspective(
                aspect,
                conf.fov,       // field of view
                clipping.0, // near
                clipping.1   // far
            );
            gl::UniformMatrix4fv(
                u_perspective,
                1,
                gl::FALSE,
                perspective_mat.as_ptr(),
            );
            let perspective_view = perspective_mat * cam;
            scene_root.draw_scene(&perspective_view, &sh, clipping);
            // Draw objects that are close (landed on planet)
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            let clipping = (0.0005, 2.5);
            let perspective_mat: glm::Mat4 = glm::perspective(
                aspect,
                conf.fov,       // field of view
                clipping.0, // near
                clipping.1   // far
            );
            let perspective_view = perspective_mat * cam;
            scene_root.draw_scene(&perspective_view, &sh, clipping);
            

            //-----------------------------------------------------------------/
            // Draw GUI if enabled
            //-----------------------------------------------------------------/
            if conf.draw_gui {
                gl::Disable(gl::DEPTH_TEST);
                gui_root.update_node_transformations(&glm::identity(), &player.position);
                gui_root.draw_scene(&perspective_view, &sh, clipping);
                gl::Enable(gl::DEPTH_TEST);
            }
        }

        context.swap_buffers().unwrap();
        frame_counter += 1;
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
        player::PlayerState::Landed(_) |
        player::PlayerState::Anchored(_) => {
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
            // horizontal angle rotates around up -> modifies right and direction
            player.direction = glm::rotate_vec3(
                &player.direction, -delta_h, &up
            );
            // vertical angle rotates around right -> modifies up and direction
            player.direction = glm::rotate_vec3(
                &player.direction, -delta_v, &player.right
            );

        }
    }
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
    use player::PlayerState::*;
    let up = player.up();
    let _flat_direction = glm::cross(&up, &player.right);

    // Transform from camera position to movement
    let mut player_position = player.position - up * player.height;
    let mut position = player_position;
    let movement_speed = conf.movement_speed;
    for key in keys.iter() {
        match key {
            /* Move left/right */
            VirtualKeyCode::A => {
                position -= match player.state {
                    FreeFloat => player.right * delta_time * movement_speed,
                    Anchored(_) | 
                    Landed(_) => player.right * delta_time * movement_speed,
                }
            },
            VirtualKeyCode::D => {
                position += match player.state {
                    FreeFloat => player.right * delta_time * movement_speed,
                    Anchored(_) | 
                    Landed(_) => player.right * delta_time * movement_speed,
                }
            },
            /* Move forward (inward)/backward, in camera direction */
            VirtualKeyCode::W => {
                position += match player.state {
                    FreeFloat => player.direction * delta_time * movement_speed,
                    Anchored(_) | 
                    Landed(_) => _flat_direction * delta_time * movement_speed,
                }
            },
            VirtualKeyCode::S => {
                position -= match player.state {
                    FreeFloat => player.direction * delta_time * movement_speed,
                    Anchored(_) | 
                    Landed(_) => _flat_direction * delta_time * movement_speed,
                }
            },
            /* Move up/down */
            VirtualKeyCode::Space => {
                match player.state {
                    Landed(_) => {
                        // Jump, set horizontal speed
                        let planet_h = closest_planet.get_height(&position);
                        let player_h = glm::length(&(
                            player.feet() - closest_planet.position
                        )); // closest_planet.position == a
                        // Not quite right, but jetpack physics is alright as well
                        if planet_h - player_h < H_ERROR {
                            player.hspeed = conf.jump_speed;
                        }
                    },
                    _ => position += up * delta_time * movement_speed,
                }
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
            VirtualKeyCode::Up => {
                let v = key_debounce.entry(VirtualKeyCode::Up).or_insert(0);
                if *v == 0 {
                    conf.movement_speed = conf.movement_speed * 1.6;
                    *v = 10;
                }
            },
            VirtualKeyCode::Down => {
                let v = key_debounce.entry(VirtualKeyCode::Down).or_insert(0);
                if *v == 0 {
                    conf.movement_speed = conf.movement_speed / 1.6;
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
                    use player::PlayerState::*;
                    player.state = match player.state {
                        FreeFloat => {
                            let a = closest_planet.position;
                            eprintln!("Player position is {:?}", position);
                            eprintln!("New origin is {:?}", a);
                            position -= a;
                            eprintln!("New player position is {:?}", position);
                            Anchored(glm::vec3(0.0, 0.0, 0.0))
                        },
                        Anchored(_) => Landed(glm::vec3(0.0, 0.0, 0.0)),
                        Landed(_) => FreeFloat,
                    };
                    *v = 10;
                }
            }
            _ => { }
        }

    }

    // Apply movement
    if matches!(player.state, Landed(_)) {
        // Apply gravitational pull
        position += up * player.hspeed;
        if player.hspeed > -MAX_H_SPEED {
            player.hspeed -= delta_time * closest_planet.gravity;
        }
    }
    let height = (100.0 * closest_planet.get_height(&position)).round() / 100.0;
    let go_to = (100.0 * glm::length(&(position - closest_planet.position))).round() / 100.0;
    if go_to >= height {
        player_position = position;
    }
    else if matches!(player.state, Landed(_) | Anchored(_)) {
        // Stick to the ground
        player_position = glm::normalize(&position) * height;
    }
    player.position = player_position + up * player.height;
}