use bevy::{prelude::*, winit::WinitPlugin};
use bevy_ratatui_camera::{RatatuiCamera, RatatuiCameraPlugin, RatatuiCameraWidget};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[test]
#[ignore = "requires a graphics adapter"]
fn renders_cells_after_the_u16_index_boundary() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
        .add_plugins(RatatuiCameraPlugin);
    app.finish();
    app.cleanup();
    let camera = app
        .world_mut()
        .spawn((
            Camera2d,
            RatatuiCamera::new(32, 32),
            Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(1., 0., 0.)),
                ..default()
            },
        ))
        .id();
    let area = Rect::new(0, 0, 400, 200);
    let mut buffer = Buffer::empty(area);
    for _ in 0..20 {
        app.update();
        if let Some(mut widget) = app.world_mut().get_mut::<RatatuiCameraWidget>(camera) {
            buffer = Buffer::empty(area);
            (&mut *widget).render(area, &mut buffer);
        }
    }
    assert_eq!(buffer[(399, 199)].fg, ratatui::style::Color::Rgb(255, 0, 0));
}
