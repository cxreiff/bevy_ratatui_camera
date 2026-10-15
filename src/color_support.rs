use ratatui::style::Color;

const ANSI_COLORS_16: [[u8; 3]; 16] = [
    [0, 0, 0],
    [128, 0, 0],
    [0, 128, 0],
    [128, 128, 0],
    [0, 0, 128],
    [128, 0, 128],
    [0, 128, 128],
    [192, 192, 192],
    [128, 128, 128],
    [255, 0, 0],
    [0, 255, 0],
    [255, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
    [0, 255, 255],
    [255, 255, 255],
];

const fn generate_ansi_colors_256() -> [[u8; 3]; 256] {
    let mut colors = [[0; 3]; 256];

    // first 16 colors are predefined.
    let mut i = 0;
    while i < 16 {
        colors[i] = ANSI_COLORS_16[i];
        i += 1;
    }

    // xterm's 6x6x6 color cube uses nonuniform channel levels.
    let levels = [0, 95, 135, 175, 215, 255];
    let mut r: u8 = 0;
    while r < 6 {
        let mut g: u8 = 0;
        while g < 6 {
            let mut b: u8 = 0;
            while b < 6 {
                let index = 16 + (r * 36) + (g * 6) + b;
                colors[index as usize] =
                    [levels[r as usize], levels[g as usize], levels[b as usize]];
                b += 1;
            }
            g += 1;
        }
        r += 1;
    }

    // next 24 colors are grayscale
    let mut i = 232;
    while i < 256 {
        let gray = 8 + (i as u8 - 232) * 10;
        colors[i] = [gray, gray, gray];
        i += 1;
    }

    colors
}

const ANSI_COLORS_256: [[u8; 3]; 256] = generate_ansi_colors_256();

/// Options for restricting the terminal colors that rendered pixels are converted to.
///
/// Many terminals support 24-bit RGB "true color", but some only support pre-defined sets of 16 or
/// 256 ANSI colors. This enum represents each of those sets of possible colors when converting
/// rendered pixels to terminal characters.
///
/// Reference for terminal color support:
/// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
#[derive(Clone, Copy, Debug, Default)]
pub enum ColorSupport {
    /// Any 24-bit color, represented by ratatui's `Color::Rgb` enum variant.
    #[default]
    TrueColor,

    /// A color from a set of 256 pre-defined colors, referred to by index (ratatui's
    /// `Color::Indexed` enum variant.
    ANSI256,

    /// A color from a set of 16 pre-defined colors, referred to by name (ratatui's named enum
    /// variants, such as `Color::Cyan` or `Color::Magenta`).
    ANSI16,
}

pub fn color_for_color_support(color: Option<Color>, support: ColorSupport) -> Option<Color> {
    color.map(|color| match support {
        ColorSupport::TrueColor => color,
        ColorSupport::ANSI256 => color_to_ansi_256(color),
        ColorSupport::ANSI16 => color_to_ansi_16(color),
    })
}

fn color_to_ansi_256(color: Color) -> Color {
    let Color::Rgb(r, g, b) = color else {
        return color;
    };

    let index = color_rgb_to_ansi_index([r, g, b], &ANSI_COLORS_256);

    Color::Indexed(index)
}

fn color_to_ansi_16(color: Color) -> Color {
    let index = match color {
        Color::Rgb(r, g, b) => color_rgb_to_ansi_index([r, g, b], &ANSI_COLORS_16),
        Color::Indexed(index) => {
            color_rgb_to_ansi_index(ANSI_COLORS_256[index as usize], &ANSI_COLORS_16)
        }
        _ => return color,
    };

    ratatui_color_from_ansi_index(index)
}

fn color_rgb_to_ansi_index(color: [u8; 3], colors: &[[u8; 3]]) -> u8 {
    colors
        .iter()
        .enumerate()
        .min_by(|&(_, &a), &(_, &b)| {
            color_distance(a, color)
                .partial_cmp(&color_distance(b, color))
                .unwrap()
        })
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

fn color_distance(a: [u8; 3], b: [u8; 3]) -> f64 {
    let [a_r, a_g, a_b] = a;
    let [b_r, b_g, b_b] = b;

    let d_r = (a_r as i32 - b_r as i32).pow(2);
    let d_g = (a_g as i32 - b_g as i32).pow(2);
    let d_b = (a_b as i32 - b_b as i32).pow(2);

    ((d_r + d_g + d_b) as f64).sqrt()
}

const fn ratatui_color_from_ansi_index(index: u8) -> Color {
    match index {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        16..=255 => Color::Indexed(index),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xterm_cube_colors_select_their_exact_indices() {
        for (rgb, index) in [
            ([95, 0, 0], 52),
            ([0, 135, 0], 28),
            ([0, 0, 175], 19),
            ([215, 95, 135], 168),
        ] {
            assert_eq!(
                color_to_ansi_256(Color::Rgb(rgb[0], rgb[1], rgb[2])),
                Color::Indexed(index)
            );
        }
    }

    #[test]
    fn named_indexed_and_true_colors_keep_their_contracts() {
        assert_eq!(color_to_ansi_256(Color::Cyan), Color::Cyan);
        assert_eq!(color_to_ansi_256(Color::Indexed(52)), Color::Indexed(52));
        assert_eq!(
            color_for_color_support(Some(Color::Rgb(95, 0, 0)), ColorSupport::TrueColor),
            Some(Color::Rgb(95, 0, 0))
        );
        assert_eq!(color_to_ansi_256(Color::Rgb(8, 8, 8)), Color::Indexed(232));
        assert_eq!(color_to_ansi_16(Color::Indexed(52)), Color::Red);
    }
}
