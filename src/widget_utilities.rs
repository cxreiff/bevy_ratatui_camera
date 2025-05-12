use image::{DynamicImage, Rgb, Rgba};
use ratatui::style::Color;

use crate::{ColorChoice, RatatuiCameraEdgeDetection};

pub fn coords_from_index(index: usize, image: &DynamicImage) -> (u16, u16) {
    (
        index as u16 % image.width() as u16,
        index as u16 / image.width() as u16,
    )
}

pub fn replace_detected_edges(
    character: char,
    fg: Option<Color>,
    sobel_value: &Rgba<u8>,
    edge_detection: &RatatuiCameraEdgeDetection,
) -> (char, Option<Color>) {
    let edge_color = edge_detection.edge_color.or(fg);
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
                (character, fg)
            }
        }
        crate::EdgeCharacters::Single(edge_character) => {
            if sobel_value.0.iter().any(|val| *val > 0) {
                (edge_character, edge_color)
            } else {
                (character, fg)
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

pub fn colors_for_color_choices(
    fg: Option<Color>,
    bg: Option<Color>,
    fg_color_choice: &Option<ColorChoice>,
    bg_color_choice: &Option<ColorChoice>,
) -> (Option<Color>, Option<Color>) {
    let new_fg = if let Some(color_choice) = fg_color_choice {
        color_for_color_choice(fg, bg, color_choice)
    } else {
        fg
    };

    let new_bg = if let Some(color_choice) = bg_color_choice {
        color_for_color_choice(fg, bg, color_choice)
    } else {
        bg
    };

    (new_fg, new_bg)
}

fn color_for_color_choice(
    fg: Option<Color>,
    bg: Option<Color>,
    color_choice: &ColorChoice,
) -> Option<Color> {
    match color_choice {
        ColorChoice::Color(color) => Some(*color),
        ColorChoice::Scale(scale) => match fg {
            Some(Color::Rgb(r, g, b)) => Some(Color::Rgb(
                (r as f32 * scale).min(u8::MAX as f32) as u8,
                (g as f32 * scale).min(u8::MAX as f32) as u8,
                (b as f32 * scale).min(u8::MAX as f32) as u8,
            )),
            _ => None,
        },
        ColorChoice::Callback(callback) => callback(fg, bg),
    }
}
