#[macro_use]
extern crate clap;
extern crate image;
extern crate julia_set;
extern crate num;

use clap::{App, Arg};
use image::{ImageBuffer, GenericImage};
use julia_set::{parallel_image, interpolate_rectilinear};
use julia_set::colorize::HSLColorizer;
use julia_set::lerp::{Lerp, LerpIter};
use num::Float;
use num::complex::Complex64;
use std::env;
use std::fs;
use std::io;
use std::path;

const PATH: [(Complex64, usize); 7] = [(Complex64 { re: 0.0, im: 0.0 }, 10),
                                       (Complex64 {
                                           re: -0.12,
                                           im: 0.0,
                                       },
                                        10),
                                       (Complex64 {
                                           re: -0.8,
                                           im: 0.2,
                                       },
                                        10),
                                       (Complex64 {
                                           re: -0.5,
                                           im: 0.5,
                                       },
                                        10),
                                       (Complex64 {
                                           re: -0.1,
                                           im: 0.8,
                                       },
                                        10),
                                       (Complex64 {
                                           re: -0.2,
                                           im: 0.6,
                                       },
                                        10),
                                       (Complex64 { re: 0.0, im: 0.0 }, 0)];

fn main() {
    let matches = App::new("animate")
       .about("generates sequences of images of julia sets for compilation to animation")
      // use crate_version! to pull the version number
      .version(crate_version!())
      .arg(Arg::with_name("colorize")
                .short("c")
                .long("colorize")
                .help("If set, colorize the output images.")
            )
      .arg(Arg::with_name("dimensions")
                .short("d")
                .long("dimensions")
                .value_names(&["WIDTH", "HEIGHT"])
                .default_value("800,600")
                .help("Set the dimensions of the output images.")
            )
      .arg(Arg::with_name("multiply")
                .short("m")
                .long("multiply")
                .value_names(&["FACTOR"])
                .default_value("1")
                .help("Multiply the number of interpolation steps between each path point.")
            )
      .get_matches();

    let colorize = matches.is_present("colorize");
    let (width, height) = {
        let dimensions = values_t!(matches, "dimensions", u32).unwrap_or_else(|e| e.exit());
        (dimensions[0], dimensions[1])
    };
    let multiply = value_t!(matches, "multiply", usize).unwrap_or_else(|e| e.exit());

    let mut path = env::current_dir().unwrap();
    path.push("animate");
    if !path.exists() {
        fs::create_dir(path.clone())
            .expect(&format!("Couldn't create output directory at {:?}", path));
    }

    println!("Output parameters:");
    println!("  Colorize:    {}", colorize);
    println!("  Dimensions:  {:?}", (width, height));
    println!("  Mul Factor:  {}", multiply);
    println!("  Output path: {:?}", path);
    print!("Clearing output path... ");
    remove_files_from(path).expect("FATAL error clearing output path!");
    println!("done");


}

fn remove_files_from<P: AsRef<path::Path>>(path: P) -> io::Result<()> {
    for entry in try!(fs::read_dir(path)) {
        let entry = try!(entry);
        if entry.path().is_file() {
            try!(fs::remove_file(entry.path()));
        }
    }
    Ok(())
}
