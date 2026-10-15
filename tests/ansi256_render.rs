use bevy::{prelude::*, winit::WinitPlugin};
use bevy_ratatui_camera::*;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

// Run explicitly on a machine with a supported GPU; no window or terminal is opened.
#[test]
#[ignore = "requires a graphics adapter"]
fn camera_to_terminal_buffer() {
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
    let area = Rect::new(0, 0, 8, 8);
    let mut red = false;
    for _ in 0..20 {
        app.update();
        if let Some(mut widget) = app.world_mut().get_mut::<RatatuiCameraWidget>(camera) {
            let mut buffer = Buffer::empty(area);
            (&mut *widget).render(area, &mut buffer);
            red |= buffer.content.iter().any(|cell| matches!(cell.fg, ratatui::style::Color::Rgb(r,g,b) if r > 200 && g < 30 && b < 30));
        }
    }
    assert!(red, "camera clear color must reach terminal cells");
    app.world_mut()
        .get_mut::<Camera>(camera)
        .unwrap()
        .clear_color = ClearColorConfig::Custom(Color::srgb_u8(95, 0, 0));
    app.world_mut()
        .entity_mut(camera)
        .insert(RatatuiCameraStrategy::HalfBlocks(HalfBlocksConfig {
            colors: ColorsConfig {
                support: ColorSupport::ANSI256,
                ..default()
            },
            ..default()
        }));
    let mut last = Buffer::empty(area);
    for _ in 0..10 {
        app.update();
        let mut widget = app
            .world_mut()
            .get_mut::<RatatuiCameraWidget>(camera)
            .unwrap();
        last = Buffer::empty(area);
        (&mut *widget).render(area, &mut last);
    }
    assert!(
        last.content
            .iter()
            .any(|cell| cell.fg == ratatui::style::Color::Indexed(52)),
        "ANSI256 must preserve exact xterm cube red (95,0,0), got {:?}",
        last.content.iter().map(|c| c.fg).collect::<Vec<_>>()
    );
}
