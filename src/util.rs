extern crate nalgebra_glm as glm;
use std::ffi::CString;
use std::f64::consts::PI;
use std::{ mem, ptr, os::raw::c_void };

//-----------------------------------------------------------------------------/
// Helper functions to make interacting with OpenGL a little bit 
// prettier. You *WILL* need these! The names should be pretty self 
// explanatory.
//-----------------------------------------------------------------------------/

// Get # of bytes in an array.
#[inline(always)]
pub fn byte_size_of_array<T>(val: &[T]) -> isize {
    mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
pub fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
#[allow(unused)]
#[inline(always)]
pub fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

#[allow(unused)]
// Get an offset in bytes for n units of type T
pub fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

//-----------------------------------------------------------------------------/
// OpenGL debug utils
//-----------------------------------------------------------------------------/

pub unsafe fn get_gl_string(name: gl::types::GLenum) -> String {
    std::ffi::CStr::from_ptr(gl::GetString(name) as *mut i8).to_string_lossy().to_string()
}

// Debug callback to panic upon enountering any OpenGL error
pub extern "system" fn debug_callback(
    source: u32, e_type: u32, id: u32,
    severity: u32, _length: i32,
    msg: *const i8, _data: *mut std::ffi::c_void
) {
    if e_type != gl::DEBUG_TYPE_ERROR { return }
    if severity == gl::DEBUG_SEVERITY_HIGH ||
       severity == gl::DEBUG_SEVERITY_MEDIUM ||
       severity == gl::DEBUG_SEVERITY_LOW
       {
           let severity_string = match severity {
            gl::DEBUG_SEVERITY_HIGH => "high",
            gl::DEBUG_SEVERITY_MEDIUM => "medium",
            gl::DEBUG_SEVERITY_LOW => "low",
            _ => "unknown",
        };
        unsafe {
            let string = CString::from_raw(msg as *mut i8);
            let error_message = String::from_utf8_lossy(string.as_bytes()).to_string();
            panic!("{}: Error of severity {} raised from {}: {}\n",
            id, severity_string, source, error_message);
        }
    }
}

//-----------------------------------------------------------------------------/
// Run configurations utils
//-----------------------------------------------------------------------------/

#[derive(Default, Debug)]
pub struct Config {
    pub fov: f32,
    pub clip_near: f32,
    pub clip_far: f32,
    pub movement_speed: f32,
    pub mouse_speed: f32,
    pub tilt_speed: f32,
    pub tilt: f32,
    pub init_position: [f32; 3],
    pub bg_color: [f32; 4],
    pub init_h_angle: f32,
    pub init_v_angle: f32,
    pub camera_position: i32,
    pub polymode: usize,
    //init_direction: [f32; 3],
}


impl Config {
    fn parse_array<T: std::str::FromStr + std::fmt::Debug, const D: usize>(val: &str) -> [T; D] {
        use std::convert::TryInto;
        let mut s = val.trim().split(",");
        let mut arr = Vec::new();
        for i in 0..D {
            arr.push(s.next().unwrap().trim().parse::<T>().unwrap_or_else(|t| panic!("parse array")));
        }
        
        arr.try_into().unwrap()
    }
    pub fn load() -> Self {
        use std::fs;
        let mut conf = Config { ..Default::default() };
        fs::read_to_string("resources/settings.conf")
            .unwrap()
            .lines()
            .filter(|&line| line.trim().len() > 0 && !line.starts_with("#")) // Filter empty lines and comments
            .for_each(|line| {
                println!("{}", line);
                let mut s = line.split("=");
                let (key, val) = (s.next().unwrap(), s.next().unwrap());
                match key {
                    "fov" => conf.fov = val.trim().parse::<f32>().unwrap(),
                    "clip_near" => conf.clip_near = val.trim().parse::<f32>().unwrap(),
                    "clip_far" => conf.clip_far = val.trim().parse::<f32>().unwrap(),
                    "movement_speed" => conf.movement_speed = val.trim().parse::<f32>().unwrap(),
                    "mouse_speed" => conf.mouse_speed = val.trim().parse::<f32>().unwrap(),
                    "tilt_speed" => conf.tilt_speed = val.trim().parse::<f32>().unwrap(),
                    "tilt" => conf.tilt = val.trim().parse::<f32>().unwrap(),
                    "init_h_angle" => conf.init_h_angle = val.trim().parse::<f32>().unwrap(),
                    "init_v_angle" => conf.init_v_angle = val.trim().parse::<f32>().unwrap(),
                    "camera_position" => conf.camera_position = val.trim().parse::<i32>().unwrap(),
                    "init_position" => conf.init_position = Self::parse_array::<f32, 3>(val),
                    "bg_color" => conf.bg_color = Self::parse_array::<f32, 4>(val),
                    "polymode" => conf.polymode = val.trim().parse::<usize>().unwrap(),
                    //"init_direction" => conf.init_direction = Self::parse_array::<f32, 3>(val),
                    &_ => (),
                }
            });
        conf
    }
}

// Connected vectors

// Calculate right camera vector from horixontal angle
pub fn vec_right(h_angle: f32) -> glm::Vec3 {
    glm::vec3(
        (h_angle - 3.14 / 2.0).sin(),
        0.0,
        (h_angle - 3.14/2.0).cos()
    )
}

/// Calculate direction vector from 
pub fn vec_direction(h_angle: f32, v_angle: f32) -> glm::Vec3 {
    glm::vec3(
        v_angle.cos() * h_angle.sin(),
        v_angle.sin(),
        v_angle.cos() * h_angle.cos()
    )
}


pub struct Heading {
    pub x     : f32,
    pub z     : f32,
    pub roll  : f32,
    pub pitch : f32,
    pub yaw   : f32,
}

pub fn simple_heading_animation(time: f32) -> Heading {
    let t             = time as f64;
    let step          = 0.05f64;
    let path_size     = 15f64;
    let circuit_speed = 0.8f64;

    let xpos      = path_size * (2.0 * (t+ 0.0) * circuit_speed).sin();
    let xpos_next = path_size * (2.0 * (t+step) * circuit_speed).sin();
    let zpos      = 3.0 * path_size * ((t+ 0.0) * circuit_speed).cos();
    let zpos_next = 3.0 * path_size * ((t+step) * circuit_speed).cos();

    let delta_pos = glm::vec2(xpos_next - xpos, zpos_next - zpos);

    let roll  = (t * circuit_speed).cos() * 0.5;
    let pitch = -0.175 * glm::length(&delta_pos);
    let yaw   = PI + delta_pos.x.atan2(delta_pos.y);

    Heading {
        x     : xpos  as f32,
        z     : zpos  as f32,
        roll  : roll  as f32,
        pitch : pitch as f32,
        yaw   : yaw   as f32,
    }
}

pub fn door_animation(mut time: f32, starttime: f32, open: bool) -> (f32, f32) {
    let (open_x, open_z) = (0.03f32, 1.5f32);
    time = time - starttime;
    if open {
        (
            open_x.min( open_x * 0.2 * time), 
            open_z.min(if time < open_x / 0.2 { 0.0 } else { 0.2 * time }),
        )
    } else {
        (
            0.0f32.max(if time < open_z / 0.2 { open_x } else { open_x - open_x * 0.2 * time }), 
            0.0f32.max(open_z - 0.2 * time),
        )
    }
}

#[derive(PartialEq)]
pub enum CameraPosition { 
    ThirdPerson,        // Third person camera on helicopter
    FirstPerson,        // Camera on pilot head (not finished)
    //Chase
    //Free              // Free movement
}

