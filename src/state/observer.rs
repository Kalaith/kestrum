//! Runtime playback controls for spectator campaigns.

#[derive(Debug, Clone, PartialEq)]
pub struct ObserverPlayback {
    pub paused: bool,
    pub speed: u8,
    elapsed_seconds: f32,
}

impl Default for ObserverPlayback {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1,
            elapsed_seconds: 0.0,
        }
    }
}

impl ObserverPlayback {
    pub const SPEEDS: [u8; 3] = [1, 2, 4];

    pub fn set_paused(&mut self, paused: bool) {
        if self.paused != paused {
            self.paused = paused;
            self.reset_clock();
        }
    }

    pub fn set_speed(&mut self, speed: u8) -> bool {
        if !Self::SPEEDS.contains(&speed) {
            return false;
        }
        if self.speed != speed {
            self.speed = speed;
            self.reset_clock();
        }
        true
    }

    pub fn reset_clock(&mut self) {
        self.elapsed_seconds = 0.0;
    }

    /// Returns at most one due step. Excess frame time is discarded to avoid bursts.
    pub fn due_step(&mut self, elapsed_seconds: f32, base_delay_seconds: f32) -> bool {
        if self.paused || !elapsed_seconds.is_finite() || elapsed_seconds <= 0.0 {
            return false;
        }
        let delay = if base_delay_seconds.is_finite() && base_delay_seconds > 0.0 {
            base_delay_seconds / f32::from(self.speed)
        } else {
            0.0
        };
        if delay == 0.0 {
            self.reset_clock();
            return true;
        }
        self.elapsed_seconds += elapsed_seconds;
        if self.elapsed_seconds < delay {
            return false;
        }
        self.reset_clock();
        true
    }
}
