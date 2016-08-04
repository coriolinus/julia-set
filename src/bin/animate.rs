#[macro_use]
extern crate clap;
extern crate image;
extern crate julia_set;
extern crate num;

use clap::{App, Arg};
use julia_set::{parallel_image, interpolate_rectilinear};
use julia_set::colorize::{Colorizer, HSLColorizer};
use julia_set::lerp::LerpIter;
use num::complex::Complex64;
use std::env;
use std::fs;
use std::io;
use std::path;

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
    remove_files_from(&path).expect("FATAL error clearing output path!");
    println!("done");
    println!("Generating images...");

    let complex_path = vec![(Complex64::new(0.0, 0.0), 10),
                            (Complex64::new(-0.12, 0.0), 10),
                            (Complex64::new(-0.8, 0.2), 10),
                            (Complex64::new(-0.5, 0.5), 10),
                            (Complex64::new(-0.1, 0.8), 10),
                            (Complex64::new(-0.2, 0.6), 10),
                            (Complex64::new(0.0, 0.0), 0)];

    let interpolate = interpolate_rectilinear(width, height, -1.1, 1.1, -1.1, 1.1);
    let colorizer = HSLColorizer::new();

    // there was a whole iterator adaptor thing which did this much more elegantly,
    // but I couldn't get it to work. Maybe come back to this later.
    let mut cp_iter = complex_path.iter();
    let mut start = None;
    let mut steps = 0;
    if let Some(&(strt, stps)) = cp_iter.next() {
        start = Some(strt);
        steps = stps;
    }
    let mut count = 0;
    loop {
        match (start, cp_iter.next()) {
            (Some(strt), Some(&(end, next_steps))) => {
                for complex_position in strt.lerp_iter(end, steps * multiply) {
                    let filename = {
                        let mut name = path.clone();
                        name.push(format!("julia_set_{:06}_({}{:+}i).png",
                                          count,
                                          complex_position.re,
                                          complex_position.im));
                        name
                    };
                    print!("Generating {:?}... ", filename.clone());

                    let image = parallel_image(width,
                                               height,
                                               &*Box::new(move |z| (z * z) + complex_position),
                                               &*interpolate,
                                               2.0);



                    if colorize {
                        print!("colorizing... ");
                        let image = colorizer.colorize(&image);
                        print!("saving... ");
                        image.save(filename).expect("Fatal IO Error");
                    } else {
                        print!("saving... ");
                        image.save(filename).expect("Fatal IO Error");
                    }

                    println!("done!");

                    count += 1;
                }

                // set up for the next loop
                start = Some(end);
                steps = next_steps;
            }
            _ => break,
        }
    }
    println!("Done!");
}

fn remove_files_from<P: AsRef<path::Path>>(path: &P) -> io::Result<()> {
    for entry in try!(fs::read_dir(path)) {
        let entry = try!(entry);
        if entry.path().is_file() {
            try!(fs::remove_file(entry.path()));
        }
    }
    Ok(())
}
