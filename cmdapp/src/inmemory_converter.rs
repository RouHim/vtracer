use std::path::PathBuf;
use std::{fs::File, io::Write};
use image::{ImageBuffer, Rgb};

use visioncortex::color_clusters::{Runner, RunnerConfig, HIERARCHICAL_MAX};
use visioncortex::{BinaryImage, Color, ColorImage, ColorName};

use super::config::{ColorMode, Config, ConverterConfig, Hierarchical};
use super::svg::SvgFile;

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


/// Converts an color image into svg xml data string
pub fn color_image_to_svg(input_buffer: ImageBuffer<Rgb<u8>, Vec<u8>>, config: Config) -> String {
    let config = config.into_converter_config();
    let width = input_buffer.width() as usize;
    let height = input_buffer.height() as usize;
    let img = ColorImage {
        pixels: input_buffer.as_raw().to_vec(),
        width,
        height,
    };

    let runner = Runner::new(
        RunnerConfig {
            diagonal: config.layer_difference == 0,
            hierarchical: HIERARCHICAL_MAX,
            batch_size: 25600,
            good_min_area: config.filter_speckle_area,
            good_max_area: (width * height),
            is_same_color_a: config.color_precision_loss,
            is_same_color_b: 1,
            deepen_diff: config.layer_difference,
            hollow_neighbours: 1,
        },
        img,
    );

    let mut clusters = runner.run();

    match config.hierarchical {
        Hierarchical::Stacked => {}
        Hierarchical::Cutout => {
            let view = clusters.view();
            let image = view.to_color_image();
            let runner = Runner::new(
                RunnerConfig {
                    diagonal: false,
                    hierarchical: 64,
                    batch_size: 25600,
                    good_min_area: 0,
                    good_max_area: (image.width * image.height),
                    is_same_color_a: 0,
                    is_same_color_b: 1,
                    deepen_diff: 0,
                    hollow_neighbours: 0,
                },
                image,
            );
            clusters = runner.run();
        }
    }

    let view = clusters.view();

    let mut svg = SvgFile::new(width, height, config.path_precision);
    for &cluster_index in view.clusters_output.iter().rev() {
        let cluster = view.get_cluster(cluster_index);
        let paths = cluster.to_compound_path(
            &view,
            false,
            config.mode,
            config.corner_threshold,
            config.length_threshold,
            config.max_iterations,
            config.splice_threshold,
        );
        svg.add_path(paths, cluster.residue_color());
    }

    format!("{}", svg)
}
