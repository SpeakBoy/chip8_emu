use chip8_emu_backend::*;
use macroquad::prelude::*;
use std::env;

// Scale window to accomodate for larger screens.
const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn window_config() -> Conf {
    Conf {
        window_title: String::from("Chip-8 Emulator"),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    clear_background(BLACK);

    'gameloop: loop {
        if is_quit_requested() {
            break 'gameloop;
        }

        next_frame().await;
    }
}
