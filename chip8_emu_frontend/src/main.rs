use chip8_emu_backend::*;
use macroquad::prelude::*;
use std::env;

// Scale window to accomodate for larger screens.
const SCALE: i32 = 15;
const WINDOW_WIDTH: i32 = (SCREEN_WIDTH as i32) * SCALE;
const WINDOW_HEIGHT: i32 = (SCREEN_HEIGHT as i32) * SCALE;

fn window_config() -> Conf {
    Conf {
        window_title: String::from("Chip-8 Emulator"),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
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
