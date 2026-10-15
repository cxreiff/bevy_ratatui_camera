use bevy_ratatui_camera::{EdgeCharacters, RatatuiCameraEdgeDetection, RatatuiCameraWidget};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

#[test]
fn edge_detection_configuration() {
    let _ = RatatuiCameraEdgeDetection {
        thickness: 1.4,
        edge_characters: EdgeCharacters::Directional {
            vertical: '|',
            horizontal: '―',
            forward_diagonal: '/',
            backward_diagonal: '\\',
        },
        edge_color: Some(ratatui::style::Color::Magenta),
        ..Default::default()
    };
}

#[allow(dead_code)]
fn draw_with_stateful_widget(widget: &mut RatatuiCameraWidget, area: Rect, buffer: &mut Buffer) {
    let depth_buffer = &mut widget.new_depth_buffer(area);
    StatefulWidget::render(&mut *widget, area, buffer, depth_buffer);
}
