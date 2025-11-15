use std::path::Path;

use bevy::{
    asset::{AssetPath, embedded_asset, io::AssetSourceId},
    core_pipeline::{
        FullscreenShader,
        core_3d::{
            DEPTH_TEXTURE_SAMPLING_SUPPORTED,
            graph::{Core3d, Node3d},
        },
        prepass::ViewPrepassTextures,
    },
    ecs::query::QueryItem,
    platform::collections::HashMap,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        extract_component::ExtractComponentPlugin,
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedPipelineState,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
            UniformBuffer,
            binding_types::{
                sampler, texture_2d, texture_depth_2d, uniform_buffer, uniform_buffer_sized,
            },
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        sync_world::MainEntity,
        texture::GpuImage,
        view::{ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
};

use crate::{RatatuiCameraEdgeDetection, camera_readback::RatatuiSobelSender};

pub struct RatatuiCameraNodeSobelPlugin;

impl Plugin for RatatuiCameraNodeSobelPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "src/", "shaders/sobel.wgsl");

        app.add_plugins(ExtractComponentPlugin::<RatatuiCameraEdgeDetection>::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(
            Render,
            prepare_config_buffer_system.in_set(RenderSystems::Prepare),
        );

        render_app
            .add_render_graph_node::<ViewNodeRunner<RatatuiCameraNodeSobel>>(
                Core3d,
                RatatuiCameraNodeSobelLabel,
            )
            .add_render_graph_edge(Core3d, Node3d::EndMainPass, RatatuiCameraNodeSobelLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<RatatuiCameraNodeSobelPipeline>()
            .init_resource::<RatatuiCameraEdgeDetectionBuffers>();
    }
}

#[derive(Default)]
pub struct RatatuiCameraNodeSobel;

#[derive(RenderLabel, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RatatuiCameraNodeSobelLabel;

impl ViewNode for RatatuiCameraNodeSobel {
    type ViewQuery = (
        &'static MainEntity,
        &'static ViewTarget,
        &'static ViewPrepassTextures,
        &'static ViewUniformOffset,
        &'static RatatuiSobelSender,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext<'_>,
        render_context: &mut RenderContext<'w>,
        (entity, view_target, prepass_textures, view_uniform_offset, sobel_sender): QueryItem<
            'w,
            '_,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let gpu_images = world.get_resource::<RenderAssets<GpuImage>>().unwrap();
        let sobel_pipeline = world.resource::<RatatuiCameraNodeSobelPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let config_buffers = world.resource::<RatatuiCameraEdgeDetectionBuffers>();

        if let CachedPipelineState::Err(pipeline_error) =
            pipeline_cache.get_render_pipeline_state(sobel_pipeline.pipeline_id)
        {
            log::error!("{pipeline_error:?}");
        };

        let Some(pipeline) = pipeline_cache.get_render_pipeline(sobel_pipeline.pipeline_id) else {
            return Ok(());
        };

        let Some(config_buffer) = config_buffers.buffers.get(entity) else {
            return Ok(());
        };

        let source = view_target.main_texture_view();
        let destination = gpu_images.get(&sobel_sender.sender_image).unwrap();
        let view_uniforms = world.resource::<ViewUniforms>();

        let (Some(depth_prepass), Some(normal_prepass)) = (
            prepass_textures.depth_view(),
            prepass_textures.normal_view(),
        ) else {
            return Ok(());
        };

        let Some(view_uniforms) = view_uniforms.uniforms.binding() else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            "ratatui_camera_node_sobel_bind_group",
            &sobel_pipeline.layout,
            &BindGroupEntries::sequential((
                source,
                &sobel_pipeline.sampler,
                depth_prepass,
                normal_prepass,
                view_uniforms,
                config_buffer,
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("ratatui_camera_node_sobel_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &destination.texture_view,
                resolve_target: None,
                ops: Operations::default(),
                depth_slice: None,
            })],
            ..default()
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[view_uniform_offset.offset]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(ShaderType, Default, Clone, Copy)]
pub struct RatatuiCameraNodeSobelConfig {
    thickness: f32,
    color_enabled: u32,
    color_threshold: f32,
    depth_enabled: u32,
    depth_threshold: f32,
    normal_enabled: u32,
    normal_threshold: f32,
}

impl From<&RatatuiCameraEdgeDetection> for RatatuiCameraNodeSobelConfig {
    fn from(value: &RatatuiCameraEdgeDetection) -> Self {
        Self {
            thickness: value.thickness,
            color_enabled: value.color_enabled.into(),
            color_threshold: value.color_threshold,
            depth_enabled: value.depth_enabled.into(),
            depth_threshold: value.depth_threshold,
            normal_enabled: value.normal_enabled.into(),
            normal_threshold: value.normal_threshold,
        }
    }
}

#[derive(Resource, Default)]
pub struct RatatuiCameraEdgeDetectionBuffers {
    buffers: HashMap<MainEntity, UniformBuffer<RatatuiCameraNodeSobelConfig>>,
}

fn prepare_config_buffer_system(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut ratatui_cameras: Query<(&MainEntity, &RatatuiCameraEdgeDetection)>,
    mut config_buffers: ResMut<RatatuiCameraEdgeDetectionBuffers>,
) {
    for (entity_id, edge_detection) in &mut ratatui_cameras {
        let config = RatatuiCameraNodeSobelConfig::from(edge_detection);

        let buffer = config_buffers.buffers.entry(*entity_id).or_default();
        buffer.set(config);
        buffer.write_buffer(&render_device, &render_queue);
    }
}

#[derive(Resource)]
struct RatatuiCameraNodeSobelPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for RatatuiCameraNodeSobelPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "ratatui_camera_node_sobel_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    // rendered texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    // depth prepass
                    texture_depth_2d(),
                    // normal prepass
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // view
                    uniform_buffer::<ViewUniform>(true),
                    // config
                    uniform_buffer_sized(false, None),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let path = Path::new("bevy_ratatui_camera").join("shaders/sobel.wgsl");
        let source = AssetSourceId::from("embedded");
        let asset_path = AssetPath::from_path(&path).with_source(source);
        let shader_handle: Handle<Shader> = world.load_asset(asset_path);

        let vertex_state = world.resource::<FullscreenShader>().to_vertex_state();
        let pipeline_cache = world.resource_mut::<PipelineCache>();

        let mut shader_defs = Vec::new();

        if DEPTH_TEXTURE_SAMPLING_SUPPORTED {
            shader_defs.push("DEPTH_TEXTURE_SAMPLING_SUPPORTED".into());
        }

        let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("ratatui_camera_node_sobel_pipeline".into()),
            layout: vec![layout.clone()],
            vertex: vertex_state,
            fragment: Some(FragmentState {
                shader: shader_handle,
                shader_defs,
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: true,
        });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
