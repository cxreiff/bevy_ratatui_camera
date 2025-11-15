use bevy::{
    core_pipeline::{
        core_2d::graph::{Core2d, Node2d},
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderApp,
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            Buffer, CommandEncoderDescriptor, Extent3d, TexelCopyBufferInfo, TexelCopyBufferLayout,
            Texture,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        view::ViewDepthTexture,
    },
};

use crate::{
    camera_image_pipe::calculate_buffer_size,
    camera_readback::{RatatuiCameraSender, RatatuiDepthSender, RatatuiSobelSender},
};

pub struct RatatuiCameraNodePlugin;

impl Plugin for RatatuiCameraNodePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_render_graph_node::<ViewNodeRunner<RatatuiCameraNode>>(Core3d, RatatuiCameraLabel);
        render_app.add_render_graph_edge(Core3d, Node3d::Upscaling, RatatuiCameraLabel);

        render_app
            .add_render_graph_node::<ViewNodeRunner<RatatuiCameraNode>>(Core2d, RatatuiCameraLabel);
        render_app.add_render_graph_edge(Core2d, Node2d::Upscaling, RatatuiCameraLabel);
    }
}

#[derive(Default)]
pub struct RatatuiCameraNode;

#[derive(RenderLabel, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RatatuiCameraLabel;

impl ViewNode for RatatuiCameraNode {
    type ViewQuery = (
        &'static ViewDepthTexture,
        &'static RatatuiCameraSender,
        Option<&'static RatatuiDepthSender>,
        Option<&'static RatatuiSobelSender>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (depth_texture, camera_sender, depth_sender, sobel_sender): QueryItem<
            'w,
            '_,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();

        let src_image = gpu_images.get(&camera_sender.sender_image).unwrap();
        copy_texture_to_buffer(
            render_context,
            world,
            &src_image.texture,
            &camera_sender.buffer,
        );

        if let Some(depth_sender) = depth_sender {
            let expected_buffer_size = calculate_buffer_size(
                depth_texture.texture.width(),
                depth_texture.texture.height(),
            );
            if expected_buffer_size == depth_sender.buffer.size() {
                copy_texture_to_buffer(
                    render_context,
                    world,
                    &depth_texture.texture,
                    &depth_sender.buffer,
                );
            }
        }

        if let Some(sobel_sender) = sobel_sender {
            let src_image_sobel = gpu_images.get(&sobel_sender.sender_image).unwrap();
            copy_texture_to_buffer(
                render_context,
                world,
                &src_image_sobel.texture,
                &sobel_sender.buffer,
            );
        }

        Ok(())
    }
}

fn copy_texture_to_buffer(
    render_context: &mut RenderContext,
    world: &World,
    src_texture: &Texture,
    buffer: &Buffer,
) {
    let mut encoder = render_context
        .render_device()
        .create_command_encoder(&CommandEncoderDescriptor::default());

    let block_dimensions = src_texture.format().block_dimensions();
    let block_size = src_texture.format().block_copy_size(None).unwrap();

    let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
        (src_texture.width() as usize / block_dimensions.0 as usize) * block_size as usize,
    );

    let texture_extent = Extent3d {
        width: src_texture.width(),
        height: src_texture.height(),
        depth_or_array_layers: 1,
    };

    encoder.copy_texture_to_buffer(
        src_texture.as_image_copy(),
        TexelCopyBufferInfo {
            buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                rows_per_image: None,
                bytes_per_row: Some(
                    std::num::NonZeroU32::new(padded_bytes_per_row as u32)
                        .unwrap()
                        .into(),
                ),
            },
        },
        texture_extent,
    );

    let render_queue = world.get_resource::<RenderQueue>().unwrap();
    render_queue.submit(std::iter::once(encoder.finish()));
}
