use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;
use frost_core::dsp::plugins::reverb::Reverb as CoreReverb;
use frost_core::dsp::plugins::AudioPlugin;

struct FrostReverbVst {
    params: Arc<FrostReverbParams>,
    core: CoreReverb,
}

#[derive(Params)]
struct FrostReverbParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "size"]
    pub size: FloatParam,
    #[id = "damping"]
    pub damping: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for FrostReverbVst {
    fn default() -> Self {
        Self {
            params: Arc::new(FrostReverbParams::default()),
            core: CoreReverb::new(44100.0),
        }
    }
}

impl Default for FrostReverbParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(450, 300),
            
            size: FloatParam::new("Room Size", 0.7, FloatRange::Linear { min: 0.0, max: 0.98 }).with_unit(" %"),
            damping: FloatParam::new("Damping", 0.2, FloatRange::Linear { min: 0.0, max: 1.0 }).with_unit(" %"),
            mix: FloatParam::new("Mix", 0.3, FloatRange::Linear { min: 0.0, max: 1.0 }).with_unit(" %"),
        }
    }
}

impl Plugin for FrostReverbVst {
    const NAME: &'static str = "Frost Reverb";
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
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(15, 10, 20)))
                    .show(egui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("FROST REVERB").color(egui::Color32::from_rgb(150, 100, 255)).size(24.0).strong());
                            ui.label(egui::RichText::new("ALGORITHMIC SPACE SIMULATOR").color(egui::Color32::from_rgb(100, 80, 120)).size(9.0));
                            ui.add_space(30.0);
                            
                            ui.columns(3, |columns| {
                                columns[0].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("SIZE").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.size, setter));
                                });
                                columns[1].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("DAMP").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.damping, setter));
                                });
                                columns[2].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("MIX").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.mix, setter));
                                });
                            });
                            
                            ui.add_space(20.0);
                        });
                    });
            },
        )
    }

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        self.core = CoreReverb::new(buffer_config.sample_rate);
        true
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        self.core.set_param(0, self.params.size.value());
        self.core.set_param(1, self.params.damping.value());
        self.core.set_param(2, self.params.mix.value());

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

impl ClapPlugin for FrostReverbVst {
    const CLAP_ID: &'static str = "com.froststudio.reverb";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Premium Space Reverb");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Reverb, ClapFeature::Stereo];
}

impl Vst3Plugin for FrostReverbVst {
    const VST3_CLASS_ID: [u8; 16] = *b"FrostReverb12345";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nih_export_clap!(FrostReverbVst);
nih_export_vst3!(FrostReverbVst);
