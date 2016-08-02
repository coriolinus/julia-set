extern crate image;
extern crate julia_set_lib;

use image::imageops::{resize, FilterType};
use julia_set_lib::{parallel_image, default_julia, interpolate_rectilinear};
use julia_set_lib::colorize::{Colorizer, HSLColorizer};
use std::env;
use std::str::FromStr;

enum JuliaResult {
    Success = 0,
    UnknownSelfName,
    WrongNumberOfArguments,
    CantParseIntegerArguments,
    IOError,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    std::process::exit(match args.len() {
        0 => {
            println!("No args found; we're done here.");
            JuliaResult::UnknownSelfName
        }
        3 => generate_julia(&args[1], &args[2], None),
        4 => generate_julia(&args[1], &args[2], Some(&args[3])),
        _ => {
            println!("Wrong number of arguments.\n\n\
                      Usage: {} WIDTH HEIGHT [PATH]\n\
                      Where WIDTH and HEIGHT are integers.\n\
                      If PATH is not specified, defaults to 'julia_set.png'",
                     args[0]);
            JuliaResult::WrongNumberOfArguments
        }
    } as i32)
}

fn generate_julia(width: &str, height: &str, path: Option<&str>) -> JuliaResult {
    let width = {
        if let Ok(w) = u32::from_str(width) {
            w
        } else {
            println!("Couldn't parse '{}' as an integer; aborting.", width);
            return JuliaResult::CantParseIntegerArguments;
        }
    };
    let height = {
        if let Ok(h) = u32::from_str(height) {
            h
        } else {
            println!("Couldn't parse '{}' as an integer; aborting.", height);
            return JuliaResult::CantParseIntegerArguments;
        }
    };
    let path = match path {
        None => {
            let mut path = env::current_dir().unwrap();
            path.push("julia_set.png");
            path
        }
        Some(path) => {
            let mut path = std::path::PathBuf::from(path);
            if path.file_name().is_none() {
                path.set_file_name("julia_set.png");
            } else if path.extension().is_none() ||
               Some(String::from("png")) !=
               path.extension().unwrap().to_str().map(|s| s.to_lowercase()) {
                path.set_extension("png");
            }
            path
        }
    };

    println!("Got parameters:");
    println!("  width:  {}", width);
    println!("  height: {}", height);
    println!("  path:   {}", path.display());

    // julia sets are only really interesting in the region [-1...1]
    let interpolate = interpolate_rectilinear(width * 2, height * 2, -1.0, 1.0, -1.0, 1.0);

    let image = parallel_image(width * 2, height * 2, &default_julia, &*interpolate, 2.0);
    let colorizer = HSLColorizer::new();
    let image = resize(&colorizer.colorize(&image),
                       width,
                       height,
                       FilterType::Lanczos3);

    match image.save(&*path.to_string_lossy()) {
        Ok(_) => JuliaResult::Success,
        Err(error) => {
            println!("Encountered error: {}", error);
            JuliaResult::IOError
        }
    }
}
