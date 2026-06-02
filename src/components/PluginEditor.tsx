import { useDawStore } from '../store/useDawStore';
import { Knob } from './Knob';
import { X, Activity } from 'lucide-react';

interface PluginParam {
    id: number;
    name: string;
    min: number;
    max: number;
    unit: string;
    step: number;
}

export default function PluginEditor({ channelId, pluginIndex, pluginName, onClose }: {
    channelId: number,
    pluginIndex: number,
    pluginName: string,
    onClose: () => void
}) {
    const { setPluginParam } = useDawStore();

    // Map of plugin parameters (normally fetched from Rust)
    const getParams = (name: string): PluginParam[] => {
        if (name.includes('Compressor')) return [
            { id: 0, name: 'THRESHOLD', min: 0.001, max: 1.0, unit: '', step: 0.01 },
            { id: 1, name: 'RATIO', min: 1.0, max: 20.0, unit: ':1', step: 1.0 },
            { id: 2, name: 'ATTACK', min: 0.0001, max: 1.0, unit: 's', step: 0.01 },
            { id: 3, name: 'RELEASE', min: 0.001, max: 5.0, unit: 's', step: 0.01 },
            { id: 4, name: 'MAKEUP', min: 0.0, max: 10.0, unit: 'x', step: 0.1 },
        ];
        if (name.includes('EQ')) return [
            { id: 0, name: 'FREQ 1', min: 20, max: 20000, unit: 'Hz', step: 1 },
            { id: 1, name: 'GAIN 1', min: -24, max: 24, unit: 'dB', step: 1 },
            { id: 3, name: 'FREQ 2', min: 20, max: 20000, unit: 'Hz', step: 1 },
            { id: 4, name: 'GAIN 2', min: -24, max: 24, unit: 'dB', step: 1 },
        ];
        return [];
    };

    const params = getParams(pluginName);

    return (
        <div className="absolute inset-x-8 top-1/2 -translate-y-1/2 bg-[#18181B] border border-indigo-500/30 rounded-2xl shadow-[0_0_60px_rgba(0,0,0,0.8)] z-50 overflow-hidden flex flex-col Frost-Studio h-[300px]">
            <div className="flex items-center justify-between px-6 py-4 bg-indigo-500/10 border-b border-indigo-500/20">
                <div className="flex items-center gap-3">
                    <Activity size={18} className="text-indigo-400" />
                    <div>
                        <h3 className="text-xs font-black text-white uppercase tracking-[0.2em]">{pluginName}</h3>
                        <p className="text-[9px] text-indigo-300/50 uppercase font-bold tracking-widest">FROST_NATIVE_CORE_V1</p>
                    </div>
                </div>
                <button onClick={onClose} className="p-2 hover:bg-white/5 rounded-full text-gray-500 hover:text-white transition-all" title="Close Color">
                    <X size={18} />
                </button>
            </div>

            <div className="flex-1 flex items-center justify-center gap-12 p-8">
                {params.map((p) => (
                    <div key={p.id} className="flex flex-col items-center gap-4">
                        <Knob
                            label={p.name}
                            min={p.min}
                            max={p.max}
                            step={p.step}
                            value={params.find(x => x.id === p.id)?.min || 0} // Placeholder for current value
                            onChange={(v) => setPluginParam(channelId, pluginIndex, p.id, v)}
                            size={56}
                        />
                        <div className="flex flex-col items-center">
                            <span className="text-[10px] text-gray-400 font-mono tracking-tighter">
                                {(params.find(x => x.id === p.id)?.min || 0).toFixed(2)}{p.unit}
                            </span>
                        </div>
                    </div>
                ))}
            </div>

            <div className="px-6 py-3 bg-black/40 border-t border-white/5 flex justify-between items-center">
                <div className="flex gap-2">
                    <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse shadow-[0_0_8px_rgba(34,197,94,0.5)]" />
                    <span className="text-[8px] text-gray-500 font-bold uppercase tracking-widest">Processor Active</span>
                </div>
                <span className="text-[8px] text-gray-600 font-mono">LATENCY: 0.0ms</span>
            </div>
        </div>
    );
}
