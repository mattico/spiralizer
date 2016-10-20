extern crate clap;
extern crate image;
extern crate memmap;

use std::path::{PathBuf, Path};
use std::process::exit;
use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;

use image::{Rgb, DynamicImage, ImageFormat, RgbImage, ImageBuffer};
use memmap::{Mmap, Protection};

const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const APP_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let matches = clap::App::new("Spiralizer")
        .version(APP_VERSION)
        .author(APP_AUTHORS)
        .about("Helps create a swirly timelapse gif")
        .arg(clap::Arg::with_name("INPUT")
            .help("A directory full of video frames to use")
            .required(true))
        .arg(clap::Arg::with_name("OUTPUT")
            .help("A directory to output the video frames to")
            .required(true))
        .get_matches();

    let assert_is_dir = |d: &PathBuf| {
        if !d.is_dir() {
            println!("Argument must be directory");
            exit(0);
        }
    };

    let input_files: Vec<PathBuf> = if let Some(dir) = matches.value_of("INPUT") {
        let d = PathBuf::from(dir);
        assert_is_dir(&d);
        d.read_dir().unwrap().map(|x| x.unwrap().path()).collect()
    } else {
        unreachable!();
    };

    let out_dir: PathBuf = if let Some(dir) = matches.value_of("OUTPUT") {
        let d = PathBuf::from(dir);
        assert_is_dir(&d);
        d
    } else {
        unreachable!();
    };

    if input_files.len() > 1 {
        spiralize(&input_files, &out_dir);
    } else {
        println!("Not enough frames provided");
        exit(0);
    }
}

fn mmap_image_buffer(file: &PathBuf) -> Option<ImageBuffer<Rgb<u8>, &[u8]>> {
    let rgb_image = match image::open(file) {
        Ok(img) => img.to_rgb(),
        Err(_) => {
            println!("Unable to open file as image, ignoring: '{}'", file.to_string_lossy());
            return None;
        },
    };
    let width = rgb_image.width();
    let height = rgb_image.height();

    let mut mmap_file_path = file.clone();
    mmap_file_path.set_extension("bin");
    let mut mmap_file = File::create(mmap_file_path).unwrap();
    mmap_file.write_all(&*rgb_image.into_raw()).unwrap();
    let mmap = Mmap::open(&mmap_file, Protection::Read).unwrap();
    forget(mmap);

    ImageBuffer::from_raw(width, height, unsafe{mmap.as_slice()})
}

fn spiralize(frames: &Vec<PathBuf>, out_dir: &PathBuf) {   
    let frames: Vec<ImageBuffer<Rgb<u8>, &[u8]>> = frames.iter().filter_map(mmap_image_buffer).collect();

    let width = frames[0].width();
    let height = frames[0].height();

    let mut output = RgbImage::new(width, height);
    let mut percent = 0;

    for i in 0..frames.len() {
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let pixel_angle = f64::atan2((width/2 - x) as f64, (height/2 - y) as f64);
            let source_frame: usize = (((pixel_angle / (2f64*PI)) 
                * frames.len() as f64 + i as f64).floor()) as usize;
            *pixel = *frames[source_frame].get_pixel(x, y);
        }
        output.save(out_dir.join(format!("frame_{}.png", i)).to_str().unwrap()).unwrap();

        let new_percent = i * 100 / frames.len();
        if new_percent > percent {
            percent = new_percent;
            println!("{}%", percent);
        }
    }
}
