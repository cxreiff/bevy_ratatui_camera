use bevy::{
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        camera::RenderTarget,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        renderer::RenderDevice,
    },
};

use crate::{
    RatatuiCamera, RatatuiCameraEdgeDetection, RatatuiCameraStrategy, RatatuiCameraWidget,
    RatatuiSubcamera,
    camera_image_pipe::{
        ImageReceiver, ImageSender, create_image_pipe, receive_image, send_image_buffer,
    },
};

pub struct RatatuiCameraReadbackPlugin;

impl Plugin for RatatuiCameraReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<RatatuiCameraSender>::default(),
            ExtractComponentPlugin::<RatatuiSobelSender>::default(),
        ))
        .add_event::<CameraTargetingEvent>()
        .add_observer(handle_ratatui_camera_insert_system)
        .add_observer(handle_ratatui_camera_removal_system)
        .add_observer(handle_ratatui_edge_detection_insert_system)
        .add_observer(handle_ratatui_edge_detection_removal_system)
        .add_observer(handle_ratatui_subcamera_insert_system)
        .add_systems(
            First,
            (
                handle_camera_targeting_events_system,
                (
                    update_ratatui_camera_readback_system,
                    update_ratatui_edge_detection_readback_system,
                    receive_camera_images_system,
                    receive_sobel_images_system,
                ),
                create_ratatui_camera_widgets_system,
            )
                .chain(),
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (send_camera_images_system, send_sobel_images_system).after(RenderSet::Render),
        );
    }
}

#[derive(Component, ExtractComponent, Deref, DerefMut, Clone, Debug)]
pub struct RatatuiCameraSender(ImageSender);

#[derive(Component, Deref, DerefMut, Debug)]
pub struct RatatuiCameraReceiver(ImageReceiver);

#[derive(Component, ExtractComponent, Deref, DerefMut, Clone, Debug)]
pub struct RatatuiSobelSender(ImageSender);

#[derive(Component, Deref, DerefMut, Debug)]
pub struct RatatuiSobelReceiver(ImageReceiver);

#[derive(Event, Debug)]
pub struct CameraTargetingEvent {
    pub targeter_entity: Entity,
    pub target_entity: Entity,
}

#[derive(Event)]
pub struct RatatuiCameraResize {
    pub dimensions: (u32, u32),
}

fn handle_ratatui_camera_insert_system(
    trigger: Trigger<OnInsert, RatatuiCamera>,
    mut commands: Commands,
    ratatui_cameras: Query<&RatatuiCamera>,
    mut camera_targeting_event: EventWriter<CameraTargetingEvent>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if let Ok(ratatui_camera) = ratatui_cameras.get(trigger.entity()) {
        commands
            .entity(trigger.entity())
            .observe(handle_ratatui_camera_resize);

        insert_camera_readback_components(
            &mut commands,
            trigger.entity(),
            &mut image_assets,
            &render_device,
            ratatui_camera,
            &mut camera_targeting_event,
        );
    }
}

fn handle_ratatui_camera_resize(
    trigger: Trigger<RatatuiCameraResize>,
    mut ratatui_cameras: Query<&mut RatatuiCamera>,
) {
    let mut ratatui_camera = ratatui_cameras.get_mut(trigger.entity()).unwrap();
    ratatui_camera.dimensions = trigger.event().dimensions;
}

fn handle_ratatui_subcamera_insert_system(
    trigger: Trigger<OnInsert, RatatuiSubcamera>,
    mut ratatui_subcameras: Query<&RatatuiSubcamera>,
    mut camera_targeting_event: EventWriter<CameraTargetingEvent>,
) {
    let RatatuiSubcamera(target_entity) = ratatui_subcameras.get_mut(trigger.entity()).unwrap();

    camera_targeting_event.send(CameraTargetingEvent {
        targeter_entity: trigger.entity(),
        target_entity: *target_entity,
    });
}

fn handle_ratatui_camera_removal_system(
    trigger: Trigger<OnRemove, RatatuiCamera>,
    mut commands: Commands,
) {
    let mut entity = commands.entity(trigger.entity());
    entity.remove::<(RatatuiCameraSender, RatatuiCameraReceiver)>();
}

fn handle_ratatui_edge_detection_insert_system(
    trigger: Trigger<OnInsert, RatatuiCameraEdgeDetection>,
    mut commands: Commands,
    ratatui_cameras: Query<&RatatuiCamera>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if let Ok(ratatui_camera) = ratatui_cameras.get(trigger.entity()) {
        insert_edge_detection_readback_components(
            &mut commands,
            trigger.entity(),
            &mut image_assets,
            &render_device,
            ratatui_camera,
        );
    }
}

fn handle_ratatui_edge_detection_removal_system(
    trigger: Trigger<OnRemove, RatatuiCameraEdgeDetection>,
    mut commands: Commands,
) {
    let mut entity = commands.entity(trigger.entity());
    entity.remove::<(RatatuiSobelSender, RatatuiSobelReceiver)>();
}

fn update_ratatui_camera_readback_system(
    mut commands: Commands,
    ratatui_cameras: Query<(Entity, &RatatuiCamera), Changed<RatatuiCamera>>,
    mut camera_targeting_event: EventWriter<CameraTargetingEvent>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, ratatui_camera) in &ratatui_cameras {
        insert_camera_readback_components(
            &mut commands,
            entity,
            &mut image_assets,
            &render_device,
            ratatui_camera,
            &mut camera_targeting_event,
        );
    }
}

fn update_ratatui_edge_detection_readback_system(
    mut commands: Commands,
    ratatui_cameras: Query<
        (Entity, &RatatuiCamera),
        (With<RatatuiCameraEdgeDetection>, Changed<RatatuiCamera>),
    >,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, ratatui_camera) in &ratatui_cameras {
        insert_edge_detection_readback_components(
            &mut commands,
            entity,
            &mut image_assets,
            &render_device,
            ratatui_camera,
        );
    }
}

