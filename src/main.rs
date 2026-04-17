use clap::Parser;
use render3d::{
    cli::CliArgs,
    display::{Display, DisplayConfig},
    fire::Fire,
    mesh::{Mesh, ModelSource},
    scene::Scene,
};

fn main() {
    println!("Renderer 3D, by Idaho06");

    let args = CliArgs::parse();

    let model_source = if args.model == "builtin" {
        ModelSource::BuiltinCube
    } else {
        let parts: Vec<&str> = args.model.splitn(2, ',').collect();
        if parts.len() != 2 {
            eprintln!("--model expects 'builtin' or 'obj_path,png_path'");
            std::process::exit(1);
        }
        ModelSource::Obj {
            obj_path: parts[0].to_string(),
            png_path: parts[1].to_string(),
        }
    };

    let config = DisplayConfig { vsync: args.vsync };
    let mut display = Display::with_config(config);
    display.cls();

    let max_frames = args.max_frames();

    let mut frame_time: u32;
    let mut last_frame_delta: u32 = 0;
    let mut frame = 0_u32;

    let mut fire = Fire::new(&mut display);
    let mut mesh = Mesh::new(&mut display, model_source);

    'running: loop {
        frame += 1;
        frame_time = display.get_frame_time();

        display.update_user_input();

        if display.user_input.quit {
            break 'running;
        }
        if display.user_input.mouse.left.changed && display.user_input.mouse.left.pressed {
            display.switch_relative_mouse_mode();
        }

        fire.update(last_frame_delta, &display);
        mesh.update(last_frame_delta, &display);
        fire.render(&mut display);
        mesh.render(&mut display);
        display.present_canvas();

        last_frame_delta = display.get_frame_time() - frame_time;

        if let Some(limit) = max_frames
            && frame > limit
        {
            break;
        }
    }

    #[allow(clippy::cast_precision_loss)]
    {
        println!("Total frames: {frame}");
        println!(
            "Average FPS: {}",
            frame as f32 / (display.get_frame_time() as f32 / 1000.0)
        );
        println!(
            "Total time: {} seconds",
            display.get_frame_time() as f32 / 1000.0
        );
    }
}
