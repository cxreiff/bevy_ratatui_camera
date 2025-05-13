use std::f32::consts::PI;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::color::Color;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_ratatui::RatatuiContext;
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui::event::MouseEvent;
use bevy_ratatui::kitty::KittyEnabled;
use bevy_ratatui_camera::RatatuiCamera;
use bevy_ratatui_camera::RatatuiCameraDepthBuffer;
use bevy_ratatui_camera::RatatuiCameraDepthDetection;
use bevy_ratatui_camera::RatatuiCameraLastArea;
use bevy_ratatui_camera::RatatuiCameraPlugin;
use bevy_ratatui_camera::RatatuiCameraWidget;
use crossterm::event::MouseEventKind;
use log::LevelFilter;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::StatefulWidgetRef;

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
        .add_systems(PreUpdate, shared::handle_input_system)
        .add_systems(
            Update,
            (
                shared::rotate_spinners_system,
                draw_scene_system,
                sphere_movement_system,
                mouse_follow_system,
            ),
        )
        .run();
}

#[derive(Component, Clone, Debug, Default)]
pub struct ConeMarker;

#[derive(Component, Clone, Debug, Default)]
pub struct CenterConeMarker;

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

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        ConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.2))),
        MeshMaterial3d(materials.add(Color::srgb(1., 0., 0.))),
    ));

    commands.spawn((
        ConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.2))),
        MeshMaterial3d(materials.add(Color::srgb(1., 1., 0.))),
    ));

    commands.spawn((
        ConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.2))),
        MeshMaterial3d(materials.add(Color::srgb(0., 0., 1.))),
    ));

    commands.spawn((
        CenterConeMarker,
        Mesh3d(meshes.add(Cone::new(0.5, 1.2))),
        MeshMaterial3d(materials.add(Color::srgb(0., 1., 0.))),
    ));

    commands.spawn((
        PointLight {
            intensity: 1_500_000.,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(4., 4., 4.),
    ));

    commands.spawn((
        RatatuiCamera::default(),
        RatatuiCameraDepthDetection,
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.5, 3.5).looking_at(Vec3::ZERO, Vec3::Y),
        Msaa::Off,
    ));
}

fn setup_labels_system(mut commands: Commands, cones: Query<Entity, With<ConeMarker>>) {
    let mut cones = cones.iter();
    commands.entity(cones.next().unwrap()).with_child((
        RatatuiTextLabel::new("red"),
        Transform::from_xyz(0., 0., 0.3),
    ));
    commands.entity(cones.next().unwrap()).with_child((
        RatatuiTextLabel::new("yellow"),
        Transform::from_xyz(0., 0., 0.3),
    ));
    commands.entity(cones.next().unwrap()).with_child((
        RatatuiTextLabel::new("blue"),
        Transform::from_xyz(0., 0., 0.3),
    ));
}

fn sphere_movement_system(mut cones: Query<&mut Transform, With<ConeMarker>>, time: Res<Time>) {
    let elapsed = time.elapsed_secs() * 0.5;
    for (i, mut cone) in cones.iter_mut().enumerate() {
        let elapsed_offset = elapsed + PI * (2. / 3.) * i as f32;
        cone.translation = Vec3::new(elapsed_offset.sin(), 0.0, elapsed_offset.cos());
    }
}

fn mouse_follow_system(
    mut mouse_events: EventReader<MouseEvent>,
    ratatui_camera: Single<(
        &Camera,
        &GlobalTransform,
        &RatatuiCameraWidget,
        &RatatuiCameraLastArea,
    )>,
    mut center_cone: Single<&mut Transform, With<CenterConeMarker>>,
) {
    let Some(mouse_position) = mouse_events
        .read()
        .last()
        .filter(|event| matches!(event.kind, MouseEventKind::Moved))
        .map(|event| IVec2::new(event.column as i32, event.row as i32))
    else {
        return;
    };

    let (camera, camera_transform, widget, last_area) = *ratatui_camera;

    let ndc = widget.cell_to_ndc(**last_area, mouse_position);

    let world_position = camera.ndc_to_world(camera_transform, ndc).unwrap();

    let viewport_position = camera
        .world_to_viewport(camera_transform, world_position)
        .unwrap();

    let ray = camera
        .viewport_to_world(camera_transform, viewport_position)
        .unwrap();

    let Some(intersect_d) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };

    let intersect = ray.get_point(intersect_d);

    center_cone.translation = intersect;
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

        let depth_buffer = &mut widget.new_depth_buffer(area);

        widget.render(area, frame.buffer_mut(), depth_buffer);

        // generate a widget for each label by converting its NDC coordinates to a buffer cell.
        let mut label_widgets = labels
            .iter()
            .filter_map(|(label, label_transform)| {
                let ndc = camera.world_to_ndc(camera_transform, label_transform.translation())?;
                let text = format!(
                    "{}: {:>+01.1}, {:>+01.1}, {:>+01.3}",
                    label.text.clone(),
                    ndc.x,
                    ndc.y,
                    ndc.z,
                );
                let IVec2 { x, y } = widget.ndc_to_cell(area, ndc);

                let depth = ndc.z;

                let overlay_widget = RatatuiTextLabelWidget { text, x, y, depth };

                Some(overlay_widget)
            })
            .collect::<Vec<_>>();

        // use `render_overlay_with_depth` to make sure area is corrected for aspect ratio, widget
        // is skipped during resize frames, and draws are occluded based on the depth buffer.
        while let Some(label_widget) = label_widgets.pop() {
            widget.render_overlay_with_depth(area, frame.buffer_mut(), &label_widget, depth_buffer);
        }
    })?;

    Ok(())
}

#[derive(Debug)]
pub struct RatatuiTextLabelWidget {
    text: String,
    x: i32,
    y: i32,
    depth: f32,
}

impl StatefulWidgetRef for RatatuiTextLabelWidget {
    type State = RatatuiCameraDepthBuffer;

    fn render_ref(&self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        let mut buffer = Buffer::empty(buf.area);

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

        {
            use ratatui::widgets::Widget;
            span.render(block.inner(label_area), &mut buffer);
            block.render(label_area, &mut buffer);
        }

        if left_cropped {
            let cell_coords = (x_adjusted as u16 + 1, y_adjusted as u16 + 1);
            if area.contains(cell_coords.into()) {
                if let Some(cell) = buffer.cell_mut(cell_coords) {
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
                if let Some(cell) = buffer.cell_mut(cell_coords) {
                    cell.set_char('…');
                }
            }
        }

        for i in label_area.x..(label_area.x + label_area.width) {
            for j in label_area.y..(label_area.y + label_area.height) {
                let position = (i, j);
                let Some(cell) = buf.cell_mut(position) else {
                    continue;
                };

                let Some(temp_cell) = buffer.cell(position) else {
                    continue;
                };

                let bg_draw = state.compare_and_update(
                    i as usize - area.x as usize,
                    (j as usize - area.y as usize) * 2,
                    self.depth,
                );
                let fg_draw = state.compare_and_update(
                    i as usize - area.x as usize,
                    (j as usize - area.y as usize) * 2 + 1,
                    self.depth,
                );

                if bg_draw.is_some_and(|draw| draw) {
                    cell.set_bg(temp_cell.bg);
                }

                if fg_draw.is_some_and(|draw| draw) {
                    cell.set_fg(temp_cell.fg).set_symbol(temp_cell.symbol());
                }
            }
        }
    }
}
