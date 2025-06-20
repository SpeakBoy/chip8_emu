#![windows_subsystem = "windows"]

use chip8_emu_backend::*;
use macroquad::prelude::*;
use rfd::{FileDialog, MessageDialog, MessageLevel};
use std::{fs::File, io::Read};

// Scale window to accomodate for larger screens.
const SCALE: i32 = 10;
const WINDOW_WIDTH: i32 = (SCREEN_WIDTH as i32) * SCALE;
const WINDOW_HEIGHT: i32 = (SCREEN_HEIGHT as i32) * SCALE;

const KEYS: [KeyCode; 16] = [
    KeyCode::X,    // 0
    KeyCode::Key1, // 1
    KeyCode::Key2, // 2
    KeyCode::Key3, // 3
    KeyCode::Q,    // 4
    KeyCode::W,    // 5
    KeyCode::E,    // 6
    KeyCode::A,    // 7
    KeyCode::S,    // 8
    KeyCode::D,    // 9
    KeyCode::Z,    // A
    KeyCode::C,    // B
    KeyCode::Key4, // C
    KeyCode::R,    // D
    KeyCode::F,    // E
    KeyCode::V,    // F
];

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

    let (screen_buf, screen_width, _, _) = cpu.get_display();

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            // Convert 1D array's index into 2D (x, y) position
            let x = (i % screen_width) as i32;
            let y = (i / screen_width) as i32;

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

async fn setup() -> Option<(Chip8Variant, Vec<u8>)> {
    let mut variant: Option<Chip8Variant> = None;

    loop {
        draw_text("Chip-8 Emulator", 155.9375, 50.0, 50.0, WHITE);
        draw_text("Press [1] for Chip-8", 188.75, 100.0, 30.0, WHITE);
        draw_text("Press [2] for SuperChip", 169.0625, 130.0, 30.0, WHITE);
        draw_text("Press [Enter] to load ROM", 155.9375, 200.0, 30.0, YELLOW);

        if let Some(v) = variant {
            draw_text(&format!("{:?}", v), 40.0, 280.0, 30.0, GREEN);
        }

        if is_key_pressed(KeyCode::Key1) {
            variant = Some(Chip8Variant::Chip8);
        } else if is_key_pressed(KeyCode::Key2) {
            variant = Some(Chip8Variant::SuperChip);
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(v) = variant {
                let file = FileDialog::new()
                    .add_filter("CHIP-8 ROM", &["ch8", "rom"])
                    .add_filter("All Files", &["*"])
                    .pick_file();

                if let Some(path) = file {
                    let mut rom = File::open(path).expect("Unable to open file");
                    let mut buffer = Vec::new();
                    rom.read_to_end(&mut buffer).unwrap();
                    return Some((v, buffer));
                } else {
                    MessageDialog::new()
                        .set_title("Error")
                        .set_description("No ROM selected!")
                        .set_level(MessageLevel::Error)
                        .show();
                    return None;
                }
            }
        }

        next_frame().await;
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let Some((variant, rom_data)) = setup().await else {
        return;
    };

    clear_background(BLACK);

    let audio = AudioManager::new().await;

    let mut chip8 = Cpu::new(audio, variant);

    chip8.load(&rom_data);

    // Initalize prev_res to the default resolution (lores)
    let mut prev_res = DisplayMode::LoRes;

    'gameloop: loop {
        if is_quit_requested() || is_key_pressed(KeyCode::Escape) {
            break 'gameloop;
        }
        for (key, &keycode) in KEYS.iter().enumerate() {
            let pressed = is_key_down(keycode);
            chip8.keypress(key, pressed);
        }

        let (_, w, h, display_mode) = chip8.get_display();

        let ticks_per_frame = config::ticks_per_frame(variant);

        for _ in 0..ticks_per_frame {
            chip8.tick();
        }
        chip8.tick_timers();

        // Update display size when changing from LoRes to HiRes (and vice versa)
        if display_mode != prev_res {
            request_new_screen_size((w as i32 * SCALE) as f32, (h as i32 * SCALE) as f32);
            prev_res = display_mode;
        }

        draw_screen(&chip8);

        next_frame().await;
    }
}
