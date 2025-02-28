#![warn(missing_debug_implementations, missing_docs)]

//! Bevy rendered to the terminal!

mod camera;
mod camera_edge_detection;
mod camera_image_pipe;
mod camera_node;
mod camera_node_sobel;
mod camera_readback;
mod plugin;
mod widget;
mod widget_halfblocks;
mod widget_luminance;
mod widget_none;

pub use camera::{LuminanceConfig, RatatuiCamera, RatatuiCameraStrategy, RatatuiSubcamera};
pub use camera_edge_detection::{EdgeCharacters, RatatuiCameraEdgeDetection};
pub use plugin::RatatuiCameraPlugin;
pub use widget::RatatuiCameraWidget;
