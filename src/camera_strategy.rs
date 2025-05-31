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

    /// Given a range of unicode characters sorted in increasing order of opacity, use each pixel's
    /// depth to select a character from the range.
    ///
    /// NOTE: The [RatatuiCameraDepthDetection](crate::RatatuiCameraDepthDetection) component is
    /// required on the same camera entity for this strategy to function, as it relies on the depth
    /// texture.
    Depth(DepthConfig),

    /// Does not print characters by itself, but edge detection will still print. Use with edge
    /// detection for a "wireframe".
    None,
}

impl RatatuiCameraStrategy {
    /// A range of braille unicode characters in increasing order of opacity.
    pub const CHARACTERS_BRAILLE: &'static [char] = &[' ', '⠂', '⠒', '⠖', '⠶', '⠷', '⠿', '⡿', '⣿'];

    /// A range of miscellaneous characters in increasing order of opacity.
    pub const CHARACTERS_MISC: &'static [char] =
        &[' ', '.', ':', '+', '=', '!', '*', '?', '#', '%', '&', '@'];

    /// A range of block characters in increasing order of opacity.
    pub const CHARACTERS_SHADING: &'static [char] = &[' ', '░', '▒', '▓', '█'];

    /// A range of block characters in increasing order of size.
    pub const CHARACTERS_BLOCKS: &'static [char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
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

