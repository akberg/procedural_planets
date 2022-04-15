use crate::scene_graph::{Node, SceneNode, SceneNodeType};
use crate::procedural_planet as planet;


pub fn create_scene() -> (Vec<planet::Planet>, Vec<Node>, Vec<usize>) {
    let mut planets = vec![];
    let mut planet_nodes = vec![];
    let mut lightsources = vec![];

    // sun
    let mut planet = planet::Planet::with_seed(498765401);
        let planet_sun = planet.planet_id;
        planet.max_height = 0.005;   // relative to scale
        planet.noise_size = 500.0;
        planet.max_lod = 2;
        planet.has_ocean = false;
        planet.color_scheme = [
            glm::vec3(0.7608, 0.1535, 0.1),
            glm::vec3(0.8608, 0.2029, 0.1),
            glm::vec3(0.9608, 0.2235, 0.1),
            glm::vec3(0.9608, 0.3729, 0.1),
            glm::vec3(0.9908, 0.4335, 0.1),
        ];
        planet.color_thresholds = [
            -0.0007, -0.0001, 0.0004, 0.0008
        ];
        planet.emission = glm::vec3(1.0, 0.5, 0.3);
        planet.lightsource = true;
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 65.0;
        planet_node.position = glm::vec3(00.0, 0.0, 0.0);
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        lightsources.push(planet.planet_id);
        planets.push(planet);
        planet_nodes.push(planet_node);

    // Small earth-like planet
    let mut planet = planet::Planet::with_seed(43932);
        let planet_earth0 = planet.planet_id;
        planet.max_height = 0.03;
        planet.noise_size = 25.0;
        planet.ocean_dark_color = glm::vec3(0.001, 0.03, 0.01);
        planet.ocean_light_color = glm::vec3(0.04, 0.37, 0.33);
        planet.emission = glm::vec3(0.03, 0.32, 0.37);
        planet.color_scheme = [
            glm::vec3(0.4, 0.4, 0.3),
            glm::vec3(0.7, 0.55, 0.0),
            glm::vec3(0.2, 0.6, 0.4),
            glm::vec3(0.5, 0.4, 0.4),
            glm::vec3(0.91, 1.0, 1.0),
        ];
        planet.color_thresholds = [
            -0.0005, 0.0008, 0.019, 0.022
        ];
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 23.0;
        planet.trajectory = 970.0;
        planet.traj_speed = 0.012;
        planet.init_angle = glm::vec3(6.24f32, 0.5, 1.0f32);
        planet_node.position = glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);

    // Other planet
    let mut planet = planet::Planet::with_seed(1834327);
        let planet_earth1 = planet.planet_id;
        planet.max_height = 0.08;
        planet.noise_size = 4.0;
        planet.emission = glm::vec3(0.02, 0.26, 0.36);
        planet.ocean_dark_color = glm::vec3(0.01, 0.06, 0.11);
        planet.ocean_light_color = glm::vec3(0.05, 0.20, 0.40);
        planet.color_scheme = [
            glm::vec3(0.6118, 0.3137, 0.1961),
            glm::vec3(0.6118, 0.3137, 0.1961),
            glm::vec3(0.1686, 0.3922, 0.3176),
            glm::vec3(0.4588, 0.4588, 0.4588),
            glm::vec3(0.91, 1.0, 1.0),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.028
        ];
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 16.0;
        planet.trajectory = 650.0;
        planet.traj_speed = 0.03;
        planet.init_angle = glm::vec3(0.08f32, 0.3, 1.0);
        planet_node.position = glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);

    // Small mars-like planet
    let mut planet = planet::Planet::with_seed(94333);
        let planet_mars = planet.planet_id;
        planet.parent_id = planet_sun; // default
        planet.max_height = 0.03;
        planet.noise_size = 10.0;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.6118, 0.1255, 0.1255);
        planet.color_scheme = [
            glm::vec3(0.6118, 0.1255, 0.1255),
            glm::vec3(0.7, 0.55, 0.0),
            glm::vec3(0.7804, 0.2275, 0.0118),
            glm::vec3(0.8275, 0.302, 0.0),
            glm::vec3(0.91, 1.0, 1.0),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.026
        ];
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 15.3;
        planet.trajectory = 440.0;
        planet.init_angle = glm::vec3(6.24, 0.1, 1.0);
        planet_node.position = glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        eprintln!("Mars is {} away from the sun", glm::length(&planet_node.position));
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);

    // Moon of mars-like planet
    let mut planet = planet::Planet::with_seed(4329713);
        planet.parent_id = planet_mars;
        planet.max_height = 0.003;
        planet.noise_size = 6.0;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.118, 0.1255, 0.1255);
        planet.color_scheme = [
            glm::vec3(0.118, 0.1255, 0.1255),
            glm::vec3(0.118, 0.255, 0.255),
            glm::vec3(0.018, 0.20, 0.20),
            glm::vec3(0.08, 0.1055, 0.1055),
            glm::vec3(0.118, 0.1255, 0.1255),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.026
        ];
        planet.init_angle = glm::vec3(0.02, 0.0, 1.0);
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 4.0;
        planet.trajectory = 50.0;
        planet.traj_speed = 0.8;
        planet_node.position = planet_nodes[2].position + glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);
        
    // Moon of closest earth-like planet
    let mut planet = planet::Planet::with_seed(35462);
        planet.parent_id = planet_earth0;
        planet.max_height = 0.09;
        planet.noise_size = 5.4;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.118, 0.1255, 0.1255);
        planet.color_scheme = [
            glm::vec3(0.118, 0.1255, 0.1255),
            glm::vec3(0.118, 0.255, 0.255),
            glm::vec3(0.018, 0.20, 0.20),
            glm::vec3(0.08, 0.1055, 0.1055),
            glm::vec3(0.118, 0.1255, 0.1255),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.026
        ];
        planet.init_angle = glm::vec3(0.7, 0.0, 1.0);
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 4.4;
        planet.trajectory = 48.0;
        planet.traj_speed = 0.8;
        planet_node.position = planet_nodes[1].position + glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);
        
    // Moon 1 of second earth-like planet
    let mut planet = planet::Planet::with_seed(87635462);
        planet.parent_id = planet_earth1;
        planet.max_height = 0.12;
        planet.noise_size = 3.4;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.118, 0.1255, 0.1255);
        planet.color_scheme = [
            glm::vec3(0.118, 0.1255, 0.1255),
            glm::vec3(0.118, 0.255, 0.255),
            glm::vec3(0.018, 0.20, 0.20),
            glm::vec3(0.08, 0.1055, 0.1055),
            glm::vec3(0.118, 0.1255, 0.1255),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.026
        ];
        planet.init_angle = glm::vec3(3.13, 0.0, 3.7);
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 4.8;
        planet.trajectory = 64.0;
        planet.traj_speed = 0.8;
        planet_node.position = planet_nodes[0].position + glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);
        
    // Moon 2 of second earth-like planet
    let mut planet = planet::Planet::with_seed(192743);
        planet.parent_id = planet_earth1;
        planet.max_height = 0.09;
        planet.noise_size = 3.6;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.118, 0.1255, 0.1255);
        planet.color_scheme = [
            glm::vec3(0.30, 0.41, 0.2),
            glm::vec3(0.70, 0.61, 0.17),
            glm::vec3(0.20, 0.06, 0.0),
            glm::vec3(0.502, 0.4706, 0.349),
            glm::vec3(0.8588, 0.7725, 0.3882),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.026
        ];
        planet.init_angle = glm::vec3(0.46, 0.0, 2.2);
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 3.1;
        planet.trajectory = 48.0;
        planet.traj_speed = 0.8;
        planet_node.position = planet_nodes[0].position + glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);
        
    // Moon 3 of second earth-like planet
    let mut planet = planet::Planet::with_seed(12342);
        planet.parent_id = planet_earth1;
        planet.max_height = 0.04;
        planet.noise_size = 2.7;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.118, 0.1255, 0.1255);
        planet.color_scheme = [
            glm::vec3(0.30, 0.41, 0.2),
            glm::vec3(0.70, 0.61, 0.17),
            glm::vec3(0.20, 0.06, 0.0),
            glm::vec3(0.502, 0.4706, 0.349),
            glm::vec3(0.8588, 0.7725, 0.3882),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.026
        ];
        planet.init_angle = glm::vec3(3.80, 0.0, 2.8);
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 3.9;
        planet.trajectory = 36.0;
        planet.traj_speed = 0.8;
        planet_node.position = planet_nodes[0].position + glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);

    // Blue small planet in outer rim
    let mut planet = planet::Planet::with_seed(71772);
        planet.parent_id = planet_sun;
        planet.max_height = 0.02;
        planet.noise_size = 8.2;
        planet.has_ocean = false;
        planet.emission = glm::vec3(0.0941, 0.1922, 0.5216);
        planet.color_scheme = [
            glm::vec3(0.1686, 0.3412, 0.9216),
            glm::vec3(0.0941, 0.1922, 0.5216),
            glm::vec3(0.2078, 0.3412, 0.7804),
            glm::vec3(0.0941, 0.1922, 0.5216),
            glm::vec3(0.1686, 0.3412, 0.9216),
        ];
        planet.color_thresholds = [
            -0.01, 0.001, 0.010, 0.016
        ];
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 13.5;
        planet.trajectory = 1690.0;
        planet.traj_speed = 0.1;
        planet.init_angle = glm::vec3(-6.22, 1.1, 3.5);
        planet_node.position = glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);
    
    // Yellow ish planet close to sun
    let mut planet = planet::Planet::with_seed(98732);
        planet.parent_id = planet_sun;
        planet.max_height = 0.023;
        planet.noise_size = 6.7;
        planet.has_ocean = true;
        planet.ocean_dark_color = glm::vec3(0.20, 0.06, 0.0);
        planet.ocean_light_color = glm::vec3(0.70, 0.61, 0.17);
        planet.emission = glm::vec3(0.50, 0.41, 0.01);
        planet.color_scheme = [
            glm::vec3(0.30, 0.41, 0.2),
            glm::vec3(0.60, 0.41, 0.01),
            glm::vec3(0.4941, 0.3804, 0.2784),
            glm::vec3(0.502, 0.4706, 0.349),
            glm::vec3(0.8588, 0.7725, 0.3882),
        ];
        planet.color_thresholds = [
            -0.01, 0.001, 0.010, 0.016
        ];
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 10.0;
        planet.trajectory = 190.0;
        planet.traj_speed = 0.1;
        planet.init_angle = glm::vec3(6.20, 1.1, 3.4);
        planet_node.position = glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);

    // Quite large planet
    let mut planet = planet::Planet::with_seed(87546432);
        planet.parent_id = planet_sun;
        planet.max_height = 0.08;
        planet.noise_size = 4.0;
        planet.emission = glm::vec3(0.4588, 0.6588, 0.4588);
        planet.ocean_dark_color = glm::vec3(0.06, 0.06, 0.11);
        planet.ocean_light_color = glm::vec3(0.15, 0.14, 0.40);
        planet.color_scheme = [
            glm::vec3(0.6118, 0.3137, 0.1961),
            glm::vec3(0.6118, 0.3137, 0.1961),
            glm::vec3(0.1686, 0.7922, 0.3176),
            glm::vec3(0.4588, 0.6588, 0.4588),
            glm::vec3(0.91, 1.0, 1.0),
        ];
        planet.color_thresholds = [
            -0.0005, 0.001, 0.014, 0.058
        ];
        let mut planet_node = SceneNode::with_type(SceneNodeType::Empty);
        planet_node.planet_id = planet.planet_id;
        planet_node.scale *= 46.0;
        planet.trajectory = 1250.0;
        planet.traj_speed = 0.03;
        planet.init_angle = glm::vec3(0.08f32, 0.3, 2.3);
        planet_node.position = glm::vec3(
            planet.init_angle.x.sin() * planet.trajectory, 
            planet.init_angle.y, 
            planet.init_angle.x.cos() * planet.trajectory
        );
        planet.node = planet_node.node_id;
        planet.node = planet_node.node_id;
        planets.push(planet);
        planet_nodes.push(planet_node);

    (
        planets,
        planet_nodes,
        lightsources
    )
}