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

impl Player {
    pub fn up(&mut self) -> glm::TVec3<f32> {
        match self.state {
            // Gravitational force from planet
            PlayerState::Anchored(a) => {
                let up = glm::normalize(&(self.position - a));
                // Anchored planet sets horizontal direction
                self.right = glm::normalize(&glm::cross(&self.direction, &up));
                up
            },
            PlayerState::FreeFloat => glm::normalize(&glm::cross(&self.right, &self.direction)),
        }
    }
}