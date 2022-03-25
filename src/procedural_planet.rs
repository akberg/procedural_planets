use nalgebra_glm as glm;

/// Procedurally generated planet. Will use a quad-tree form, each side
/// either drawing a plane or subdividing into nodes covering recursively
/// smaller planes. 
/// 
/// SceneNodeType::Empty used to mark a layer 
///
/// Expected usage:
/// - Create object containing parameters for generating planet
/// - Object controls generating subdivided meshes as needed, and how deep
/// to render, connecting with the scene graph
/// ```
/// let planet0 = Planet::new(600.0)    // radius
///     .position(glm::vec3(0.0, 0.0, 0.0))
///     .height(1.0)
///     .noise_params({ size: 3.5, niter: 5, .. });
/// 
/// scene_root.add_child(planet0.node);
/// ```
pub struct Planet {
    pub position: glm::TVec3<f32>,
    pub rotation: glm::TVec3<f32>,

    pub radius: f32,                // Radius to ocean level
    pub emission: glm::TVec3<f32>,  // Emission colour and intensity
    pub has_ocean: bool,            // Set false to 

    pub noise_fn: dyn noise::NoiseFn<[f32;3]>,

}

impl Planet {
    // pub fn new(radius: f32) -> Self {

    // }
}

/// Noise parameters to unambiguously generate a planet terrain. Should be able
/// to generate both terrain and texture (?)
pub struct PlanetParameters {
    pub size: f32,
    pub niter: usize,
    pub height: f32,                // Distance from radius to highest point
}