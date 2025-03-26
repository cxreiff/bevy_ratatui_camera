use ratatui::style::Color;

/// Options for restricting the terminal colors that rendered pixels are converted to.
///
/// Many terminals support 24-bit RGB colors, but some only support pre-defined sets of 16 or 256
/// ANSI colors. This enum represents each of those sets of possible colors when converting
/// rendered pixels to terminal characters.
///
/// Reference for terminal color support:
/// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
#[derive(Clone, Copy, Debug)]
pub enum TerminalColorRange {
    /// Any 24-bit color, represented by ratatui's `Color::Rgb` enum variant.
    Rgb,

    /// A color from a set of 256 pre-defined colors, referred to by index (ratatui's
    /// `Color::Indexed` enum variant.
    ANSI256,

    /// A color from a set of 16 pre-defined colors, referred to by name (ratatui's named enum
    /// variants, such as `Color::Cyan` or `Color::Magenta`).
    ANSI16,
}

pub fn color_for_color_range(color: Color, range: TerminalColorRange) -> Color {
    match range {
        TerminalColorRange::Rgb => color,
        TerminalColorRange::ANSI256 => color_to_ansi_256(color),
        TerminalColorRange::ANSI16 => color_to_ansi_16(color),
    }
}

fn color_to_ansi_256(color: Color) -> Color {
    let Color::Rgb(r, g, b) = color else {
        return color;
    };

    let index = color_rgb_to_ansi_index([r, g, b], &ANSI_COLORS);

    Color::Indexed(index)
}

