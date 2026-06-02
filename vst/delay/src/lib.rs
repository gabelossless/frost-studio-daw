use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;
use frost_core::dsp::plugins::delay::Delay as CoreDelay;
use frost_core::dsp::plugins::AudioPlugin;

struct FrostDelayVst {
    params: Arc<FrostDelayParams>,
    core: CoreDelay,
}

#[derive(Params)]
struct FrostDelayParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "time"]
    pub time: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for FrostDelayVst {
    fn default() -> Self {
        Self {
            params: Arc::new(FrostDelayParams::default()),
            core: CoreDelay::new(44100.0),
        }
    }
}

impl Default for FrostDelayParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(450, 300),
            
            time: FloatParam::new("Time", 0.5, FloatRange::Linear { min: 0.01, max: 2.0 }).with_unit(" s"),
            feedback: FloatParam::new("Feedback", 0.3, FloatRange::Linear { min: 0.0, max: 0.95 }).with_unit(" %"),
            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }).with_unit(" %"),
        }
    }
}

impl Plugin for FrostDelayVst {
    const NAME: &'static str = "Frost Delay";
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
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(10, 15, 25)))
                    .show(egui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("FROST DELAY").color(egui::Color32::from_rgb(0, 150, 255)).size(24.0).strong());
                            ui.label(egui::RichText::new("ECHO REFLECTION ENGINE").color(egui::Color32::from_rgb(80, 100, 120)).size(9.0));
                            ui.add_space(30.0);
                            
                            ui.columns(3, |columns| {
                                columns[0].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("TIME").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.time, setter));
                                });
                                columns[1].vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("FEEDBACK").color(egui::Color32::WHITE).size(12.0));
                                    ui.add(widgets::ParamSlider::for_param(&params.feedback, setter));
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
        self.core = CoreDelay::new(buffer_config.sample_rate);
        true
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        self.core.set_param(0, self.params.time.value());
        self.core.set_param(1, self.params.feedback.value());
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

impl ClapPlugin for FrostDelayVst {
    const CLAP_ID: &'static str = "com.froststudio.delay";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Premium Echo Delay");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Delay, ClapFeature::Stereo];
}

impl Vst3Plugin for FrostDelayVst {
    const VST3_CLASS_ID: [u8; 16] = *b"FrostDelay123456";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}

nih_export_clap!(FrostDelayVst);
nih_export_vst3!(FrostDelayVst);
