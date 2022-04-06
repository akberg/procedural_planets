use nalgebra_glm as glm;
// Player controller
// - Mark closest planet and apply UP with an on/off switch

#[derive(Debug)]
pub enum PlayerState { FreeFloat, Anchored(glm::TVec3<f32>) }
impl Default for PlayerState {
    fn default() -> Self {
        Self::FreeFloat
    }
}

#[derive(Debug, Default)]
pub struct Player {
    pub position: glm::TVec3<f32>,
    pub direction: glm::TVec3<f32>,
    pub right: glm::TVec3<f32>,
    pub state: PlayerState,

    pub closest_planet_id: i32,
}