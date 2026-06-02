import { useState, useMemo } from 'react';
import { useDawStore } from '../store/useDawStore';
import { Search, X, Plus, Music, Layers, Activity, Wind } from 'lucide-react';
import presetsData from '../presets/pro-trap-neo.json';

interface PluginInfo {
    name: string;
    category: string;
    type: 'internal' | 'vst3';
    description: string;
}

const NATIVE_PLUGINS: PluginInfo[] = [
    { name: 'Frost Compressor', category: 'Dynamics', type: 'internal', description: 'VCA-style compression' },
    { name: 'Frost EQ', category: 'EQ', type: 'internal', description: '5-band parametric equalizer' },
    { name: 'Frost Limiter', category: 'Dynamics', type: 'internal', description: 'Brickwall limiter' },
    { name: 'Frost Bass', category: 'Synth', type: 'internal', description: 'Sub-bass synthesizer' },
    { name: 'Frost Delay', category: 'Delay', type: 'internal', description: 'Stereo delay with feedback' },
    { name: 'Frost Reverb', category: 'Reverb', type: 'internal', description: 'Algorithmic plate reverb' },
];

export default function PluginInsert() {
    const { isPluginPickerOpen, setPluginPickerOpen, addPlugin, selectedTrackId, tracks } = useDawStore();
    const [searchQuery, setSearchQuery] = useState('');
    const [activeCategory, setActiveCategory] = useState('All');

    const categories = ['All', 'Trap', 'Bass', 'Lead', 'Pad', 'FX', 'Dynamics', 'EQ', 'Synth'];

    const filteredPlugins = useMemo(() => {
        const allItems = [
            ...NATIVE_PLUGINS,
            ...presetsData.map(p => ({
                name: p.name,
                category: p.category.charAt(0).toUpperCase() + p.category.slice(1),
                type: 'internal' as const,
                description: p.description
            }))
        ];

        return allItems.filter(p => {
            const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                                p.description.toLowerCase().includes(searchQuery.toLowerCase());
            const matchesCategory = activeCategory === 'All' || p.category === activeCategory;
            return matchesSearch && matchesCategory;
        });
    }, [searchQuery, activeCategory]);

    const handleSelect = (plugin: PluginInfo) => {
        const trackIndex = tracks.findIndex(t => t.id === selectedTrackId);
        if (trackIndex !== -1) {
            addPlugin(trackIndex, plugin.name.toLowerCase()).catch(console.error);
        }
        setPluginPickerOpen(false);
    };

    if (!isPluginPickerOpen) return null;

    return (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-[100] animate-in fade-in duration-200">
            <div className="bg-[#111] border border-white/10 rounded-2xl w-full max-w-4xl max-h-[85vh] flex flex-col shadow-2xl overflow-hidden">
                {/* Header */}
                <div className="p-6 border-b border-white/5 flex items-center justify-between bg-gradient-to-r from-blue-500/10 to-transparent">
                    <div>
                        <h2 className="text-2xl font-bold text-white flex items-center gap-3">
                            <Layers className="text-blue-400" />
                            Plugin Browser
                        </h2>
                        <p className="text-gray-400 text-sm mt-1">Select a plugin or preset to insert into the channel rack</p>
                    </div>
                    <button 
                        onClick={() => setPluginPickerOpen(false)}
                        title="Close Browser"
                        className="p-2 hover:bg-white/5 rounded-full text-gray-400 hover:text-white transition-colors"
                    >
                        <X size={24} />
                    </button>
                </div>

                <div className="flex flex-1 overflow-hidden">
                    {/* Sidebar Categories */}
                    <div className="w-48 border-r border-white/5 p-4 bg-black/20 overflow-y-auto">
                        <div className="space-y-1">
                            {categories.map(cat => (
                                <button
                                    key={cat}
                                    onClick={() => setActiveCategory(cat)}
                                    className={`w-full text-left px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                                        activeCategory === cat 
                                        ? 'bg-blue-600 text-white shadow-lg shadow-blue-600/20' 
                                        : 'text-gray-400 hover:bg-white/5 hover:text-gray-200'
                                    }`}
                                >
                                    {cat}
                                </button>
                            ))}
                        </div>
                    </div>

                    {/* Main Content */}
                    <div className="flex-1 flex flex-col bg-[#0d0d0d]">
                        {/* Search Bar */}
                        <div className="p-4 border-b border-white/5">
                            <div className="relative">
                                <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" />
                                <input
                                    type="text"
                                    placeholder="Search plugins, presets, effects..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    className="w-full bg-white/5 border border-white/10 rounded-xl py-3 pl-12 pr-4 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 transition-all font-medium"
                                    autoFocus
                                />
                            </div>
                        </div>

                        {/* Plugin Grid */}
                        <div className="flex-1 overflow-y-auto p-6 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            {filteredPlugins.map((plugin, idx) => (
                                <button
                                    key={`${plugin.name}-${idx}`}
                                    onClick={() => handleSelect(plugin)}
                                    className="group relative bg-[#1a1a1a] border border-white/5 rounded-xl p-4 text-left hover:bg-white/5 hover:border-blue-500/30 transition-all active:scale-95"
                                >
                                    <div className="flex justify-between items-start mb-2">
                                        <div className="p-2 bg-blue-500/10 rounded-lg group-hover:bg-blue-500/20 transition-colors">
                                            {plugin.category === 'Synth' ? <Music className="text-blue-400" /> : 
                                             plugin.category === 'Dynamics' ? <Activity className="text-orange-400" /> :
                                             <Wind className="text-cyan-400" />}
                                        </div>
                                        <span className="text-[10px] font-bold text-gray-600 uppercase tracking-tighter">
                                            {plugin.type}
                                        </span>
                                    </div>
                                    <h3 className="text-white font-bold mb-1 truncate">{plugin.name}</h3>
                                    <p className="text-gray-500 text-xs line-clamp-2 leading-relaxed">
                                        {plugin.description}
                                    </p>
                                    <div className="absolute bottom-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                        <Plus className="text-blue-400 shadow-sm" />
                                    </div>
                                </button>
                            ))}
                        </div>
                    </div>
                </div>

                {/* Footer */}
                <div className="p-4 bg-black/40 border-t border-white/5 flex justify-between items-center px-8">
                    <span className="text-gray-500 text-xs font-medium">
                        Showing {filteredPlugins.length} available plugins/presets
                    </span>
                    <div className="flex gap-2">
                        <button 
                            className="px-6 py-2 bg-white/5 hover:bg-white/10 text-white rounded-lg text-sm font-bold transition-all"
                            onClick={() => setPluginPickerOpen(false)}
                        >
                            Cancel
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
