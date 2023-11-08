mod config;
mod converter;
mod svg;
use vtracer::{convert_image_to_svg, Config};

fn main() {
    let config = config::Config::from_args();
    let result = converter::convert_image_to_svg(config);
    match result {
        Ok(()) => {
            println!("Conversion successful.");
        }
        Err(msg) => {
            panic!("Conversion failed with error message: {}", msg);
        }
    }
}
