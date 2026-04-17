use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "renderer3d", about = "CPU software rasterizer")]
pub struct CliArgs {
    /// Number of frames to render (overrides --unlimited)
    #[arg(long, conflicts_with = "unlimited")]
    pub frames: Option<u32>,

    /// Run forever until the window is closed
    #[arg(long)]
    pub unlimited: bool,

    /// Enable vsync
    #[arg(long)]
    pub vsync: bool,

    /// Model to render: `"builtin"` for the built-in cube, or `"obj_path,png_path"`
    #[arg(long, default_value = "builtin")]
    pub model: String,
}

impl CliArgs {
    #[must_use]
    pub fn max_frames(&self) -> Option<u32> {
        if self.unlimited {
            None
        } else {
            Some(self.frames.unwrap_or(60 * 25))
        }
    }
}
