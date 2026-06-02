use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;
use frost_core::dsp::plugins::bass::BassSynthPlugin as CoreBass;
use frost_core::dsp::plugins::AudioPlugin;

struct FrostBassVst {
    params: Arc<FrostBassParams>,
    core: CoreBass,
}

#[derive(Params)]
struct FrostBassParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "cutoff"]
    pub cutoff: FloatParam,
    #[id = "octave"]
    pub octave: IntParam,
}

impl Default for FrostBassVst {
    fn default() -> Self {
        Self {
            params: Arc::new(FrostBassParams::default()),
            core: CoreBass::new(44100.0),
        }
    }
}

impl Default for FrostBassParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(400, 300),
            
            cutoff: FloatParam::new("Cutoff", 1000.0, FloatRange::Skewed { min: 40.0, max: 5000.0, factor: FloatRange::skew_factor(-2.0) }).with_unit(" Hz"),
            octave: IntParam::new("Octave", 0, IntRange::Linear { min: -2, max: 2 }),
        }
    }
}

impl Plugin for FrostBassVst {
    const NAME: &'static str = "Frost Bass";
    const VENDOR: &'static str = "Frost Studio";
    const URL: &'static str = "https://froststudio.app";
    const EMAIL: &'static str = "support@froststudio.app";
    const VERSION: &'static str = "1.0.0";
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> { self.params.clone() }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(10, 20, 15)))
                    .show(egui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("FROST BASS").color(egui::Color32::from_rgb(0, 255, 150)).size(24.0).strong());
                            ui.label(egui::RichText::new("ANALOG SUB-PHONIC ENHANCER").color(egui::Color32::from_rgb(80, 120, 100)).size(9.0));
                            ui.add_space(30.0);
                            
                            ui.label(egui::RichText::new("CUTOFF").color(egui::Color32::WHITE).size(12.0));
                            ui.add(widgets::ParamSlider::for_param(&params.cutoff, setter));
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("OCTAVE").color(egui::Color32::WHITE).size(12.0));
                            ui.add(widgets::ParamSlider::for_param(&params.octave, setter));
                            
                            ui.add_space(20.0);
                        });
                    });
            },
        )
    }

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        self.core = CoreBass::new(buffer_config.sample_rate);
        true
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        self.core.set_param(1, self.params.cutoff.value());
        
        // This plugin layers the bass on top of the input signal
        for samples in buffer.iter_samples() {
            let mut s = samples.into_iter();
            if let (Some(l), Some(r)) = (s.next(), s.next()) {
                let (out_l, out_r) = self.core.process(*l, *r);
                *l = out_l;
                *r = out_r;
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for FrostBassVst {
    const CLAP_ID: &'static str = "com.froststudio.bass";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Premium Sub-Bass Enhancer");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Synthesizer, ClapFeature::Stereo];
}

impl Vst3Plugin for FrostBassVst {
    const VST3_CLASS_ID: [u8; 16] = *b"FrostBass1234567";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(FrostBassVst);
nih_export_vst3!(FrostBassVst);