    /// Depth strategy with a provided list of characters.
    pub fn depth_with_characters(characters: &[char]) -> Self {
        Self::Depth(DepthConfig {
            characters: CharactersConfig {
                list: characters.into(),
                scale: DepthConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Depth strategy with a range of braille unicode characters in increasing order of opacity.
    pub fn depth_braille() -> Self {
        Self::Depth(DepthConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_BRAILLE.into(),
                scale: DepthConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Depth strategy with a range of miscellaneous characters in increasing order of opacity.
    pub fn depth_misc() -> Self {
        Self::Depth(DepthConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_MISC.into(),
                scale: DepthConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Depth strategy with a range of block characters in increasing order of opacity.
    pub fn depth_shading() -> Self {
        Self::Depth(DepthConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_SHADING.into(),
                scale: DepthConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Depth strategy with a range of block characters in increasing order of size.
    pub fn depth_blocks() -> Self {
        Self::Depth(DepthConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_BLOCKS.into(),
                scale: DepthConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Luminance strategy with a provided list of characters.
    pub fn luminance_with_characters(characters: &[char]) -> Self {
        Self::Luminance(LuminanceConfig {
            characters: CharactersConfig {
                list: characters.into(),
                scale: LuminanceConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Luminance strategy with a range of braille unicode characters in increasing order of opacity.
    pub fn luminance_braille() -> Self {
        Self::Luminance(LuminanceConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_BRAILLE.into(),
                scale: LuminanceConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Luminance strategy with a range of miscellaneous characters in increasing order of opacity.
    pub fn luminance_misc() -> Self {
        Self::Luminance(LuminanceConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_MISC.into(),
                scale: LuminanceConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Luminance strategy with a range of block characters in increasing order of opacity.
    pub fn luminance_shading() -> Self {
        Self::Luminance(LuminanceConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_SHADING.into(),
                scale: LuminanceConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }

    /// Luminance strategy with a range of block characters in increasing order of size.
    pub fn luminance_blocks() -> Self {
        Self::Luminance(LuminanceConfig {
            characters: CharactersConfig {
                list: Self::CHARACTERS_BLOCKS.into(),
                scale: LuminanceConfig::SCALE_DEFAULT,
            },
            ..default()
        })
    }
}

/// Configuration for the RatatuiCameraStrategy::HalfBlock terminal rendering strategy.
///
/// # Example:
///
/// The following would configure the widget to use ANSI colors.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{
/// #   RatatuiCamera, RatatuiCameraStrategy, HalfBlocksConfig, ColorsConfig, ColorSupport
/// # };
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// # commands.spawn((
/// #     RatatuiCamera::default(),
///     RatatuiCameraStrategy::HalfBlocks(HalfBlocksConfig {
///         colors: ColorsConfig {
///             support: ColorSupport::ANSI16,
///             ..default()
///         },
///         ..default()
///     }),
/// # ));
/// # };
/// ```
///
#[derive(Clone, Debug, Default)]
pub struct HalfBlocksConfig {
    /// Configuration options common to all strategies.
    pub common: CommonConfig,

    /// Configuration for determining the resulting colors.
    pub colors: ColorsConfig,
}

/// Configuration for the RatatuiCameraStrategy::Depth terminal rendering strategy.
///
/// NOTE: The [RatatuiCameraDepthDetection](crate::RatatuiCameraDepthDetection) component is
/// required on the same camera entity for this strategy to function, as it relies on the depth
/// texture.
///
/// # Example:
///
/// The following configures the widget to use '@' for close surfaces and '+' for more distant
/// surfaces.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{
/// #   RatatuiCamera, RatatuiCameraStrategy, DepthConfig, CharactersConfig
/// # };
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// # commands.spawn((
/// #     RatatuiCamera::default(),
///     RatatuiCameraStrategy::Depth(DepthConfig {
///         characters: CharactersConfig {
///             list: vec![' ', '+', '@'],
///             scale: DepthConfig::SCALE_DEFAULT,
///         },
///         ..default()
///     }),
/// # ));
/// # };
/// ```
#[derive(Clone, Debug)]
pub struct DepthConfig {
    /// Configuration options common to all strategies.
    pub common: CommonConfig,

    /// Configuration for determining the resulting characters.
    pub characters: CharactersConfig,

    /// Configuration for determining the resulting colors.
    pub colors: ColorsConfig,
}

impl DepthConfig {
    /// The default scaling value to multiply pixel depth by.
    pub const SCALE_DEFAULT: f32 = 30.;
}

impl Default for DepthConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig::default(),
            characters: CharactersConfig {
                list: RatatuiCameraStrategy::CHARACTERS_MISC.into(),
                scale: DepthConfig::SCALE_DEFAULT,
            },
            colors: ColorsConfig::default(),
        }
    }
}

/// Configuration for the RatatuiCameraStrategy::Luminance terminal rendering strategy.
///
/// # Example:
///
/// The following configures the widget to multiply each pixel's luminance value by 5.0, and use
/// ' ' and '.' for dimmer areas, use '+' and '#' for brighter areas.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{
/// #   RatatuiCamera, RatatuiCameraStrategy, LuminanceConfig, CharactersConfig
/// # };
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// # commands.spawn((
/// #     RatatuiCamera::default(),
///     RatatuiCameraStrategy::Luminance(LuminanceConfig {
///         characters: CharactersConfig {
///             list: vec![' ', '.', '+', '#'],
///             scale: 5.0,
///         },
///         ..default()
///     }),
/// # ));
/// # };
/// ```
///
#[derive(Clone, Debug)]
pub struct LuminanceConfig {
    /// Configuration options common to all strategies.
    pub common: CommonConfig,

    /// Configuration for determining the resulting characters.
    pub characters: CharactersConfig,

    /// Configuration for determining the resulting colors.
    pub colors: ColorsConfig,
}

impl LuminanceConfig {
    /// The default scaling value to multiply pixel luminance by.
    pub const SCALE_DEFAULT: f32 = 10.;
}

impl Default for LuminanceConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig::default(),
            characters: CharactersConfig {
                list: RatatuiCameraStrategy::CHARACTERS_MISC.into(),
                scale: LuminanceConfig::SCALE_DEFAULT,
            },
            colors: ColorsConfig::default(),
        }
    }
}

/// General configuration not specific to particular strategies.
#[derive(Clone, Debug)]
pub struct CommonConfig {
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
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self { transparent: true }
    }
}

/// Configuration pertaining to character selection, based on criteria determined by the strategy.
#[derive(Clone, Debug)]
pub struct CharactersConfig {
    /// The list of characters, in increasing order of opacity, to use for printing. For example,
    /// put an '@' symbol after a '+' symbol because it is more "opaque", taking up more space in
    /// the cell it is printed in, and so when printed in bright text on a dark background, it
    /// appears to be "brighter".
    pub list: Vec<char>,

    /// The number that each value used for character selection is multiplied by before being used
    /// to select a character. This is useful because many combinations of scenes and character
    /// selection metrics will not occupy the full range between 0.0 and 1.0, and so each luminance
    /// value can be multiplied by a scaling value first to tune the character selection.
    pub scale: f32,
}

/// Configuration pertaining to color selection.
#[derive(Clone, Debug, Default)]
pub struct ColorsConfig {
    /// If present, customizes how the foreground color should be chosen per character.
    pub foreground: Option<ColorChoice>,

    /// If present, customizes how the background color should be chosen per character.
    pub background: Option<ColorChoice>,

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
    pub support: ColorSupport,
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
