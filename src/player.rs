use nalgebra_glm as glm;
// Player controller
// - Mark closest planet and apply UP with an on/off switch

pub const MAX_H_SPEED: f32 = 10.0;  // Maximum horizontal speed from gravity
pub const H_ERROR: f32 = 0.1;       // Margin of error for height computation

#[derive(Debug)]
pub enum PlayerState { 
    FreeFloat, 
    Anchored(glm::TVec3<f32>),
    Landed(glm::TVec3<f32>)
}
impl Default for PlayerState {
    fn default() -> Self {
        Self::FreeFloat
    }
}

#[derive(Debug, Default)]
pub struct Player {
    pub position: glm::TVec3<f32>,  // Global camera position
    pub direction: glm::TVec3<f32>, // Global direction of camera
    pub right: glm::TVec3<f32>,     // Right vector of camera
    pub state: PlayerState,
    pub height: f32,                // Camera height over movement position (feet)
    pub hspeed: f32,                // Horizontal speed, for simple physics

    pub closest_planet_id: i32,
}

impl Player {
    pub fn up(&mut self) -> glm::TVec3<f32> {
        use PlayerState::*;
        match self.state {
            // Gravitational force from planet
            Anchored(a) | Landed(a) => {
                let up = glm::normalize(&(self.position - a));
                // Anchored planet sets horizontal direction
                self.right = glm::normalize(&glm::cross(&self.direction, &up));
                up
            },
            FreeFloat => {
                
                glm::normalize(&glm::cross(&self.right, &self.direction))
            },
        }
    }

    pub fn feet(&mut self) -> glm::TVec3<f32> {
        self.position - self.up() * self.height
    }
}