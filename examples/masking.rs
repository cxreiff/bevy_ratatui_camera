use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui_camera::EdgeCharacters;
use bevy_ratatui_camera::LuminanceConfig;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraEdgeDetection;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraStrategy;
use bevy_ratatui_camera::RatatuiCameraWidget;

mod shared;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),
            FrameTimeDiagnosticsPlugin,
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .init_resource::<shared::Flags>()
        .init_resource::<shared::InputState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .add_systems(PreUpdate, shared::handle_input_system)
        .add_systems(Update, shared::rotate_spinners_system)
        .run();
}

#[derive(Component)]
pub struct Foreground;

#[derive(Component)]
pub struct Background;

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    shared::spawn_3d_scene(&mut commands, &mut meshes, &mut materials);

    commands.spawn((
        Foreground,
        RatatuiCamera::default(),
        RatatuiCameraStrategy::Luminance(LuminanceConfig {
            mask_color: Some(ratatui::style::Color::Rgb(0, 0, 0)),
            ..default()
        }),
        RatatuiCameraEdgeDetection {
            edge_color: Some(ratatui::style::Color::Rgb(255, 0, 255)),
            edge_characters: EdgeCharacters::Single('#'),
            ..Default::default()
        },
        Camera3d::default(),
        Transform::from_xyz(6., 0., 2.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        Background,
        RatatuiCamera::default(),
        RatatuiCameraStrategy::luminance_misc(),
        Camera3d::default(),
        Transform::from_xyz(3., 0., 1.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn draw_scene_system(
    mut commands: Commands,
    mut ratatui: ResMut<RatatuiContext>,
    foreground_widget: Query<&RatatuiCameraWidget, With<Foreground>>,
    background_widget: Query<&RatatuiCameraWidget, With<Background>>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> std::io::Result<()> {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        background_widget
            .single()
            .render_autoresize(area, frame.buffer_mut(), &mut commands);
        foreground_widget
            .single()
            .render_autoresize(area, frame.buffer_mut(), &mut commands);
    })?;

    Ok(())
}
