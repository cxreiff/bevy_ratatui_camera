use image::{DynamicImage, GenericImageView};
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

use crate::RatatuiCameraEdgeDetection;
use crate::widget_utilities::{
    average_in_rgb, calculate_render_area, coords_from_index, replace_detected_edges,
    resize_image_to_area,
};

pub struct RatatuiCameraWidgetNone<'a> {
    camera_image: &'a DynamicImage,
    sobel_image: &'a Option<DynamicImage>,
    edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
}

impl<'a> RatatuiCameraWidgetNone<'a> {
    pub fn new(
        camera_image: &'a DynamicImage,
        sobel_image: &'a Option<DynamicImage>,
        edge_detection: &'a Option<RatatuiCameraEdgeDetection>,
    ) -> Self {
        Self {
            camera_image,
            sobel_image,
            edge_detection,
        }
    }
}

impl WidgetRef for RatatuiCameraWidgetNone<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let Self {
            camera_image,
            sobel_image,
            edge_detection,
        } = self;

        let (Some(sobel_image), Some(edge_detection)) = (sobel_image, edge_detection) else {
            return;
        };

        let camera_image = resize_image_to_area(area, camera_image);

        let render_area = calculate_render_area(area, &camera_image);

        let mut color_characters = convert_image_to_colors(&camera_image);

        let sobel_image = resize_image_to_area(area, sobel_image);

        for (index, color) in color_characters.iter_mut().enumerate() {
            let mut character = ' ';
            let (x, y) = coords_from_index(index, &camera_image);

            if x >= render_area.width || y >= render_area.height {
                continue;
            }

            if !sobel_image.in_bounds(x as u32, y as u32 * 2) {
                continue;
            }

            let sobel_value = sobel_image.get_pixel(x as u32, y as u32 * 2);

            (character, *color) =
                replace_detected_edges(character, *color, &sobel_value, edge_detection);

            if let Some(cell) = buf.cell_mut((render_area.x + x, render_area.y + y)) {
                cell.set_fg(*color).set_char(character);
            }
        }
    }
}

fn convert_image_to_colors(camera_image: &DynamicImage) -> Vec<Color> {
    let rgb_triplets = convert_image_to_rgb_triplets(camera_image);
    let colors = rgb_triplets
        .iter()
        .map(|rgb| Color::Rgb(rgb[0], rgb[1], rgb[2]));

    colors.collect()
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
                rgb_triplets[position] = average_in_rgb(&rgb_triplets[position], pixel);
            }
        }
    }

    rgb_triplets
}
