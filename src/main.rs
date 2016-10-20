extern crate clap;
extern crate image;
extern crate memmap;
extern crate pbr;
extern crate tempdir;

use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

use image::{Rgb, RgbImage, ImageBuffer};
use memmap::{Mmap, Protection};
use tempdir::TempDir;

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

fn spiralize(frames: &Vec<PathBuf>, out_dir: &PathBuf) {   
    let mut width = 0;
    let mut height = 0;
    let temp_dir = TempDir::new("spiralizer").unwrap();

    let maps: Vec<Mmap> = frames.iter().filter_map(|file| {
        let rgb_image = match image::open(file) {
            Ok(img) => img.to_rgb(),
            Err(_) => {
                println!("Unable to open file as image, ignoring: '{}'", file.to_string_lossy());
                return None;
            },
        };
        if width == 0 && height == 0 {
            width = rgb_image.width();
            height = rgb_image.height();
        } else if width != rgb_image.width() || height != rgb_image.height() {
            println!("Images must all be the same size");
            exit(0);
        }
        let mut mmap_file_path = temp_dir.path().join(file.file_name().unwrap());
        mmap_file_path.set_extension("bin");
        let mut mmap_file = File::create(&mmap_file_path).unwrap();
        mmap_file.write_all(&*rgb_image.into_raw()).unwrap();
        Some(Mmap::open_path(&mmap_file_path, Protection::Read).unwrap())
    }).collect();

    let frames: Vec<ImageBuffer<Rgb<u8>, &[u8]>> = maps.iter()
        .filter_map(|m| ImageBuffer::from_raw(width, height, unsafe { m.as_slice() }))
        .collect();

    let mut output = RgbImage::new(width, height);
    let mut progress_bar = pbr::ProgressBar::new(frames.len() as u64);

    for i in 0..frames.len() {
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let two_pi = 2f64 * PI;
            let c_x = (height/2) as i32 - y as i32;
            let c_y = (width/2) as i32 - x as i32;
            let pixel_angle = f64::atan2(c_x as f64, c_y as f64) + two_pi;
            let angle_modifier = i as f64 / frames.len() as f64 * two_pi;
            let time_of_day = ((pixel_angle + angle_modifier) % two_pi) / two_pi;
            let source_frame: usize = (time_of_day * frames.len() as f64).floor() as usize;
            *pixel = *frames[source_frame].get_pixel(x, y);
        }
        output.save(out_dir.join(format!("frame_{}.png", i)).to_str().unwrap()).unwrap();

        progress_bar.inc();
    }

    progress_bar.finish_print(&format!("Saved {} images to {}\n", frames.len(), out_dir.to_string_lossy()));
}
