# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.14.1](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.14.0...v0.14.1) - 2025-05-25

### Other

- wasm compatibility changes

## [0.14.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.13.0...v0.14.0) - 2025-05-13

### Other

- workflows fixed
- bevy_ratatui version 0.9.0 bump
- expanded `bg_color_scale` to custom color system
- refactored depth detection to have less side effects
- depth recording and widget occlusion example
- successful depth readback and parsing, unutilized
- extend text_labels example to show cell_to_ndc
- cell-ndc conversion math and ergonomics fixes
- refactored some area calculations, overlay widgets
- reworked resizing to use Widget trait render

## [0.13.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.12.0...v0.13.0) - 2025-04-27

### Other

- doc fix for subcameras
- smoothed FPS counter
- bevy 0.16 migration

## [0.12.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.11.1...v0.12.0) - 2025-04-03

### Fixed

- saturating add causing washed out colors

### Other

- deduped some widget code into widget_utilities
- replaced ratatui-image with HalfBlocks strategy
- luminance_characters(), bg_color_scale

## [0.11.1](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.11.0...v0.11.1) - 2025-03-26

### Other

- generate 256 ANSI colors with const fn

## [0.11.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.10.0...v0.11.0) - 2025-03-26

### Other

- color support (rgb, ansi256, ansi16) ([#37](https://github.com/cxreiff/bevy_ratatui_camera/pull/37))
- exit conversion early when cell is invalid
- reborrow commands instead of &mut
- added badges to README
- mask on alpha instead of color
- insert/update/remove user-facing components example
- added a system set for First schedule systems

## [0.10.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.9.0...v0.10.0) - 2025-03-02

### Other

- reworked autoresize to work with render area

## [0.9.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.8.2...v0.9.0) - 2025-03-01

### Other

- transparency feature with masking color
- added subcamera for compositing cameras
- rust 2024 edition migration

## [0.8.2](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.8.1...v0.8.2) - 2024-12-31

### Fixed

- better diagonals font compatibility

## [0.8.1](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.8.0...v0.8.1) - 2024-12-23

### Other

- README fixes, new None strategy

## [0.8.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.7.0...v0.8.0) - 2024-12-23

### Other

- added edge_detection example

## [0.7.0](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.6.0...v0.7.0) - 2024-12-05

### Added

- [**breaking**] bevy 0.15 migration

### Other

- Bump ruzstd from 0.7.0 to 0.7.3 in the cargo group across 1 directory
- updated README for version bump

## [0.5.8](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.5.7...v0.5.8) - 2024-10-11

### Other

- disable LogPlugin in docs and examples

## [0.5.7](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.5.6...v0.5.7) - 2024-10-11

### Other

- shortened comments in docs/README
- version upgrades
- update docs and examples to remove WinitPlugin

## [0.5.6](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.5.5...v0.5.6) - 2024-08-15

### Other
- version bumps, workflow updates

## [0.5.5](https://github.com/cxreiff/bevy_ratatui_camera/compare/v0.5.4...v0.5.5) - 2024-08-07

### Other
- re-added workflows
- autoresize initialization fix
