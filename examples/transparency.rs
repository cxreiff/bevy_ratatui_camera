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
use bevy_ratatui_camera::EdgeCharacters;
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
            FrameTimeDiagnosticsPlugin::default(),
            RatatuiPlugins::default(),
            RatatuiCameraPlugin,
        ))
        .init_resource::<shared::Flags>()
        .init_resource::<shared::InputState>()
        .insert_resource(ClearColor(Color::srgba(0., 0., 0., 0.)))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system)
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
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    shared::spawn_3d_scene(commands.reborrow(), meshes, materials);

    commands.spawn((
        Foreground,
        RatatuiCamera::default(),
        RatatuiCameraStrategy::luminance_braille(),
        RatatuiCameraEdgeDetection {
            edge_color: Some(ratatui::style::Color::Magenta),
            edge_characters: EdgeCharacters::Single('#'),
            ..Default::default()
        },
        Camera3d::default(),
        Camera {
            // by setting this camera's clear_color transparent, background pixels will be given an
            // alpha value of zero and so will be skipped when the ratatui buffer is drawn
            clear_color: ClearColorConfig::Custom(Color::srgba(0., 0., 0., 0.)),
            ..default()
        },
        Transform::from_xyz(6., 0., 2.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
    commands.spawn((
        Background,
        RatatuiCamera::default(),
        RatatuiCameraStrategy::luminance_misc(),
        RatatuiCameraEdgeDetection {
            edge_color: Some(ratatui::style::Color::Cyan),
            edge_characters: EdgeCharacters::Single('#'),
            ..Default::default()
        },
        Camera3d::default(),
        Transform::from_xyz(3., 0., 1.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn draw_scene_system(
    mut commands: Commands,
    mut ratatui: ResMut<RatatuiContext>,
    foreground_widget: Single<&RatatuiCameraWidget, With<Foreground>>,
    background_widget: Single<&RatatuiCameraWidget, With<Background>>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> Result {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        background_widget.render_autoresize(area, frame.buffer_mut(), &mut commands);
        foreground_widget.render_autoresize(area, frame.buffer_mut(), &mut commands);
    })?;

    Ok(())
}
