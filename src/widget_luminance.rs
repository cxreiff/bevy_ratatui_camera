use bevy::color::Luminance;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::color_support::color_for_color_support;
use crate::{ColorSupport, LuminanceConfig, RatatuiCameraEdgeDetection};

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

        let cell_candidates = convert_image_to_cell_candidates(
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

        for (index, (mut character, mut color, mut skip)) in cell_candidates.enumerate() {
            let x = index as u16 % camera_image.width() as u16;
            let y = index as u16 / camera_image.width() as u16;
            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            let Some(cell) = buf.cell_mut((render_area.x + x, render_area.y + y)) else {
                continue;
            };

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
                            skip = false;
                        } else if is_max_sobel(sobel_value[1]) {
                            character = horizontal;
                            color = edge_detection.edge_color.unwrap_or(color);
                            skip = false;
                        } else if is_max_sobel(sobel_value[2]) {
                            character = forward_diagonal;
                            color = edge_detection.edge_color.unwrap_or(color);
                            skip = false;
                        } else if is_max_sobel(sobel_value[3]) {
                            character = backward_diagonal;
                            color = edge_detection.edge_color.unwrap_or(color);
                            skip = false;
                        }
                    }
                    crate::EdgeCharacters::Single(edge_character) => {
                        if sobel_value.0.iter().any(|val| *val > 0) {
                            character = edge_character;
                            color = edge_detection.edge_color.unwrap_or(color);
                            skip = false;
                        }
                    }
                }
            };

            if strategy_config.transparent && skip {
                continue;
            }

            color = color_for_color_support(color, strategy_config.color_support);

            if strategy_config.bg_color_scale > 0.0 {
                if let ColorSupport::TrueColor = strategy_config.color_support {
                    if let Color::Rgb(r, g, b) = color {
                        let bg = Color::Rgb(
                            (r as f32 * 0.4) as u8,
                            (g as f32 * 0.4) as u8,
                            (b as f32 * 0.4) as u8,
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
) -> impl Iterator<Item = (char, Color, bool)> {
    let rgba_quads = convert_image_to_rgba_quads(camera_image);

    rgba_quads.into_iter().map(move |rgba| {
        let character =
            convert_rgba_quads_to_character(&rgba, luminance_characters, luminance_scale);
        let color = Color::Rgb(rgba[0], rgba[1], rgba[2]);
        let skip = rgba[3] == 0;
        (character, color, skip)
    })
}

fn convert_image_to_rgba_quads(camera_image: &DynamicImage) -> Vec<[u8; 4]> {
    let mut rgba_quad =
        vec![[0; 4]; (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for (y, row) in camera_image.to_rgba8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (camera_image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgba_quad[position] = pixel.0;
            } else {
                rgba_quad[position][0] =
                    ((rgba_quad[position][0] as u16 + pixel[0] as u16) / 2) as u8;
                rgba_quad[position][1] =
                    ((rgba_quad[position][1] as u16 + pixel[1] as u16) / 2) as u8;
                rgba_quad[position][2] =
                    ((rgba_quad[position][2] as u16 + pixel[2] as u16) / 2) as u8;
                rgba_quad[position][3] =
                    ((rgba_quad[position][3] as u16 + pixel[3] as u16) / 2) as u8;
            }
        }
    }

    rgba_quad
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
