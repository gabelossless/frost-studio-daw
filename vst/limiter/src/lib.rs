use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
use std::sync::Arc;
use frost_core::dsp::plugins::limiter::Limiter as CoreLimiter;
use frost_core::dsp::plugins::AudioPlugin;

struct FrostLimiterVst {
    params: Arc<FrostLimiterParams>,
    core: CoreLimiter,
}

#[derive(Params)]
struct FrostLimiterParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "threshold"]
    pub threshold: FloatParam,
    #[id = "ceiling"]
    pub ceiling: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
}

impl Default for FrostLimiterVst {
    fn default() -> Self {
        Self {
            params: Arc::new(FrostLimiterParams::default()),
            core: CoreLimiter::new(44100.0),
        }
    }
}

impl Default for FrostLimiterParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(400, 300),
            
            threshold: FloatParam::new("Threshold", 0.0, FloatRange::Linear { min: -40.0, max: 0.0 }).with_unit(" dB"),
            ceiling: FloatParam::new("Ceiling", -0.1, FloatRange::Linear { min: -6.0, max: 0.0 }).with_unit(" dB"),
            release: FloatParam::new("Release", 50.0, FloatRange::Skewed { min: 1.0, max: 1000.0, factor: FloatRange::skew_factor(-2.0) }).with_unit(" ms"),
        }
    }
}

impl Plugin for FrostLimiterVst {
    const NAME: &'static str = "Frost Limiter";
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
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 10, 10)))
                    .show(egui_ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(15.0);
                            ui.label(egui::RichText::new("FROST LIMITER").color(egui::Color32::from_rgb(255, 50, 50)).size(24.0).strong());
                            ui.label(egui::RichText::new("BRICKWALL SAFETY ENGINE").color(egui::Color32::from_rgb(120, 80, 80)).size(9.0));
                            ui.add_space(25.0);
                            
                            ui.label(egui::RichText::new("THRESHOLD").color(egui::Color32::WHITE).size(12.0));
                            ui.add(widgets::ParamSlider::for_param(&params.threshold, setter));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("CEILING").color(egui::Color32::WHITE).size(12.0));
                            ui.add(widgets::ParamSlider::for_param(&params.ceiling, setter));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("RELEASE").color(egui::Color32::WHITE).size(12.0));
                            ui.add(widgets::ParamSlider::for_param(&params.release, setter));
                            
                            ui.add_space(20.0);
                        });
                    });
            },
        )
    }

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        self.core = CoreLimiter::new(buffer_config.sample_rate);
        true
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, _context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        let thresh_lin = 10.0f32.powf(self.params.threshold.value() / 20.0);
        let ceil_lin = 10.0f32.powf(self.params.ceiling.value() / 20.0);
        
        self.core.set_param(0, thresh_lin);
        self.core.set_param(1, ceil_lin);
        self.core.set_param(3, self.params.release.value() / 1000.0);

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

impl ClapPlugin for FrostLimiterVst {
    const CLAP_ID: &'static str = "com.froststudio.limiter";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Premium Brickwall Limiter");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Limiter, ClapFeature::Stereo];
}

impl Vst3Plugin for FrostLimiterVst {
    const VST3_CLASS_ID: [u8; 16] = *b"FrostLimit123456";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(FrostLimiterVst);
nih_export_vst3!(FrostLimiterVst);
