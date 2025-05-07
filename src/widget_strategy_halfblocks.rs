use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::RatatuiCameraEdgeDetection;
use crate::camera_strategy::HalfBlocksConfig;
use crate::color_support::color_for_color_support;
use crate::widget_utilities::{coords_from_index, replace_detected_edges};

pub struct RatatuiCameraWidgetHalf<'a> {
    camera_image: DynamicImage,
    depth_image: DynamicImage,
    sobel_image: Option<DynamicImage>,
    strategy_config: &'a HalfBlocksConfig,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetHalf<'a> {
    pub fn new(
        camera_image: DynamicImage,
        depth_image: DynamicImage,
        sobel_image: Option<DynamicImage>,
        strategy_config: &'a HalfBlocksConfig,
        edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
    ) -> Self {
        Self {
            camera_image,
            depth_image,
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
            depth_image,
            sobel_image,
            strategy_config,
            edge_detection,
        } = self;

        let cell_candidates = convert_image_to_cell_candidates(camera_image, strategy_config);

        for (index, (mut bg, mut fg)) in cell_candidates.enumerate() {
            let mut character = '▄';
            let (x, y) = coords_from_index(index, camera_image);

            if x >= area.width || y >= area.height {
                continue;
            }

            let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) else {
                continue;
            };

            if x == 0 && y == 0 {
                let depth_bytes = depth_image.get_pixel(x as u32, y as u32);
                let _depth_value = f32::from_le_bytes(depth_bytes.0);
                // TODO: update a buffer of depth values per terminal cell.
            }

            if let (Some(sobel_image), Some(edge_detection)) = (&sobel_image, edge_detection) {
                if !sobel_image.in_bounds(x as u32, y as u32 * 2) {
                    continue;
                }

                let sobel_value = sobel_image.get_pixel(x as u32, y as u32 * 2);

                (character, fg) =
                    replace_detected_edges(character, fg, &sobel_value, edge_detection);
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
    let mut rgba_quad_pairs =
        vec![[[0; 4]; 2]; (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for (y, row) in camera_image.to_rgba8().rows().enumerate() {
        for (x, pixel) in row.enumerate() {
            let position = x + (camera_image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgba_quad_pairs[position][0] = pixel.0;
            } else {
                rgba_quad_pairs[position][1] = pixel.0;
            }
        }
    }

    rgba_quad_pairs
}
