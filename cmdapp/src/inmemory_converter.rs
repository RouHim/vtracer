use image::GenericImageView;

use super::svg::SvgFile;

use crate::Config;
use visioncortex::{BinaryImage, Color, ColorName};

/// Converts an binary image into svg xml data string
pub fn binary_image_to_svg(input_buffer: &BinaryImage, config: Config) -> String {
    let config = config.into_converter_config();

    let clusters = input_buffer.to_clusters(false);

    let mut svg = SvgFile::new(
        input_buffer.width,
        input_buffer.height,
        config.path_precision,
    );
    for i in 0..clusters.len() {
        let cluster = clusters.get_cluster(i);
        if cluster.size() >= config.filter_speckle_area {
            let paths = cluster.to_compound_path(
                config.mode,
                config.corner_threshold,
                config.length_threshold,
                config.max_iterations,
                config.splice_threshold,
            );
            svg.add_path(paths, Color::color(&ColorName::Black));
        }
    }

    format!("{}", svg)
}
