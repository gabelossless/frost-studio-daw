use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthPreset {
    pub name: String,
    pub category: String,
    pub params: Vec<f32>, // Generic parameter list
}

pub struct SynthBank {
    pub summit_presets: Vec<SynthPreset>,
    pub eruption_presets: Vec<SynthPreset>,
    pub nebula_presets: Vec<SynthPreset>,
}

impl SynthBank {
    pub fn new() -> Self {
        let mut bank = Self {
            summit_presets: Vec::with_capacity(50),
            eruption_presets: Vec::with_capacity(50),
            nebula_presets: Vec::with_capacity(50),
        };
        
        bank.init_summit();
        bank.init_eruption();
        bank.init_nebula();
        
        bank
    }

    fn init_summit(&mut self) {
        // Categories: Leads, Pads, Bass, Plucks, Keys
        let categories = ["Lead", "Pad", "Bass", "Pluck", "Keys"];
        for i in 1..=50 {
            let cat = categories[(i - 1) % 5];
            self.summit_presets.push(SynthPreset {
                name: format!("Summit {} {}", cat, (i-1)/5 + 1),
                category: cat.to_string(),
                params: vec![0.1 * i as f32, 0.5, 0.8, 0.2], // Example params
            });
        }
    }

    fn init_eruption(&mut self) {
        let categories = ["Acid", "Fat Bass", "Modular", "Poly", "Strings"];
        for i in 1..=50 {
            let cat = categories[(i - 1) % 5];
            self.eruption_presets.push(SynthPreset {
                name: format!("Eruption {} {}", cat, (i-1)/5 + 1),
                category: cat.to_string(),
                params: vec![0.2, 0.4 * i as f32, 0.6, 0.1],
            });
        }
    }

    fn init_nebula(&mut self) {
        let categories = ["Space", "Cloud", "Atmosphere", "FX", "Bells"];
        for i in 1..=50 {
            let cat = categories[(i - 1) % 5];
            self.nebula_presets.push(SynthPreset {
                name: format!("Nebula {} {}", cat, (i-1)/5 + 1),
                category: cat.to_string(),
                params: vec![0.5, 0.5, 0.5, 0.5 * i as f32],
            });
        }
    }
}
