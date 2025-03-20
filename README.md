# bevy_ratatui_camera

[![Crate Badge]][Crate]
[![Docs Badge]][Docs]
[![Downloads Badge]][Downloads]
[![License Badge]][License]

<p float="left">
<img src="https://assets.cxreiff.com/github/cube.gif" width="30%" alt="cube">
<img src="https://assets.cxreiff.com/github/foxes.gif" width="30%" alt="foxes">
<img src="https://assets.cxreiff.com/github/sponza.gif" width="30%" alt="sponza test scene">
<p>

Bevy inside the terminal!

Uses bevy headless rendering,
[ratatui](https://github.com/ratatui-org/ratatui),
[ratatui_image](https://github.com/benjajaja/ratatui-image), and
[bevy_ratatui](https://github.com/cxreiff/bevy_ratatui) to print your bevy
application's rendered frames to the terminal.

> [!IMPORTANT]  
> This crate was renamed from `bevy_ratatui_render` to `bevy_ratatui_camera`.

## getting started

`cargo add bevy_ratatui_camera bevy_ratatui`

```rust
fn main() {
    App::new()
        .add_plugins((
            // disable WinitPlugin as it panics in environments without a display server.
            // disable LogPlugin as it interferes with terminal output.
            DefaultPlugins.build()
                .disable::<WinitPlugin>()
                .disable::<LogPlugin>(),

            // create windowless loop and set its frame rate.
            ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1. / 60.)),

            // set up the Ratatui context and forward terminal input events.
            RatatuiPlugins::default(),

            // add the ratatui camera plugin.
            RatatuiCameraPlugin,
        ))
        .add_systems(Startup, setup_scene_system)
        .add_systems(PostUpdate, draw_scene_system.map(error));
}

// add RatatuiCamera to your scene's camera.
fn setup_scene_system(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        RatatuiCamera::default(),
    ));
}

// a RatatuiCameraWidget component will be available in your camera entity.
fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    camera_widget: Query<&RatatuiCameraWidget>,
) -> std::io::Result<()> {
    ratatui.draw(|frame| {
        camera_widget.single().render(frame.area(), frame.buffer_mut());
    })?;

    Ok(())
}
```

As shown above, when `RatatuiCameraPlugin` is added to your application, any
bevy camera entities that you add a `RatatuiCamera` component to, will have
a `RatatuiCameraWidget` inserted that you can query for. Each
`RatatuiCameraWidget` is a ratatui widget that when drawn will print the most
recent frame rendered by the associated bevy camera, as unicode characters.

Use [bevy_ratatui](https://github.com/cxreiff/bevy_ratatui) for setting ratatui
up and receiving terminal events (keyboard, focus, mouse, paste, resize) inside
bevy.

## strategies

The method by which the rendered image is converted into unicode characters
depends on the `RatatuiCameraStrategy` that you choose. Insert a variant of the
component alongside the `RatatuiCamera` to change the behavior from the
default. Refer to the `RatatuiCameraStrategy` documentation for descriptions of
each variant.

For example, to use the "Luminance" strategy:

```rust
commands.spawn((
    Camera3d::default(),
    RatatuiCamera::default(),
    RatatuiCameraStrategy::Luminance(LuminanceConfig::default()),
));
```

## autoresize

By default, the size of the texture the camera renders to will stay constant,
and when rendered to the ratatui buffer with `RatatuiCameraWidget::render(...)`
it will retain its aspect ratio. If you use the
`RatatuiCameraWidget::render_autoresize` variant instead, whenever the render
texture doesn't match the size of the render area, rendering will be skipped
that frame and a resize of the render texture will be triggered instead.

```rust
ratatui.draw(|frame| {
    camera_widget.single().render_autoresize(
        frame.area(),
        frame.buffer_mut(),
        &mut commands,
    );
})?;
```

## edge detection

When using the `RatatuiCameraStrategy::Luminance` strategy and a 3d camera, you
can also optionally insert a `RatatuiCameraEdgeDetection` component into your
camera in order to add an edge detection step in the render graph. When
printing to the ratatui buffer, special characters and an override color can be
used based on the detected edges and their directions. This can be useful for
certain visual effects, and distinguishing detail when the text rendering
causes edges to blend together.

Set `edge_characters` to `EdgeCharacters::Single(..)` for a single dedicated
edge character, or set it to `EdgeCharacters::Directional { .. }` to set
different characters based on the "direction" of the edge, for example using
'―', '|', '/', and '\\' characters to draw edge "lines". Detecting the correct
edge direction is a bit fuzzy, so you may need to experiment with
color/depth/normal thresholds for good results.

```rust
RatatuiCameraEdgeDetection {
    thickness: 1.4,
    edge_characters: Self::Directional {
        vertical: '|',
        horizontal: '―',
        forward_diagonal: '/',
        backward_diagonal: '\\',
    },
    edge_color: Some(ratatui::style::Color::Magenta),
    ..default()
}
```

## multiple cameras

`RatatuiCamera` can be added to multiple camera entities. To access the correct
render, use marker components on your cameras to use when querying
`RatatuiCameraWidget`.

If you need multiple cameras to render to one image, create one `RatatuiCamera`
main camera that will define the dimensions, strategy, etcetera, and then create
additional `RatatuiSubcamera` cameras that point to the main camera.

## supported terminals

Printing to terminal relies on the terminal supporting 24-bit color. I've
personally tested and confirmed that the following terminals display correctly:

- Alacritty
- Kitty
- iTerm
- WezTerm
- Rio
- Ghostty

...but any terminal with 24-bit color support should work fine, if its
performance is adequate.

## compatibility

| bevy  | bevy_ratatui_camera |
|-------|---------------------|
| 0.15  | 0.10                |
| 0.14  | 0.6                 |

[Crate]: https://crates.io/crates/bevy_ratatui_camera
[Crate Badge]: https://img.shields.io/crates/v/bevy_ratatui_camera
[Docs]: https://docs.rs/bevy_ratatui_camera
[Docs Badge]: https://img.shields.io/badge/docs-bevy_ratatui_camera-886666
[Downloads]: https://crates.io/crates/bevy_ratatui_camera
[Downloads Badge]: https://img.shields.io/crates/d/bevy_ratatui_camera.svg
[License]: ./LICENSE-MIT
[License Badge]: https://img.shields.io/crates/l/bevy_ratatui_camera
