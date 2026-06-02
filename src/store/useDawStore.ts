import { create } from 'zustand';
import { temporal } from 'zundo';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type TrackType = 'midi' | 'audio';

declare global {
    interface Window {
        __TAURI_INTERNALS__?: any;
    }
}

export interface EqBandParams {
    freq_hz: number;
    gain_db: number;
    q: number;
}

export interface MeterLevel {
    channel_id: number;
    rms_left: number;
    rms_right: number;
    peak_left: number;
    peak_right: number;
    playhead_beats: number;
}

export interface Track {
    id: string;
    name: string;
    color: string;
    type: TrackType;
    volume: number; // 0.0 to 1.0 (linear for now)
    pan: number;    // -1.0 to 1.0
    muted: boolean;
    soloed: boolean;
    armed: boolean;
    eq_low: EqBandParams;
    eq_mid: EqBandParams;
    eq_high: EqBandParams;
    send_1_amount: number;
    send_2_amount: number;
    sidechain_source_id?: number | null;
    sidechain_ratio?: number;
    plugins?: string[];
}

export interface Clip {
    id: string;
    trackId: string;
    start: number; // in beats
    duration: number; // in beats
    name: string;
    color: string;
    samplePath?: string; // Absolute path to file on disk
}

export interface Note {
    id: string;
    clipId: string;
    pitch: string; // e.g., 'C4'
    start: number; // relative to clip start, in beats
    duration: number; // in beats
    velocity: number; // 0-127
}

interface DawState {
    // Transport
    isPlaying: boolean;
    isRecording: boolean;
    metronomeEnabled: boolean;
    tempo: number;
    timeSignature: [number, number]; // [numerator, denominator]
    playheadPosition: number; // in beats

    // Project Data
    tracks: Track[];
    clips: Clip[];
    notes: Note[];

    // Audio State
    masterVolume: number;
    meters: Record<number, MeterLevel>;
    synthParams: {
        waveform: 'Sine' | 'Square' | 'Saw';
        attack: number;
        decay: number;
        sustain: number;
        release: number;
    };
    masterLimiterParams: {
        threshold: number;
        ceiling: number;
        attack_ms: number;
        release_ms: number;
    };

    // UI State
    selectedTrackId: string | null;
    selectedClipId: string | null;
    activeDetailTab: 'piano-roll' | 'mixer' | 'audio' | 'instrument' | 'plugins' | 'visualizer' | 'plugin-insert';
    gridSize: number;
    isPluginManagerOpen: boolean;
    isFullScreenMixer: boolean;
    isExportModalOpen: boolean;
    isPluginPickerOpen: boolean;
    midiDevices: string[];
    selectedMidiDeviceId: string | null;

    // Actions
    togglePlay: () => Promise<void>;
    stop: () => Promise<void>;
    toggleRecord: () => void;
    toggleMetronome: () => void;
    setTempo: (tempo: number) => void;

    addTrack: (track: Omit<Track, 'id'>) => void;
    updateTrack: (id: string, updates: Partial<Track>) => void;
    deleteTrack: (id: string) => void;
    setMasterVolume: (volume: number) => void;

    addClip: (clip: Omit<Clip, 'id'>) => void;
    updateClip: (id: string, updates: Partial<Clip>) => void;
    deleteClip: (id: string) => void;
    setClips: (clips: Clip[]) => void;

    addNote: (note: Omit<Note, 'id'>) => void;
    updateNote: (id: string, updates: Partial<Note>) => void;
    deleteNote: (id: string) => void;
    syncMidi: () => Promise<void>;

    setSelectedTrack: (id: string | null) => void;
    setSelectedClip: (id: string | null) => void;
    setSelectedMidiDevice: (id: string | null) => void;
    setActiveDetailTab: (tab: 'piano-roll' | 'mixer' | 'audio' | 'instrument' | 'plugins' | 'visualizer' | 'plugin-insert') => void;
    setGridSize: (size: number) => void;
    togglePluginManager: () => void;
    toggleFullScreenMixer: () => void;
    setExportModalOpen: (open: boolean) => void;
    setPluginPickerOpen: (open: boolean) => void;
    setMasterLimiterParams: (params: Partial<DawState['masterLimiterParams']>) => void;
    setSynthParams: (params: Partial<DawState['synthParams']>) => void;
    setSynthType: (channelId: number, synthType: string) => Promise<void>;
    fetchPresets: (synthType: string) => Promise<void>;
    addPlugin: (channelId: number, pluginType: string) => Promise<void>;
    removePlugin: (channelId: number, index: number) => Promise<void>;
    setPluginParam: (channelId: number, pluginIndex: number, paramId: number, value: number) => Promise<void>;
    setSamplerSample: (channelId: number, path: string) => Promise<void>;
    syncAudioTracks: () => Promise<void>;
    addAudioClipFromPath: (path: string) => Promise<void>;
    initAudioEngine: () => Promise<void>;
}

