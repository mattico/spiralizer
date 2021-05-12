use std::f64::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use memmap::Mmap;
use tempfile::TempDir;

const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const APP_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

fn main() {
    let matches = clap::App::new("Spiralizer")
        .version(APP_VERSION)
        .author(APP_AUTHORS)
        .about("Helps create a swirly timelapse gif")
        .after_help(
            "Supported input formats: PNG JPG GIF ICO BMP\n\
                     Output images will be saved in PNG format.",
        )
        .arg(
            clap::Arg::with_name("INPUT")
                .help("A directory full of images to use")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("OUTPUT")
                .help("A directory to output the images to")
                .required(true),
        )
        .get_matches();

    let assert_is_dir = |d: &PathBuf| {
        if !d.is_dir() {
            println!(
                "ERROR: Arguments must be directories.\n{} is not a directory",
                d.to_string_lossy()
            );
            exit(0);
        }
    };

    let mut input_files: Vec<PathBuf> = {
        let dir = matches.value_of("INPUT").unwrap();
        let d = PathBuf::from(dir);
        assert_is_dir(&d);
        d.read_dir().unwrap().map(|x| x.unwrap().path()).collect()
    };
    input_files.sort();

    let out_dir: PathBuf = {
        let dir = matches.value_of("OUTPUT").unwrap();
        let d = PathBuf::from(dir);
        assert_is_dir(&d);
        d
    };

    if input_files.len() > 1 {
        spiralize(&input_files, &out_dir);
    } else {
        println!("ERROR: Not enough valid frames provided. Need at least 2.");
        exit(0);
    }
}

fn spiralize(input_files: &Vec<PathBuf>, out_dir: &PathBuf) {
    let mut width = 0;
    let mut height = 0;
    let temp_dir = TempDir::new_in("spiralizer").unwrap();

    println!("= Loading images =");

    let mut load_progress_bar = pbr::ProgressBar::new(input_files.len() as u64);

    let mmaps: Vec<Mmap> = input_files
        .iter()
        .filter_map(|file| {
            let rgb_image = match image::open(file) {
                Ok(img) => img.to_rgb8(),
                Err(_) => return None,
            };
            if width == 0 && height == 0 {
                width = rgb_image.width();
                height = rgb_image.height();
            } else if width != rgb_image.width() || height != rgb_image.height() {
                println!("ERROR: Images must all be the same size.");
                exit(0);
            }
            let mut mmap_file_path = temp_dir.path().join(file.file_name().unwrap());
            mmap_file_path.set_extension("bin");
            let mut mmap_file = File::create(&mmap_file_path).unwrap();
            mmap_file.write_all(&*rgb_image.into_raw()).unwrap();
            load_progress_bar.inc();
            Some(unsafe { Mmap::map(&mmap_file).unwrap() })
        })
        .collect();

    load_progress_bar.finish_print(&format!(
        "Loaded {} images. {} files ignored.\n",
        mmaps.len(),
        input_files.len() - mmaps.len()
    ));

    let frames: Vec<ImageBuffer<Rgb<u8>, &[u8]>> = mmaps
        .iter()
        .filter_map(|m| ImageBuffer::from_raw(width, height, m.as_ref()))
        .collect();

    let mut output = RgbImage::new(width, height);
    let mut save_progress_bar = pbr::ProgressBar::new(frames.len() as u64);

    println!("= Saving images =");

    for i in 0..frames.len() {
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let two_pi = 2f64 * PI;
            let c_x = (height / 2) as i32 - y as i32;
            let c_y = (width / 2) as i32 - x as i32;
            let mut pixel_angle = f64::atan2(c_x as f64, c_y as f64);
            if pixel_angle < 0.0 {
                pixel_angle += two_pi;
            }
            let angle_modifier = i as f64 / frames.len() as f64 * two_pi;
            let time_of_day = ((pixel_angle + angle_modifier) % two_pi) / two_pi;
            let source_frame_frac = time_of_day * frames.len() as f64;
            let source_frame_round = source_frame_frac.round();
            let source_frame = source_frame_frac.floor() as isize;

            let blend_source_frame = if source_frame_frac > source_frame_round {
                source_frame - 1
            } else {
                source_frame + 1
            };

            if blend_source_frame < 0 || blend_source_frame >= frames.len() as isize {
                *pixel = *frames[source_frame as usize].get_pixel(x, y);
                continue;
            }

            let blend_val = (source_frame_frac - source_frame_round).abs();

            let mut main_pixel = frames[source_frame as usize].get_pixel(x, y).to_rgba();
            let mut blend_pixel = frames[blend_source_frame as usize]
                .get_pixel(x, y)
                .to_rgba();

            blend_pixel[3] = (blend_val * 256.0) as u8;
            main_pixel.blend(&blend_pixel);

            *pixel = main_pixel.to_rgb();
        }
        output
            .save(
                out_dir
                    .join(format!("frame_{:04}.png", i))
                    .to_str()
                    .unwrap(),
            )
            .unwrap();
        save_progress_bar.inc();
    }

    save_progress_bar.finish_print(&format!(
        "Saved {} images to {}\n",
        frames.len(),
        out_dir.to_string_lossy()
    ));
}
