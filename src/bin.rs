extern crate julia_set_lib;

use julia_set_lib::{save_image, default_julia};
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

    match save_image(width, height, &default_julia, 2.0, &*path.to_string_lossy()) {
        Ok(_) => JuliaResult::Success,
        Err(error) => {
            println!("Encountered error: {}", error);
            JuliaResult::IOError
        }
    }
}
