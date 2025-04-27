use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui_camera::LuminanceConfig;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraStrategy;
use bevy_ratatui_camera::RatatuiCameraWidget;
use log::LevelFilter;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;

mod shared;

fn main() {
    shared::setup_tui_logger(LevelFilter::Info);

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 120.)),
            FrameTimeDiagnosticsPlugin::default(),
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
        RatatuiCameraStrategy::luminance_with_characters(&[' ', '-', '+', '=', '#']),
        Camera3d::default(),
        Transform::from_xyz(0., 3., 0.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        RatatuiCamera::default(),
        RatatuiCameraStrategy::Luminance(LuminanceConfig {
            bg_color_scale: 0.2,
            luminance_characters: LuminanceConfig::LUMINANCE_CHARACTERS_BRAILLE.into(),
            ..default()
        }),
        Camera3d::default(),
        Transform::from_xyz(0., 0., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        RatatuiCamera::default(),
        RatatuiCameraStrategy::luminance_with_characters(&[' ', '.', 'o', 'O', '0']),
        Camera3d::default(),
        Transform::from_xyz(2., 2., 2.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn draw_scene_system(
    mut commands: Commands,
    mut ratatui: ResMut<RatatuiContext>,
    camera_widgets: Query<&RatatuiCameraWidget>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> Result {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        let widgets = camera_widgets.iter().enumerate().collect::<Vec<_>>();

        let layout = Layout::new(
            Direction::Horizontal,
            vec![Constraint::Fill(1); widgets.len()],
        )
        .split(area);

        for (i, widget) in widgets {
            widget.render_autoresize(layout[i], frame.buffer_mut(), &mut commands);
        }
    })?;

    Ok(())
}