const defaultEq = {
    eq_low: { freq_hz: 100.0, gain_db: 0.0, q: 0.707 },
    eq_mid: { freq_hz: 1000.0, gain_db: 0.0, q: 0.707 },
    eq_high: { freq_hz: 8000.0, gain_db: 0.0, q: 0.707 },
};

const INITIAL_TRACKS: Track[] = [
    { id: 't1', name: 'Audio 1', color: 'bg-emerald-500', type: 'audio', volume: 0.8, pan: 0, muted: false, soloed: false, armed: false, send_1_amount: 0.0, send_2_amount: 0.0, ...defaultEq },
];

const INITIAL_CLIPS: Clip[] = [];


// Helper to convert pitch string to MIDI number
const PITCH_TO_MIDI: Record<string, number> = {
    'C': 0, 'C#': 1, 'D': 2, 'D#': 3, 'E': 4, 'F': 5, 'F#': 6, 'G': 7, 'G#': 8, 'A': 9, 'A#': 10, 'B': 11
};
function pitchToMidiNumber(pitch: string): number {
    const note = pitch.slice(0, -1);
    const octave = parseInt(pitch.slice(-1));
    return (octave + 1) * 12 + PITCH_TO_MIDI[note];
}

let meterUnlisten: (() => void) | null = null;
let audioTickInterval: number | null = null;
let rafTickId: number | null = null;
let midiAccessCleanup: (() => void) | null = null;

