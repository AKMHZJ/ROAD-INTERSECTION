use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::Duration;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 800;
const ROAD_WIDTH: u32 = 40;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsytstem = sdl_context.video()?;

    let window = video_subsytstem
        .window("Traffic Simulation", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                }
                _ => {}
            }
        }

        // Update

        // Render
        canvas.set_draw_color(Color::RGB(34, 139, 34));
        canvas.clear();

        // Roads
        let h_road = Rect::new(
            0,
            (WINDOW_HEIGHT / 2 - ROAD_WIDTH) as i32,
            WINDOW_WIDTH,
            ROAD_WIDTH * 2
        );
        let v_road = Rect::new(
            (WINDOW_WIDTH / 2 - ROAD_WIDTH) as i32,
            0,
            ROAD_WIDTH * 2,
            WINDOW_HEIGHT
        );

        canvas.set_draw_color(Color::RGB(105, 105, 105));
        canvas.fill_rect(h_road)?;
        canvas.fill_rect(v_road)?;

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // 60 FPS
    }

    Ok(())
}
