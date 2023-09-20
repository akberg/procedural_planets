use crate::util;
use image::io::Reader as ImageReader;

/// Generate a texture binding for an RGBA8 image
unsafe fn get_texture_id(img: &image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) -> u32 {
    let mut tex_id = 0;
    gl::GenTextures(1, &mut tex_id);

    gl::BindTexture(gl::TEXTURE_2D, tex_id);

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::TexParameteri(
        gl::TEXTURE_2D,
        gl::TEXTURE_MIN_FILTER,
        gl::NEAREST_MIPMAP_LINEAR as i32,
    );
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        img.dimensions().0 as i32,
        img.dimensions().1 as i32,
        0,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        util::pointer_to_array(img),
    );

    gl::GenerateMipmap(gl::TEXTURE_2D);

    tex_id
}

pub fn load_texture(path: &str) -> u32 {
    let timer = std::time::SystemTime::now();
    eprint!("Loading texture '{}' . . . ", path);
    let img = ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .flipv()
        .into_rgba8();
    let t_id = unsafe { get_texture_id(&img) };
    eprintln!("took {:?}", timer.elapsed().unwrap());
    t_id
}