fn send_camera_images_system(
    ratatui_camera_senders: Query<&RatatuiCameraSender>,
    render_device: Res<RenderDevice>,
) {
    for camera_sender in &ratatui_camera_senders {
        send_image_buffer(&render_device, &camera_sender.buffer, &camera_sender.sender);
    }
}

fn send_sobel_images_system(
    ratatui_sobel_senders: Query<&RatatuiSobelSender>,
    render_device: Res<RenderDevice>,
) {
    for sobel_sender in &ratatui_sobel_senders {
        send_image_buffer(&render_device, &sobel_sender.buffer, &sobel_sender.sender);
    }
}

fn receive_camera_images_system(mut camera_receivers: Query<&mut RatatuiCameraReceiver>) {
    for mut camera_receiver in &mut camera_receivers {
        receive_image(&mut camera_receiver);
    }
}

fn receive_sobel_images_system(mut sobel_receivers: Query<&mut RatatuiSobelReceiver>) {
    for mut sobel_receiver in &mut sobel_receivers {
        receive_image(&mut sobel_receiver);
    }
}

fn create_ratatui_camera_widgets_system(
    mut commands: Commands,
    ratatui_cameras: Query<(
        Entity,
        &RatatuiCamera,
        &RatatuiCameraStrategy,
        Option<&RatatuiCameraEdgeDetection>,
        &RatatuiCameraReceiver,
        Option<&RatatuiSobelReceiver>,
    )>,
) {
    for (entity_id, ratatui_camera, strategy, edge_detection, camera_receiver, sobel_receiver) in
        &ratatui_cameras
    {
        let mut entity = commands.entity(entity_id);

        let camera_image = match camera_receiver.receiver_image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create camera image buffer {e:?}"),
        };

        let sobel_image = sobel_receiver.as_ref().map(|image_sobel| {
            match image_sobel.receiver_image.clone().try_into_dynamic() {
                Ok(image) => image,
                Err(e) => panic!("failed to create sobel image buffer {e:?}"),
            }
        });

        let widget = RatatuiCameraWidget {
            entity: entity_id,
            ratatui_camera: ratatui_camera.clone(),
            camera_image,
            sobel_image,
            strategy: strategy.clone(),
            edge_detection: edge_detection.cloned(),
        };

        entity.insert(widget);
    }
}

// TODO: When observers can be explicitly ordered, use another observer ordered after the
// RatatuiCamera observers instead.
//
// TODO: When bevy 0.16 relations arrive, turn RatatuiSubcamera into relation and use automatically
// managed "RatatuiSubcameras" component instead of iterating through targeter_cameras for matching
// subcameras to update their render target.
//
/// Handles camera targeting events to point cameras at the correct render targets. An event
/// handler is used here instead of observers to make sure that the render target is created after
/// the RatatuiCamera insert/update observers run and so the camera entity definitely already has
/// its RatatuiCameraSender component. Otherwise, for example, if a RatatuiCamera and related
/// RatatuiSubcamera is spawned in a single system run, we could potentially try to update the
/// subcamera's render target before the main camera's render texture is created.
fn handle_camera_targeting_events_system(
    target_cameras: Query<&RatatuiCameraSender, With<RatatuiCamera>>,
    mut targeter_cameras: Query<(&mut Camera, Option<&RatatuiSubcamera>)>,
    mut camera_targeting_events: EventReader<CameraTargetingEvent>,
) {
    for CameraTargetingEvent {
        targeter_entity,
        target_entity,
    } in camera_targeting_events.read()
    {
        let sender = target_cameras
            .get(*target_entity)
            .expect("CameraTargetingEvent sent with invalid targeting entity");

        let render_target = RenderTarget::from(sender.sender_image.clone());

        for (mut subcamera, _) in targeter_cameras.iter_mut().filter(|(_, subcamera_entity)| {
            if let Some(RatatuiSubcamera(subcamera_entity)) = subcamera_entity {
                return target_entity == subcamera_entity;
            }

            false
        }) {
            subcamera.target = render_target.clone();
        }

        let (mut camera, _) = targeter_cameras
            .get_mut(*targeter_entity)
            .expect("CameraTargetingEvent sent with invalid target entity");

        camera.target = render_target;
    }
}

fn insert_camera_readback_components(
    commands: &mut Commands,
    entity: Entity,
    image_assets: &mut Assets<Image>,
    render_device: &RenderDevice,
    ratatui_camera: &RatatuiCamera,
    camera_targeting_event: &mut EventWriter<CameraTargetingEvent>,
) {
    let mut entity_commands = commands.entity(entity);

    let (sender, receiver) =
        create_image_pipe(image_assets, render_device, ratatui_camera.dimensions);

    camera_targeting_event.send(CameraTargetingEvent {
        targeter_entity: entity,
        target_entity: entity,
    });

    entity_commands.insert((RatatuiCameraSender(sender), RatatuiCameraReceiver(receiver)));
}

fn insert_edge_detection_readback_components(
    commands: &mut Commands,
    entity: Entity,
    image_assets: &mut Assets<Image>,
    render_device: &RenderDevice,
    ratatui_camera: &RatatuiCamera,
) {
    let mut entity = commands.entity(entity);

    let (sender, receiver) =
        create_image_pipe(image_assets, render_device, ratatui_camera.dimensions);

    entity.insert((
        RatatuiSobelSender(sender),
        RatatuiSobelReceiver(receiver),
        DepthPrepass,
        NormalPrepass,
        Msaa::Off,
    ));
}
