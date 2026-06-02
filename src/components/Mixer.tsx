import { useDawStore } from '../store/useDawStore';
import { PeakMeter } from './PeakMeter';
import { Knob } from './Knob';
import { Maximize2, Minimize2, Plus, X } from 'lucide-react';
import PluginEditor from './PluginEditor';
import { useState } from 'react';

export default function Mixer() {
    const {
        tracks,
        updateTrack,
        masterVolume,
        setMasterVolume,
        meters,
        isFullScreenMixer,
        toggleFullScreenMixer,
        masterLimiterParams,
        setMasterLimiterParams,
        addPlugin,
        removePlugin,
        setSelectedTrack,
        setActiveDetailTab,
    } = useDawStore();

    const [editingPlugin, setEditingPlugin] = useState<{ channelId: number, index: number, name: string } | null>(null);
    const [addingToSlot, setAddingToSlot] = useState<{ channelId: number, slot: number } | null>(null);

    const pluginTypes = ['Compressor', 'EQ', 'Limiter', 'Bass', 'Delay', 'Reverb'];

    return (
        <div className={`flex flex-col flex-1 bg-[#0f0f11] overflow-hidden Frost-Studio ${isFullScreenMixer ? 'h-full' : ''}`}>
            {/* Mixer Header */}
            <div className="flex justify-between items-center px-4 py-2 border-b border-[#27272a] bg-[#18181b]">
                <h2 className="text-sm font-semibold tracking-wide text-gray-300">Mixer</h2>
                <button
                    onClick={toggleFullScreenMixer}
                    className="p-1 rounded hover:bg-[#27272a] text-gray-400 hover:text-white transition-colors"
                    title={isFullScreenMixer ? "Restore View" : "Full Screen Mixer"}
                >
                    {isFullScreenMixer ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
                </button>
            </div>

            {/* Mixer Channels Container */}
            <div className="flex-1 flex overflow-x-auto p-2 gap-2 custom-scrollbar items-start relative">
                {tracks.map((track, trackIndex) => {
                    const meter = meters[trackIndex];
                    return (
                        <div
                            key={track.id}
                            className="w-32 bg-[#1e1e20] border border-black/50 rounded-lg flex flex-col items-center flex-shrink-0 shadow-2xl relative overflow-hidden"
                        >
                            {/* Pro Channel Label */}
                            <div className="w-full bg-[#1a1a1a] border-b border-black/60 py-1 mb-1 text-center shadow-inner">
                                <span className="text-[9px] font-bold text-gray-500 uppercase tracking-widest">{track.type}</span>
                            </div>

                            {/* Instrument Slot (MIDI tracks carry synth engines) */}
                            {track.type === 'midi' && (
                                <div className="w-full px-1.5 mb-2">
                                    <div className="text-[7px] text-gray-500 font-bold uppercase tracking-tighter mb-0.5 px-1">Instrument</div>
                                    <button
                                        onClick={() => {
                                            setSelectedTrack(track.id);
                                            setActiveDetailTab('instrument');
                                            if (isFullScreenMixer) toggleFullScreenMixer();
                                        }}
                                        className="w-full h-5 bg-[#1a1a20] border border-indigo-500/20 rounded-sm flex items-center px-1.5 hover:bg-[#202028] hover:border-indigo-500/40 cursor-pointer transition-all border-l-2 border-l-indigo-400 group/inst"
                                        title="Open Instrument Editor"
                                    >
                                        <span className="text-[9px] text-indigo-300 font-bold truncate">Synth</span>
                                        <div className="ml-auto w-1 h-1 rounded-full bg-indigo-400/80 shadow-[0_0_4px_rgba(99,102,241,0.6)] group-hover/inst:bg-indigo-300" />
                                    </button>
                                </div>
                            )}

                            {/* Inserts (FX Chain) - Logic style slots */}
                            <div className="w-full px-1.5 flex flex-col gap-0.5 mb-4">
                                <div className="text-[8px] text-gray-500 font-bold uppercase tracking-tighter mb-1 px-1">Audio FX</div>
                                {[0, 1, 2, 3].map(i => {
                                    const pluginName = track.plugins?.[i];
                                    return (
                                        <div key={i} className="relative">
                                            <div
                                                className={`h-5 ${pluginName ? 'bg-cyan-950/20 border-cyan-500/30' : 'bg-[#1e1e1e] border-black/40'} border rounded-sm flex items-center px-1.5 hover:bg-[#333] cursor-pointer transition-all group/slot border-l-2 ${pluginName ? 'border-l-cyan-400' : 'border-l-cyan-500/20'}`}
                                                onClick={() => {
                                                    if (pluginName) {
                                                        setEditingPlugin({ channelId: trackIndex, index: i, name: pluginName });
                                                    } else {
                                                        setAddingToSlot(addingToSlot?.channelId === trackIndex && addingToSlot?.slot === i ? null : { channelId: trackIndex, slot: i });
                                                    }
                                                }}
                                            >
                                                <span className={`text-[9px] ${pluginName ? 'text-cyan-400 font-bold' : 'text-gray-500 group-hover/slot:text-gray-300 font-medium'} truncate`}>
                                                    {pluginName || 'Empty'}
                                                </span>
                                                
                                                {pluginName ? (
                                                    <button
                                                        onClick={(e) => {
                                                            e.stopPropagation();
                                                            removePlugin(trackIndex, i);
                                                            if (editingPlugin?.channelId === trackIndex && editingPlugin?.index === i) {
                                                                setEditingPlugin(null);
                                                            }
                                                        }}
                                                        className="ml-auto opacity-0 group-hover/slot:opacity-100 p-0.5 hover:bg-black/40 rounded text-red-400 hover:text-red-300 transition-opacity"
                                                        title="Remove Plugin"
                                                    >
                                                        <X size={10} />
                                                    </button>
                                                ) : (
                                                    <Plus size={8} className="ml-auto opacity-0 group-hover/slot:opacity-50 text-cyan-400" />
                                                )}
                                            </div>

                                            {addingToSlot?.channelId === trackIndex && addingToSlot?.slot === i && (
                                                <div className="absolute left-full top-0 ml-1 w-32 bg-[#252528] border border-black/80 rounded shadow-2xl z-50 py-1 overflow-hidden ring-1 ring-white/5">
                                                    {pluginTypes.map(type => (
                                                        <button
                                                            key={type}
                                                            onClick={() => { addPlugin(trackIndex, type); setAddingToSlot(null); }}
                                                            className="w-full px-3 py-1.5 text-[10px] text-gray-300 hover:bg-indigo-500 hover:text-white text-left transition-colors font-medium"
                                                        >
                                                            {type}
                                                        </button>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
                                    );
                                })}
                            </div>

                            {/* I/O Section */}
                            <div className="w-full px-1.5 mb-4 flex flex-col gap-1">
                                <div className="text-[8px] text-gray-500 font-bold uppercase tracking-tighter px-1">Sends</div>
                                <div className="flex gap-1">
                                    <div className="flex-1 h-5 bg-[#1a1a1a] rounded-sm border border-black/40 flex items-center justify-center">
                                        <span className="text-[8px] text-indigo-400/60 font-black">S1</span>
                                    </div>
                                    <div className="flex-1 h-5 bg-[#1a1a1a] rounded-sm border border-black/40 flex items-center justify-center">
                                        <span className="text-[8px] text-cyan-400/60 font-black">S2</span>
                                    </div>
                                </div>
                            </div>

                            {/* Sidechain Section */}
                            <div className="w-full px-1.5 mb-2 flex flex-col gap-1">
                                <div className="text-[8px] text-cyan-400/80 font-bold uppercase tracking-tighter px-1">Sidechain</div>
                                <select
                                    title="Sidechain Source"
                                    value={(track.sidechain_source_id !== undefined && track.sidechain_source_id !== null) ? track.sidechain_source_id : -1}
                                    onChange={(e) => {
                                        const val = parseInt(e.target.value);
                                        updateTrack(track.id, { sidechain_source_id: val === -1 ? null : val });
                                    }}
                                    className="w-full bg-[#1a1a1a] text-[9px] text-gray-300 rounded-sm border border-black/40 py-1 px-1 font-sans focus:outline-none focus:border-cyan-500/50 cursor-pointer"
                                >
                                    <option value="-1">None</option>
                                    {tracks.map((t, tc) => tc !== trackIndex && (
                                        <option key={t.id} value={tc}>{t.name}</option>
                                    ))}
                                </select>
                                {(track.sidechain_source_id !== null && track.sidechain_source_id !== undefined && track.sidechain_source_id >= 0) && (
                                    <div className="flex items-center gap-1 mt-0.5 px-1">
                                        <span className="text-[8px] text-gray-500">Amt</span>
                                        <input
                                            type="range"
                                            title="Sidechain Amount"
                                            min="0" max="1" step="0.01"
                                            value={track.sidechain_ratio ?? 0}
                                            onChange={(e) => updateTrack(track.id, { sidechain_ratio: parseFloat(e.target.value) })}
                                            className="flex-1 h-1 appearance-none bg-black/60 rounded-full [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2 [&::-webkit-slider-thumb]:h-2 [&::-webkit-slider-thumb]:bg-cyan-400 [&::-webkit-slider-thumb]:rounded-full cursor-pointer"
                                        />
                                    </div>
                                )}
                            </div>

                            {/* Pan & Input */}
                            <div className="w-full flex flex-col items-center gap-2 mb-6">
                                <Knob
                                    label=""
                                    value={track.pan}
                                    min={-1} max={1} isPan size={28}
                                    onChange={(v) => updateTrack(track.id, { pan: v })}
                                />
                                <div className="text-[9px] font-mono text-gray-400 bg-black/40 px-2 py-0.5 rounded-full border border-white/5">
                                    {track.pan === 0 ? 'C' : track.pan < 0 ? `L${Math.abs(Math.round(track.pan * 64))}` : `R${Math.round(track.pan * 64)}`}
                                </div>
                            </div>

                            {/* Fader Area */}
                            <div className="flex-1 flex flex-col items-center w-full px-2 min-h-[220px] bg-gradient-to-b from-transparent to-black/10 pb-4">
                                <div className="flex-1 flex gap-2 w-full justify-center pt-2">
                                    {/* Fader with Scale */}
                                    <div className="w-8 relative flex flex-col items-center group">
                                        <div className="absolute inset-y-0 -left-1 flex flex-col justify-between text-[7px] text-gray-600 font-mono py-1 pointer-events-none">
                                            <span>+6</span><span>0</span><span>-6</span><span>-12</span><span>-24</span><span>-48</span><span>-oo</span>
                                        </div>
                                        <input
                                            type="range"
                                            min="0"
                                            max="1.2"
                                            step="0.01"
                                            value={track.volume}
                                            onChange={(e) => updateTrack(track.id, { volume: parseFloat(e.target.value) })}
                                            className="absolute w-[180px] h-8 -rotate-90 origin-center translate-y-[80px] appearance-none bg-transparent cursor-pointer z-10 
                                                [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-6 [&::-webkit-slider-thumb]:h-10 [&::-webkit-slider-thumb]:bg-[#dbdbdb] 
                                                [&::-webkit-slider-thumb]:border-x [&::-webkit-slider-thumb]:border-black/50 [&::-webkit-slider-thumb]:rounded-sm [&::-webkit-slider-thumb]:shadow-[0_4px_8px_rgba(0,0,0,0.6)]
                                                [&::-webkit-slider-thumb]:hover:bg-white [&::-webkit-slider-thumb]:active:scale-95"
                                        />
                                        <div className="w-1 h-full bg-black/80 rounded-full shadow-inner border border-white/5 pointer-events-none mt-2" />
                                    </div>
                                    <PeakMeter level={meter} height={180} />
                                </div>

                                {/* Mute / Solo - Classic Squares */}
                                <div className="flex gap-1 w-full px-1 mt-4">
                                    <button
                                        onClick={() => updateTrack(track.id, { soloed: !track.soloed })}
                                        className={`w-7 h-7 rounded-sm text-[10px] font-black border transition-all ${track.soloed ? 'bg-yellow-500 border-yellow-300 text-black' : 'bg-[#3d3d3d] border-black/40 text-gray-400'}`}
                                    >
                                        S
                                    </button>
                                    <button
                                        onClick={() => updateTrack(track.id, { muted: !track.muted })}
                                        className={`w-7 h-7 rounded-sm text-[10px] font-black border transition-all ${track.muted ? 'bg-red-500 border-red-300 text-white' : 'bg-[#3d3d3d] border-black/40 text-gray-400'}`}
                                    >
                                        M
                                    </button>
                                    <div className="flex-1 flex items-center justify-center bg-[#222] border border-black/60 rounded-sm">
                                        <span className="text-[10px] font-mono text-cyan-400">
                                            {track.volume === 0 ? '-inf' : (20 * Math.log10(track.volume)).toFixed(1)}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            {/* Name Badge */}
                            <div className="w-full bg-[#1a1a1a] border-t border-black/60 py-2 mt-auto">
                                <div className={`mx-auto w-[85%] h-5 bg-[#333] border border-black/80 rounded-md flex items-center justify-center shadow-inner overflow-hidden border-t-${track.color.split('-')[1]}-500/40`}>
                                    <span className="text-[10px] font-bold text-gray-200 truncate px-1 uppercase tracking-tighter">{track.name}</span>
                                </div>
                            </div>
                        </div>
                    );
                })}

                {/* Master Bus - Professional High-Contrast Strip */}
                <div className="w-48 bg-[#252528] border-x border-t border-indigo-500/40 rounded-t-xl flex flex-col items-center pt-1 pb-1 flex-shrink-0 ml-auto shadow-[0_0_50px_rgba(99,102,241,0.15)] relative overflow-hidden">
                    <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-indigo-500 to-transparent opacity-80" />

                    <div className="w-full bg-[#1a1a1a] border-b border-black/60 py-1 mb-2 text-center shadow-inner">
                        <span className="text-[10px] font-black text-indigo-400 uppercase tracking-[0.2em] drop-shadow-[0_0_5px_rgba(99,102,241,0.5)]">Master</span>
                    </div>

                    {/* Master Limiter Section - Logic Style Group */}
                    <div className="w-full px-3 flex flex-col gap-4 mb-4 bg-black/30 py-4 rounded-lg border border-indigo-500/20 mx-2 shadow-inner">
                        <div className="flex justify-between items-center mb-1">
                            <div className="text-[8px] text-indigo-300 font-bold uppercase tracking-tighter">Limiter</div>
                            <div className="w-2 h-2 rounded-full bg-indigo-500 shadow-[0_0_8px_rgba(99,102,241,1)]" />
                        </div>
                        <div className="flex justify-around gap-2">
                            <Knob
                                label="THRSH"
                                value={masterLimiterParams.threshold}
                                min={0.1} max={1.0} step={0.01} size={36}
                                onChange={(v) => setMasterLimiterParams({ threshold: v })}
                            />
                            <Knob
                                label="CEIL"
                                value={masterLimiterParams.ceiling}
                                min={0.5} max={1.0} step={0.01} size={36}
                                onChange={(v) => setMasterLimiterParams({ ceiling: v })}
                            />
                        </div>
                    </div>

                    {/* Master Fader Area */}
                    <div className="flex-1 flex flex-col items-center w-full px-4 min-h-[220px] bg-gradient-to-b from-transparent to-indigo-500/5 pb-4">
                        <div className="flex-1 flex gap-3 w-full justify-center pt-2">
                            {/* Master Fader with Detailed Scale */}
                            <div className="w-10 relative flex flex-col items-center group">
                                <div className="absolute inset-y-0 -left-2 flex flex-col justify-between text-[7px] text-indigo-300/40 font-mono py-1 pointer-events-none">
                                    <span>+6</span><span>0</span><span>-6</span><span>-12</span><span>-24</span><span>-48</span><span>-oo</span>
                                </div>
                                <input
                                    type="range"
                                    min="0"
                                    max="1.5"
                                    step="0.01"
                                    value={masterVolume}
                                    onChange={(e) => setMasterVolume(parseFloat(e.target.value))}
                                    className="absolute w-[180px] h-10 -rotate-90 origin-center translate-y-[80px] appearance-none bg-transparent cursor-pointer z-10 
                                        [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-8 [&::-webkit-slider-thumb]:h-12 [&::-webkit-slider-thumb]:bg-indigo-300 
                                        [&::-webkit-slider-thumb]:border-x-2 [&::-webkit-slider-thumb]:border-black [&::-webkit-slider-thumb]:rounded-sm [&::-webkit-slider-thumb]:shadow-[0_0_20px_rgba(99,102,241,0.6)]
                                        [&::-webkit-slider-thumb]:hover:bg-white [&::-webkit-slider-thumb]:transition-all"
                                />
                                <div className="w-1.5 h-full bg-black/60 rounded-full shadow-inner border border-indigo-500/30 pointer-events-none mt-2" />
                            </div>

                            {/* Large Master Meter */}
                            <PeakMeter level={meters[255]} height={180} width={28} />
                        </div>

                        {/* Master Output Value */}
                        <div className="mt-4 w-full flex items-center justify-center bg-black/60 border border-indigo-500/30 rounded py-1 mb-2">
                            <span className="text-[12px] font-mono text-indigo-400 font-bold">
                                {masterVolume === 0 ? '-inf' : (20 * Math.log10(masterVolume)).toFixed(1)} <span className="text-[8px] opacity-50 ml-0.5">dB</span>
                            </span>
                        </div>
                    </div>

                    <div className="w-full bg-[#1a1a1a] border-t border-black/60 py-2.5 mt-auto">
                        <div className="mx-auto w-[90%] h-6 bg-[#222] border border-indigo-500/40 rounded-md flex items-center justify-center shadow-[inset_0_2px_10px_rgba(0,0,0,0.8)]">
                            <span className="text-[11px] font-black text-indigo-300 uppercase tracking-[0.15em]">Stereo Out</span>
                        </div>
                    </div>
                </div>

                {/* Plugin Editor Overlay */}
                {editingPlugin && (
                    <PluginEditor
                        channelId={editingPlugin.channelId}
                        pluginIndex={editingPlugin.index}
                        pluginName={editingPlugin.name}
                        onClose={() => setEditingPlugin(null)}
                    />
                )}
            </div>

            {/* Global dismiss layer for menus */}
            {(addingToSlot) && (
                <div
                    className="fixed inset-0 z-40"
                    onClick={() => { setAddingToSlot(null); }}
                />
            )}
        </div >
    );
}
