import { useDawStore } from '../store/useDawStore';
import SynthLCD from './SynthLCD';
import PresetBrowser from './PresetBrowser';
import { Music, Zap, Layers, Cpu, ChevronDown } from 'lucide-react';
import { useState, useEffect, useRef } from 'react';

function OscillatorVisualizer({ color }: { color: string }) {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        let frame = 0;
        let animationId: number;

        const draw = () => {
            frame++;
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            ctx.beginPath();
            ctx.strokeStyle = color;
            ctx.lineWidth = 2;
            ctx.lineCap = 'round';
            ctx.lineJoin = 'round';

            const width = canvas.width;
            const height = canvas.height;
            const midY = height / 2;
            const amplitude = height * 0.3;
            const frequency = 0.05;

            for (let x = 0; x < width; x++) {
                // Sine wave + subtle saw tooth for texture
                const sine = Math.sin(x * frequency + frame * 0.1);
                const saw = (x * 0.1 + frame * 0.05) % 1;
                const y = midY + (sine * 0.7 + (saw - 0.5) * 0.3) * amplitude;

                if (x === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
            }
            ctx.stroke();

            // Glow effect
            ctx.shadowBlur = 10;
            ctx.shadowColor = color;
            ctx.stroke();

            animationId = requestAnimationFrame(draw);
        };

        draw();
        return () => cancelAnimationFrame(animationId);
    }, [color]);

    return <canvas ref={canvasRef} width={200} height={60} className="w-full h-full opacity-80" />;
}

