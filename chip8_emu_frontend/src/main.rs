use chip8_emu_backend::*;
use macroquad::prelude::*;
use std::env;
use std::fs::File;
use std::io::Read;

// Scale window to accomodate for larger screens.
const SCALE: i32 = 15;
const WINDOW_WIDTH: i32 = (SCREEN_WIDTH as i32) * SCALE;
const WINDOW_HEIGHT: i32 = (SCREEN_HEIGHT as i32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

fn window_config() -> Conf {
    Conf {
        window_title: String::from("Chip-8 Emulator"),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        ..Default::default()
    }
}

fn draw_screen(cpu: &Cpu) {
    // Clear window and make background black
    clear_background(BLACK);

    let screen_buf = cpu.get_display();

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            // Convert 1D array's index into 2D (x, y) position
            let x = (i % SCREEN_WIDTH) as i32;
            let y = (i / SCREEN_WIDTH) as i32;

            // Draw rectangle at (x, y), scaled up by SCALE
            draw_rectangle(
                (x * SCALE) as f32,
                (y * SCALE) as f32,
                SCALE as f32,
                SCALE as f32,
                WHITE,
            );
        }
    }
}

fn key2btn(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Key1 => Some(0x1),
        KeyCode::Key2 => Some(0x2),
        KeyCode::Key3 => Some(0x3),
        KeyCode::Key4 => Some(0xC),
        KeyCode::Q => Some(0x4),
        KeyCode::W => Some(0x5),
        KeyCode::E => Some(0x6),
        KeyCode::R => Some(0xD),
        KeyCode::A => Some(0x7),
        KeyCode::S => Some(0x8),
        KeyCode::D => Some(0x9),
        KeyCode::F => Some(0xE),
        KeyCode::Z => Some(0xA),
        KeyCode::X => Some(0x0),
        KeyCode::C => Some(0xB),
        KeyCode::V => Some(0xF),
        _ => None,
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let mut chip8 = Cpu::new();

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        if is_quit_requested() || is_key_pressed(KeyCode::Escape) {
            break 'gameloop;
        }
        for key in get_keys_pressed() {
            if let Some(k) = key2btn(key) {
                chip8.keypress(k, true);
            }
        }
        for key in get_keys_released() {
            if let Some(k) = key2btn(key) {
                chip8.keypress(k, false);
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();
        draw_screen(&chip8);
        next_frame().await;
    }
}
