use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;
use frost_core::dsp::plugins::compressor::Compressor as CoreCompressor;
use frost_core::dsp::plugins::AudioPlugin;

struct FrostCompressorVst {
    params: Arc<FrostCompressorParams>,
    core: CoreCompressor,
}

#[derive(Params)]
struct FrostCompressorParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "ratio"]
    pub ratio: FloatParam,

    #[id = "makeup"]
    pub makeup: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "release"]
    pub release: FloatParam,
}

impl Default for FrostCompressorVst {
    fn default() -> Self {
        Self {
            params: Arc::new(FrostCompressorParams::default()),
            core: CoreCompressor::new(44100.0),
        }
    }
}

impl Default for FrostCompressorParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(500, 400),
            
            threshold: FloatParam::new(
                "Threshold",
                -20.0,
                FloatRange::Linear { min: -60.0, max: 0.0 },
            )
            .with_unit(" dB"),

            ratio: FloatParam::new(
                "Ratio",
                4.0,
                FloatRange::Linear { min: 1.0, max: 20.0 },
            )
            .with_unit(":1"),

            makeup: FloatParam::new(
                "Makeup",
                0.0,
                FloatRange::Linear { min: -20.0, max: 20.0 },
            )
            .with_unit(" dB"),

            attack: FloatParam::new(
                "Attack",
                10.0,
                FloatRange::Skewed { min: 0.1, max: 500.0, factor: FloatRange::skew_factor(-2.0) },
            )
            .with_unit(" ms"),

            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Skewed { min: 1.0, max: 2000.0, factor: FloatRange::skew_factor(-2.0) },
            )
            .with_unit(" ms"),
        }
    }
}

impl Plugin for FrostCompressorVst {
    const NAME: &'static str = "Frost Compressor";
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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(15, 15, 20)))
                    .show(egui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(
                                egui::RichText::new("FROST COMPRESSOR")
                                    .color(egui::Color32::from_rgb(255, 80, 0))
                                    .size(28.0)
                                    .strong()
                            );
                            ui.label(
                                egui::RichText::new("PREMIUM ANALOG MODELED VCA")
                                    .color(egui::Color32::from_rgb(100, 100, 120))
                                    .size(10.0)
                            );
                            ui.add_space(30.0);
                            
                            // Moog-style control panels
                            ui.columns(5, |columns| {
                                columns[0].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("THRESH").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.threshold, setter));
                                });
                                columns[1].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("RATIO").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.ratio, setter));
                                });
                                columns[2].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("ATTACK").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.attack, setter));
                                });
                                columns[3].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("RELEASE").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.release, setter));
                                });
                                columns[4].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("GAIN").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.makeup, setter));
                                });
                            });
                            
                            ui.add_space(40.0);
                            // Industrial Footer
                            ui.painter().rect_filled(
                                ui.available_rect_before_wrap(),
                                0.0,
                                egui::Color32::from_rgb(30, 30, 35)
                            );
                        });
                    });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.core = CoreCompressor::new(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Map VST params (dB/ms) to Core params (Lin/Sec)
        let threshold_lin = 10.0f32.powf(self.params.threshold.value() / 20.0);
        let makeup_lin = 10.0f32.powf(self.params.makeup.value() / 20.0);
        
        self.core.set_param(0, threshold_lin);
        self.core.set_param(1, self.params.ratio.value());
        self.core.set_param(2, self.params.attack.value() / 1000.0);
        self.core.set_param(3, self.params.release.value() / 1000.0);
        self.core.set_param(4, makeup_lin);

        // Check if we have at least 2 channels
        let num_channels = buffer.channels();
        let _num_samples = buffer.samples();
        if num_channels < 2 {
            return ProcessStatus::Normal;
        }

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

impl ClapPlugin for FrostCompressorVst {
    const CLAP_ID: &'static str = "com.froststudio.compressor";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Premium VCA Compressor");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Compressor, ClapFeature::Stereo];
}

impl Vst3Plugin for FrostCompressorVst {
    const VST3_CLASS_ID: [u8; 16] = *b"FrostComp1234567";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(FrostCompressorVst);
nih_export_vst3!(FrostCompressorVst);
