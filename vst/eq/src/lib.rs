use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;
use frost_core::dsp::plugins::eq::ParametricEQ as CoreEq;
use frost_core::dsp::plugins::AudioPlugin;

struct FrostEqVst {
    params: Arc<FrostEqParams>,
    core: CoreEq,
}

#[derive(Params)]
struct FrostEqParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "low_gain"]
    pub low_gain: FloatParam,
    #[id = "mid_gain"]
    pub mid_gain: FloatParam,
    #[id = "high_gain"]
    pub high_gain: FloatParam,

    #[id = "low_freq"]
    pub low_freq: FloatParam,
    #[id = "mid_freq"]
    pub mid_freq: FloatParam,
    #[id = "high_freq"]
    pub high_freq: FloatParam,
}

impl Default for FrostEqVst {
    fn default() -> Self {
        Self {
            params: Arc::new(FrostEqParams::default()),
            core: CoreEq::new(44100.0),
        }
    }
}

impl Default for FrostEqParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(500, 350),
            
            low_gain: FloatParam::new("Low Gain", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 }).with_unit(" dB"),
            mid_gain: FloatParam::new("Mid Gain", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 }).with_unit(" dB"),
            high_gain: FloatParam::new("High Gain", 0.0, FloatRange::Linear { min: -24.0, max: 24.0 }).with_unit(" dB"),

            low_freq: FloatParam::new("Low Freq", 100.0, FloatRange::Skewed { min: 20.0, max: 1000.0, factor: FloatRange::skew_factor(-2.0) }).with_unit(" Hz"),
            mid_freq: FloatParam::new("Mid Freq", 1000.0, FloatRange::Skewed { min: 200.0, max: 8000.0, factor: FloatRange::skew_factor(-2.0) }).with_unit(" Hz"),
            high_freq: FloatParam::new("High Freq", 8000.0, FloatRange::Skewed { min: 2000.0, max: 20000.0, factor: FloatRange::skew_factor(-2.0) }).with_unit(" Hz"),
        }
    }
}

impl Plugin for FrostEqVst {
    const NAME: &'static str = "Frost EQ";
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
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(10, 10, 15)))
                    .show(egui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("FROST EQ").color(egui::Color32::from_rgb(0, 200, 255)).size(24.0).strong());
                            ui.label(egui::RichText::new("PRECISION SPECTRAL SCULPTOR").color(egui::Color32::from_rgb(80, 80, 100)).size(9.0));
                            ui.add_space(25.0);
                            
                            ui.columns(3, |columns| {
                                columns[0].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("LOW").color(egui::Color32::WHITE).size(11.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.low_gain, setter));
                                    ui.add_space(5.0);
                                    ui.add(widgets::ParamSlider::for_param(&params.low_freq, setter));
                                });
                                columns[1].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("MID").color(egui::Color32::WHITE).size(11.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.mid_gain, setter));
                                    ui.add_space(5.0);
                                    ui.add(widgets::ParamSlider::for_param(&params.mid_freq, setter));
                                });
                                columns[2].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("HIGH").color(egui::Color32::WHITE).size(11.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.high_gain, setter));
                                    ui.add_space(5.0);
                                    ui.add(widgets::ParamSlider::for_param(&params.high_freq, setter));
                                });
                            });
                            
                            ui.add_space(20.0);
                        });
                    });
            },
        )
    }

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        self.core = CoreEq::new(buffer_config.sample_rate);
        true
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        // Sync bands (0: low, 2: mid, 4: high)
        self.core.set_param(0, self.params.low_freq.value());
        self.core.set_param(1, self.params.low_gain.value());
        self.core.set_param(6, self.params.mid_freq.value());
        self.core.set_param(7, self.params.mid_gain.value());
        self.core.set_param(12, self.params.high_freq.value());
        self.core.set_param(13, self.params.high_gain.value());

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

impl ClapPlugin for FrostEqVst {
    const CLAP_ID: &'static str = "com.froststudio.eq";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Precision 3-Band Parametric EQ");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Equalizer, ClapFeature::Stereo];
}

impl Vst3Plugin for FrostEqVst {
    const VST3_CLASS_ID: [u8; 16] = *b"FrostEQ123456789";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Eq];
}

nih_export_clap!(FrostEqVst);
nih_export_vst3!(FrostEqVst);
