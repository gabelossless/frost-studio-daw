import React, { useRef, useState, useEffect } from 'react';
import { useDawStore } from '../store/useDawStore';
import { Plus, Music, Mic, MousePointer, Scissors, Eraser } from 'lucide-react';
import '../styles/Arrangement.css';

export default function Arrangement() {
    const {
        tracks,
        clips,
        playheadPosition,
        tempo,
        updateClip,
        updateTrack,
        addTrack,
        meters
    } = useDawStore();

    const containerRef = useRef<HTMLDivElement>(null);
    const [currentTool, setCurrentTool] = useState<'pointer' | 'scissors' | 'eraser'>('pointer');
    const [isDragging, setIsDragging] = useState<{ id: string, startX: number, originalStart: number } | null>(null);
    const [isResizing, setIsResizing] = useState<{ id: string, startX: number, originalDuration: number } | null>(null);

    // Pixels per beat
    const beatWidth = 80;
    const totalBeats = 128;

    const handleMouseMove = (e: MouseEvent) => {
        if (isDragging) {
            const deltaX = e.clientX - isDragging.startX;
            const deltaBeats = Math.round(deltaX / beatWidth);
            const newStart = Math.max(0, isDragging.originalStart + deltaBeats);
            updateClip(isDragging.id, { start: newStart });
        } else if (isResizing) {
            const deltaX = e.clientX - isResizing.startX;
            const deltaBeats = Math.round(deltaX / beatWidth);
            const newDuration = Math.max(0.25, isResizing.originalDuration + deltaBeats);
            updateClip(isResizing.id, { duration: newDuration });
        }
    };

    const handleMouseUp = () => {
        setIsDragging(null);
        setIsResizing(null);
    };

    useEffect(() => {
        if (isDragging || isResizing) {
            window.addEventListener('mousemove', handleMouseMove);
            window.addEventListener('mouseup', handleMouseUp);
        }
        return () => {
            window.removeEventListener('mousemove', handleMouseMove);
            window.removeEventListener('mouseup', handleMouseUp);
        };
    }, [isDragging, isResizing]);

    const handleRulerClick = (e: React.MouseEvent) => {
        if (!containerRef.current) return;
        const rect = containerRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left + containerRef.current.scrollLeft;
        const newPos = Math.max(0, x / beatWidth);
        useDawStore.setState({ playheadPosition: newPos });
    };

    return (
        <div className="flex flex-col h-full bg-[#0A0A0B] border-b border-[#27272a] select-none overflow-hidden font-sans">
            {/* Arrangement Top Bar */}
            <div className="h-10 flex items-center px-4 bg-[#18181B] border-b border-[#27272a] justify-between z-30">
                <div className="flex items-center gap-3">
                    <span className="text-[10px] font-bold text-gray-500 uppercase tracking-widest">Arrangement</span>
                    
                    {/* Tool Selector */}
                    <div className="flex items-center bg-[#121214] rounded p-0.5 border border-[#27272a] gap-0.5 ml-2">
                        <button 
                            onClick={() => setCurrentTool('pointer')}
                            className={`p-1 rounded ${currentTool === 'pointer' ? 'bg-indigo-500 text-white' : 'text-gray-400 hover:bg-[#27272a]'}`}
                            title="Pointer Tool (Move)"
                        >
                            <MousePointer size={14} />
                        </button>
                        <button 
                            onClick={() => setCurrentTool('scissors')}
                            className={`p-1 rounded ${currentTool === 'scissors' ? 'bg-indigo-500 text-white' : 'text-gray-400 hover:bg-[#27272a]'}`}
                            title="Scissors Tool (Split)"
                        >
                            <Scissors size={14} />
                        </button>
                        <button 
                            onClick={() => setCurrentTool('eraser')}
                            className={`p-1 rounded ${currentTool === 'eraser' ? 'bg-indigo-500 text-white' : 'text-gray-400 hover:bg-[#27272a]'}`}
                            title="Eraser Tool (Delete)"
                        >
                            <Eraser size={14} />
                        </button>
                    </div>

                    <div className="flex items-center gap-1 border-l border-[#27272a] pl-2 ml-1">
                        <button
                            onClick={() => addTrack({ name: `Inst ${tracks.length + 1}`, color: 'bg-indigo-500', type: 'midi', volume: 0.8, pan: 0, muted: false, soloed: false, armed: false, send_1_amount: 0, send_2_amount: 0, eq_low: { freq_hz: 100, gain_db: 0, q: 0.707 }, eq_mid: { freq_hz: 1000, gain_db: 0, q: 0.707 }, eq_high: { freq_hz: 8000, gain_db: 0, q: 0.707 } })}
                            className="p-1.5 rounded hover:bg-[#27272a] text-gray-400 hover:text-white transition-colors flex items-center gap-1"
                            title="Add Instrument Track"
                        >
                            <Plus size={14} />
                            <span className="text-[9px] font-bold">INSTRUMENT</span>
                        </button>
                        <button
                            onClick={() => addTrack({ name: `Audio ${tracks.length + 1}`, color: 'bg-emerald-500', type: 'audio', volume: 0.8, pan: 0, muted: false, soloed: false, armed: false, send_1_amount: 0, send_2_amount: 0, eq_low: { freq_hz: 100, gain_db: 0, q: 0.707 }, eq_mid: { freq_hz: 1000, gain_db: 0, q: 0.707 }, eq_high: { freq_hz: 8000, gain_db: 0, q: 0.707 } })}
                            className="p-1.5 rounded hover:bg-[#27272a] text-gray-400 hover:text-white transition-colors flex items-center gap-1"
                            title="Add Audio Track"
                        >
                            <Plus size={14} />
                            <span className="text-[9px] font-bold">AUDIO</span>
                        </button>
                    </div>
                </div>
                <div className="flex items-center gap-4 text-[11px] font-mono text-indigo-400 bg-black/40 px-3 py-1 rounded border border-indigo-500/20 shadow-inner">
                    <span>{Math.floor(playheadPosition / 4) + 1} . {Math.floor(playheadPosition % 4) + 1} . 1</span>
                    <span className="text-gray-600">|</span>
                    <span>{tempo} BPM</span>
                </div>
            </div>

            <div className="flex-1 flex overflow-hidden">
                {/* Track Headers */}
                <div className="w-64 flex flex-col border-r border-[#27272a] bg-[#121214] z-20 shadow-[10px_0_30px_rgba(0,0,0,0.5)]">
                    <div className="h-8 border-b border-[#27272a] flex items-center px-4 bg-[#18181B]/50">
                        <span className="text-[9px] text-gray-500 font-bold uppercase tracking-tighter">Tracks</span>
                    </div>
                    <div className="flex-1 overflow-y-auto overflow-x-hidden custom-scrollbar">
                        {tracks.map((track) => (
                            <div
                                key={track.id}
                                className="h-20 border-b border-[#27272a] p-3 flex flex-col justify-between hover:bg-white/[0.02] transition-colors group relative"
                            >
                                <div className={`absolute left-0 top-0 bottom-0 w-1 ${track.color}`} />
                                <div className="flex items-center justify-between">
                                    <span className="text-xs font-semibold text-gray-300 group-hover:text-white truncate pr-2">{track.name}</span>
                                    <div className="flex gap-1 opacity-40 group-hover:opacity-100 transition-opacity">
                                        <button
                                            onClick={() => updateTrack(track.id, { muted: !track.muted })}
                                            title="Mute"
                                            className={`w-5 h-5 rounded flex items-center justify-center text-[9px] font-bold ${track.muted ? 'bg-red-500/80 text-white' : 'bg-[#27272a] text-gray-400 hover:bg-[#3f3f46]'}`}
                                        >
                                            M
                                        </button>
                                        <button
                                            onClick={() => updateTrack(track.id, { soloed: !track.soloed })}
                                            title="Solo"
                                            className={`w-5 h-5 rounded flex items-center justify-center text-[9px] font-bold ${track.soloed ? 'bg-yellow-500/80 text-black' : 'bg-[#27272a] text-gray-400 hover:bg-[#3f3f46]'}`}
                                        >
                                            S
                                        </button>
                                    </div>
                                </div>
                                <div className="flex items-center gap-3">
                                    <div className="flex-1 h-1.5 bg-black/60 rounded-full overflow-hidden border border-white/5">
                                        {(() => {
                                            const trackIndex = tracks.findIndex(t => t.id === track.id);
                                            const trackMeter = meters[trackIndex + 1] || { rms_left: 0, rms_right: 0 };
                                            const rms = (trackMeter.rms_left + trackMeter.rms_right) / 2;
                                            const fillWidth = Math.min(100, Math.max(0, rms * 180));
                                            return (
                                                <div 
                                                    className="h-full bg-emerald-400 rounded-full transition-all duration-75 shadow-[0_0_4px_#34d399]" 
                                                    style={{ width: `${fillWidth}%` }} 
                                                />
                                            );
                                        })()}
                                    </div>
                                    <div className="text-gray-600 group-hover:text-gray-400 transition-colors">
                                        {track.type === 'midi' ? <Music size={12} /> : <Mic size={12} />}
                                    </div>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Timeline Grid */}
                <div className="flex-1 overflow-x-auto relative custom-scrollbar group/timeline bg-[#050506]" ref={containerRef}>
                    {/* Time Ruler */}
                    <div
                        className="h-8 sticky top-0 bg-[#121214]/95 backdrop-blur-md border-b border-[#27272a] z-40 flex text-[9px] font-mono text-gray-500 shadow-sm cursor-pointer"
                        onClick={handleRulerClick}
                    >
                        {Array.from({ length: totalBeats }).map((_, i) => (
                            <div
                                key={i}
                                className="arrangement-ruler-beat"
                                style={{ '--beat-width': `${beatWidth}px` } as React.CSSProperties}
                            >
                                <span className="pl-1.5 pointer-events-none">
                                    {i % 4 === 0 ? <span className="text-gray-400 font-bold">{(i / 4) + 1}</span> : `${(i / 4) + 1}.${(i % 4) + 1}`}
                                </span>
                            </div>
                        ))}
                    </div>

                    {/* Timeline Interaction Layer */}
                    <div
                        className="arrangement-grid-container"
                        style={{
                            '--timeline-width': `${totalBeats * beatWidth}px`,
                            '--timeline-height': `${tracks.length * 80}px`
                        } as React.CSSProperties}
                    >
                        {/* Playhead */}
                        <div
                            className="arrangement-playhead"
                            style={{ '--playhead-left': `${playheadPosition * beatWidth}px` } as React.CSSProperties}
                        >
                            <div className="arrangement-playhead-cap" />
                        </div>

                        {/* Visual Grid Lines */}
                        <div className="arrangement-grid-vertical-lines">
                            {Array.from({ length: totalBeats }).map((_, i) => (
                                <div
                                    key={i}
                                    className={`arrangement-grid-line ${i % 4 === 0 ? 'arrangement-grid-line-bar' : ''}`}
                                    style={{ '--beat-width': `${beatWidth}px` } as React.CSSProperties}
                                />
                            ))}
                        </div>

                        {/* Tracks Lane */}
                        <div className="flex flex-col">
                            {tracks.map((track) => (
                                <div key={track.id} className="arrangement-lane">
                                    {/* Clips on this track */}
                                    {clips.filter(c => c.trackId === track.id).map(clip => (
                                        <div
                                            key={clip.id}
                                            className={`arrangement-clip ${clip.color} ${currentTool === 'scissors' ? 'cursor-cell' : currentTool === 'eraser' ? 'cursor-not-allowed' : 'cursor-move'}`}
                                            onDoubleClick={(e) => {
                                                e.stopPropagation();
                                                const { setSelectedClip, setActiveDetailTab } = useDawStore.getState();
                                                setSelectedClip(clip.id);
                                                if (track.type === 'audio') {
                                                    setActiveDetailTab('audio');
                                                } else {
                                                    setActiveDetailTab('piano-roll');
                                                }
                                            }}
                                            onMouseDown={(e) => {
                                                e.stopPropagation();
                                                useDawStore.getState().setSelectedClip(clip.id); // Also select on single click
                                                if (currentTool === 'pointer') {
                                                    setIsDragging({ id: clip.id, startX: e.clientX, originalStart: clip.start });
                                                } else if (currentTool === 'eraser') {
                                                    useDawStore.getState().deleteClip(clip.id);
                                                } else if (currentTool === 'scissors') {
                                                    const rect = e.currentTarget.getBoundingClientRect();
                                                    const x = e.clientX - rect.left;
                                                    const beatOffset = x / 80; // beatWidth = 80
                                                    const absoluteBeat = clip.start + beatOffset;
                                                    const snappedBeat = Math.round(absoluteBeat / 0.25) * 0.25;
                                                    if (snappedBeat > clip.start && snappedBeat < clip.start + clip.duration) {
                                                        const leftDur = snappedBeat - clip.start;
                                                        const rightDur = clip.duration - leftDur;
                                                        updateClip(clip.id, { duration: leftDur });
                                                        useDawStore.getState().addClip({
                                                            ...clip,
                                                            start: snappedBeat,
                                                            duration: rightDur,
                                                            name: `${clip.name} (Split)`
                                                        });
                                                    }
                                                }
                                            }}
                                            style={{
                                                '--clip-left': `${clip.start * beatWidth}px`,
                                                '--clip-width': `${clip.duration * beatWidth}px`
                                            } as React.CSSProperties}
                                        >
                                            <div className="bg-black/30 px-2 py-1 flex items-center justify-between border-b border-white/10 pointer-events-none">
                                                <span className="text-[10px] font-bold text-white truncate drop-shadow-sm uppercase tracking-tighter">{clip.name}</span>
                                            </div>
                                            <div className="flex-1 opacity-40 p-1 flex items-center justify-center pointer-events-none">
                                                {track.type === 'audio' ? (
                                                    <div className="w-full h-full flex items-center gap-[1px] px-2 overflow-hidden">
                                                        {Array.from({ length: 48 }).map((_, i) => (
                                                            <div 
                                                                key={i} 
                                                                className="flex-1 bg-white/40" 
                                                                style={{ 
                                                                    height: `${Math.abs(Math.sin(i * 0.4 + clip.start)) * 80 + 10}%`,
                                                                    opacity: (i % 3 === 0) ? 0.8 : 0.4
                                                                }} 
                                                            />
                                                        ))}
                                                    </div>
                                                ) : (
                                                    /* Visual MIDI representation */
                                                    <div className="w-full h-full flex flex-col justify-around px-1">
                                                        {[1, 2, 3, 4, 5].map(i => (
                                                            <div
                                                                key={i}
                                                                className="arrangement-midi-note-bar"
                                                                style={{
                                                                    '--midi-bar-width': `${(Math.sin(i * 1.5 + clip.start) * 0.3 + 0.5) * 100}%`,
                                                                    '--midi-bar-margin': `${Math.abs(Math.cos(i * 2.2 + clip.start)) * 20}%`
                                                                } as React.CSSProperties}
                                                            />
                                                        ))}
                                                    </div>
                                                )}
                                            </div>

                                            {/* Resize handles */}
                                            <div
                                                className="absolute right-0 top-0 bottom-0 w-2 cursor-ew-resize hover:bg-white/40 z-10"
                                                onMouseDown={(e) => {
                                                    e.stopPropagation();
                                                    setIsResizing({ id: clip.id, startX: e.clientX, originalDuration: clip.duration });
                                                }}
                                            />
                                        </div>
                                    ))}
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
