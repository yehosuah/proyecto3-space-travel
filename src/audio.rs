//! Síntesis propia de propulsores y señales; raylib reproduce PCM y la pista local.
use std::f32::consts::TAU;

pub struct Synth {
    phase: f32,
    noise: u32,
    filtered: f32,
    pub cue: u8,
    cue_left: f32,
}
impl Default for Synth {
    fn default() -> Self {
        Self {
            phase: 0.0,
            noise: 2026,
            filtered: 0.0,
            cue: 0,
            cue_left: 0.0,
        }
    }
}
impl Synth {
    pub fn signal(&mut self, cue: u8) {
        self.cue = cue;
        self.cue_left = if cue == 2 { 0.75 } else { 0.22 };
    }
    pub fn fill(&mut self, buffer: &mut [f32], speed: f32, thrust: f32, boost: f32, active: bool) {
        let frequency = 48.0 + speed * 0.75 + thrust * 34.0 + boost * 35.0;
        for sample in buffer {
            self.phase = (self.phase + frequency / 48000.0).fract();
            self.noise ^= self.noise << 13;
            self.noise ^= self.noise >> 17;
            self.noise ^= self.noise << 5;
            let noise = self.noise as f32 / u32::MAX as f32 * 2.0 - 1.0;
            self.filtered += 0.1 * (noise - self.filtered);
            let engine = (TAU * self.phase).sin() * 0.12
                + (TAU * self.phase * 2.0).sin() * 0.045
                + self.filtered * (0.12 + boost * 0.28);
            let ambient = (TAU * self.phase * 0.25).sin() * 0.015;
            let mut value = if active {
                engine * (0.18 + thrust * 0.52 + boost * 0.3) + ambient
            } else {
                0.0
            };
            if self.cue_left > 0.0 {
                let f = match self.cue {
                    2 => 660.0 + 330.0 * (self.cue_left * 5.0).fract(),
                    3 => 880.0,
                    _ => 1174.0,
                };
                value += (TAU * self.cue_left * f).sin() * self.cue_left.min(0.05) * 2.5;
                self.cue_left -= 1.0 / 48000.0;
            }
            *sample = value.clamp(-0.8, 0.8);
        }
    }
}
