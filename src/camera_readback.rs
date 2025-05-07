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
    RatatuiCamera, RatatuiCameraEdgeDetection, RatatuiCameraSet, RatatuiCameraStrategy,
    RatatuiCameraWidget, RatatuiSubcamera, RatatuiSubcameras,
    camera::RatatuiCameraLastArea,
    camera_image_pipe::{
        ImageReceiver, ImageSender, create_image_pipe, receive_image, send_image_buffer,
    },
};

pub struct RatatuiCameraReadbackPlugin;

impl Plugin for RatatuiCameraReadbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<RatatuiCameraSender>::default(),
            ExtractComponentPlugin::<RatatuiDepthSender>::default(),
            ExtractComponentPlugin::<RatatuiSobelSender>::default(),
        ))
        .add_event::<CameraTargetingEvent>()
        .add_observer(handle_ratatui_camera_insert_observer)
        .add_observer(handle_ratatui_subcamera_insert_observer)
        .add_observer(ratatui_depth_readback_insert_observer)
        .add_observer(handle_ratatui_edge_detection_insert_observer)
        .add_observer(handle_ratatui_camera_removal_observer)
        .add_observer(ratatui_depth_readback_removal_observer)
        .add_observer(handle_ratatui_edge_detection_removal_observer)
        .add_observer(resize_ratatui_camera_observer)
        .add_systems(
            First,
            (
                create_ratatui_camera_widgets_system,
                handle_camera_targeting_events_system,
                (
                    update_ratatui_camera_readback_system,
                    update_ratatui_depth_readback_system,
                    update_ratatui_edge_detection_readback_system,
                    receive_camera_images_system,
                    receive_depth_images_system,
                    receive_sobel_images_system,
                ),
            )
                .chain()
                .in_set(RatatuiCameraSet),
        );

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                send_camera_images_system,
                send_depth_images_system,
                send_sobel_images_system,
            )
                .after(RenderSet::Render),
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

#[derive(Component, ExtractComponent, Deref, DerefMut, Clone, Debug)]
pub struct RatatuiDepthSender(ImageSender);

#[derive(Component, Deref, DerefMut, Debug)]
pub struct RatatuiDepthReceiver(ImageReceiver);

#[derive(Event, Debug)]
pub struct CameraTargetingEvent {
    pub targeter_entity: Entity,
    pub target_entity: Entity,
}

fn handle_ratatui_camera_insert_observer(
    trigger: Trigger<OnInsert, RatatuiCamera>,
    mut commands: Commands,
    ratatui_cameras: Query<&RatatuiCamera>,
    mut camera_targeting_event: EventWriter<CameraTargetingEvent>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if let Ok(ratatui_camera) = ratatui_cameras.get(trigger.target()) {
        insert_camera_readback_components(
            commands.reborrow(),
            trigger.target(),
            &mut image_assets,
            &render_device,
            ratatui_camera,
            &mut camera_targeting_event,
        );
    }
}

fn handle_ratatui_subcamera_insert_observer(
    trigger: Trigger<OnInsert, RatatuiSubcamera>,
    mut ratatui_subcameras: Query<&RatatuiSubcamera>,
    mut camera_targeting_event: EventWriter<CameraTargetingEvent>,
) {
    let RatatuiSubcamera(target_entity) = ratatui_subcameras.get_mut(trigger.target()).unwrap();

    camera_targeting_event.write(CameraTargetingEvent {
        targeter_entity: trigger.target(),
        target_entity: *target_entity,
    });
}

fn ratatui_depth_readback_insert_observer(
    trigger: Trigger<OnInsert, RatatuiCamera>,
    mut commands: Commands,
    ratatui_cameras: Query<&RatatuiCamera>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if let Ok(ratatui_camera) = ratatui_cameras.get(trigger.target()) {
        insert_ratatui_depth_readback_components(
            commands.reborrow(),
            trigger.target(),
            &mut image_assets,
            &render_device,
            ratatui_camera,
        );
    }
}

