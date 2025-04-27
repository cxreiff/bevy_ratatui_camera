use bevy::prelude::*;

use crate::camera_strategy::RatatuiCameraStrategy;

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
}

impl Default for RatatuiCamera {
    fn default() -> Self {
        Self {
            dimensions: (256, 256),
        }
    }
}

impl RatatuiCamera {
    /// Creates a new RatatuiCamera that renders to an image of the provided dimensions.
    pub fn new(dimensions: (u32, u32)) -> Self {
        Self { dimensions }
    }
}

/// Bevy relation that allows you to create subcameras that render to a main camera's render
/// texture instead of creating their own. When `RatatuiSubcamera` is within into a camera entity
/// (instead of a `RatatuiCamera`), rather than creating its own render texture for unicode
/// conversion, this camera will render to the texture of the RatatuiCamera main camera entity
/// indicated by the relation. The composite render from both cameras will then be converted to
/// unicode as one image.
///
/// Example:
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_ratatui_camera::{RatatuiCamera, RatatuiSubcameras};
/// #
/// # #[derive(Component)]
/// # pub struct POVCamera;
/// # #[derive(Component)]
/// # pub struct FXCamera;
/// #
/// # fn setup_scene_system(mut commands: Commands) {
/// let main_camera = commands.spawn((
///     RatatuiCamera::default(),
///     Camera3d::default(),
///     related!(RatatuiSubcameras[
///         (Camera3d::default(), POVCamera),
///         (Camera3d::default(), FXCamera),
///     ]),
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
#[relationship(relationship_target = RatatuiSubcameras)]
pub struct RatatuiSubcamera(pub Entity);

/// Bevy relation target for subcameras that will render to this camera entity's render target.
#[derive(Component, Debug)]
#[relationship_target(relationship = RatatuiSubcamera)]
pub struct RatatuiSubcameras(Vec<Entity>);

/// System set for the systems that perform this crate's functionality. Because important pieces of
/// this crate's functionality are provided by components that are not added by the user directly,
/// but are inserted and updated by this crate's observers and event handlers (e.g.
/// RatatuiCameraWidget), it is important to order your systems relative to this system set to make
/// sure certain components are present and up-to-date.
///
/// System set that runs in the [First] schedule, for the systems that create the
/// RatatuiCameraWidget components each frame, retrieve rendered images from the GPU, and keep the
/// mechanisms for performing that retrieval up-to-date (e.g. after resizes).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RatatuiCameraSet;
