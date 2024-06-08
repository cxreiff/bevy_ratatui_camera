# bevy_ratatui_render

Bevy inside the terminal!

Uses bevy headless rendering, [ratatui](https://github.com/ratatui-org/ratatui), and
[ratatui_image](https://github.com/benjajaja/ratatui-image) to print the rendered output
of your bevy application to the terminal using unicode halfblocks.

![cube example](https://assets.cxreiff.com/github/cube.gif)![foxes](https://assets.cxreiff.com/github/foxes.gif)![sponza test scene](https://assets.cxreiff.com/github/sponza.gif)

> examples/cube.rs, bevy many_foxes example, sponza test scene

Use [bevy_ratatui](https://github.com/joshka/bevy_ratatui/tree/main) for setting ratatui up
and receiving terminal events (keyboard, focus, mouse, paste, resize) inside bevy.

## getting started

`cargo add bevy_ratatui_render bevy_ratatui`

```rust
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RatatuiPlugins::default(),
            RatatuiRenderPlugin::new().add_render((256, 256)),
        ))
        .add_systems(Startup, setup_scene_system)
        .add_systems(Update, draw_scene_system.map(error))
        .run();
}

fn setup_scene_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ratatui_render: Res<RatatuiRenderContext>,
) {
    // spawn objects into your scene

    ...

    commands.spawn(Camera3dBundle {
        camera: Camera {
            target: ratatui_render.target(0),
            ..default()
        },
        ..default()
    });
}

fn draw_scene_system(
    mut ratatui: ResMut<RatatuiContext>,
    ratatui_render: Res<RatatuiRenderContext>,
) -> io::Result<()> {
    ratatui.draw(|frame| {
        frame.render_widget(ratatui_render.widget(0), frame.size());
    })?;

    Ok(())
}
```

There is a convenience function if you do not need access to the ratatui draw loop and just would like
the render to print to the full terminal (use instead of adding the `draw_scene` system above):

```rust
RatatuiRenderPlugin::new().add_render((256, 256)).print_full_terminal(0)
```

I also recommend telling bevy you don't need a window or anti-aliasing:

```rust
DefaultPlugins
    .set(ImagePlugin::default_nearest())
    .set(WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        close_when_requested: false,
    })
```

## multiple renders

When you call `add_render((width, height))` a new render target and destination will be created,
associated with an index that increments from zero. For multiple renders, just call `add_render` multiple
times, and then use the index for the order it was created in the `target(index)`, `widget(index)`, and
`print_full_terminal(index)` methods (I may implement string IDs in the future, but for now using an
index kept the code simple).

## supported terminals

This relies on the terminal supporting 24-bit color. I've personality tested and confirmed that the
following terminals display color correctly:

- Alacritty
- Kitty
- iTerm
- WezTerm

## what's next?

I am still refining the interface for creating render targets- I would like devs to be able to identify
render targets with descriptive string IDs instead of integer indices.

Also, it may be slightly nicer to create multiple render targets by instantiating the plugin multiple
times, rather than the current builder pattern.

## credits

* Headless rendering code adapted from bevy's
[headless_render](https://github.com/bevyengine/bevy/blob/main/examples/app/headless_renderer.rs)
example (@bugsweeper, @alice-i-cecile, @mockersf).
* bevy's [many_foxes](https://github.com/bevyengine/bevy/blob/main/examples/stress_tests/many_foxes.rs)
example used for example gif.
* [bevy_sponza_scene](https://github.com/DGriffin91/bevy_sponza_scene) used for example gif.
