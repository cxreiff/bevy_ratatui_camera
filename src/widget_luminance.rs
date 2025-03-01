use bevy::color::Luminance;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::{LuminanceConfig, RatatuiCameraEdgeDetection};

pub struct RatatuiCameraWidgetLuminance<'a> {
    camera_image: &'a DynamicImage,
    sobel_image: &'a Option<DynamicImage>,
    strategy_config: &'a LuminanceConfig,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetLuminance<'a> {
    pub fn new(
        camera_image: &'a DynamicImage,
        sobel_image: &'a Option<DynamicImage>,
        strategy_config: &'a LuminanceConfig,
        edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
    ) -> Self {
        Self {
            camera_image,
            sobel_image,
            strategy_config,
            edge_detection,
        }
    }
}

impl WidgetRef for RatatuiCameraWidgetLuminance<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let Self {
            camera_image,
            sobel_image,
            strategy_config,
            edge_detection,
        } = self;

        let camera_image = camera_image.resize(
            area.width as u32,
            area.height as u32 * 2,
            FilterType::Nearest,
        );

        let render_area = Rect {
            x: area.x + area.width.saturating_sub(camera_image.width() as u16) / 2,
            y: area.y + (area.height).saturating_sub(camera_image.height() as u16 / 2) / 2,
            width: camera_image.width() as u16,
            height: camera_image.height() as u16 / 2,
        };

        let color_characters = convert_image_to_color_characters(
            &camera_image,
            &strategy_config.luminance_characters,
            strategy_config.luminance_scale,
        );

        let sobel_image = sobel_image.as_ref().map(|sobel_image| {
            sobel_image.resize(
                area.width as u32,
                area.height as u32 * 2,
                FilterType::Nearest,
            )
        });

        for (index, &(mut character, mut color)) in color_characters.iter().enumerate() {
            let x = index as u16 % camera_image.width() as u16;
            let y = index as u16 / camera_image.width() as u16;
            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            if let (Some(sobel_image), Some(edge_detection)) = (&sobel_image, edge_detection) {
                if !sobel_image.in_bounds(x as u32, y as u32 * 2) {
                    continue;
                }

                let sobel_value = sobel_image.get_pixel(x as u32, y as u32 * 2);

                match edge_detection.edge_characters {
                    crate::EdgeCharacters::Directional {
                        vertical,
                        horizontal,
                        forward_diagonal,
                        backward_diagonal,
                    } => {
                        let is_max_sobel = |current: u8| {
                            sobel_value
                                .0
                                .iter()
                                .all(|val| (current > 0) && (current >= *val))
                        };

                        if is_max_sobel(sobel_value[0]) {
                            character = vertical;
                            color = edge_detection.edge_color.unwrap_or(color);
                        } else if is_max_sobel(sobel_value[1]) {
                            character = horizontal;
                            color = edge_detection.edge_color.unwrap_or(color);
                        } else if is_max_sobel(sobel_value[2]) {
                            character = forward_diagonal;
                            color = edge_detection.edge_color.unwrap_or(color);
                        } else if is_max_sobel(sobel_value[3]) {
                            character = backward_diagonal;
                            color = edge_detection.edge_color.unwrap_or(color);
                        }
                    }
                    crate::EdgeCharacters::Single(edge_character) => {
                        if sobel_value.0.iter().any(|val| *val > 0) {
                            character = edge_character;
                            color = edge_detection.edge_color.unwrap_or(color);
                        }
                    }
                }
            };

            if let Some(cell) = buf.cell_mut((render_area.x + x, render_area.y + y)) {
                cell.set_fg(color).set_char(character);
            }
        }
    }
}

fn convert_image_to_color_characters(
    camera_image: &DynamicImage,
    luminance_characters: &[char],
    luminance_scale: f32,
) -> Vec<(char, Color)> {
    let rgb_triplets = convert_image_to_rgb_triplets(camera_image);
    let characters = rgb_triplets
        .iter()
        .map(|rgb| convert_rgb_triplet_to_character(rgb, luminance_characters, luminance_scale));
    let colors = rgb_triplets
        .iter()
        .map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2]));

    characters.zip(colors).collect()
}

fn convert_image_to_rgb_triplets(camera_image: &DynamicImage) -> Vec<[u8; 3]> {
    let mut rgb_triplets =
        vec![[0; 3]; (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for (y, row) in camera_image.to_rgb8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (camera_image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgb_triplets[position] = pixel.0;
            } else {
                rgb_triplets[position][0] =
                    (rgb_triplets[position][0].saturating_add(pixel[0])) / 2;
                rgb_triplets[position][1] =
                    (rgb_triplets[position][1].saturating_add(pixel[1])) / 2;
                rgb_triplets[position][2] =
                    (rgb_triplets[position][2].saturating_add(pixel[2])) / 2;
            }
        }
    }

    rgb_triplets
}

fn convert_rgb_triplet_to_character(
    rgb_triplet: &[u8; 3],
    luminance_characters: &[char],
    luminance_scale: f32,
) -> char {
    let luminance =
        bevy::color::Color::srgb_u8(rgb_triplet[0], rgb_triplet[1], rgb_triplet[2]).luminance();
    let scaled_luminance = (luminance * luminance_scale).min(1.0);
    let character_index = ((scaled_luminance * luminance_characters.len() as f32) as usize)
        .min(luminance_characters.len() - 1);

    let Some(character) = luminance_characters.get(character_index) else {
        return ' ';
    };

    *character
}
