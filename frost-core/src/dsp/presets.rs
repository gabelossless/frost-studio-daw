use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthPreset {
    pub name: String,
    pub category: String,
    pub params: Vec<f32>, // Generic parameter list
}

/// On-disk JSON shape produced by `scripts/generate_presets.cjs`.
#[derive(Debug, Deserialize)]
struct PresetFile {
    name: String,
    category: String,
    cutoff: Option<f32>,
    resonance: Option<f32>,
    attack: Option<f32>,
    decay: Option<f32>,
    sustain: Option<f32>,
    release: Option<f32>,
    osc_mix: Option<f32>,
}

/// Load `.json` preset files from a directory. Returns an empty vec if the
/// directory is missing or contains no parseable patches.
pub fn load_presets_from_dir(dir: &Path) -> Vec<SynthPreset> {
    let mut presets = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return presets;
    };

    let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();

    for path in paths {
        if path.extension().map_or(false, |ext| ext == "json") {
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            let Ok(pf) = serde_json::from_str::<PresetFile>(&text) else { continue };
            presets.push(SynthPreset {
                name: pf.name,
                category: pf.category,
                params: vec![
                    pf.cutoff.unwrap_or(0.5),
                    pf.resonance.unwrap_or(0.5),
                    pf.attack.unwrap_or(0.1),
                    pf.decay.unwrap_or(0.2),
                    pf.sustain.unwrap_or(0.7),
                    pf.release.unwrap_or(0.2),
                    pf.osc_mix.unwrap_or(0.5),
                ],
            });
        }
    }
    presets
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
    }    fn init_summit(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn factory_bank_generates_50_per_synth() {
        let bank = SynthBank::new();
        assert_eq!(bank.summit_presets.len(), 50);
        assert_eq!(bank.eruption_presets.len(), 50);
        assert_eq!(bank.nebula_presets.len(), 50);
    }

    #[test]
    fn load_presets_from_dir_parses_generated_json() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("src-tauri/presets/summit");
        let presets = load_presets_from_dir(&dir);
        assert_eq!(presets.len(), 50, "expected 50 summit patches on disk");
        assert_eq!(presets[0].params.len(), 7, "cutoff..osc_mix = 7 params");
        assert!(!presets[0].name.is_empty());
        assert!(!presets[0].category.is_empty());
    }

    #[test]
    fn load_presets_from_dir_missing_directory_returns_empty() {
        let presets = load_presets_from_dir(Path::new("/nonexistent/frost-presets"));
        assert!(presets.is_empty());
    }
}
