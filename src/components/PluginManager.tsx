import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, Monitor, Package, RefreshCw } from 'lucide-react';

interface VstPlugin {
    name: string;
    path: string;
    vendor: string;
    category: string;
}

export default function PluginManager() {
    const [plugins, setPlugins] = useState<VstPlugin[]>([]);
    const [loading, setLoading] = useState(false);
    const [searchTerm, setSearchTerm] = useState('');

    const scanPlugins = async () => {
        setLoading(true);
        try {
            const data: VstPlugin[] = await invoke('get_available_vst3_plugins');
            setPlugins(data);
        } catch (e) {
            console.error('Failed to scan plugins:', e);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        scanPlugins();
    }, []);

    const filteredPlugins = plugins.filter(p =>
        p.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        p.vendor.toLowerCase().includes(searchTerm.toLowerCase())
    );

    return (
        <div className="flex flex-col h-full bg-[#050506] border-t border-white/5 Frost-Studio p-6">
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h2 className="text-xl font-bold bg-gradient-to-r from-indigo-400 to-cyan-400 bg-clip-text text-transparent">Plugin Manager</h2>
                    <p className="text-[10px] text-gray-500 uppercase tracking-widest font-mono">VST3 / CLAP / AU SCANNER</p>
                </div>
                <div className="flex items-center gap-3">
                    <div className="relative">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={14} />
                        <input
                            type="text"
                            placeholder="Search plugins..."
                            value={searchTerm}
                            onChange={(e) => setSearchTerm(e.target.value)}
                            className="bg-black/60 border border-indigo-500/20 rounded-lg pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-indigo-500/50 w-64 shadow-inner"
                        />
                    </div>
                    <button
                        onClick={scanPlugins}
                        disabled={loading}
                        className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white px-4 py-2 rounded-lg text-sm font-semibold transition-all active:scale-95 disabled:opacity-50"
                    >
                        <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
                        Rescan
                    </button>
                </div>
            </div>

            <div className="flex-1 bg-black/40 rounded-xl border border-indigo-500/10 overflow-hidden flex flex-col shadow-[0_4px_30px_rgba(0,0,0,0.5)]">
                <div className="grid grid-cols-4 bg-[#0f0f11] border-b border-white/5 px-6 py-2 text-[10px] uppercase font-bold text-indigo-300 tracking-tighter">
                    <span>Plugin Name</span>
                    <span>Vendor</span>
                    <span>Category</span>
                    <span>Format</span>
                </div>

                <div className="flex-1 overflow-y-auto custom-scrollbar">
                    {filteredPlugins.length > 0 ? (
                        filteredPlugins.map((plugin, i) => (
                            <div
                                key={i}
                                className="grid grid-cols-4 px-6 py-4 border-b border-white/[0.02] hover:bg-white/[0.02] transition-colors group cursor-pointer"
                            >
                                <div className="flex items-center gap-3">
                                    <div className="w-8 h-8 rounded bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
                                        <Package size={16} />
                                    </div>
                                    <span className="text-sm font-medium text-gray-200 group-hover:text-indigo-400 transition-colors">{plugin.name}</span>
                                </div>
                                <div className="flex items-center text-sm text-gray-500">{plugin.vendor}</div>
                                <div className="flex items-center text-sm text-gray-500">
                                    <span className="px-2 py-0.5 rounded-full bg-indigo-500/5 border border-indigo-500/10 text-[10px] uppercase">{plugin.category}</span>
                                </div>
                                <div className="flex items-center text-sm text-gray-500 font-mono text-[11px] opacity-40">VST3</div>
                            </div>
                        ))
                    ) : (
                        <div className="flex flex-col items-center justify-center h-full gap-4 text-gray-600">
                            <Monitor size={48} className="opacity-10" />
                            <p className="italic">No plugins found in build path. Try rescanning.</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
