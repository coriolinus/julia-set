#[macro_use]
extern crate clap;
extern crate csv;
extern crate image;
extern crate julia_set;
extern crate lerp;
extern crate num;

use clap::{App, Arg};
use julia_set::{parallel_image, interpolate_rectilinear};
use julia_set::colorize::{Colorizer, HSLColorizer};
use julia_set::iter::DuplicateFirst;
use lerp::LerpIter;
use num::complex::Complex64;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::str::FromStr;

fn main() {
    // --------------------
    // set up configuration
    // --------------------
    let conf = match AnimationConfiguration::new() {
        Ok(conf) => conf,
        Err(err) => panic!(err),
    };

    let out_path = conf.basepath.join("animate");
    if !out_path.exists() {
        fs::create_dir(out_path.clone())
            .expect(&format!("Couldn't create output directory at {:?}", out_path));
    }

    println!("Input parameters:");
    println!("  Points file: {:?}", conf.pointsfile);
    println!("Output parameters:");
    println!("  Colorize:    {}", conf.colorize);
    println!("  Dimensions:  {:?}", (conf.width, conf.height));
    println!("  Mul Factor:  {}", conf.multiply);
    println!("  Output path: {:?}", out_path);
    print!("Clearing output path... ");
    remove_files_from(&out_path).expect("FATAL error clearing output path!");
    println!("done");

    println!("Generating images...");

    // ---------------------------
    // set up prerequisite objects
    // ---------------------------
    let interpolate = interpolate_rectilinear(conf.width, conf.height, -1.1, 1.1, -1.1, 1.1);
    let colorizer = HSLColorizer::new();
    let mut rdr = csv::Reader::from_file(conf.pointsfile.clone()).unwrap().flexible(true);

    // determine at runtime if we have headers
    {
        let headers = rdr.headers().unwrap();
        if headers.len() == 3 &&
           (f64::from_str(&headers[0]).is_err() || f64::from_str(&headers[1]).is_err() ||
            usize::from_str(&headers[2]).is_err()) {
            rdr = rdr.has_headers(true);
        } else {
            rdr = rdr.has_headers(false);
        }
    }

    // ---------
    // main loop
    // ---------
    //
    // this looks complex, but it's all just a sequence of operations on iterators:
    //   - get a row from the CSV reader
    //   - map it to a (Complex64, usize)
    //   - map it to (Complex64, usize, Complex64) so we know our bounds
    //   - fill in the appropriate default number of steps if unspecified
    //   - map it to a long sequence of Complex64 by lerping
    //   - enumerate it
    //   - for each of the (enumeration, complex), act out the body of the loop
    for (count, complex_position) in rdr.decode()
        .map(|record| {
            let (real, imag, steps): (f64, f64, Option<usize>) =
                record.expect("Invalid format in input CSV");
            (Complex64::new(real, imag), steps)
        })
        .duplicate_first()
        .map(|(start, steps, end)| {
            let steps = match steps {
                Some(s) => s,
                None => {
                    // if steps wasn't specified, generate steps from
                    // the distance between the two points
                    const DEFAULT_STEPS_PER_UNIT: usize = 5;
                    ((end - start).norm() * DEFAULT_STEPS_PER_UNIT as f64).ceil() as usize
                }
            };
            (start, steps, end)
        })
        .flat_map(|(start, steps, end)| start.lerp_iter(end, steps * conf.multiply))
        .enumerate() {

        let filename = format!("julia_set_{:06}.png", count);
        let file_path = out_path.join(filename.clone());
        print!("Generating {:?}... ", filename.clone());

        let image = parallel_image(conf.width,
                                   conf.height,
                                   &move |z| (z * z) + complex_position,
                                   &*interpolate,
                                   2.0);



        if conf.colorize {
            print!("colorizing... ");
            let image = colorizer.colorize(&image);
            print!("saving... ");
            image.save(file_path).expect("Fatal IO Error");
        } else {
            print!("saving... ");
            image.save(file_path).expect("Fatal IO Error");
        }

        println!("done!");

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

struct AnimationConfiguration {
    colorize: bool,
    width: u32,
    height: u32,
    multiply: usize,
    basepath: path::PathBuf,
    pointsfile: path::PathBuf,
}

impl AnimationConfiguration {
    fn build_cli() -> App<'static, 'static> {
        App::new("animate")
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
          .arg(Arg::with_name("pointsfile")
                    .short("p")
                    .long("points-file")
                    .value_names(&["PATH"])
                    .default_value("animation-steps.csv")
                    .help("CSV file from which to load the points data for this animation.")
                )
    }

    /// Construct a new animation configuration object by reading and parsing the command line.
    fn new() -> Result<AnimationConfiguration, String> {
        AnimationConfiguration::unpack_matches(AnimationConfiguration::build_cli().get_matches())
    }

    fn unpack_matches(matches: clap::ArgMatches) -> Result<AnimationConfiguration, String> {
        let colorize = matches.is_present("colorize");
        let (width, height) = {
            let dimensions = values_t!(matches, "dimensions", u32).unwrap_or_else(|e| e.exit());
            (dimensions[0], dimensions[1])
        };
        let multiply = value_t!(matches, "multiply", usize).unwrap_or_else(|e| e.exit());
        let pointsfile = value_t!(matches, "pointsfile", String).unwrap_or_else(|e| e.exit());

        let path = env::current_dir().unwrap();
        let pointsfile = path.join(pointsfile);
        if !pointsfile.exists() || !pointsfile.is_file() {
            return Err(format!("Points file at {:?} does not exist or is not a file.",
                               pointsfile));
        }

        Ok(AnimationConfiguration {
            colorize: colorize,
            width: width,
            height: height,
            multiply: multiply,
            basepath: path,
            pointsfile: pointsfile,
        })
    }

    /// Returns a string which resembles the way the program was called
    fn called_as() -> String {
        env::args().collect::<Vec<_>>().join(" ")
    }
}

impl Default for AnimationConfiguration {
    fn default() -> AnimationConfiguration {
        AnimationConfiguration::unpack_matches(AnimationConfiguration::build_cli()
                .get_matches_from(Vec::<String>::new()))
            .unwrap()
    }
}
