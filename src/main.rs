use render3d::{cube::Cube, display::Display, fire::Fire, scene::Scene};
use sdl2::{event::Event, keyboard::Keycode};

fn main() -> Result<(), String> {
    println!("Renderer 3D, by Idaho06");

    let mut display = Display::new();
    display.cls();
    let mut event_pump = display.get_event_pump();

    //display.cls();

    //let color = Color::RGB(128, 0, 0);

    let mut frame_time: u32;
    let mut last_frame_delta: u32 = 0;

    let mut fire = Fire::new(&mut display);
    let mut cube = Cube::new(&mut display);

    optick::start_capture();
    'running: loop {
        frame_time = display.get_frame_time();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }
        //display.clear_streaming_buffer("fire", color);
        //display.streaming_buffer_to_canvas("fire");
        fire.update(last_frame_delta, &display, &None);
        cube.update(last_frame_delta, &display, &None);
        fire.render(&mut display);
        cube.render(&mut display);
        display.present_canvas();

        last_frame_delta = display.get_frame_time() - frame_time;
    }
    optick::stop_capture("./profile/render3d");
    Ok(())
}
