/// Base trait for all native Frost Studio plugins.
pub trait AudioPlugin: Send {
    fn name(&self) -> &'static str;
    fn process(&mut self, left: f32, right: f32) -> (f32, f32);
    fn set_param(&mut self, id: u32, value: f32);
    fn get_param(&self, id: u32) -> f32;
    fn reset(&mut self);
}

pub mod compressor;
pub mod eq;
pub mod limiter;
pub mod bass;
pub mod delay;
pub mod reverb;
pub mod distortion;

// Bridge to dynamic VST3 host Node flawless
pub use crate::vst::host::Vst3PluginInstance;