fn handle_ratatui_edge_detection_insert_observer(
    trigger: Trigger<OnInsert, RatatuiCameraEdgeDetection>,
    mut commands: Commands,
    ratatui_cameras: Query<&RatatuiCamera>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    if let Ok(ratatui_camera) = ratatui_cameras.get(trigger.target()) {
        insert_edge_detection_readback_components(
            commands.reborrow(),
            trigger.target(),
            &mut image_assets,
            &render_device,
            ratatui_camera,
        );
    }
}

fn handle_ratatui_camera_removal_observer(
    trigger: Trigger<OnRemove, RatatuiCamera>,
    mut commands: Commands,
) {
    let mut entity = commands.entity(trigger.target());
    entity.remove::<(RatatuiCameraSender, RatatuiCameraReceiver)>();
}

fn ratatui_depth_readback_removal_observer(
    trigger: Trigger<OnRemove, RatatuiCamera>,
    mut commands: Commands,
) {
    let mut entity = commands.entity(trigger.target());
    entity.remove::<(RatatuiDepthSender, RatatuiDepthReceiver)>();
}

fn handle_ratatui_edge_detection_removal_observer(
    trigger: Trigger<OnRemove, RatatuiCameraEdgeDetection>,
    mut commands: Commands,
) {
    let mut entity = commands.entity(trigger.target());
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
            commands.reborrow(),
            entity,
            &mut image_assets,
            &render_device,
            ratatui_camera,
            &mut camera_targeting_event,
        );
    }
}

