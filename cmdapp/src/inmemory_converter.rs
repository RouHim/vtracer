use crate::{find_unused_color_in_image, should_key_image};
use image::RgbaImage;
use visioncortex::color_clusters::{KeyingAction, Runner, RunnerConfig, HIERARCHICAL_MAX};
use visioncortex::{BinaryImage, Color, ColorImage, ColorName};

use super::config::{Config, Hierarchical};
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
pub fn color_image_to_svg(input_image: RgbaImage, config: Config) -> String {
    let config = config.into_converter_config();
    let width = input_image.width() as usize;
    let height = input_image.height() as usize;
    let mut img = ColorImage {
        pixels: input_image.as_raw().to_vec(),
        width,
        height,
    };

    let key_color = if should_key_image(&img) {
        let key_color = find_unused_color_in_image(&img).unwrap();
        for y in 0..height {
            for x in 0..width {
                if img.get_pixel(x, y).a == 0 {
                    img.set_pixel(x, y, &key_color);
                }
            }
        }
        key_color
    } else {
        // The default color is all zeroes, which is treated by visioncortex as a special value meaning no keying will be applied.
        Color::default()
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
            key_color,
            keying_action: if matches!(config.hierarchical, Hierarchical::Cutout) {
                KeyingAction::Keep
            } else {
                KeyingAction::Discard
            },
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
                    good_max_area: (image.width * image.height) as usize,
                    is_same_color_a: 0,
                    is_same_color_b: 1,
                    deepen_diff: 0,
                    hollow_neighbours: 0,
                    key_color: Color::default(),
                    keying_action: if matches!(config.hierarchical, Hierarchical::Cutout) {
                        KeyingAction::Keep
                    } else {
                        KeyingAction::Discard
                    },
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
