extern crate image;
extern crate num;

use std::env;
use std::fs::File;
use std::path::Path;
use std::vec::Vec;

use image::{
    ImageBuffer,
    GenericImage,
    Pixel,
};

use num::ToPrimitive;

#[inline(always)]
fn sqdist(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let xdiff = x2 - x1;
    let ydiff = y2 - y1;
    return xdiff * xdiff + ydiff * ydiff;
}

#[inline(always)]
fn is_on(luma: u8) -> bool {
    return luma > 250;
}

fn generate_bitmap<I: GenericImage + 'static>(source_image: &I) -> Vec<bool> {
    let source_image_dimentions = source_image.dimensions();
    let bitmap_size = (source_image_dimentions.0 * source_image_dimentions.1) as usize;
    let mut bitmap = Vec::with_capacity(bitmap_size);
    for (_, _, pixel) in source_image.pixels() {
        bitmap.push(is_on(pixel.to_luma().data[0].to_u8().unwrap()));
    }

    return bitmap;
}

fn generate_sdf<I: GenericImage + 'static>(source_image: &I, spread: u8)
    -> ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let rad = spread as u32;
    let max_dist = rad * rad;
    let source_image_dimentions = source_image.dimensions();
    let mut target_image = ImageBuffer::new(
        source_image_dimentions.0, source_image_dimentions.1);
    let bitmap = generate_bitmap(source_image);
    for (x, y, pixel) in target_image.enumerate_pixels_mut() {
        let current_on_state = bitmap[(y * source_image_dimentions.0 + x) as usize];
        let mut dist = max_dist;
        let mut bx = 0;
        if x > rad {
            bx = x - rad;
        }

        let mut by = 0;
        if y > rad {
            by = y - rad;
        }

        let ex = std::cmp::min(x+rad, source_image_dimentions.0);
        let ey = std::cmp::min(y+rad, source_image_dimentions.1);
        for sy in by..ey {
            for sx in bx..ex {
                let on_state = bitmap[(sy * source_image_dimentions.0 + sx) as usize];
                if current_on_state != on_state {
                    dist = std::cmp::min(sqdist(x as i32, y as i32, sx as i32, sy as i32) as u32, dist);
                }
            }
        }

        let mut sdist = f32::sqrt(dist as f32) - 0.5f32;
        if !current_on_state {
            sdist = -sdist;
        }

        let norm_dist = ((sdist / (spread as f32)) * 128f32 + 127.5f32) as u8;

        *pixel = image::Luma([norm_dist]);
    }

    return target_image;
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("No input image specified");
    }

    let source_image_path = &args[1];
    let mut spread = 16u8;
    if args.len() > 2 {
        spread = args[2].parse::<u8>().unwrap();
    }

    let image = match image::open(&Path::new(source_image_path)) {
        Ok(image) => image,
        Err(error) => {println!("Could not load image {}", error); return;},
    };

    let sdf_image = generate_sdf(&image, spread);

    let ref mut fout = File::create(&Path::new("sdf.png")).unwrap();
    match image::ImageLuma8(sdf_image).save(fout, image::PNG) {
        Ok(_) => println!("SDF created"),
        Err(err) => println!("Could not save output image: {}", err),
    }
}
