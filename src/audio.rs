use macroquad::audio::{Sound, PlaySoundParams, play_sound, set_sound_volume, stop_sound};

pub struct AudioManager {
    pub music: Sound,
    pub is_muted: bool,
    pub music_volume: f32,
}

impl AudioManager {
    pub async fn new() -> Self {
        // let base = std::path::Path::new("assets/audio/");
        let music = macroquad::audio::load_sound("assets/audio/Cheerful_Music.ogg")
            .await
            .expect("Failed to load music");

        Self {
            music,
            is_muted: false,
            music_volume: 1.0,
        }
    }

    pub fn play_music(&self) {
        play_sound(
            &self.music,
            PlaySoundParams {
                looped: true,
                volume: if self.is_muted { 0.0 } else { self.music_volume },
            },
        );
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;

        let volume = if self.is_muted { 0.0 } else { self.music_volume };
        set_sound_volume(&self.music, volume);
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);

        if !self.is_muted {
            set_sound_volume(&self.music, self.music_volume);
        }
    }

    pub fn stop_music(&self) {
        stop_sound(&self.music);
    }
}
