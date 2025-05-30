use macroquad::audio::{PlaySoundParams, Sound, load_sound, play_sound, stop_sound};

pub struct AudioManager {
    beep: Sound,
    is_playing: bool,
}

impl AudioManager {
    pub async fn new() -> Self {
        let beep = load_sound("assets/beep.wav").await.unwrap();
        Self {
            beep: beep,
            is_playing: false,
        }
    }

    pub fn start_beep(&mut self) {
        if !self.is_playing {
            play_sound(
                &self.beep,
                PlaySoundParams {
                    looped: true,
                    volume: 0.2,
                },
            );
            self.is_playing = true;
        }
    }

    pub fn stop_beep(&mut self) {
        stop_sound(&self.beep);
        self.is_playing = false;
    }
}
