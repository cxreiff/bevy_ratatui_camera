use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::RatatuiCameraEdgeDetection;
use crate::camera_strategy::HalfBlocksConfig;
use crate::color_support::color_for_color_support;

pub struct RatatuiCameraWidgetHalf<'a> {
    camera_image: &'a DynamicImage,
    sobel_image: &'a Option<DynamicImage>,
    strategy_config: &'a HalfBlocksConfig,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetHalf<'a> {
    pub fn new(
        camera_image: &'a DynamicImage,
        sobel_image: &'a Option<DynamicImage>,
        strategy_config: &'a HalfBlocksConfig,
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

impl WidgetRef for RatatuiCameraWidgetHalf<'_> {
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

        let cell_candidates = convert_image_to_cell_candidates(&camera_image, strategy_config);

        let sobel_image = sobel_image.as_ref().map(|sobel_image| {
            sobel_image.resize(
                area.width as u32,
                area.height as u32 * 2,
                FilterType::Nearest,
            )
        });

        for (index, (mut bg, mut fg)) in cell_candidates.enumerate() {
            let x = index as u16 % camera_image.width() as u16;
            let y = index as u16 / camera_image.width() as u16;
            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            let Some(cell) = buf.cell_mut((render_area.x + x, render_area.y + y)) else {
                continue;
            };

            let mut character = '▄';

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
                            fg = edge_detection.edge_color.unwrap_or(fg);
                        } else if is_max_sobel(sobel_value[1]) {
                            character = horizontal;
                            fg = edge_detection.edge_color.unwrap_or(fg);
                        } else if is_max_sobel(sobel_value[2]) {
                            character = forward_diagonal;
                            fg = edge_detection.edge_color.unwrap_or(fg);
                        } else if is_max_sobel(sobel_value[3]) {
                            character = backward_diagonal;
                            fg = edge_detection.edge_color.unwrap_or(fg);
                        }
                    }
                    crate::EdgeCharacters::Single(edge_character) => {
                        if sobel_value.0.iter().any(|val| *val > 0) {
                            character = edge_character;
                            fg = edge_detection.edge_color.unwrap_or(fg);
                        }
                    }
                }
            };

            if !matches!(bg, Color::Reset) {
                bg = color_for_color_support(bg, strategy_config.color_support);
                cell.set_bg(bg);
            };

            if !matches!(fg, Color::Reset) {
                fg = color_for_color_support(fg, strategy_config.color_support);
                cell.set_fg(fg);
            };

            if !matches!(bg, Color::Reset) && !matches!(fg, Color::Reset) {
                cell.set_char(character);
            };
        }
    }
}

fn convert_image_to_cell_candidates(
    camera_image: &DynamicImage,
    strategy_config: &HalfBlocksConfig,
) -> impl Iterator<Item = (Color, Color)> {
    let rgba_quads = convert_image_to_rgba_quads(camera_image);

    rgba_quads.into_iter().map(move |rgbas| {
        let bg = if strategy_config.transparent && rgbas[0][3] == 0 {
            Color::Reset
        } else {
            Color::Rgb(rgbas[0][0], rgbas[0][1], rgbas[0][2])
        };
        let fg = if strategy_config.transparent && rgbas[1][3] == 0 {
            Color::Reset
        } else {
            Color::Rgb(rgbas[1][0], rgbas[1][1], rgbas[1][2])
        };

        (bg, fg)
    })
}

fn convert_image_to_rgba_quads(camera_image: &DynamicImage) -> Vec<[[u8; 4]; 2]> {
    let mut rgba_quad =
        vec![[[0; 4]; 2]; (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for (y, row) in camera_image.to_rgba8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (camera_image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgba_quad[position][0] = pixel.0;
            } else {
                rgba_quad[position][1] = pixel.0;
            }
        }
    }

    rgba_quad
}