fn color_to_ansi_16(color: Color) -> Color {
    let index = match color {
        Color::Rgb(r, g, b) => color_rgb_to_ansi_index([r, g, b], &ANSI_COLORS[..16]),
        Color::Indexed(index) => {
            color_rgb_to_ansi_index(ANSI_COLORS[index as usize], &ANSI_COLORS[..16])
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

const ANSI_COLORS: [[u8; 3]; 256] = [
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
    [0, 0, 0],
    [0, 0, 95],
    [0, 0, 135],
    [0, 0, 175],
    [0, 0, 215],
    [0, 0, 255],
    [0, 95, 0],
    [0, 95, 95],
    [0, 95, 135],
    [0, 95, 175],
    [0, 95, 215],
    [0, 95, 255],
    [0, 135, 0],
    [0, 135, 95],
    [0, 135, 135],
    [0, 135, 175],
    [0, 135, 215],
    [0, 135, 255],
    [0, 175, 0],
    [0, 175, 95],
    [0, 175, 135],
    [0, 175, 175],
    [0, 175, 215],
    [0, 175, 255],
    [0, 215, 0],
    [0, 215, 95],
    [0, 215, 135],
    [0, 215, 175],
    [0, 215, 215],
    [0, 215, 255],
    [0, 255, 0],
    [0, 255, 95],
    [0, 255, 135],
    [0, 255, 175],
    [0, 255, 215],
    [0, 255, 255],
    [95, 0, 0],
    [95, 0, 95],
    [95, 0, 135],
    [95, 0, 175],
    [95, 0, 215],
    [95, 0, 255],
    [95, 95, 0],
    [95, 95, 95],
    [95, 95, 135],
    [95, 95, 175],
    [95, 95, 215],
    [95, 95, 255],
    [95, 135, 0],
    [95, 135, 95],
    [95, 135, 135],
    [95, 135, 175],
    [95, 135, 215],
    [95, 135, 255],
    [95, 175, 0],
    [95, 175, 95],
    [95, 175, 135],
    [95, 175, 175],
    [95, 175, 215],
    [95, 175, 255],
    [95, 215, 0],
    [95, 215, 95],
    [95, 215, 135],
    [95, 215, 175],
    [95, 215, 215],
    [95, 215, 255],
    [95, 255, 0],
    [95, 255, 95],
    [95, 255, 135],
    [95, 255, 175],
    [95, 255, 215],
    [95, 255, 255],
    [135, 0, 0],
    [135, 0, 95],
    [135, 0, 135],
    [135, 0, 175],
    [135, 0, 215],
    [135, 0, 255],
    [135, 95, 0],
    [135, 95, 95],
    [135, 95, 135],
    [135, 95, 175],
    [135, 95, 215],
    [135, 95, 255],
    [135, 135, 0],
    [135, 135, 95],
    [135, 135, 135],
    [135, 135, 175],
    [135, 135, 215],
    [135, 135, 255],
    [135, 175, 0],
    [135, 175, 95],
    [135, 175, 135],
    [135, 175, 175],
    [135, 175, 215],
    [135, 175, 255],
    [135, 215, 0],
    [135, 215, 95],
    [135, 215, 135],
    [135, 215, 175],
    [135, 215, 215],
    [135, 215, 255],
    [135, 255, 0],
    [135, 255, 95],
    [135, 255, 135],
    [135, 255, 175],
    [135, 255, 215],
    [135, 255, 255],
    [175, 0, 0],
    [175, 0, 95],
    [175, 0, 135],
    [175, 0, 175],
    [175, 0, 215],
    [175, 0, 255],
    [175, 95, 0],
    [175, 95, 95],
    [175, 95, 135],
    [175, 95, 175],
    [175, 95, 215],
    [175, 95, 255],
    [175, 135, 0],
    [175, 135, 95],
    [175, 135, 135],
    [175, 135, 175],
    [175, 135, 215],
    [175, 135, 255],
    [175, 175, 0],
    [175, 175, 95],
    [175, 175, 135],
    [175, 175, 175],
    [175, 175, 215],
    [175, 175, 255],
    [175, 215, 0],
    [175, 215, 95],
    [175, 215, 135],
    [175, 215, 175],
    [175, 215, 215],
    [175, 215, 255],
    [175, 255, 0],
    [175, 255, 95],
    [175, 255, 135],
    [175, 255, 175],
    [175, 255, 215],
    [175, 255, 255],
    [215, 0, 0],
    [215, 0, 95],
    [215, 0, 135],
    [215, 0, 175],
    [215, 0, 215],
    [215, 0, 255],
    [215, 95, 0],
    [215, 95, 95],
    [215, 95, 135],
    [215, 95, 175],
    [215, 95, 215],
    [215, 95, 255],
    [215, 135, 0],
    [215, 135, 95],
    [215, 135, 135],
    [215, 135, 175],
    [215, 135, 215],
    [215, 135, 255],
    [215, 175, 0],
    [215, 175, 95],
    [215, 175, 135],
    [215, 175, 175],
    [215, 175, 215],
    [215, 175, 255],
    [215, 215, 0],
    [215, 215, 95],
    [215, 215, 135],
    [215, 215, 175],
    [215, 215, 215],
    [215, 215, 255],
    [215, 255, 0],
    [215, 255, 95],
    [215, 255, 135],
    [215, 255, 175],
    [215, 255, 215],
    [215, 255, 255],
    [255, 0, 0],
    [255, 0, 95],
    [255, 0, 135],
    [255, 0, 175],
    [255, 0, 215],
    [255, 0, 255],
    [255, 95, 0],
    [255, 95, 95],
    [255, 95, 135],
    [255, 95, 175],
    [255, 95, 215],
    [255, 95, 255],
    [255, 135, 0],
    [255, 135, 95],
    [255, 135, 135],
    [255, 135, 175],
    [255, 135, 215],
    [255, 135, 255],
    [255, 175, 0],
    [255, 175, 95],
    [255, 175, 135],
    [255, 175, 175],
    [255, 175, 215],
    [255, 175, 255],
    [255, 215, 0],
    [255, 215, 95],
    [255, 215, 135],
    [255, 215, 175],
    [255, 215, 215],
    [255, 215, 255],
    [255, 255, 0],
    [255, 255, 95],
    [255, 255, 135],
    [255, 255, 175],
    [255, 255, 215],
    [255, 255, 255],
    [8, 8, 8],
    [18, 18, 18],
    [28, 28, 28],
    [38, 38, 38],
    [48, 48, 48],
    [58, 58, 58],
    [68, 68, 68],
    [78, 78, 78],
    [88, 88, 88],
    [98, 98, 98],
    [108, 108, 108],
    [118, 118, 118],
    [128, 128, 128],
    [138, 138, 138],
    [148, 148, 148],
    [158, 158, 158],
    [168, 168, 168],
    [178, 178, 178],
    [188, 188, 188],
    [198, 198, 198],
    [208, 208, 208],
    [218, 218, 218],
    [228, 228, 228],
    [238, 238, 238],
];
