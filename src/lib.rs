#![warn(missing_debug_implementations, missing_docs)]

//! Bevy rendered to the terminal!

mod camera;
mod camera_edge_detection;
mod camera_image_pipe;
mod camera_node;
mod camera_node_sobel;
mod camera_readback;
mod camera_strategy;
mod color_support;
mod plugin;
mod widget;
mod widget_halfblocks;
mod widget_luminance;
mod widget_none;

pub use camera::{RatatuiCamera, RatatuiCameraSet, RatatuiSubcamera};
pub use camera_edge_detection::{EdgeCharacters, RatatuiCameraEdgeDetection};
pub use camera_strategy::{LuminanceConfig, RatatuiCameraStrategy};
pub use color_support::ColorSupport;
pub use plugin::RatatuiCameraPlugin;
pub use widget::RatatuiCameraWidget;
