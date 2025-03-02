use bevy::prelude::{Commands, Component, Entity};
use image::DynamicImage;
use ratatui::widgets::Widget;
use ratatui::{prelude::*, widgets::WidgetRef};

use crate::camera_readback::RatatuiCameraResize;
use crate::widget_halfblocks::RatatuiCameraWidgetHalfblocks;
use crate::widget_luminance::RatatuiCameraWidgetLuminance;
use crate::widget_none::RatatuiCameraWidgetNone;
use crate::{RatatuiCamera, RatatuiCameraEdgeDetection, RatatuiCameraStrategy};

/// Ratatui widget that will be inserted into each RatatuiCamera containing entity and updated each
/// frame with the last image rendered by the camera. When drawn in a ratatui buffer, it will use
/// the RatatuiCamera's specified RatatuiCameraStrategy to convert the rendered image to unicode
/// characters, and will draw them in the buffer.
///
#[derive(Component, Debug)]
pub struct RatatuiCameraWidget {
    /// Associated entity.
    pub entity: Entity,

    /// Associated RatatuiCamera.
    pub ratatui_camera: RatatuiCamera,

    /// RatatuiCamera camera's rendered image copied back from the GPU.
    pub camera_image: DynamicImage,

    /// RatatuiCamera camera's sobel texture generated by the GPU, if any.
    pub sobel_image: Option<DynamicImage>,

    /// Strategy used to convert the rendered image to unicode.
    pub strategy: RatatuiCameraStrategy,

    /// RatatuiCamera's edge detection settings, if any.
    pub edge_detection: Option<RatatuiCameraEdgeDetection>,
}

impl Widget for &RatatuiCameraWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.strategy {
            RatatuiCameraStrategy::HalfBlocks => {
                RatatuiCameraWidgetHalfblocks::new(&self.camera_image).render_ref(area, buf)
            }
            RatatuiCameraStrategy::Luminance(ref strategy_config) => {
                RatatuiCameraWidgetLuminance::new(
                    &self.camera_image,
                    &self.sobel_image,
                    strategy_config,
                    &self.edge_detection,
                )
                .render_ref(area, buf);
            }
            RatatuiCameraStrategy::None => {
                RatatuiCameraWidgetNone::new(
                    &self.camera_image,
                    &self.sobel_image,
                    &self.edge_detection,
                )
                .render_ref(area, buf);
            }
        }
    }
}

impl RatatuiCameraWidget {
    /// Resize the associated RatatuiCamera to the dimensions of the provided area.
    ///
    /// Returns `true` if a resize was triggered, `false` otherwise.
    pub fn resize(&self, commands: &mut Commands, area: Rect) -> bool {
        let dimensions = (area.width as u32 * 2, area.height as u32 * 4);

        if self.ratatui_camera.dimensions != dimensions {
            commands
                .entity(self.entity)
                .trigger(RatatuiCameraResize { dimensions });

            return true;
        }

        false
    }

    /// Resizes if a resize is needed, otherwise renders.
    pub fn render_autoresize(&self, area: Rect, buf: &mut Buffer, commands: &mut Commands) {
        if !self.resize(commands, area) {
            self.render(area, buf);
        }
    }
}
