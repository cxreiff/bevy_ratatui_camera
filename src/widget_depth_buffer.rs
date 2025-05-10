use image::{DynamicImage, GenericImageView};

/// A depth buffer for keeping track of the bevy world-space depth of each character drawn to the
/// terminal buffer, for occluding characters "behind" others with respect to a bevy camera.
///
/// For a given render area (in terminal cell dimensions), the depth buffer is the same width as
/// the render area, and twice the height (as terminal cells are twice as high as they are wide).
/// Coordinates provided when calling the depth buffer's functions should be starting from the top
/// left.
///
/// Depth values follow Bevy's convention, which is 1/Z with the near plane being 1.0, and the far
/// plane being 0.0. This means that this buffer will record the highest value seen for a given
/// coordinate pair.
#[derive(Clone, Debug, Default)]
pub struct RatatuiCameraDepthBuffer {
    width: usize,
    height: usize,
    pub(crate) buffer: Vec<f32>,
}

impl RatatuiCameraDepthBuffer {
    /// Create a new depth buffer matching the provided area. Height is doubled because there are
    /// two pixels vertically per terminal cell, therefore two depths.
    pub fn new(area: ratatui::layout::Rect) -> Self {
        Self {
            width: area.width as usize,
            height: area.height as usize * 2,
            buffer: vec![0.0; area.width as usize * area.height as usize * 2],
        }
    }

    /// Retrieve the depth value for the provided coordinates.
    pub fn get(&self, x: usize, y: usize) -> Option<f32> {
        let index = self.index(x, y)?;

        Some(self.buffer[index])
    }

    /// Set the depth value for the provided coordinates.
    pub fn set(&mut self, x: usize, y: usize, depth: f32) -> Option<f32> {
        let index = self.index(x, y)?;

        let previous = self.buffer[index];

        self.buffer[index] = depth;

        Some(previous)
    }

    /// Compare a provided depth value against the depth value already recorded at the same
    /// coordinates. There are three possible results:
    /// - If the new depth value is equal or higher (closer), the existing depth value will be
    ///   replaced and the returned option will contain true (meaning a character at the new depth
    ///   should be drawn).
    /// - If the new depth is less than previous recorded (further), the recorded depth will be
    ///   left as is, and the returned option will contain false (meaning a character at the new
    ///   depth is occluded and should not be drawn).
    /// - If the provided coordinates are outside of the depth buffer, `None` is returned.
    pub fn compare_and_update(&mut self, x: usize, y: usize, depth: f32) -> Option<bool> {
        let previous_depth = self.get(x, y)?;

        if depth >= previous_depth {
            self.set(x, y, depth);
            return Some(true);
        }

        Some(false)
    }

    /// See [RatatuiCameraDepthBuffer::compare_and_update]. This variant takes a depth image
    /// instead of a depth value, and takes care of retrieving the correct depth value based on the
    /// provided coordinates (assuming that the depth image dimensions match the buffer).
    pub fn compare_and_update_from_image(
        &mut self,
        x: u32,
        y: u32,
        depth_image: &DynamicImage,
    ) -> Option<bool> {
        if x > depth_image.width() || y > depth_image.height() {
            return None;
        }

        let depth_bytes = depth_image.get_pixel(x, y);
        let depth = f32::from_le_bytes(depth_bytes.0);
        self.compare_and_update(x as usize, y as usize, depth)
    }

    /// Convert the provided 2D coordinates to an index in our flat buffer, returning None if the
    /// coordinates lie outside the bounds.
    fn index(&self, x: usize, y: usize) -> Option<usize> {
        if !self.valid_coordinates(x, y) {
            return None;
        }

        Some(x + y * self.width)
    }

    /// Validate that the provided coordinates lie inside the width and height of the buffer.
    fn valid_coordinates(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }
}
