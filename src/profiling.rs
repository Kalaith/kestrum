//! Opt-in rendered-frame measurements; F3 reports without changing game rules.
use macroquad::prelude::*;

#[derive(Default)]
pub struct FrameProfile {
    active: bool,
    frames: u32,
    cpu: Vec<f64>,
    presented: Vec<f64>,
    input: Vec<f64>,
}

pub fn benchmark_frames() -> u32 {
    macroquad_toolkit::capture::env_u32("KESTRUM_PROFILE_FRAMES", 0)
}

impl FrameProfile {
    pub fn new() -> Self {
        Self {
            active: benchmark_frames() > 0,
            ..Self::default()
        }
    }

    pub fn enabled(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self, context: &str) {
        if self.active {
            self.report(context);
        }
        self.active = !self.active;
        self.frames = 0;
        self.cpu.clear();
        self.presented.clear();
        self.input.clear();
        info!("K18_FRAME_PROFILE active={} {context}", self.active);
    }

    pub fn record(&mut self, cpu: f64, presented: f64, input: bool, context: &str) -> bool {
        if !self.active {
            return false;
        }
        self.frames += 1;
        // Warm fonts/GPU and exclude initial activation / generated-state setup.
        if self.frames > 60 {
            self.cpu.push(cpu * 1000.0);
            self.presented.push(presented * 1000.0);
            if input {
                self.input.push(presented * 1000.0);
            }
        }
        if self.cpu.len() >= 600 {
            self.report(context);
            self.cpu.clear();
            self.presented.clear();
            self.input.clear();
        }
        let done = benchmark_frames() > 0 && self.frames >= benchmark_frames();
        if done {
            self.report(context);
        }
        done
    }

    fn report(&self, context: &str) {
        info!("K18_RENDER samples={} cpu_ms={} frame_ms={} input_samples={} input_to_next_frame_ms={} {context}",
            self.cpu.len(), summary(&self.cpu), summary(&self.presented), self.input.len(), summary(&self.input));
    }
}

fn summary(values: &[f64]) -> String {
    if values.is_empty() {
        return "unmeasured".into();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    format!(
        "p50:{:.3},p95:{:.3},max:{:.3}",
        sorted[sorted.len() / 2],
        sorted[sorted.len() * 95 / 100],
        sorted[sorted.len() - 1]
    )
}
