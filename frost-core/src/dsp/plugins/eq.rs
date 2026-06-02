use super::AudioPlugin;
use crate::dsp::filter::{BiquadFilter, FilterType};

pub struct ParametricEQ {
    bands: Vec<BiquadFilter>,
}

impl ParametricEQ {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            bands: vec![
                BiquadFilter::new(FilterType::LowShelf, 100.0, 0.0, 0.707, sample_rate),
                BiquadFilter::new(FilterType::Peaking, 400.0, 0.0, 0.707, sample_rate),
                BiquadFilter::new(FilterType::Peaking, 1000.0, 0.0, 0.707, sample_rate),
                BiquadFilter::new(FilterType::Peaking, 4000.0, 0.0, 0.707, sample_rate),
                BiquadFilter::new(FilterType::HighShelf, 10000.0, 0.0, 0.707, sample_rate),
            ],
        }
    }
}

impl AudioPlugin for ParametricEQ {
    fn name(&self) -> &'static str { "Frost Parametric EQ" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let mut l = left;
        let mut r = right;
        for band in &mut self.bands {
            l = band.process(l);
            r = band.process(r);
        }
        (l, r)
    }

    fn set_param(&mut self, id: u32, value: f32) {
        let band_idx = (id / 3) as usize;
        let param_type = id % 3; // 0: freq, 1: gain, 2: q
        
        if band_idx < self.bands.len() {
            let mut current = self.bands[band_idx].get_params();
            match param_type {
                0 => { current.0 = value.clamp(20.0, 20000.0); }, // freq
                1 => { current.1 = value.clamp(-24.0, 24.0); },   // gain
                2 => { current.2 = value.clamp(0.1, 10.0); },     // q
                _ => (),
            }
            self.bands[band_idx].set_params(current.0, current.1, current.2);
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        let band_idx = (id / 3) as usize;
        let param_type = id % 3;
        if band_idx < self.bands.len() {
            let params = self.bands[band_idx].get_params();
            match param_type {
                0 => params.0,
                1 => params.1,
                2 => params.2,
                _ => 0.0,
            }
        } else {
            0.0
        }
    }

    fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }
}