export const useDawStore = create<DawState>()(
    temporal(
        (set, get) => ({
            // Initial State
            isPlaying: false,
            isRecording: false,
            metronomeEnabled: true,
            tempo: 120,
            timeSignature: [4, 4] as [number, number],
            playheadPosition: 0,

            tracks: INITIAL_TRACKS,
            clips: INITIAL_CLIPS,
            notes: [],

            masterVolume: 0.8,
            meters: {},
            synthParams: {
                waveform: 'Sine',
                attack: 0.01,
                decay: 0.1,
                sustain: 0.5,
                release: 0.2,
            },
            masterLimiterParams: {
                threshold: 1.0,
                ceiling: 0.99,
                attack_ms: 1.0,
                release_ms: 50.0,
            },

            selectedTrackId: 't1',
            selectedClipId: 'c1',
            activeDetailTab: 'piano-roll',
            gridSize: 0.25,
            isPluginManagerOpen: false,
            isFullScreenMixer: false,
            isExportModalOpen: false,
            isPluginPickerOpen: false,
            midiDevices: [],
            selectedMidiDeviceId: null,

            // Actions
            togglePlay: async () => {
                const playing = !get().isPlaying;
                set({ isPlaying: playing });
                try {
                    const { resumeAudio } = await import('../audioContext');
                    resumeAudio();
                } catch (e) { console.error('Failed to resume audio context', e); }

                if ((window as any).__TAURI__) {
                    await invoke('set_transport', { playing }).catch(console.error);
                }

                if (playing && !audioTickInterval) {
                    if (rafTickId) { cancelAnimationFrame(rafTickId); rafTickId = null; }
                    let lastFrame = performance.now();
                    const tick = () => {
                        if (!get().isPlaying) return;
                        if ((window as any).__TAURI__) {
                            invoke('process_audio_tick').catch(console.error);
                        } else {
                            const now = performance.now();
                            const dt = (now - lastFrame) / 1000;
                            lastFrame = now;
                            const bpm = get().tempo || 120;
                            const beatsPerSecond = bpm / 60;
                            set(state => ({ playheadPosition: state.playheadPosition + beatsPerSecond * dt }));
                        }
                        rafTickId = requestAnimationFrame(tick);
                    };
                    rafTickId = requestAnimationFrame(tick);
                } else if (!playing) {
                    if (rafTickId) { cancelAnimationFrame(rafTickId); rafTickId = null; }
                }
            },
            stop: async () => {
                set({ isPlaying: false, isRecording: false, playheadPosition: 0 });
                if (window.__TAURI_INTERNALS__ || true) {
                    await invoke('set_transport', { playing: false }).catch(console.error);
                }
                if (rafTickId) { cancelAnimationFrame(rafTickId); rafTickId = null; }
            },
            toggleRecord: () => set((state) => ({ isRecording: !state.isRecording, isPlaying: true })),
            toggleMetronome: () => set((state) => ({ metronomeEnabled: !state.metronomeEnabled })),
            setTempo: (tempo: number) => {
                set({ tempo });
                if (window.__TAURI_INTERNALS__ || true) {
                    invoke('set_tempo', { tempo }).catch(console.error);
                }
            },

            addTrack: (track: Omit<Track, 'id'>) => set((state) => ({
                tracks: [...state.tracks, { ...track, id: `t${Date.now()}` }]
            })),
            updateTrack: (id: string, updates: Partial<Track>) => {
                set((state) => {
                    const newTracks = state.tracks.map(t => t.id === id ? { ...t, ...updates } : t);
                    const trackIndex = state.tracks.findIndex(t => t.id === id);
                    if ((trackIndex !== -1 && window.__TAURI_INTERNALS__) || trackIndex !== -1) {
                        const track = newTracks[trackIndex];
                        invoke('set_channel_params', {
                            params: {
                                channel_id: trackIndex,
                                volume: track.volume,
                                pan: track.pan,
                                muted: track.muted,
                                soloed: track.soloed,
                                eq_low: track.eq_low,
                                eq_mid: track.eq_mid,
                                eq_high: track.eq_high,
                                send_1_amount: track.send_1_amount,
                                send_2_amount: track.send_2_amount,
                                sidechain_source_id: track.sidechain_source_id ?? null,
                                sidechain_ratio: track.sidechain_ratio ?? 0.0
                            }
                        }).catch(console.error);
                    }
                    return { tracks: newTracks };
                });
            },
            deleteTrack: (id: string) => set((state) => ({
                tracks: state.tracks.filter(t => t.id !== id),
                clips: state.clips.filter(c => c.trackId !== id)
            })),

            setMasterVolume: (volume: number) => {
                set({ masterVolume: volume });
                if (window.__TAURI_INTERNALS__ || true) {
                    invoke('set_master_volume', { volume }).catch(console.error);
                }
            },

            addClip: (clip: Omit<Clip, 'id'>) => set((state) => ({
                clips: [...state.clips, { ...clip, id: `c${Date.now()}` }]
            })),
            updateClip: (id: string, updates: Partial<Clip>) => {
                set((state) => ({
                    clips: state.clips.map(c => c.id === id ? { ...c, ...updates } : c)
                }));
                get().syncMidi();
            },
            deleteClip: (id: string) => set((state) => ({
                clips: state.clips.filter(c => c.id !== id),
                notes: state.notes.filter(n => n.clipId !== id)
            })),
            setClips: (clips: Clip[]) => {
                set({ clips });
                get().syncMidi();
            },

            addNote: (note: Omit<Note, 'id'>) => {
                set((state) => ({
                    notes: [...state.notes, { ...note, id: `n${Date.now()}` }]
                }));
                get().syncMidi();
            },
            updateNote: (id: string, updates: Partial<Note>) => {
                set((state) => ({
                    notes: state.notes.map(n => n.id === id ? { ...n, ...updates } : n)
                }));
                get().syncMidi();
            },
            deleteNote: (id: string) => {
                set((state) => ({
                    notes: state.notes.filter(n => n.id !== id)
                }));
                get().syncMidi();
            },
            syncMidi: async () => {
                if (!window.__TAURI_INTERNALS__ && typeof window !== 'undefined' && !(window as any).__TAURI__) {
                    // console.warn("Tauri not detected, skipping syncMidi");
                    // return; 
                }

                const state = get();
                const allRustNotes: any[] = [];

                state.tracks.forEach((track, trackIndex) => {
                    const trackClips = state.clips.filter(c => c.trackId === track.id);
                    trackClips.forEach(clip => {
                        const clipNotes = state.notes.filter(n => n.clipId === clip.id);
                        clipNotes.forEach(n => {
                            allRustNotes.push({
                                channel_id: trackIndex,
                                pitch: pitchToMidiNumber(n.pitch),
                                velocity: n.velocity / 127,
                                start_tick: Math.floor((clip.start + n.start) * 960),
                                duration_ticks: Math.floor(n.duration * 960)
                            });
                        });
                    });
                });

                await invoke('sync_midi_data', { notes: allRustNotes }).catch(console.error);
            },

            setSelectedTrack: (id: string | null) => set({ selectedTrackId: id }),
            setSelectedClip: (id: string | null) => set({ selectedClipId: id }),
            setActiveDetailTab: (tab: 'piano-roll' | 'mixer' | 'audio' | 'instrument' | 'plugins' | 'visualizer' | 'plugin-insert') => set({ activeDetailTab: tab }),
            setGridSize: (size: number) => set({ gridSize: size }),
            togglePluginManager: () => set((state) => ({ isPluginManagerOpen: !state.isPluginManagerOpen })),
            toggleFullScreenMixer: () => set((state) => ({ isFullScreenMixer: !state.isFullScreenMixer })),
            setExportModalOpen: (open: boolean) => set({ isExportModalOpen: open }),
            setPluginPickerOpen: (open: boolean) => set({ isPluginPickerOpen: open }),
            setMasterLimiterParams: (params: Partial<DawState['masterLimiterParams']>) => {
                set((state) => {
                    const newParams = { ...state.masterLimiterParams, ...params };
                    if ((window as any).__TAURI__) {
                        invoke('set_master_limiter_params', { ...newParams }).catch(console.error);
                    }
                    return { masterLimiterParams: newParams as DawState['masterLimiterParams'] };
                });
            },
            setSynthParams: (params: Partial<DawState['synthParams']>) => {
                set((state) => {
                    const newParams = { ...state.synthParams, ...params };
                    if ((window as any).__TAURI__) {
                        invoke('set_synth_params', { params: newParams }).catch(console.error);
                    }
                    return { synthParams: newParams as DawState['synthParams'] };
                });
            },
            setSynthType: async (channelId: number, synthType: string) => {
                if ((window as any).__TAURI__) {
                    await invoke('set_synth_type', { channelId, synthType }).catch(console.error);
                }
                get().fetchPresets(synthType);
            },
            fetchPresets: async (synthType: string) => {
                if ((window as any).__TAURI__) {
                    const presets = await invoke<any[]>('get_synth_presets', { synthType }).catch(() => []);
                    console.log(`Fetched ${presets.length} presets for ${synthType}`);
                }
            },
            addPlugin: async (channelId: number, pluginType: string) => {
                if ((window as any).__TAURI__) {
                    const names = await invoke<string[]>('add_native_plugin', { channelId, pluginType }).catch(console.error);
                    if (names) {
                        const updatedTracks = [...get().tracks];
                        if (updatedTracks[channelId]) {
                            updatedTracks[channelId] = { ...updatedTracks[channelId], plugins: names };
                            set({ tracks: updatedTracks });
                        }
                    }
                }
            },
            removePlugin: async (channelId: number, index: number) => {
                if ((window as any).__TAURI__) {
                    const names = await invoke<string[]>('remove_native_plugin', { channelId, index }).catch(console.error);
                    if (names) {
                        const updatedTracks = [...get().tracks];
                        if (updatedTracks[channelId]) {
                            updatedTracks[channelId] = { ...updatedTracks[channelId], plugins: names };
                            set({ tracks: updatedTracks });
                        }
                    }
                }
            },
            setPluginParam: async (channelId: number, pluginIndex: number, paramId: number, value: number) => {
                if ((window as any).__TAURI__) {
                    await invoke('set_plugin_param', { channelId, pluginIndex, paramId, value }).catch(console.error);
                }
            },
            setSamplerSample: async (channelId: number, path: string) => {
                if ((window as any).__TAURI__) {
                    await invoke('set_sampler_sample', { channelId, path }).catch(console.error);
                }
            },
            syncAudioTracks: async () => {
                const state = get();
                const rustTracks = state.tracks
                    .filter(t => t.type === 'audio')
                    .map((t, idx) => ({
                        id: idx,
                        name: t.name,
                        clips: state.clips
                            .filter(c => c.trackId === t.id)
                            .map(c => ({
                                id: c.id,
                                sample_path: c.samplePath || c.name,
                                start_tick: Math.floor(c.start * 960),
                                duration_ticks: Math.floor(c.duration * 960),
                                offset_samples: 0,
                                gain: 1.0
                            })),
                        muted: t.muted,
                        volume: t.volume,
                        pan: t.pan,
                        channel_id: idx // Basic routing for now
                    }));
                
                if ((window as any).__TAURI__) {
                    await invoke('sync_audio_tracks', { tracks: rustTracks }).catch(console.error);
                }
            },

            addAudioClipFromPath: async (path: string) => {
                const state = get();
                // 1. Ensure we have an audio track
                let targetTrack = state.tracks.find(t => t.type === 'audio');
                if (!targetTrack) {
                    targetTrack = {
                        id: `t${Date.now()}`,
                        name: 'Audio Track 1',
                        color: 'bg-emerald-500',
                        type: 'audio',
                        volume: 0.8, pan: 0, muted: false, soloed: false, armed: false,
                        send_1_amount: 0, send_2_amount: 0,
                        eq_low: { freq_hz: 100, gain_db: 0, q: 1.0 }, eq_mid: { freq_hz: 1000, gain_db: 0, q: 1.0 }, eq_high: { freq_hz: 8000, gain_db: 0, q: 1.0 }
                    };
                    state.addTrack(targetTrack);
                }

                // 2. Add Clip
                const name = path.split(/[/\\]/).pop() || 'Imported Audio';
                const newClip: Clip = {
                    id: `c${Date.now()}`,
                    trackId: targetTrack.id,
                    start: 0,
                    duration: 16, // Default large enough
                    name,
                    color: targetTrack.color,
                    samplePath: path
                };

                set({ clips: [...get().clips, newClip] });
                await get().syncAudioTracks();
            },

            setSelectedMidiDevice: (id: string | null) => set({ selectedMidiDeviceId: id }),

            initAudioEngine: async () => {
                if (!(window as any).__TAURI__) {
                    console.log("Tauri not detected, running in browser-only mode.");
                    return;
                }
                const tracks = get().tracks;
                for (let i = 0; i < tracks.length; i++) {
                    const t = tracks[i];
                    await invoke('set_channel_params', {
                        params: {
                            channel_id: i, volume: t.volume, pan: t.pan, muted: t.muted, soloed: t.soloed,
                            eq_low: t.eq_low, eq_mid: t.eq_mid, eq_high: t.eq_high,
                            send_1_amount: t.send_1_amount, send_2_amount: t.send_2_amount
                        }
                    }).catch(console.error);
                }
                await invoke('set_master_volume', { volume: get().masterVolume }).catch(console.error);
                await invoke('set_tempo', { tempo: get().tempo }).catch(console.error);
                await invoke('set_synth_params', { params: get().synthParams }).catch(console.error);
                await get().syncAudioTracks();
                await get().syncMidi();

                // Initialize Web MIDI
                if (navigator.requestMIDIAccess) {
                    try {
                        if (midiAccessCleanup) {
                            midiAccessCleanup();
                            midiAccessCleanup = null;
                        }
                        const midiAccess = await navigator.requestMIDIAccess();
                        const devices: string[] = [];
                        const inputs = midiAccess.inputs.values();
                        const handlers: { input: MIDIInput, handler: (e: MIDIMessageEvent) => void }[] = [];
                        
                        for (let input = inputs.next(); input && !input.done; input = inputs.next()) {
                            const device = input.value;
                            devices.push(device.name || 'Unknown Device');
                            
                            const handler = (e: any) => {
                                const [status, note, velocity] = e.data;
                                const type = status & 0xf0;
                                const state = get();
                                
                                const activeTrack = state.tracks.find((t: any) => t.armed) || state.tracks.find((t: any) => t.id === state.selectedTrackId);
                                const channelId = state.tracks.indexOf(activeTrack || state.tracks[0]);
                                if (channelId === -1) return;

                                if (type === 144 && velocity > 0) {
                                    invoke('trigger_note_on', { channelId, note, velocity }).catch(console.error);
                                } else if (type === 128 || (type === 144 && velocity === 0)) {
                                    invoke('trigger_note_off', { channelId, note }).catch(console.error);
                                }
                            };
                            device.onmidimessage = handler;
                            handlers.push({ input: device, handler });
                        }
                        midiAccessCleanup = () => {
                            for (const { input, handler } of handlers) {
                                input.onmidimessage = null;
                            }
                        };
                        set({ midiDevices: devices });
                    } catch (err) {
                        console.error("Failed to access MIDI devices:", err);
                    }
                }

                if (meterUnlisten) meterUnlisten();
                meterUnlisten = await listen<MeterLevel[]>('meter-levels', (event) => {
                    const metersObj: Record<number, MeterLevel> = {};
                    let newPlayhead = get().playheadPosition;
                    if (event.payload.length > 0) {
                        newPlayhead = event.payload[0].playhead_beats;
                    }
                    event.payload.forEach(m => { metersObj[m.channel_id] = m; });
                    set({ meters: metersObj, playheadPosition: newPlayhead });
                });
            }
        }),
        {
            partialize: (state) => ({
                tracks: state.tracks,
                clips: state.clips,
                notes: state.notes,
                tempo: state.tempo,
                timeSignature: state.timeSignature,
            }),
        }
    )
);

// Auto-initialize when store loads
if (typeof window !== 'undefined') {
    useDawStore.getState().initAudioEngine();
}
