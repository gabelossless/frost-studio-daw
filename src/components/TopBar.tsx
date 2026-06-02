import { useEffect, useRef } from 'react';
import { Play, Square, Circle, Activity, Maximize2, Plus } from 'lucide-react';
import { useDawStore } from '../store/useDawStore';
import '../styles/TopBar.css';

export default function TopBar() {
    const {
        isPlaying,
        isRecording,
        metronomeEnabled,
        tempo,
        timeSignature,
        playheadPosition,
        meters,
        togglePlay,
        stop,
        toggleRecord,
        toggleMetronome,
        setTempo,
        toggleFullScreenMixer,
        setPluginPickerOpen,
        setActiveDetailTab,
        masterVolume,
        setMasterVolume
    } = useDawStore();

    const canvasRef = useRef<HTMLCanvasElement>(null);
    const historyRef = useRef<number[]>([]);
    const animationRef = useRef<number>(0);

    // Audio Analyzer Data Rolling Loop
    useEffect(() => {
        const draw = () => {
            const canvas = canvasRef.current;
            if (!canvas) {
                animationRef.current = requestAnimationFrame(draw);
                return;
            }
            const ctx = canvas.getContext('2d');
            if (!ctx) return;

            // Fetch Master Level (Channel 0 usually)
            const master = meters[0] || { rms_left: 0, rms_right: 0 };
            const rms = (master.rms_left + master.rms_right) / 2;

            // Push to rolling history
            historyRef.current.push(rms);
            if (historyRef.current.length > 80) {
                historyRef.current.shift();
            }

            // Draw Oscilloscope Line
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            ctx.lineWidth = 2;
            ctx.strokeStyle = '#10b981'; // emerald-500
            ctx.shadowColor = '#10b981';
            ctx.shadowBlur = 4;
            ctx.beginPath();

            const sliceWidth = canvas.width / 80;
            let x = 0;

            for (let i = 0; i < historyRef.current.length; i++) {
                // Scale value to fit canvas height
                const val = historyRef.current[i];
                // Smoothly clamped height multiplier
                const amp = Math.min(1.0, val * 3.0); 
                const y = (canvas.height / 2) - (amp * canvas.height / 2) + (Math.random() * 2 - 1); // add micro jitter for vibe

                if (i === 0) {
                    ctx.moveTo(x, y);
                } else {
                    ctx.lineTo(x, y);
                }
                x += sliceWidth;
            }
            ctx.stroke();

            // Draw a subtle gradient glow underneath
            ctx.shadowBlur = 0; // reset
            ctx.lineTo(canvas.width, canvas.height);
            ctx.lineTo(0, canvas.height);
            ctx.closePath();
            const grad = ctx.createLinearGradient(0, 0, 0, canvas.height);
            grad.addColorStop(0, 'rgba(16, 185, 129, 0.1)');
            grad.addColorStop(1, 'rgba(16, 185, 129, 0)');
            ctx.fillStyle = grad;
            ctx.fill();

            animationRef.current = requestAnimationFrame(draw);
        };
        animationRef.current = requestAnimationFrame(draw);
        return () => cancelAnimationFrame(animationRef.current);
    }, [meters]);

    return (
        <div className="h-14 bg-[#1f1f22]/80 border-b border-[#27272a] flex items-center px-4 justify-between z-10 shrink-0 backdrop-blur-xl">
            <div className="flex items-center gap-3">
                <div className="p-1.5 bg-gradient-to-br from-indigo-500/10 to-cyan-500/10 rounded-lg border border-white/5 shadow-lg overflow-hidden">
                    <img src="/src/assets/snowflake.png" alt="Frost" className="w-5 h-5 object-contain" />
                </div>
                <div className="flex flex-col">
                    <span className="text-xs font-black text-white tracking-[0.2em]">FROST STUDIO</span>
                    <span className="text-[8px] text-indigo-400 uppercase font-bold tracking-tighter opacity-70">Digital Audio Workstation</span>
                </div>
            </div>

            {/* Transport Controls - Absolute Centered */}
            <div className="absolute left-1/2 -translate-x-1/2 flex items-center gap-1.5 z-20">
                <button
                    onClick={stop}
                    title="Stop"
                    className="w-9 h-9 flex items-center justify-center rounded-lg premium-button text-gray-400 hover:text-white transition-all group"
                >
                    <Square fill="currentColor" size={12} className="group-hover:scale-95 transition-transform" />
                </button>
                <button
                    onClick={togglePlay}
                    title={isPlaying ? "Pause" : "Play"}
                    className={`w-11 h-9 flex items-center justify-center rounded-lg transition-all ${isPlaying ? 'bg-green-500 text-white shadow-[0_0_15px_rgba(34,197,94,0.4)] scale-95' : 'premium-button text-gray-400 hover:text-white'}`}
                >
                    <Play fill="currentColor" size={16} />
                </button>
                <button
                    onClick={toggleRecord}
                    title={isRecording ? "Stop Recording" : "Record"}
                    className={`w-9 h-9 flex items-center justify-center rounded-lg transition-all ${isRecording ? 'bg-red-500 text-white shadow-[0_0_15px_rgba(239,68,68,0.4)] scale-95 animate-pulse' : 'premium-button text-gray-400 hover:text-red-400'}`}
                >
                    <Circle fill="currentColor" size={12} />
                </button>
                <div className="w-px h-5 bg-[#27272a] mx-1"></div>
                
                <button
                    onClick={toggleMetronome}
                    title="Toggle Metronome"
                    className={`p-1.5 rounded-lg transition-colors ${metronomeEnabled ? 'text-indigo-400 bg-indigo-500/10' : 'text-gray-500 hover:text-gray-300 premium-button'}`}
                >
                    <Activity size={16} />
                </button>

                <button
                    onClick={() => {
                        setActiveDetailTab('plugin-insert');
                        setPluginPickerOpen(true);
                    }}
                    title="Insert Plugin"
                    className="w-9 h-9 flex items-center justify-center rounded-lg premium-button text-blue-400 hover:text-white transition-all hover:bg-blue-500/10"
                >
                    <Plus size={16} />
                </button>
            </div>

            {/* Right Controls containing LCD Dashboard and Master */}
            <div className="flex items-center gap-3 z-20">
                {/* Central Display Panel - Unified Premium Dashboard */}
                <div className="flex items-center h-10 lcd-container rounded-lg border border-[#232326] px-3 gap-3 min-w-[320px] scale-95 origin-right">
                    <div className="lcd-grid" />
                    <canvas ref={canvasRef} width={320} height={40} className="absolute inset-0 w-full h-full opacity-30 pointer-events-none z-10" />

                    {/* 1. Position Readout */}
                    <div className="flex items-baseline gap-1 font-mono text-emerald-400 z-10 glow-emerald select-none">
                        <span className="text-base font-black tracking-tight">
                            {(Math.floor(playheadPosition / 4) + 1).toString().padStart(3, '0')}
                        </span>
                        <span className="text-xs text-emerald-500/40">.</span>
                        <span className="text-base font-black tracking-tight">
                            {((Math.floor(playheadPosition) % 4) + 1)}
                        </span>
                    </div>

                    <div className="w-px h-5 bg-[#242426] z-10"></div>

                    {/* 2. Tempo & Signature */}
                    <div className="flex items-center gap-2 z-10">
                        <div className="flex flex-col select-none">
                            <span className="text-[6px] text-gray-500 uppercase font-black tracking-wider">BPM</span>
                            <input
                                type="number"
                                value={tempo}
                                onChange={(e) => setTempo(Number(e.target.value))}
                                title="Project Tempo (BPM)"
                                className="w-10 bg-transparent text-emerald-400 font-bold font-mono text-xs focus:outline-none p-0 glow-emerald [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                            />
                        </div>
                        <div className="flex flex-col select-none border-l border-[#242426] pl-2">
                            <span className="text-[6px] text-gray-500 uppercase font-black tracking-wider">SIGN</span>
                            <span className="text-emerald-300 font-bold font-mono text-xs">{timeSignature[0]}/{timeSignature[1]}</span>
                        </div>
                    </div>

                    <div className="w-px h-5 bg-[#242426] z-10"></div>

                    {/* 3. Micro Linear Peak Meters */}
                    <div className="flex flex-col flex-1 gap-1 z-10 justify-center translate-y-[1px] min-w-[80px]">
                        <div className="flex items-center gap-1">
                            <span className="text-[5px] text-gray-600 font-black">L</span>
                            <div className="lcd-meter-bar flex-1 h-1">
                                <div className="lcd-meter-fill" style={{ width: `${Math.min(100, Math.max(0, (meters[0]?.rms_left || 0) * 100))}%` }} />
                            </div>
                        </div>
                        <div className="flex items-center gap-1">
                            <span className="text-[5px] text-gray-600 font-black">R</span>
                            <div className="lcd-meter-bar flex-1 h-1">
                                <div className="lcd-meter-fill" style={{ width: `${Math.min(100, Math.max(0, (meters[0]?.rms_right || 0) * 100))}%` }} />
                            </div>
                        </div>
                    </div>
                </div>

                {/* Master Volume Slider */}
                <div className="flex items-center gap-2 bg-[#18181b]/30 border border-[#27272a]/80 rounded-lg px-2.5 py-1 shadow-inner">
                    <span className="text-[8px] text-gray-500 uppercase font-black tracking-wider">MSTR</span>
                    <input
                        type="range"
                        min="0"
                        max="1"
                        step="0.01"
                        title="Master Volume"
                        value={masterVolume}
                        onChange={(e) => setMasterVolume(parseFloat(e.target.value))}
                        className="w-16 h-1 bg-[#27272a] rounded-lg appearance-none cursor-pointer accent-emerald-500 hover:accent-emerald-400 transition-all"
                    />
                </div>

                <button
                    onClick={toggleFullScreenMixer}
                    className="flex items-center gap-2 px-3 h-8 premium-button text-gray-300 hover:text-indigo-300 rounded-lg text-xs font-semibold transition-all group"
                >
                    <Maximize2 size={12} className="group-hover:scale-110 transition-transform text-indigo-400" /> Mixer
                </button>
            </div>

        </div>
    );
}
