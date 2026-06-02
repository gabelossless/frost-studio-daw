/// MIDI Event System and Master Clock for Frost Studio.

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NoteEvent {
    pub channel_id: usize,
    pub pitch: u8,
    pub velocity: f32,
    pub start_tick: u64,
    pub duration_ticks: u64,
}

pub struct MasterClock {
    pub current_tick: u64,
    pub bpm: f32,
    pub sample_rate: f32,
    pub is_playing: bool,
    
    // Internal accumulator for precise tick calculation
    tick_accumulator: f32,
}

impl MasterClock {
    pub fn new(sample_rate: f32, bpm: f32) -> Self {
        Self {
            current_tick: 0,
            bpm,
            sample_rate,
            is_playing: false,
            tick_accumulator: 0.0,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
    }

    pub fn start(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
    }

    pub fn reset(&mut self) {
        self.current_tick = 0;
        self.tick_accumulator = 0.0;
    }

    /// Advance the clock by one sample and return the new tick if it changed.
    /// Ticks are 960 PPQ (Pulses Per Quarter Note).
    pub fn tick(&mut self) -> Option<u64> {
        if !self.is_playing { return None; }

        let ticks_per_second = (self.bpm / 60.0) * 960.0;
        let ticks_per_sample = ticks_per_second / self.sample_rate;
        
        self.tick_accumulator += ticks_per_sample;
        
        if self.tick_accumulator >= 1.0 {
            let ticks_to_add = self.tick_accumulator.floor() as u64;
            self.current_tick += ticks_to_add;
            self.tick_accumulator -= ticks_to_add as f32;
            Some(self.current_tick)
        } else {
            None
        }
    }

    pub fn get_position_beats(&self) -> f32 {
        self.current_tick as f32 / 960.0
    }
}

pub struct MidiPlaylist {
    pub notes: Vec<NoteEvent>,
}

impl MidiPlaylist {
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }

    pub fn update(&mut self, notes: Vec<NoteEvent>) {
        self.notes = notes;
        // Sort by start_tick for efficient processing
        self.notes.sort_by_key(|n| n.start_tick);
    }
}
