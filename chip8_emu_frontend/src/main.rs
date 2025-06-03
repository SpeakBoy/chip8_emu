use chip8_emu_backend::*;
use macroquad::prelude::*;
use rfd::FileDialog;
use rfd::MessageDialog;
use rfd::MessageLevel;
use std::fs::File;
use std::io::Read;

// Scale window to accomodate for larger screens.
const SCALE: i32 = 15;
const WINDOW_WIDTH: i32 = (SCREEN_WIDTH as i32) * SCALE;
const WINDOW_HEIGHT: i32 = (SCREEN_HEIGHT as i32) * SCALE;
const TICKS_PER_FRAME: usize = 8;

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

#[macroquad::main(window_config)]
async fn main() {
    let file = FileDialog::new()
        .add_filter("CHIP-8 ROM", &["ch8", "rom"])
        .add_filter("All Files", &["*"])
        .pick_file();

    if file.is_none() {
        MessageDialog::new()
            .set_title("Error")
            .set_description("No ROM selected!")
            .set_level(MessageLevel::Error)
            .show();
        return;
    }

    let audio = AudioManager::new().await;

    let variant = Chip8Variant::SuperChip;

    let mut chip8 = Cpu::new(audio, variant);

    let mut rom = File::open(file.unwrap()).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    'gameloop: loop {
        if is_quit_requested() || is_key_pressed(KeyCode::Escape) {
            break 'gameloop;
        }
        for (key, &keycode) in KEYS.iter().enumerate() {
            let pressed = is_key_down(keycode);
            chip8.keypress(key, pressed);
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();
        draw_screen(&chip8);
        next_frame().await;
    }
}
