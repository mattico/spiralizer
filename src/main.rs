
extern crate clap;
extern crate image;

use std::path::{PathBuf, Path};
use std::process::exit;
use std::f64::consts::PI;

use image::{GenericImage, DynamicImage, ImageFormat, RgbImage, ImageBuffer};

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

    let assert_dir = |d: &PathBuf| {
        if !d.is_dir() {
            println!("Argument must be directory");
            exit(0);
        }
    };

    let input_files: Vec<PathBuf> = if let Some(dir) = matches.value_of("INPUT") {
        let d = PathBuf::from(dir);
        assert_dir(&d);
        d.read_dir().unwrap().map(|x| x.unwrap().path()).collect()
    } else {
        unreachable!();
    };

    let out_dir: PathBuf = if let Some(dir) = matches.value_of("OUTPUT") {
        let d = PathBuf::from(dir);
        assert_dir(&d);
        d
    } else {
        unreachable!();
    };

    let frames: Vec<RgbImage> = input_files.iter().filter_map(|f| match image::open(f) {
        Ok(image) => Some(image.to_rgb()),
        Err(_) => {
            println!("Unable to open file as image, ignoring: '{}'", f.to_string_lossy());
            None
        },
    }).collect();

    if frames.len() > 1 {
        println!("{} frames loaded", frames.len());
        spiralize(&input_files, &out_dir);
    } else {
        println!("Not enough frames loaded");
        exit(0);
    }
}

fn spiralize(frames: &Vec<PathBuf>, out_dir: &PathBuf) {   
    let width = frames[0].width();
    let height = frames[0].height();

    let mut output = RgbImage::new(width, height);
    let mut percent = 0;

    for i in 0..frames.len() {
        let mut frame_offset = i;
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let pixel_angle = f64::atan2((width/2 - x) as f64, (height/2 - y) as f64);
            let source_frame: usize = ((pixel_angle / (2f64*PI)) 
                * frames.len() as f64 + frame_offset as f64) as usize;
            *pixel = *frames[source_frame].get_pixel(x, y);
            frame_offset += 1;
        }

        output.save(out_dir.join(format!("frame_{}.png", i)).to_str().unwrap());

        let new_percent = i * 100 / frames.len();
        if new_percent > percent {
            percent = new_percent;
            println!("{}%", percent);
        }
    }
}
