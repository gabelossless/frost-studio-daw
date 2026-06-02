import { Search, Music, Zap, Layers, Wind, Bell } from 'lucide-react';
import { useState, useEffect } from 'react';

export default function PresetBrowser({ synthType }: { synthType: string }) {
    const [presets, setPresets] = useState<any[]>([]);
    const [search, setSearch] = useState('');

    useEffect(() => {
        // Mocking the fetch since we don't have a state list for presets yet
        import('@tauri-apps/api/core').then(({ invoke }) => {
            invoke<any[]>('get_synth_presets', { synthType }).then(setPresets);
        });
    }, [synthType]);

    const filtered = presets.filter(p =>
        p.name.toLowerCase().includes(search.toLowerCase()) ||
        p.category.toLowerCase().includes(search.toLowerCase())
    );

    const getCategoryIcon = (cat: string) => {
        switch (cat.toLowerCase()) {
            case 'lead': return <Zap size={14} className="text-orange-400" />;
            case 'pad': return <Layers size={14} className="text-indigo-400" />;
            case 'bass': return <Wind size={14} className="text-cyan-400" />;
            case 'pluck': return <Music size={14} className="text-green-400" />;
            case 'keys': return <Bell size={14} className="text-purple-400" />;
            default: return <Music size={14} className="text-gray-500" />;
        }
    };

    return (
        <div className="flex flex-col h-full bg-[#121214] border-l border-white/5 w-64 Frost-Studio shadow-2xl overflow-hidden">
            <div className="p-4 border-b border-white/5 space-y-4">
                <div className="flex items-center justify-between">
                    <h3 className="text-[10px] font-black uppercase tracking-[0.2em] text-gray-400">Preset Library</h3>
                    <span className="text-[9px] bg-indigo-500/20 text-indigo-400 px-2 py-0.5 rounded-full font-bold">{presets.length}</span>
                </div>
                <div className="relative">
                    <Search size={12} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
                    <input
                        type="text"
                        placeholder="Search patches..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="w-full bg-black/40 border border-white/10 rounded-lg pl-8 pr-4 py-2 text-[11px] focus:outline-none focus:border-indigo-500/50 transition-all font-medium text-gray-300"
                    />
                </div>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-1">
                {filtered.map((p, i) => (
                    <button
                        key={i}
                        className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-white/5 transition-all group text-left border border-transparent hover:border-white/5"
                    >
                        {getCategoryIcon(p.category)}
                        <div className="flex flex-col">
                            <span className="text-[11px] font-bold text-gray-300 group-hover:text-white transition-colors">{p.name}</span>
                            <span className="text-[8px] uppercase tracking-tighter text-gray-600 font-black group-hover:text-indigo-400/70">{p.category}</span>
                        </div>
                    </button>
                ))}
            </div>
        </div>
    );
}
