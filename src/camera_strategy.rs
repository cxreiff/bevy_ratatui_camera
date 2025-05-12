use std::{fmt::Debug, sync::Arc};

use bevy::prelude::*;

use crate::color_support::ColorSupport;

/// Specify the strategy used for converting the camera's rendered image to unicode characters for
/// the terminal buffer. Insert a variant of this component alongside your `RatatuiCamera` to
/// change the default behavior.
///
#[derive(Component, Clone, Debug)]
pub enum RatatuiCameraStrategy {
    /// Print to the terminal using unicode halfblock characters. By using both the halfblock
    /// (foreground) color and the background color, we can draw two pixels per buffer cell.
    HalfBlocks(HalfBlocksConfig),

    /// Given a range of unicode characters sorted in increasing order of opacity, use each pixel's
    /// luminance to select a character from the range.
    Luminance(LuminanceConfig),

    /// Does not print characters by itself, but edge detection will still print. Use with edge
    /// detection for a "wireframe".
    None,
}

impl Default for RatatuiCameraStrategy {
    fn default() -> Self {
        Self::halfblocks()
    }
}

impl RatatuiCameraStrategy {
    /// Halfblocks strategy using unicode halfblock characters, and the foreground and background
    /// colors of each cell.
    pub fn halfblocks() -> Self {
        Self::HalfBlocks(HalfBlocksConfig::default())
    }

    /// Luminance strategy with a provided list of characters.
    pub fn luminance_with_characters(characters: &[char]) -> Self {
        Self::Luminance(LuminanceConfig {
            luminance_characters: characters.into(),
            ..default()
        })
    }

    /// Luminance strategy with a range of braille unicode characters in increasing order of opacity.
    pub fn luminance_braille() -> Self {
        Self::Luminance(LuminanceConfig {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_BRAILLE.into(),
            ..default()
        })
    }

    /// Luminance strategy with a range of miscellaneous characters in increasing order of opacity.
    pub fn luminance_misc() -> Self {
        Self::Luminance(LuminanceConfig {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_MISC.into(),
            ..default()
        })
    }

    /// Luminance strategy with a range of block characters in increasing order of opacity.
    pub fn luminance_shading() -> Self {
        Self::Luminance(LuminanceConfig {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_SHADING.into(),
            ..default()
        })
    }

    /// Luminance strategy with a range of block characters in increasing order of size.
    pub fn luminance_blocks() -> Self {
        Self::Luminance(LuminanceConfig {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_BLOCKS.into(),
            ..default()
        })
    }
}

/// Configuration for the RatatuiCameraStrategy::HalfBlock terminal rendering strategy.
///
/// # Example:
///
/// The following would configure the widget to skip transparent pixels.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{
/// #   RatatuiCamera, RatatuiCameraStrategy, HalfBlocksConfig
/// # };
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// # commands.spawn((
/// #     RatatuiCamera::default(),
///     RatatuiCameraStrategy::HalfBlocks(HalfBlocksConfig {
///         transparent: true,
///         ..default()
///     }),
/// # ));
/// # };
/// ```
///
#[derive(Clone, Debug)]
pub struct HalfBlocksConfig {
    /// If the alpha value of a rendered pixel is zero, skip writing that character to the ratatui
    /// buffer. Useful for compositing camera images together.
    ///
    /// Normally if two camera widgets are rendered in the same buffer area, the first image will
    /// be completely overwritten by the background of the second, even if the background is empty.
    /// But, with this option enabled, transparent pixels in the second image will skip being drawn
    /// and will leave the first layer as-is.
    ///
    /// Make sure to set the `Camera` component's `clear_color` to fully transparent for your
    /// transparent camera entity. Only fully transparent pixels will be skipped. See the
    /// `transparency` example for more detail.
    pub transparent: bool,

    /// The sets of terminal colors to convert to. Many terminals support 24-bit RGB "true color",
    /// but some only support pre-defined sets of 16 or 256 ANSI colors. By default the `RGB` enum
    /// variant will be used, which transparently uses the rgb u8 triplet to create a ratatui
    /// `Color::RGB` color. If set to the `ANSI16` or `ANSI256` enum variants, this strategy will
    /// find the ANSI color within those sets closest to the original rgb color (by Euclidean
    /// distance), and then convert to the corresponding ratatui `Color::Indexed` (for 256 colors)
    /// or named ANSI color, like `Color::Cyan` (for 16 colors).
    ///
    /// Colors that are from a more limited set will not be converted "upwards" to the more
    /// expansive set- for example, if you set an edge detection color of `Color::Cyan` and the
    /// `ColorSupport::ANSI256` variant, the color will be left as-is rather than being converted
    /// to `Color::Indexed(6)` (the equivalent indexed color for cyan).
    ///
    /// Reference for terminal color support:
    /// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
    pub color_support: ColorSupport,
}

impl Default for HalfBlocksConfig {
    fn default() -> Self {
        Self {
            transparent: true,
            color_support: ColorSupport::TrueColor,
        }
    }
}

