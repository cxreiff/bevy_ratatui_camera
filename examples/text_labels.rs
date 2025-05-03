use std::f32::consts::PI;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::color::Color;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui::terminal::RatatuiContext;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraWidget;
use log::LevelFilter;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;

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
            RatatuiPlugins {
                enable_mouse_capture: true,
                ..default()
            },
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
    x: i32,
    y: i32,
}

impl WidgetRef for RatatuiTextLabelWidget {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let mut width = self.text.len() as u16 + 4;
        let height = 3;
        let mut span = Line::from(format!(" {} ", self.text.clone()));
        let mut left_cropped = false;
        let mut right_cropped = false;

        let x = {
            let left_margin = self.x - area.x as i32;
            if width as i32 / 2 > left_margin {
                width = ((width as i32 / 2) + left_margin).max(0) as u16;
                span = span.right_aligned();
                left_cropped = true;
            }

            self.x - (width / 2) as i32
        };

        if width < 3 {
            return;
        }

        let x_adjusted = x.max(area.x as i32);
        let y_adjusted = self.y.max(area.y as i32);

        let max_width = ((area.x as i32 + area.width as i32) - x).max(0) as u16;
        if width > max_width {
            right_cropped = true;
            if max_width < 3 {
                return;
            }
        }
        let width_adjusted = width.min(max_width);
        let max_height = (area.y + area.height).saturating_sub(y_adjusted.max(0) as u16);
        if max_height < 3 {
            return;
        }
        let height_adjusted = height.min(max_height);

        if x_adjusted < 0 || y_adjusted < 0 {
            return;
        }

        let label_area = Rect {
            x: x_adjusted as u16,
            y: y_adjusted as u16,
            width: width_adjusted,
            height: height_adjusted,
        };

        let block = Block::bordered()
            .fg(ratatui::style::Color::White)
            .bg(ratatui::style::Color::Black);

        span.render(block.inner(label_area), buf);
        block.render(label_area, buf);

        if left_cropped {
            let cell_coords = (x_adjusted as u16 + 1, y_adjusted as u16 + 1);
            if area.contains(cell_coords.into()) {
                if let Some(cell) = buf.cell_mut(cell_coords) {
                    cell.set_char('…');
                }
            }
        }

        if right_cropped {
            let cell_coords = (
                x_adjusted as u16 + width_adjusted as u16 - 2,
                y_adjusted as u16 + 1,
            );
            if area.contains(cell_coords.into()) {
                if let Some(cell) = buf.cell_mut(cell_coords) {
                    cell.set_char('…');
                }
            }
        }
    }
}

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

    commands.spawn((
        ConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.))),
        MeshMaterial3d(materials.add(Color::srgb(1., 1., 0.))),
        Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
    ));

    commands.spawn((
        ConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.))),
        MeshMaterial3d(materials.add(Color::srgb(0., 0., 1.))),
        Transform::from_rotation(Quat::from_rotation_x(PI / 2.)),
    ));

    commands.spawn((
        PointLight {
            intensity: 2_000_000.,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(3., 4., 6.),
    ));

    commands.spawn((
        RatatuiCamera::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 4., 3.).looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

fn setup_labels_system(mut commands: Commands, cones: Query<Entity, With<ConeMarker>>) {
    let mut cones = cones.iter();
    commands
        .entity(cones.next().unwrap())
        .with_child(RatatuiTextLabel::new("red"));
    commands
        .entity(cones.next().unwrap())
        .with_child(RatatuiTextLabel::new("yellow"));
    commands
        .entity(cones.next().unwrap())
        .with_child(RatatuiTextLabel::new("blue"));
}

fn sphere_movement_system(mut cones: Query<&mut Transform, With<ConeMarker>>, time: Res<Time>) {
    let elapsed = time.elapsed_secs() * 0.5;
    for (i, mut cone) in cones.iter_mut().enumerate() {
        let elapsed_offset = elapsed + PI * (2. / 3.) * i as f32;
        cone.translation = Vec3::new(elapsed_offset.sin(), elapsed_offset.cos(), 0.);
    }
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

        widget.render(area, frame.buffer_mut());

        // generate a widget for each label by converting its NDC coordinates to a buffer cell.
        let mut label_widgets = labels
            .iter()
            .filter_map(|(label, label_transform)| {
                let ndc = camera.world_to_ndc(camera_transform, label_transform.translation())?;
                let text = format!(
                    "{}: {:.1}, {:.1}, {:.1}",
                    label.text.clone(),
                    ndc.x,
                    ndc.y,
                    ndc.z,
                );
                let IVec2 { x, y } = widget.ndc_to_cell(area, ndc);

                let overlay_widget = RatatuiTextLabelWidget { text, x, y };

                Some((overlay_widget, ndc.z))
            })
            .collect::<Vec<_>>();

        // sort by camera-space depth so "further" labels are covered by "closer" labels.
        label_widgets.sort_by(|(_, a), (_, b)| a.total_cmp(b).reverse());

        // use `render_overlay` to make sure area is corrected for aspect ratio and widget is
        // skipped during resize frames.
        while let Some((label_widget, _)) = label_widgets.pop() {
            widget.render_overlay(area, frame.buffer_mut(), &label_widget);
        }
    })?;

    Ok(())
}
