use super::AudioPlugin;

/// Native Distortion / saturator plugin
/// Provides soft-clipping, hard-clipping, and tape saturation models.
/// Optimized for the audio thread.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistortionType {
    Soft, // atan curve
    Hard, // clamp
    Tape, // cubic polynomial
}

pub struct DistortionPlugin {
    pub drive: f32, // 1.0 to 12.0 (effectively +21dB gain drive)
    pub mix: f32,   // 0.0 to 1.0 (Wet amount)
    pub mode: DistortionType,
}

impl DistortionPlugin {
    pub fn new() -> Self {
        Self {
            drive: 1.0,
            mix: 1.0,
            mode: DistortionType::Soft,
        }
    }

    pub fn set_params(&mut self, drive: f32, mix: f32, mode: DistortionType) {
        self.drive = drive.clamp(1.0, 12.0);
        self.mix = mix.clamp(0.0, 1.0);
        self.mode = mode;
    }

    #[inline(always)]
    fn process_single(&self, input: f32) -> f32 {
        let x = input * self.drive;
        let saturated = match self.mode {
            DistortionType::Soft => {
                // Continuous curve atan-based soft-clipping
                x.atan() / (std::f32::consts::PI / 2.0)
            }
            DistortionType::Hard => {
                // Instant brickwall clamp
                x.clamp(-1.0, 1.0)
            }
            DistortionType::Tape => {
                // Cubic saturator: approximation for symmetric tape saturation
                if x > 1.0 {
                    1.0
                } else if x < -1.0 {
                    -1.0
                } else {
                    // x - (x^3)/3 curve
                    x - (x.powi(3) / 3.0)
                }
            }
        };

        // Linear Wet / Dry Mix
        saturated * self.mix + input * (1.0 - self.mix)
    }
}

impl AudioPlugin for DistortionPlugin {
    fn name(&self) -> &'static str { "Frost Distortion" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        (self.process_single(left), self.process_single(right))
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => self.drive = value.clamp(1.0, 12.0),
            1 => self.mix = value.clamp(0.0, 1.0),
            2 => {
                self.mode = if value < 0.5 {
                    DistortionType::Soft
                } else if value < 1.5 {
                    DistortionType::Hard
                } else {
                    DistortionType::Tape
                };
            }
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.drive,
            1 => self.mix,
            2 => match self.mode {
                DistortionType::Soft => 0.0,
                DistortionType::Hard => 1.0,
                DistortionType::Tape => 2.0,
            },
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        // No cached state delays needed for static wave-shaping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unity_drive_nop_mix() {
        let mut p = DistortionPlugin::new();
        p.set_params(1.0, 0.0, DistortionType::Soft); // 0% mix means Dry NOP
        let out = p.process_single(0.5);
        assert!((out - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_hard_clamp() {
        let mut p = DistortionPlugin::new();
        p.set_params(10.0, 1.0, DistortionType::Hard); // driven hard
        let out = p.process_single(0.5); // 0.5 * 10.0 = 5.0 -> clamp 1.0
        assert_eq!(out, 1.0);
    }
}
