use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraWidget;
use bevy_ratatui_camera::RatatuiSubcameras;
use log::LevelFilter;
use ratatui::widgets::Widget;

mod shared;

fn main() {
    shared::setup_tui_logger(LevelFilter::Info);

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin {
                smoothing_factor: 1.0,
                ..default()
            },
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .init_resource::<shared::Flags>()
        .init_resource::<shared::InputState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system)
        .add_systems(PreUpdate, shared::handle_input_system)
        .add_systems(Update, shared::rotate_spinners_system)
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    shared::spawn_3d_scene(commands.reborrow(), meshes, materials);

    commands.spawn((
        RatatuiCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(2., 1., 1.).looking_at(Vec3::Y, Vec3::Z),
        related!(
            RatatuiSubcameras[(
                Camera3d::default(),
                Transform::from_xyz(1., 2., 1.).looking_at(Vec3::X, Vec3::Z)
            )]
        ),
    ));
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    mut camera_widget: Single<&mut RatatuiCameraWidget>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> Result {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        camera_widget.render(area, frame.buffer_mut());
    })?;

    Ok(())
}