export default function InstrumentView() {
    const { tracks, selectedTrackId, setSynthType } = useDawStore();
    const selectedTrack = tracks.find(t => t.id === selectedTrackId);
    const [isSelectingSynth, setIsSelectingSynth] = useState(false);

    if (!selectedTrack) {
        return (
            <div className="flex items-center justify-center h-full text-gray-600 italic Frost-Studio bg-[#050506]">
                Select a track to view instruments...
            </div>
        );
    }

    const synthTypes = [
        { id: 'Summit', name: 'SUMMIT', desc: 'Hybrid Wavetable Engine', color: 'text-indigo-400' },
        { id: 'Eruption', name: 'ERUPTION', desc: 'Virtual Analog Power', color: 'text-orange-400' },
        { id: 'Nebula', name: 'NEBULA', desc: 'Phase Modulation Space', color: 'text-cyan-400' },
        { id: 'Sampler', name: 'SAMPLER', desc: 'Native Sample Workflow', color: 'text-emerald-400' },
    ];

    const currentSynth = synthTypes.find(s => s.id === (selectedTrack.type === 'midi' ? 'Summit' : 'Summit')) || synthTypes[0];

    return (
        <div className="flex h-full bg-[#050506] border-t border-white/5 Frost-Studio select-none overflow-hidden">
            <div className="flex-1 flex flex-col min-w-0">
                <div className="flex-1 flex p-8 gap-12 overflow-y-auto custom-scrollbar">
                    {/* Left Column: Synth & Main Visuals */}
                    <div className="flex-[2] flex flex-col gap-8 min-w-0">
                        <div className="flex justify-between items-start">
                            <div className="min-w-0">
                                <h2 className="text-2xl font-bold text-gray-200 uppercase tracking-tighter truncate">
                                    {selectedTrack.name}
                                </h2>
                                <div className="relative mt-2">
                                    <button
                                        onClick={() => setIsSelectingSynth(!isSelectingSynth)}
                                        className="flex items-center gap-2 bg-indigo-500/10 hover:bg-indigo-500/20 border border-indigo-500/30 px-3 py-1.5 rounded-lg transition-all group shrink-0"
                                    >
                                        <Cpu size={14} className="text-indigo-400" />
                                        <span className={`text-[11px] font-bold uppercase tracking-widest ${currentSynth.color}`}>{currentSynth.name}</span>
                                        <span className="text-[9px] text-gray-500 ml-2 truncate">— {currentSynth.desc}</span>
                                        <ChevronDown size={12} className={`ml-2 text-gray-500 transition-transform ${isSelectingSynth ? 'rotate-180' : ''}`} />
                                    </button>

                                    {isSelectingSynth && (
                                        <div className="absolute top-full left-0 mt-2 w-64 bg-[#18181B] border border-white/10 rounded-xl shadow-2xl z-50 py-2 overflow-hidden backdrop-blur-xl">
                                            {synthTypes.map((s) => (
                                                <button
                                                    key={s.id}
                                                    onClick={() => { setSynthType(tracks.indexOf(selectedTrack), s.id); setIsSelectingSynth(false); }}
                                                    className="w-full flex flex-col items-start px-4 py-3 hover:bg-white/5 text-left border-b border-white/[0.03] last:border-0"
                                                >
                                                    <span className={`text-xs font-bold ${s.color}`}>{s.name}</span>
                                                    <span className="text-[9px] text-gray-500 uppercase tracking-tight">{s.desc}</span>
                                                </button>
                                            ))}
                                        </div>
                                    )}
                                </div>
                            </div>
                            <div className="flex flex-col items-end gap-1 shrink-0">
                                <span className="text-[10px] text-gray-600 font-mono tracking-widest">ENGINE_CORE_v2.4</span>
                                <div className="flex gap-1">
                                    {[1, 2, 3, 4].map(i => <div key={i} className={`w-1 h-3 rounded-full ${i <= 3 ? 'bg-indigo-500' : 'bg-white/10'}`} />)}
                                </div>
                            </div>
                        </div>

                        <SynthLCD />

                        {/* Synth Details Grid */}
                        <div className="grid grid-cols-2 gap-6">
                            <div className="glass-panel p-6 flex flex-col gap-4 group hover:border-indigo-500/20 transition-all cursor-pointer relative overflow-hidden">
                                <div className="absolute inset-0 bg-gradient-to-br from-indigo-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                                <div className="flex items-center gap-2 text-indigo-400 font-bold uppercase text-[10px] tracking-widest relative z-10">
                                    <Zap size={14} /> Modulation Sources
                                </div>
                                <div className="flex justify-between items-center text-sm text-gray-400 relative z-10">
                                    <span className="text-[11px] font-bold uppercase tracking-tighter">LFO 1 Rate</span>
                                    <span className="font-mono text-xs text-indigo-300">0.5 Hz</span>
                                </div>
                                <div className="w-full h-1 bg-black/50 rounded-full overflow-hidden shadow-inner relative z-10">
                                    <div className="h-full bg-indigo-500 w-1/2 shadow-[0_0_12px_rgba(99,102,241,0.8)]" />
                                </div>
                                <div className="flex justify-between items-center text-sm text-gray-400 relative z-10 mt-auto">
                                    <span className="text-[11px] font-bold uppercase tracking-tighter">LFO 2 Shape</span>
                                    <span className="font-mono text-xs text-indigo-300 opacity-60">Sine</span>
                                </div>
                            </div>

                            <div className="glass-panel p-6 flex flex-col gap-4 group hover:border-cyan-500/20 transition-all cursor-pointer relative overflow-hidden">
                                <div className="absolute inset-0 bg-gradient-to-br from-cyan-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity" />
                                <div className="flex items-center gap-2 text-cyan-400 font-bold uppercase text-[10px] tracking-widest relative z-10">
                                    <Layers size={14} /> Oscillation Mix
                                </div>

                                {/* Animated Waveform Visualizer */}
                                <div className="w-full h-16 bg-black/60 rounded-lg border border-white/5 shadow-inner flex items-center justify-center overflow-hidden relative z-10">
                                    {selectedTrack.id === 'Sampler' ? (
                                        <div className="text-[10px] text-emerald-400 font-mono text-center px-4 uppercase italic opacity-60">
                                            Sample Data Loaded
                                        </div>
                                    ) : (
                                        <OscillatorVisualizer color="var(--accent-cyan)" />
                                    )}
                                </div>

                                <div className="flex justify-between items-center text-sm text-gray-400 relative z-10 mt-2">
                                    <span className="text-[11px] font-bold uppercase tracking-tighter">
                                        {selectedTrack.id === 'Sampler' ? 'Sample Gain' : 'Osc 1 (Saw)'}
                                    </span>
                                    <span className="font-mono text-xs text-cyan-300">75%</span>
                                </div>
                                <div className="w-full h-1 bg-black/50 rounded-full overflow-hidden shadow-inner relative z-10">
                                    <div className={`h-full ${selectedTrack.id === 'Sampler' ? 'bg-emerald-500' : 'bg-cyan-500'} w-3/4 shadow-[0_0_12px_rgba(34,211,238,0.8)]`} />
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Right Column: Signal Chain & Global Effects */}
                    <div className="flex-1 flex flex-col gap-6 min-w-[280px]">
                        <div className="bg-indigo-500/5 border border-indigo-500/20 rounded-2xl p-6 flex flex-col gap-6">
                            <h3 className="text-[10px] font-bold uppercase tracking-[0.3em] text-indigo-300 border-b border-indigo-500/20 pb-4">Signal Chain</h3>

                            <div className="flex flex-col gap-3">
                                {[
                                    { name: 'Wave Generator', icon: <Music size={14} />, status: 'active' },
                                    { name: 'ADSR Envelope', icon: <Layers size={14} />, status: 'active' },
                                    { name: 'Zero-Delay Filter', icon: <Zap size={14} />, status: 'active' },
                                    { name: 'Master Gain', icon: <Cpu size={14} />, status: 'active' }
                                ].map((s, i) => (
                                    <div key={i} className="flex items-center gap-4 bg-black/40 p-3 rounded-xl border border-white/5 group hover:border-indigo-500/30 transition-all cursor-pointer">
                                        <div className="text-indigo-400 group-hover:scale-110 transition-transform">{s.icon}</div>
                                        <span className="text-[11px] font-bold text-gray-300 uppercase tracking-tighter">{s.name}</span>
                                        <div className="ml-auto w-1.5 h-1.5 rounded-full bg-green-500/50 shadow-[0_0_8px_rgba(34,197,94,0.5)]" />
                                    </div>
                                ))}
                            </div>
                        </div>

                        <div className="flex-1 bg-white/[0.02] border border-white/[0.05] rounded-2xl p-6 flex flex-col items-center justify-center gap-4 text-center group hover:bg-white/[0.04] transition-all cursor-pointer">
                            <div className="w-12 h-12 rounded-full border-2 border-dashed border-white/10 flex items-center justify-center text-white/20 group-hover:border-indigo-500/30 group-hover:text-indigo-500/50 transition-all">
                                +
                            </div>
                            <p className="text-[9px] text-gray-500 uppercase font-black tracking-widest group-hover:text-gray-300 transition-colors">Add Audio Effect</p>
                        </div>
                    </div>
                </div>
            </div>

            <PresetBrowser synthType={currentSynth.id} />
        </div>
    );
}