fn update_ratatui_depth_readback_system(
    mut commands: Commands,
    ratatui_cameras: Query<(Entity, &RatatuiCamera), Changed<RatatuiCamera>>,
    mut image_assets: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    for (entity, ratatui_camera) in &ratatui_cameras {
        insert_ratatui_depth_readback_components(
            commands.reborrow(),
            entity,
            &mut image_assets,
            &render_device,
            ratatui_camera,
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
            commands.reborrow(),
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

fn send_depth_images_system(
    ratatui_depth_senders: Query<&RatatuiDepthSender>,
    render_device: Res<RenderDevice>,
) {
    for depth_sender in &ratatui_depth_senders {
        send_image_buffer(&render_device, &depth_sender.buffer, &depth_sender.sender);
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

fn receive_depth_images_system(mut depth_receivers: Query<&mut RatatuiDepthReceiver>) {
    for mut depth_receiver in &mut depth_receivers {
        receive_image(&mut depth_receiver);
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
        &RatatuiCameraStrategy,
        &RatatuiCameraLastArea,
        Option<&RatatuiCameraEdgeDetection>,
        &RatatuiCameraReceiver,
        &RatatuiDepthReceiver,
        Option<&RatatuiSobelReceiver>,
    )>,
) {
    for (
        entity_id,
        strategy,
        last_area,
        edge_detection,
        camera_receiver,
        depth_receiver,
        sobel_receiver,
    ) in &ratatui_cameras
    {
        let mut entity = commands.entity(entity_id);

        let camera_image = match camera_receiver.receiver_image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create camera image from buffer {e:?}"),
        };

        let depth_image = match depth_receiver.receiver_image.clone().try_into_dynamic() {
            Ok(image) => image,
            Err(e) => panic!("failed to create depth image from buffer {e:?}"),
        };

        let sobel_image = sobel_receiver.as_ref().map(|image_sobel| {
            match image_sobel.receiver_image.clone().try_into_dynamic() {
                Ok(image) => image,
                Err(e) => panic!("failed to create sobel image buffer {e:?}"),
            }
        });

        let widget = RatatuiCameraWidget {
            entity: entity_id,
            camera_image,
            depth_image,
            sobel_image,
            strategy: strategy.clone(),
            edge_detection: edge_detection.cloned(),
            last_area: **last_area,
            next_last_area: **last_area,
        };

        entity.insert(widget);
    }
}

fn resize_ratatui_camera_observer(
    trigger: Trigger<OnReplace, RatatuiCameraWidget>,
    mut commands: Commands,
    widgets: Query<(&RatatuiCameraWidget, &RatatuiCameraLastArea)>,
    mut ratatui_cameras: Query<&mut RatatuiCamera>,
) -> Result {
    let (widget, last_area) = widgets.get(trigger.target())?;

    commands
        .entity(trigger.target())
        .insert(RatatuiCameraLastArea(widget.next_last_area));

    if last_area.width == widget.next_last_area.width
        && last_area.height == widget.next_last_area.height
    {
        return Ok(());
    }

    if !ratatui_cameras.get(trigger.target())?.autoresize {
        return Ok(());
    }

    let mut ratatui_camera = ratatui_cameras.get_mut(trigger.target())?;
    ratatui_camera.dimensions = UVec2::new(
        (widget.next_last_area.width as u32 * 2).max(1),
        (widget.next_last_area.height as u32 * 4).max(1),
    );

    Ok(())
}

// TODO: When observers can be explicitly ordered, use another observer ordered after the
// RatatuiCamera observers instead.
//
/// Handles camera targeting events to point cameras at the correct render targets. An event
/// handler is used here instead of observers to make sure that the render target is created after
/// the RatatuiCamera insert/update observers run and so the camera entity definitely already has
/// its RatatuiCameraSender component. Otherwise, for example, if a RatatuiCamera and related
/// RatatuiSubcamera is spawned in a single system run, we could potentially try to update the
/// subcamera's render target before the main camera's render texture is created.
fn handle_camera_targeting_events_system(
    target_cameras: Query<(&RatatuiCameraSender, Option<&RatatuiSubcameras>), With<RatatuiCamera>>,
    mut cameras: Query<&mut Camera>,
    mut camera_targeting_events: EventReader<CameraTargetingEvent>,
) {
    for CameraTargetingEvent {
        targeter_entity,
        target_entity,
    } in camera_targeting_events.read()
    {
        let (sender, targeting_subcameras) = target_cameras
            .get(*target_entity)
            .expect("CameraTargetingEvent sent with invalid targeting entity");

        let render_target = RenderTarget::from(sender.sender_image.clone());

        if let Some(targeting_subcameras) = targeting_subcameras {
            for targeting_subcamera in targeting_subcameras.iter() {
                if let Ok(mut camera) = cameras.get_mut(targeting_subcamera) {
                    camera.target = render_target.clone()
                }
            }
        }

        let mut camera = cameras
            .get_mut(*targeter_entity)
            .expect("CameraTargetingEvent sent with invalid target entity");

        camera.target = render_target;
    }
}

fn insert_camera_readback_components(
    mut commands: Commands,
    entity: Entity,
    image_assets: &mut Assets<Image>,
    render_device: &RenderDevice,
    ratatui_camera: &RatatuiCamera,
    camera_targeting_event: &mut EventWriter<CameraTargetingEvent>,
) {
    let mut entity_commands = commands.entity(entity);

    let (sender, receiver) =
        create_image_pipe(image_assets, render_device, ratatui_camera.dimensions);

    camera_targeting_event.write(CameraTargetingEvent {
        targeter_entity: entity,
        target_entity: entity,
    });

    entity_commands.insert((RatatuiCameraSender(sender), RatatuiCameraReceiver(receiver)));
}

fn insert_edge_detection_readback_components(
    mut commands: Commands,
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

fn insert_ratatui_depth_readback_components(
    mut commands: Commands,
    entity: Entity,
    image_assets: &mut Assets<Image>,
    render_device: &RenderDevice,
    ratatui_camera: &RatatuiCamera,
) {
    let mut entity = commands.entity(entity);

    let (sender, receiver) =
        create_image_pipe(image_assets, render_device, ratatui_camera.dimensions);

    entity.insert((
        RatatuiDepthSender(sender),
        RatatuiDepthReceiver(receiver),
        DepthPrepass,
        Msaa::Off,
    ));
}
