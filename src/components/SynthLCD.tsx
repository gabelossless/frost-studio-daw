import { useState, useEffect } from 'react';
import { useDawStore } from '../store/useDawStore';
import { invoke } from '@tauri-apps/api/core';
import { Knob } from './Knob';

export default function SynthLCD() {
    const { tracks, selectedTrackId, synthParams, setSynthParams } = useDawStore();
    const selectedTrackIndex = tracks.findIndex(t => t.id === selectedTrackId);
    const [synthType, setSynthType] = useState('Summit');
    const [analyzerData, setAnalyzerData] = useState<number[]>(new Array(64).fill(0));

    // Simulated waveform/analyzer data
    useEffect(() => {
        const interval = setInterval(() => {
            setAnalyzerData(prev => prev.map(() => Math.random() * 50 + 10));
        }, 100);
        return () => clearInterval(interval);
    }, []);

    const handleSynthChange = async (type: string) => {
        setSynthType(type);
        if (selectedTrackIndex !== -1 && (window as any).__TAURI_INTERNALS__) {
            try {
                await invoke('set_synth_type', { channelId: selectedTrackIndex, synthType: type });
            } catch (e) {
                console.error(e);
            }
        }
    };

    return (
        <div className="w-full h-48 bg-[#050506] rounded-xl border border-indigo-500/30 overflow-hidden relative shadow-[inset_0_2px_20px_rgba(0,0,0,0.8)] flex flex-col">
            {/* Header / Tabs */}
            <div className="flex bg-[#0f0f11] border-b border-indigo-500/20 px-4">
                {['Summit', 'Eruption', 'Nebula'].map(t => (
                    <button
                        key={t}
                        onClick={() => handleSynthChange(t)}
                        className={`px-4 py-2 text-[10px] font-bold tracking-widest uppercase transition-all ${synthType === t ? 'text-indigo-400 border-b-2 border-indigo-500 bg-indigo-500/5' : 'text-gray-500 hover:text-gray-300'}`}
                    >
                        {t}
                    </button>
                ))}
            </div>

            <div className="flex-1 flex p-4 gap-6">
                {/* Visualizer Area */}
                <div className="flex-1 bg-black/40 rounded-lg border border-indigo-500/10 flex items-end p-2 gap-[1px]">
                    {analyzerData.map((v, i) => (
                        <div
                            key={i}
                            data-height={v}
                            className="synth-lcd-bar"
                        />
                    ))}

                    {/* Floating Info Overlay */}
                    <div className="absolute top-20 left-8 text-indigo-400 font-mono text-[10px] opacity-70">
                        SR: 44100Hz<br />
                        VOICES: 8<br />
                        OSC: ACTIVE
                    </div>
                </div>

                {/* ADSR Control Panel */}
                <div className="w-56 flex flex-col gap-3">
                    <div className="bg-indigo-500/5 rounded border border-indigo-500/10 p-2 flex flex-col mb-1">
                        <span className="text-[8px] text-indigo-300/60 uppercase font-bold tracking-tighter">Envelope ADSR</span>
                    </div>

                    <div className="flex justify-between items-center gap-1 bg-black/20 p-2 rounded-lg border border-white/5 shadow-inner">
                        <Knob
                            label="ATK"
                            value={synthParams.attack}
                            min={0.001} max={2.0} step={0.01} size={30}
                            onChange={(v) => setSynthParams({ attack: v })}
                        />
                        <Knob
                            label="DCY"
                            value={synthParams.decay}
                            min={0.001} max={2.0} step={0.01} size={30}
                            onChange={(v) => setSynthParams({ decay: v })}
                        />
                        <Knob
                            label="SUS"
                            value={synthParams.sustain}
                            min={0.0} max={1.0} step={0.01} size={30}
                            onChange={(v) => setSynthParams({ sustain: v })}
                        />
                        <Knob
                            label="REL"
                            value={synthParams.release}
                            min={0.001} max={5.0} step={0.01} size={30}
                            onChange={(v) => setSynthParams({ release: v })}
                        />
                    </div>

                    <div className="flex-1 grid grid-cols-2 gap-2 mt-auto">
                        <div className="bg-black/40 rounded flex flex-col items-center justify-center border border-white/5 py-1">
                            <span className="text-[7px] text-gray-500 uppercase tracking-tighter">Polyphony</span>
                            <span className="text-[10px] font-mono text-indigo-400">8 Voice</span>
                        </div>
                        <div className="bg-black/40 rounded flex flex-col items-center justify-center border border-white/5 py-1">
                            <span className="text-[7px] text-gray-500 uppercase tracking-tighter">Mod Source</span>
                            <span className="text-[10px] font-mono text-cyan-400">LFO 1</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Glassmorphic Reflection Overlay */}
            <div className="absolute inset-0 pointer-events-none bg-gradient-to-br from-white/5 to-transparent opaicty-20" />
        </div>
    );
}
