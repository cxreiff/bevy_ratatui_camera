use image::{DynamicImage, Rgb, Rgba};
use ratatui::style::Color;

use crate::RatatuiCameraEdgeDetection;

pub fn coords_from_index(index: usize, image: &DynamicImage) -> (u16, u16) {
    (
        index as u16 % image.width() as u16,
        index as u16 / image.width() as u16,
    )
}

pub fn replace_detected_edges(
    character: char,
    color: Color,
    sobel_value: &Rgba<u8>,
    edge_detection: &RatatuiCameraEdgeDetection,
) -> (char, Color) {
    let edge_color = edge_detection.edge_color.unwrap_or(color);
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
                (vertical, edge_color)
            } else if is_max_sobel(sobel_value[1]) {
                (horizontal, edge_color)
            } else if is_max_sobel(sobel_value[2]) {
                (forward_diagonal, edge_color)
            } else if is_max_sobel(sobel_value[3]) {
                (backward_diagonal, edge_color)
            } else {
                (character, color)
            }
        }
        crate::EdgeCharacters::Single(edge_character) => {
            if sobel_value.0.iter().any(|val| *val > 0) {
                (edge_character, edge_color)
            } else {
                (character, color)
            }
        }
    }
}

pub fn average_in_rgb(rgb_triplet: &[u8; 3], pixel: &Rgb<u8>) -> [u8; 3] {
    [
        ((rgb_triplet[0] as u16 + pixel[0] as u16) / 2) as u8,
        ((rgb_triplet[1] as u16 + pixel[1] as u16) / 2) as u8,
        ((rgb_triplet[2] as u16 + pixel[2] as u16) / 2) as u8,
    ]
}

pub fn average_in_rgba(rgba_quad: &[u8; 4], pixel: &Rgba<u8>) -> [u8; 4] {
    [
        ((rgba_quad[0] as u16 + pixel[0] as u16) / 2) as u8,
        ((rgba_quad[1] as u16 + pixel[1] as u16) / 2) as u8,
        ((rgba_quad[2] as u16 + pixel[2] as u16) / 2) as u8,
        ((rgba_quad[3] as u16 + pixel[3] as u16) / 2) as u8,
    ]
}
