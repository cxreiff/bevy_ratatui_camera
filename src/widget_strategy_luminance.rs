use bevy::color::Luminance;
use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::color_support::color_for_color_support;
use crate::widget_utilities::{average_in_rgba, coords_from_index, replace_detected_edges};
use crate::{ColorSupport, LuminanceConfig, RatatuiCameraEdgeDetection};

pub struct RatatuiCameraWidgetLuminance<'a> {
    camera_image: DynamicImage,
    sobel_image: Option<DynamicImage>,
    strategy_config: &'a LuminanceConfig,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetLuminance<'a> {
    pub fn new(
        camera_image: DynamicImage,
        sobel_image: Option<DynamicImage>,
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

        let cell_candidates = convert_image_to_cell_candidates(
            camera_image,
            &strategy_config.luminance_characters,
            strategy_config.luminance_scale,
        );

        for (index, (mut character, mut color)) in cell_candidates.enumerate() {
            let (x, y) = coords_from_index(index, camera_image);

            if x >= area.width || y >= area.height {
                continue;
            }

            let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) else {
                continue;
            };

            if let (Some(sobel_image), Some(edge_detection)) = (&sobel_image, edge_detection) {
                if !sobel_image.in_bounds(x as u32, y as u32 * 2) {
                    continue;
                }

                let sobel_value = sobel_image.get_pixel(x as u32, y as u32 * 2);

                (character, color) =
                    replace_detected_edges(character, color, &sobel_value, edge_detection);
            };

            if strategy_config.transparent && matches!(color, Color::Reset) {
                continue;
            }

            color = color_for_color_support(color, strategy_config.color_support);

            if strategy_config.bg_color_scale > 0.0 {
                if let ColorSupport::TrueColor = strategy_config.color_support {
                    if let Color::Rgb(r, g, b) = color {
                        let bg = Color::Rgb(
                            (r as f32 * strategy_config.bg_color_scale) as u8,
                            (g as f32 * strategy_config.bg_color_scale) as u8,
                            (b as f32 * strategy_config.bg_color_scale) as u8,
                        );
                        cell.set_bg(bg);
                    }
                }
            }

            cell.set_fg(color).set_char(character);
        }
    }
}

fn convert_image_to_cell_candidates(
    camera_image: &DynamicImage,
    luminance_characters: &[char],
    luminance_scale: f32,
) -> impl Iterator<Item = (char, Color)> {
    let rgba_quads = convert_image_to_rgba_quads(camera_image);

    rgba_quads.into_iter().map(move |rgba| {
        let character =
            convert_rgba_quads_to_character(&rgba, luminance_characters, luminance_scale);
        let color = if rgba[3] == 0 {
            Color::Reset
        } else {
            Color::Rgb(rgba[0], rgba[1], rgba[2])
        };
        (character, color)
    })
}

fn convert_image_to_rgba_quads(camera_image: &DynamicImage) -> Vec<[u8; 4]> {
    let mut rgba_quads =
        vec![[0; 4]; (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for (y, row) in camera_image.to_rgba8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (camera_image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgba_quads[position] = pixel.0;
            } else {
                rgba_quads[position] = average_in_rgba(&rgba_quads[position], pixel);
            }
        }
    }

    rgba_quads
}

fn convert_rgba_quads_to_character(
    rgba_quad: &[u8; 4],
    luminance_characters: &[char],
    luminance_scale: f32,
) -> char {
    let luminance =
        bevy::color::Color::srgba_u8(rgba_quad[0], rgba_quad[1], rgba_quad[2], rgba_quad[3])
            .luminance();
    let scaled_luminance = (luminance * luminance_scale).min(1.0);
    let character_index = ((scaled_luminance * luminance_characters.len() as f32) as usize)
        .min(luminance_characters.len() - 1);

    let Some(character) = luminance_characters.get(character_index) else {
        return ' ';
    };

    *character
}
