use bevy::prelude::*;

/// Spawn this component with your bevy camera in order to send each frame's rendered image to
/// a RatatuiCameraWidget that will be inserted into the same camera entity.
///
/// Example:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::RatatuiCamera;
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// commands.spawn((
///     RatatuiCamera::default(),
///     Camera3d::default(),
/// ));
/// # };
/// ```
///
#[derive(Component, Clone, Debug)]
#[require(RatatuiCameraStrategy)]
pub struct RatatuiCamera {
    /// Dimensions (width, height) of the image the camera will render to.
    pub dimensions: (u32, u32),

    /// If true, the rendered image dimensions will be resized to match the size and aspect ratio
    /// of the terminal window (at startup and whenever a terminal resize event is received).
    pub autoresize: bool,

    /// When autoresize is true, this function will be used to transform the new terminal
    /// dimensions into the rendered image dimensions. For example, use `|(w, h)| (w*4, h*3)` to
    /// maintain a 4:3 aspect ratio.
    pub autoresize_fn: fn((u32, u32)) -> (u32, u32),
}

impl Default for RatatuiCamera {
    fn default() -> Self {
        Self {
            dimensions: (256, 256),
            autoresize: false,
            autoresize_fn: |(w, h)| (w * 2, h * 2),
        }
    }
}

impl RatatuiCamera {
    /// Creates a new RatatuiCamera that renders to an image of the provided dimensions.
    pub fn new(dimensions: (u32, u32)) -> Self {
        Self {
            dimensions,
            ..default()
        }
    }

    /// Creates a new RatatuiCamera that will automatically resize to the terminal dimensions.
    ///
    /// Providing dimensions is not necessary as they will be replaced on the first frame.
    pub fn autoresize() -> Self {
        Self {
            autoresize: true,
            ..default()
        }
    }

    /// Mutates RatatuiCamera to use new provided dimensions.
    pub fn with_dimensions(mut self, dimensions: (u32, u32)) -> Self {
        self.dimensions = dimensions;
        self
    }

    /// Mutates RatatuiCamera to use new provided autoresize setting.
    pub fn with_autoresize(mut self, autoresize: bool) -> Self {
        self.autoresize = autoresize;
        self
    }

    /// Mutates RatatuiCamera to use new provided autoresize function.
    pub fn with_autoresize_fn(mut self, autoresize_fn: fn((u32, u32)) -> (u32, u32)) -> Self {
        self.autoresize_fn = autoresize_fn;
        self
    }
}

/// When inserted into a camera entity, rather than creating its own render texture for unicode
/// conversion, this camera will render to the texture of the RatatuiCamera camera entity indicated
/// by the provided entity id. The composite render from both cameras will then be converted to
/// unicode as one image.
///
/// Example:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{RatatuiCamera, RatatuiSubcamera};
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// let main_camera = commands.spawn((
///     RatatuiCamera::default(),
///     Camera3d::default(),
/// )).id();
///
/// commands.spawn((
///     RatatuiSubcamera(main_camera),
///     Camera3d::default(),
/// ));
/// # };
/// ```
///
#[derive(Component, Debug)]
// TODO: When bevy 0.16 arrives, use new relations feature.
// #[relationship(relationship_target = RatatuiCameraTargetedBy)]
pub struct RatatuiSubcamera(pub Entity);

// TODO: When bevy 0.16 arrives, use new relations feature.
// /// All camera entities that are rendering to this camera entity's render target.
// #[derive(Component, Debug)]
// #[relationship_target(relationship = RatatuiCameraTargeting)]
// struct RatatuiCameraTargetedBy(Vec<Entity>);

/// Specify the strategy used for converting the camera's rendered image to unicode characters for
/// the terminal buffer. Insert a variant of this component alongside your `RatatuiCamera` to
/// change the default behavior.
///
#[derive(Component, Clone, Debug, Default)]
pub enum RatatuiCameraStrategy {
    /// Print to the terminal using unicode halfblock characters. By using both the halfblock
    /// (foreground) color and the background color, we can draw two pixels per buffer cell.
    #[default]
    HalfBlocks,

    /// Given a range of unicode characters sorted in increasing order of opacity, use each pixel's
    /// luminance to select a character from the range.
    Luminance(LuminanceConfig),

    /// Does not print characters by itself, but edge detection will still print. Use with edge
    /// detection for a "wireframe".
    None,
}

impl RatatuiCameraStrategy {
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
}

/// Configuration for the RatatuiCameraStrategy::Luminance terminal rendering strategy.
///
/// # Example:
///
/// The following would configure the widget to multiply each pixel's luminance value by 5.0, use
/// ' ' and '.' for dimmer areas, use '+' and '#' for brighter areas, and skip using a mask color:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{RatatuiCamera, RatatuiCameraStrategy, LuminanceConfig};
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// # commands.spawn((
/// #     RatatuiCamera::default(),
///     RatatuiCameraStrategy::Luminance(LuminanceConfig {
///         luminance_characters: vec![' ', '.', '+', '#'],
///         luminance_scale: 5.0,
///         mask_color: None,
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

    /// An optional mask color for creating transparency effects. Skips writing any character to
    /// the terminal buffer that matches the provided color.
    ///
    /// For example, set it to `Some(Color::Rgb(0, 0, 0)` in a scene with a foreground object in
    /// front of a black background, to render that object without the background space overwriting
    /// the cells currently in the buffer.
    ///
    /// Useful for compositing together multiple rendered layers. There should generally always be
    /// at least one non-masked layer furthest back, as otherwise stray cells in the terminal
    /// buffer might not get replaced between frames.
    pub mask_color: Option<ratatui::style::Color>,
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

    /// The default scaling value to multiply pixel luminance by.
    const LUMINANCE_SCALE_DEFAULT: f32 = 10.;
}

impl Default for LuminanceConfig {
    fn default() -> Self {
        Self {
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_BRAILLE.into(),
            luminance_scale: LuminanceConfig::LUMINANCE_SCALE_DEFAULT,
            mask_color: None,
        }
    }
}
