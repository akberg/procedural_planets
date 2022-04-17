/// Collect project-wide global values and constants

//-gamelogic.rs----------------------------------------------------------------/

pub const SCALING_FACTOR: f32 = 10.0;
pub const WORLD_SPEED: f32 = 0.5;

//-main.rs---------------------------------------------------------------------/

pub const SCREEN_W: u32 = 1920;
pub const SCREEN_H: u32 = 1080;

//-mesh.rs---------------------------------------------------------------------/

// Iterations of fractal noise
pub const FRACTAL_ITERATIONS: usize = 8;

//-player.rs-------------------------------------------------------------------/

pub const MAX_H_SPEED: f32 = 1.0;   // Maximum horizontal speed from gravity
pub const H_ERROR: f32 = 0.001;     // Margin of error for height computation

//-procedural_planets.rs-------------------------------------------------------/

/// Thresholds for level of detail
pub const MAX_LOD: usize = 4;
pub const MAX_IN_FLIGHT: u64 = 4;
//const THRESHOLD: [f32; MAX_LOD] = [128.0, 32.0, 16.0, 8.0, 4.0, 2.0];
pub const SUBDIVS_PER_LEVEL: usize = 16; // 256: 480+380=860ms, 128: 127+98=225ms
pub const N_LAYERS: usize = 5;  // Must match with scene.frag:22