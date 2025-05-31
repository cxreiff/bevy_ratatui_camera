use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;

use crate::camera_strategy::DepthConfig;
use crate::color_support::color_for_color_support;
use crate::widget_utilities::{
    average_in_rgba, colors_for_color_choices, coords_from_index, replace_detected_edges,
};
use crate::{RatatuiCameraDepthBuffer, RatatuiCameraEdgeDetection};

#[derive(Debug)]
pub struct RatatuiCameraWidgetDepth<'a> {
    camera_image: DynamicImage,
    depth_image: Option<DynamicImage>,
    sobel_image: Option<DynamicImage>,
    depth_buffer: Option<&'a mut RatatuiCameraDepthBuffer>,
    strategy_config: &'a DepthConfig,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetDepth<'a> {
    pub fn new(
        camera_image: DynamicImage,
        depth_image: Option<DynamicImage>,
        sobel_image: Option<DynamicImage>,
        depth_buffer: Option<&'a mut RatatuiCameraDepthBuffer>,
        strategy_config: &'a DepthConfig,
        edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
    ) -> Self {
        Self {
            camera_image,
            depth_image,
            sobel_image,
            depth_buffer,
            strategy_config,
            edge_detection,
        }
    }
}

impl Widget for &mut RatatuiCameraWidgetDepth<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(ref depth_image) = self.depth_image else {
            return;
        };

        let cell_candidates = convert_image_to_cell_candidates(
            &self.camera_image,
            depth_image,
            &self.strategy_config.characters.list,
            self.strategy_config.characters.scale,
        );

        for (index, (mut character, mut fg)) in cell_candidates.enumerate() {
            let mut bg = None;
            let (x, y) = coords_from_index(index, &self.camera_image);

            if x >= area.width || y >= area.height {
                continue;
            }

            let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) else {
                continue;
            };

            if let (Some(depth_image), Some(depth_buffer)) =
                (&self.depth_image, &mut self.depth_buffer)
            {
                if depth_buffer
                    .compare_and_update_from_image(x as u32, y as u32 * 2, depth_image)
                    .is_none_or(|draw| !draw)
                {
                    continue;
                }
                if depth_buffer
                    .compare_and_update_from_image(x as u32, y as u32 * 2 + 1, depth_image)
                    .is_none_or(|draw| !draw)
                {
                    continue;
                }
            }

            if let (Some(sobel_image), Some(edge_detection)) =
                (&self.sobel_image, self.edge_detection)
            {
                if !sobel_image.in_bounds(x as u32, y as u32 * 2) {
                    continue;
                }

                let sobel_value = sobel_image.get_pixel(x as u32, y as u32 * 2);

                (character, fg) =
                    replace_detected_edges(character, fg, &sobel_value, edge_detection);
            };

            (fg, bg) = colors_for_color_choices(
                fg,
                bg,
                &self.strategy_config.colors.foreground,
                &self.strategy_config.colors.background,
            );

            if self.strategy_config.common.transparent && fg.is_none() {
                continue;
            }

            fg = color_for_color_support(fg, self.strategy_config.colors.support);
            bg = color_for_color_support(bg, self.strategy_config.colors.support);

            fg.map(|fg| cell.set_fg(fg).set_char(character));
            bg.map(|bg| cell.set_bg(bg));
        }
    }
}

fn convert_image_to_cell_candidates(
    camera_image: &DynamicImage,
    depth_image: &DynamicImage,
    depth_characters: &[char],
    depth_scale: f32,
) -> impl Iterator<Item = (char, Option<Color>)> {
    let rgba_quads = convert_image_to_rgba_quads(camera_image, depth_image);

    rgba_quads.into_iter().map(move |(rgba, depth)| {
        let character = convert_depth_to_character(depth, depth_characters, depth_scale);
        let color = if rgba[3] == 0 || depth == 0.0 {
            None
        } else {
            Some(Color::Rgb(rgba[0], rgba[1], rgba[2]))
        };
        (character, color)
    })
}

fn convert_image_to_rgba_quads(
    camera_image: &DynamicImage,
    depth_image: &DynamicImage,
) -> Vec<([u8; 4], f32)> {
    let mut rgba_quads =
        vec![([0; 4], 0.0); (camera_image.width() * camera_image.height().div_ceil(2)) as usize];

    for ((y, row), depth_row) in camera_image
        .to_rgba8()
        .rows()
        .enumerate()
        .zip(depth_image.to_rgba8().rows())
    {
        for ((x, pixel), depth) in row.enumerate().zip(depth_row) {
            let position = x + (camera_image.width() as usize) * (y / 2);
            if y % 2 == 0 {
                rgba_quads[position].0 = pixel.0;
            } else {
                rgba_quads[position].0 = average_in_rgba(&rgba_quads[position].0, pixel);
            }
            rgba_quads[position].1 = f32::from_le_bytes(depth.0);
        }
    }

    rgba_quads
}

fn convert_depth_to_character(depth: f32, depth_characters: &[char], depth_scale: f32) -> char {
    let scaled_depth = (depth * depth_scale).min(1.0);
    let character_index =
        ((scaled_depth * depth_characters.len() as f32) as usize).min(depth_characters.len() - 1);

    let Some(character) = depth_characters.get(character_index) else {
        return ' ';
    };

    *character
}