/// Configuration for the RatatuiCameraStrategy::Luminance terminal rendering strategy.
///
/// # Example:
///
/// The following would configure the widget to multiply each pixel's luminance value by 5.0, use
/// ' ' and '.' for dimmer areas, use '+' and '#' for brighter areas, and skip transparent pixels.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{
/// #   RatatuiCamera, RatatuiCameraStrategy, LuminanceConfig
/// # };
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// # commands.spawn((
/// #     RatatuiCamera::default(),
///     RatatuiCameraStrategy::Luminance(LuminanceConfig {
///         luminance_characters: vec![' ', '.', '+', '#'],
///         luminance_scale: 5.0,
///         transparent: true,
///         ..default()
///     }),
/// # ));
/// # };
/// ```
///
#[derive(Clone, Debug)]
pub struct LuminanceConfig {
    /// The list of characters, in increasing order of opacity, to use for printing. For example,
    /// put an '@' symbol after a '+' symbol because it is more "opaque", taking up more space in
    /// the cell it is printed in, and so when printed in bright text on a dark background, it
    /// appears to be "brighter".
    pub luminance_characters: Vec<char>,

    /// The number that each luminance value is multiplied by before being used to select
    /// a character. Because most scenes do not occupy the full range of luminance between 0.0 and
    /// 1.0, each luminance value is multiplied by a scaling value first.
    pub luminance_scale: f32,

    /// If present, customizes how the foreground color should be chosen per character.
    pub foreground_color: Option<ColorChoice>,

    /// If present, customizes how the background color should be chosen per character.
    pub background_color: Option<ColorChoice>,

    /// Please refer to the same field in [HalfBlocksConfig].
    pub transparent: bool,

    /// Please refer to the same field in [HalfBlocksConfig].
    pub color_support: ColorSupport,
}

impl LuminanceConfig {
    /// A range of braille unicode characters in increasing order of opacity.
    pub const LUMINANCE_CHARACTERS_BRAILLE: &'static [char] =
        &[' ', '⠂', '⠒', '⠖', '⠶', '⠷', '⠿', '⡿', '⣿'];

    /// A range of miscellaneous characters in increasing order of opacity.
    pub const LUMINANCE_CHARACTERS_MISC: &'static [char] =
        &[' ', '.', ':', '+', '=', '!', '*', '?', '#', '%', '&', '@'];

    /// A range of block characters in increasing order of opacity.
    pub const LUMINANCE_CHARACTERS_SHADING: &'static [char] = &[' ', '░', '▒', '▓', '█'];

    /// A range of block characters in increasing order of size.
    pub const LUMINANCE_CHARACTERS_BLOCKS: &'static [char] =
        &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    /// The default scaling value to multiply pixel luminance by.
    const LUMINANCE_SCALE_DEFAULT: f32 = 10.;
}

impl Default for LuminanceConfig {
    fn default() -> Self {
        Self {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_BRAILLE.into(),
            luminance_scale: LuminanceConfig::LUMINANCE_SCALE_DEFAULT,
            foreground_color: None,
            background_color: None,
            transparent: true,
            color_support: ColorSupport::TrueColor,
        }
    }
}

/// Options for customizing a terminal buffer color (foreground or background). Customization
/// happens after depth detection and edge detection, and before the conversion for color support
/// and the transparency check.
#[derive(Clone)]
pub enum ColorChoice {
    /// Overrides the color with a single provided color.
    Color(ratatui::style::Color),

    /// Color will be determined by scaling the foreground color by the provided value. For
    /// example, `ColorChoice::Scale(0.5)` will be half as bright as the calculated foreground
    /// color.
    Scale(f32),

    /// Provide a callback that will be used to determine the color. When the callback is called,
    /// the first argument is the foreground color, and the second argument is the background
    /// color, as determined by the conversion strategy. Both are an `Option`, as they may be
    /// `None` in cases where the strategy has determined it should skip drawing that pixel or cell
    /// (e.g. if the alpha for that pixel is zero). The result is also an
    /// `Option<ratatui::style::Color>`, as you can signal that drawing the foreground or
    /// background should be skipped by conditionally returning `None` from the callback. Your
    /// callback needs to be wrapped in an `Arc` as `RatatuiCameraStrategy` is cloned during
    /// render (or you can use the `from_callback()` convenience method which wraps it for you).
    Callback(
        Arc<
            dyn Fn(
                    Option<ratatui::style::Color>,
                    Option<ratatui::style::Color>,
                ) -> Option<ratatui::style::Color>
                + Send
                + Sync
                + 'static,
        >,
    ),
}

impl Default for ColorChoice {
    fn default() -> Self {
        Self::Scale(0.5)
    }
}

impl Debug for ColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorChoice::Color(color) => write!(f, "ColorChoice::Color({:?})", color),
            ColorChoice::Scale(scale) => write!(f, "ColorChoice::Scale({})", scale),
            ColorChoice::Callback(_) => write!(f, "ColorChoice::Callback(...)"),
        }
    }
}

impl ColorChoice {
    /// See [ColorChoice::Callback]. This convenience method creates a `ColorChoice::Callback` enum
    /// variant by wrapping the provided callback in an `Arc`.
    pub fn from_callback<F>(callback: F) -> Self
    where
        F: Fn(
                Option<ratatui::style::Color>,
                Option<ratatui::style::Color>,
            ) -> Option<ratatui::style::Color>
            + Send
            + Sync
            + 'static,
    {
        Self::Callback(Arc::new(callback))
    }
}
