use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound, stop_sound};

pub struct AudioManager {
    beep: Sound,
    is_playing: bool,
}

impl AudioManager {
    pub async fn new() -> Self {
        let beep_data = include_bytes!("../../assets/beep.wav");
        let beep = load_sound_from_bytes(beep_data).await.unwrap();
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
