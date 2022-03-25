use nalgebra_glm as glm;
// Player controller
// - Mark closest planet and apply UP with an on/off switch

#[derive(Debug)]
pub enum PlayerState { FreeFloat, Anchored(glm::TVec3<f32>) }

pub struct Player {
    pub position: glm::TVec3<f32>,
    pub direction: glm::TVec3<f32>,

    pub closest_planet_id: i32,
}