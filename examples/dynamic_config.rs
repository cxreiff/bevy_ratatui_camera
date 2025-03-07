use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::color::Color;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::system::RegisteredSystemError;
use bevy::ecs::system::SystemState;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::utils::error;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::event::KeyEvent;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui_camera::LuminanceConfig;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraEdgeDetection;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraStrategy;
use bevy_ratatui_camera::RatatuiCameraWidget;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use log::LevelFilter;

mod shared;

fn main() {
    shared::setup_tui_logger(LevelFilter::Info);

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
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
        .add_systems(Update, handle_input_system.map(error))
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    shared::spawn_3d_scene(&mut commands, &mut meshes, &mut materials);

    commands.spawn((
        RatatuiCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(2.5, 2.5, 2.5).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn draw_scene_system(
    mut commands: Commands,
    mut ratatui: ResMut<RatatuiContext>,
    camera_widget: Query<&RatatuiCameraWidget>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> std::io::Result<()> {
    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        camera_widget
            .single()
            .render_autoresize(area, frame.buffer_mut(), &mut commands);
    })?;

    Ok(())
}

#[derive(Resource, Default, Clone)]
pub enum CameraState {
    #[default]
    Start,
    SwitchedStrategy,
    AddedEdges,
    ChangedCharacters,
    ChangedEdgeColor,
}

pub fn handle_input_system(
    world: &mut World,
    system_state: &mut SystemState<EventReader<KeyEvent>>,
    mut camera_state: Local<CameraState>,
) -> Result<(), RegisteredSystemError> {
    let mut event_reader = system_state.get_mut(world);
    let events: Vec<_> = event_reader.read().cloned().collect();

    for key_event in events.iter() {
        if let KeyEventKind::Press = key_event.kind {
            if let KeyCode::Char(' ') = key_event.code {
                match *camera_state {
                    CameraState::Start => {
                        world.run_system_cached(toggle_ratatui_camera_strategy)?;
                        *camera_state = CameraState::SwitchedStrategy;
                    }
                    CameraState::SwitchedStrategy => {
                        world.run_system_cached(toggle_edge_detection_system)?;
                        *camera_state = CameraState::AddedEdges;
                    }
                    CameraState::AddedEdges => {
                        world.run_system_cached(modify_ratatui_camera_strategy)?;
                        *camera_state = CameraState::ChangedCharacters;
                    }
                    CameraState::ChangedCharacters => {
                        world.run_system_cached(modify_edge_detection_system)?;
                        *camera_state = CameraState::ChangedEdgeColor;
                    }
                    CameraState::ChangedEdgeColor => {
                        world.run_system_cached(toggle_edge_detection_system)?;
                        world.run_system_cached(toggle_ratatui_camera_strategy)?;
                        *camera_state = CameraState::Start;
                    }
                }
            }
        }
    }

    Ok(())
}

fn toggle_edge_detection_system(
    mut commands: Commands,
    ratatui_camera: Query<(Entity, Option<&mut RatatuiCameraEdgeDetection>), With<RatatuiCamera>>,
) {
    let (entity, edge_detection) = ratatui_camera.single();
    if edge_detection.is_some() {
        commands
            .entity(entity)
            .remove::<RatatuiCameraEdgeDetection>();
    } else {
        commands
            .entity(entity)
            .insert(RatatuiCameraEdgeDetection::default());
    }
}

fn modify_edge_detection_system(
    mut ratatui_camera_edge_detection: Query<
        Option<&mut RatatuiCameraEdgeDetection>,
        With<RatatuiCamera>,
    >,
) {
    if let Some(mut c) = ratatui_camera_edge_detection.single_mut() {
        c.edge_color = Some(ratatui::style::Color::Magenta);
    }
}

fn modify_ratatui_camera_strategy(mut ratatui_camera_strategy: Query<&mut RatatuiCameraStrategy>) {
    let RatatuiCameraStrategy::Luminance(ref mut luminance_config) =
        *ratatui_camera_strategy.single_mut()
    else {
        return;
    };

    luminance_config.luminance_characters = vec!['.', 'o', 'O', '0'];
}

fn toggle_ratatui_camera_strategy(
    mut commands: Commands,
    mut ratatui_camera: Query<(Entity, &RatatuiCameraStrategy)>,
) {
    let (entity, strategy) = ratatui_camera.single_mut();
    commands.entity(entity).insert(match strategy {
        RatatuiCameraStrategy::HalfBlocks => {
            RatatuiCameraStrategy::Luminance(LuminanceConfig::default())
        }
        RatatuiCameraStrategy::Luminance(_) => RatatuiCameraStrategy::HalfBlocks,
        RatatuiCameraStrategy::None => RatatuiCameraStrategy::None,
    });
}
