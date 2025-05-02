use std::f32::consts::PI;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::color::Color;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::math::U16Vec2;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraOverlayWidget;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraWidget;
use log::LevelFilter;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;
use shared::Spinner;

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
        .add_systems(Startup, (setup_scene_system, setup_labels_system).chain())
        .add_systems(Update, draw_scene_system)
        .add_systems(PreUpdate, shared::handle_input_system)
        .add_systems(Update, shared::rotate_spinners_system)
        .add_systems(Update, sphere_movement_system)
        .run();
}

#[derive(Component, Clone, Debug, Default)]
pub struct ConeMarker;

#[derive(Component, Clone, Debug, Default)]
#[require(Transform)]
pub struct RatatuiTextLabel {
    text: String,
}

impl RatatuiTextLabel {
    fn new(text: &str) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug)]
pub struct RatatuiTextLabelWidget {
    text: String,
    anchor: RatatuiTextLabelWidgetAnchor,
    x: u16,
    y: u16,
}

#[derive(Debug)]
enum RatatuiTextLabelWidgetAnchor {
    Left,
    Center,
}

impl WidgetRef for RatatuiTextLabelWidget {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut width = self.text.len() as u16 + 4;
        let height = 3;
        let mut span = Line::from(format!(" {} ", self.text.clone()));

        let x = if let RatatuiTextLabelWidgetAnchor::Center = self.anchor {
            if width / 2 > self.x {
                width = width / 2 + self.x;
                span = span.right_aligned();
            }

            (area.x + self.x).saturating_sub(width / 2)
        } else {
            area.x + self.x
        };

        let x_adjusted = x.max(area.x);
        let y_adjusted = (area.y + self.y).max(area.y);
        let width_adjusted = width.min(area.x + area.width.saturating_sub(x_adjusted));
        let height_adjusted = height.min(area.y + area.height.saturating_sub(y_adjusted));

        let label_area = Rect {
            x: x_adjusted,
            y: y_adjusted,
            width: width_adjusted,
            height: height_adjusted,
        };

        let block = Block::bordered()
            .fg(ratatui::style::Color::Green)
            .bg(ratatui::style::Color::Black);

        span.render(block.inner(label_area), buf);
        block.render(label_area, buf);
    }
}

impl RatatuiCameraOverlayWidget for RatatuiTextLabelWidget {}

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        ConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.))),
        MeshMaterial3d(materials.add(Color::srgb(1., 0., 0.))),
        Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
    ));

    shared::spawn_3d_scene(commands.reborrow(), meshes, materials);

    commands.spawn((
        RatatuiCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(2.5, 2.5, 2.5).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn setup_labels_system(
    mut commands: Commands,
    cube: Single<Entity, With<Spinner>>,
    cone: Single<Entity, With<ConeMarker>>,
) {
    commands
        .entity(*cube)
        .with_child((RatatuiTextLabel::new("cube"),));
    commands
        .entity(*cone)
        .with_child((RatatuiTextLabel::new("cone"),));
}

fn sphere_movement_system(
    mut cube: Single<&mut Transform, With<Spinner>>,
    mut cone: Single<&mut Transform, (With<ConeMarker>, Without<Spinner>)>,
    time: Res<Time>,
) {
    let elapsed = time.elapsed_secs() * 0.5;
    let elapsed_plus_pi = elapsed + PI;
    cube.translation = Vec3::new(elapsed.sin(), elapsed.cos(), 0.);
    cone.translation = Vec3::new(elapsed_plus_pi.sin(), elapsed_plus_pi.cos(), 0.);
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    mut ratatui_camera_single: Single<(&Camera, &GlobalTransform, &mut RatatuiCameraWidget)>,
    labels: Query<(&RatatuiTextLabel, &GlobalTransform)>,
    flags: Res<shared::Flags>,
    diagnostics: Res<DiagnosticsStore>,
    kitty_enabled: Option<Res<KittyEnabled>>,
) -> Result {
    let (camera, camera_transform, ref mut widget) = *ratatui_camera_single;

    ratatui.draw(|frame| {
        let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());

        let mut label_widgets = labels
            .iter()
            .filter_map(|(label, label_transform)| {
                let anchor = RatatuiTextLabelWidgetAnchor::Center;
                let ndc = camera.world_to_ndc(camera_transform, label_transform.translation())?;
                let text = format!(
                    "{}: {:.1}, {:.1}, {:.1}",
                    label.text.clone(),
                    ndc.x,
                    ndc.y,
                    ndc.z,
                );
                let U16Vec2 { x, y } = widget.ndc_to_cell(area, ndc);

                let overlay_widget = RatatuiTextLabelWidget { text, anchor, x, y };

                Some((overlay_widget, ndc.z))
            })
            .collect::<Vec<_>>();

        label_widgets.sort_by(|(_, a), (_, b)| a.total_cmp(b).reverse());

        while let Some((label_widget, _)) = label_widgets.pop() {
            widget.push_overlay_widget(Box::new(label_widget));
        }

        widget.push_overlay_widget(Box::new(RatatuiTextLabelWidget {
            text: format!(
                "width: {}, height: {}",
                frame.area().width,
                frame.area().height
            ),
            x: 1,
            y: 1,
            anchor: RatatuiTextLabelWidgetAnchor::Left,
        }));

        widget.render(area, frame.buffer_mut());
    })?;

    Ok(())
}
