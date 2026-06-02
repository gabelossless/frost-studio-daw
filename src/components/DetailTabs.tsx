import { useDawStore } from '../store/useDawStore';
import { Music, ListMusic, Cpu, Settings, Puzzle, Play } from 'lucide-react';

export default function DetailTabs() {
    const { activeDetailTab, setActiveDetailTab } = useDawStore();

    const tabs = [
        { id: 'piano-roll', label: 'Piano Roll', icon: <Music size={14} /> },
        { id: 'mixer', label: 'Mixer', icon: <ListMusic size={14} /> },
        { id: 'instrument', label: 'Instrument', icon: <Cpu size={14} /> },
        { id: 'plugins', label: 'Vst Manager', icon: <Puzzle size={14} /> },
        { id: 'visualizer', label: '3D Visualizer', icon: <Play size={14} className="text-white/80 rotate-45" /> },
        { id: 'audio', label: 'Audio Engine', icon: <Settings size={14} /> },
    ];

    return (
        <div className="h-10 bg-[#121214] border-b border-[#27272a] flex items-center px-4 gap-1 z-30 select-none">
            {tabs.map((tab) => (
                <button
                    key={tab.id}
                    onClick={() => setActiveDetailTab(tab.id as any)}
                    className={`
                        flex items-center gap-2 px-4 py-1.5 rounded-md text-[11px] font-bold uppercase tracking-wider transition-all duration-200
                        ${activeDetailTab === tab.id
                            ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30'
                            : 'text-gray-500 hover:text-gray-300 hover:bg-white/5 border border-transparent'}
                    `}
                >
                    <span className={activeDetailTab === tab.id ? 'text-indigo-400' : 'text-gray-600'}>
                        {tab.icon}
                    </span>
                    {tab.label}
                </button>
            ))}
        </div>
    );
}
