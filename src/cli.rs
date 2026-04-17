//! Command-line argument parsing using [`clap`].
//!
//! Run `renderer3d --help` to see all flags.
//!
//! See book chapter: _Running the renderer_ (TODO: link when mdBook is set up).

use clap::{Parser, ValueEnum};

/// Named model presets, each mapping to a pair of asset files in `assets/`.
///
/// Passed to `--model`; defaults to [`ModelPreset::Lexus`].
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum ModelPreset {
    /// Procedural unit cube — no asset files required
    Builtin,
    /// Lexus car (`assets/lexus.obj` + `assets/lexus.png`)
    #[default]
    Lexus,
    /// Ferris crab (`assets/crab.obj` + `assets/crab.png`)
    Crab,
    /// Simple cube mesh (`assets/cube.obj` + `assets/cube.png`)
    Cube,
}

/// Command-line arguments for the renderer.
#[derive(Parser, Debug)]
#[command(name = "renderer3d", about = "CPU software rasterizer")]
pub struct CliArgs {
    /// Cap rendering at N frames then exit (default: run forever)
    #[arg(long)]
    pub frames: Option<u32>,

    /// Disable vsync (vsync is on by default)
    #[arg(long)]
    pub no_vsync: bool,

    /// Model preset to render
    #[arg(long, value_enum, default_value_t = ModelPreset::Lexus)]
    pub model: ModelPreset,
}

impl CliArgs {
    /// Returns `None` to run forever, or `Some(N)` to stop after N frames.
    #[must_use]
    pub fn max_frames(&self) -> Option<u32> {
        self.frames
    }

    /// Returns `true` if vsync should be enabled (on by default).
    #[must_use]
    pub fn vsync(&self) -> bool {
        !self.no_vsync
    }
}
