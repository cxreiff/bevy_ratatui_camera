use bevy::{prelude::*, winit::WinitPlugin};
use bevy_ratatui_camera::{RatatuiCamera, RatatuiCameraPlugin, RatatuiCameraWidget};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[test]
#[ignore = "requires a graphics adapter"]
fn resize_then_despawn_camera() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
        .add_plugins(RatatuiCameraPlugin);
    app.finish();
    app.cleanup();
    let camera = app
        .world_mut()
        .spawn((Camera2d, RatatuiCamera::default()))
        .id();
    let area = Rect::new(2, 3, 16, 8);
    for _ in 0..10 {
        app.update();
        if let Some(mut widget) = app.world_mut().get_mut::<RatatuiCameraWidget>(camera) {
            (&mut *widget).render(area, &mut Buffer::empty(area));
        }
    }
    assert_eq!(
        app.world().get::<RatatuiCamera>(camera).unwrap().dimensions,
        UVec2::new(32, 32)
    );
    assert_eq!(
        app.world()
            .get::<RatatuiCameraWidget>(camera)
            .unwrap()
            .last_area,
        area
    );
    app.world_mut().despawn(camera);
    app.update();
}
