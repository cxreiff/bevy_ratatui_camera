use bevy::math::{IVec2, Vec3};
use image::{DynamicImage, imageops::FilterType};
use ratatui::layout::Rect;

use crate::RatatuiCameraWidget;

impl RatatuiCameraWidget {
    /// Calculate the aspect ratio of the widget's render image.
    pub fn aspect_ratio(&self) -> f32 {
        (self.camera_image.width() * 2) as f32 / self.camera_image.height() as f32
    }

    /// Calculate the area that the image will actually be drawn (excluding the vertical or
    /// horizontal gutters needed to preserve the image aspect ratio).
    pub fn calculate_render_area(&self, area: Rect) -> Rect {
        let aspect_ratio = self.aspect_ratio();
        let width = (area.width as f32)
            .min(area.height as f32 * aspect_ratio)
            .round() as u16;
        let height = (area.height as f32)
            .min(area.width as f32 / aspect_ratio)
            .round() as u16;

        let x = area.x + (area.width - width) / 2;
        let y = area.y + (area.height - height) / 2;

        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Return the camera image and (if present) sobel texture, resized to fit the area parameter.
    pub fn resize_images_to_area(
        &self,
        area: Rect,
    ) -> (DynamicImage, Option<DynamicImage>, Option<DynamicImage>) {
        let width = area.width as u32;
        let height = area.height as u32 * 2;

        let camera_image = self.camera_image.resize(width, height, FilterType::Nearest);

        let depth_image = self
            .depth_image
            .as_ref()
            .map(|i| i.resize(width, height, FilterType::Nearest));

        let sobel_image = self
            .sobel_image
            .as_ref()
            .map(|i| i.resize(width, height, FilterType::Nearest));

        (camera_image, depth_image, sobel_image)
    }

    /// Convert a pair of terminal buffer cell coordinates (number of characters from the left edge
    /// and top edge of the buffer, respectively) into an NDC (Normalized Device Coordinates) value
    /// that represents a position in the camera viewport.
    pub fn cell_to_ndc(&self, area: Rect, cell_coords: IVec2) -> Vec3 {
        let render_area = self.calculate_render_area(area);
        let cell_coords = IVec2 {
            x: cell_coords.x - render_area.x as i32,
            y: cell_coords.y - render_area.y as i32,
        };

        self.relative_cell_to_ndc(render_area, cell_coords)
    }

    /// See [RatatuiCameraWidget::cell_to_ndc]. Rather than the global cell coordinates, this
    /// variant takes the cell coordinates relative to the provided area.
    pub fn relative_cell_to_ndc(&self, area: Rect, cell_coords: IVec2) -> Vec3 {
        let render_area = self.calculate_render_area(area);
        let x = (cell_coords.x as f32 / render_area.width as f32 - 0.5) * 2.;
        let y = (cell_coords.y as f32 / render_area.height as f32 - 0.5) * -2.;

        Vec3::new(x, y, 0.5)
    }

    /// Convert an NDC (Normalized Device Coordinates) value that represents a position in the
    /// camera viewport into a pair of terminal buffer cell coordinates (number of characters from
    /// the left edge and top edge of the buffer, respectively).
    pub fn ndc_to_cell(&self, area: Rect, ndc_coords: Vec3) -> IVec2 {
        let render_area = self.calculate_render_area(area);
        let cell = self.ndc_to_relative_cell(render_area, ndc_coords);

        IVec2 {
            x: cell.x + render_area.x as i32,
            y: cell.y + render_area.y as i32,
        }
    }

    /// See [RatatuiCameraWidget::ndc_to_cell]. Rather than the global cell coordinates, this
    /// variant gives the cell coordinates relative to the provided area.
    pub fn ndc_to_relative_cell(&self, area: Rect, ndc_coords: Vec3) -> IVec2 {
        let render_area = self.calculate_render_area(area);
        let x = ((ndc_coords.x / 2. + 0.5) * render_area.width as f32) as i32;
        let y = ((-ndc_coords.y / 2. + 0.5) * render_area.height as f32) as i32;

        IVec2 { x, y }
    }
}
