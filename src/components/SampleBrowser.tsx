import { useState, useEffect } from 'react';
import { Folder, FolderOpen, FileAudio, Play, Plus, Search } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useDawStore } from '../store/useDawStore';

interface SampleNode {
    name: string;
    path: string;
    isDir: boolean;
    children?: SampleNode[];
}

export default function SampleBrowser() {
    const [samples, setSamples] = useState<SampleNode[]>([]);
    const [searchQuery, setSearchQuery] = useState('');
    const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set(['root']));

    const { 
        addClip, 
        selectedTrackId, 
        tracks, 
        playheadPosition,
        setSamplerSample,
        syncAudioTracks
    } = useDawStore();

    useEffect(() => {
        const loadSamples = async () => {
            try {
                const nodes = await invoke<SampleNode[]>('scan_sample_folder', { path: null });
                setSamples(nodes);
            } catch (e) {
                console.error("Failed to scan sample folder:", e);
            }
        };
        loadSamples();
    }, []);

    const toggleFolder = (path: string) => {
        const newExpanded = new Set(expandedFolders);
        if (newExpanded.has(path)) {
            newExpanded.delete(path);
        } else {
            newExpanded.add(path);
        }
        setExpandedFolders(newExpanded);
    };

    const handlePreview = async (path: string) => {
        try {
            await invoke('load_sample_to_memory', { path });
            await invoke('preview_sample', { path });
        } catch (e) {
            console.error("Failed to preview sample", e);
        }
    };

    const handleAddToProject = async (path: string, _name: string) => {
        const selectedTrack = tracks.find(t => t.id === selectedTrackId);
        if (!selectedTrack) return;

        if (selectedTrack.type === 'audio') {
            addClip({
                trackId: selectedTrack.id,
                start: playheadPosition,
                duration: 4,
                name: path,
                color: selectedTrack.color
            });
            await syncAudioTracks();
        } else {
            await setSamplerSample(tracks.indexOf(selectedTrack), path);
        }
    };

    const renderTree = (nodes: SampleNode[], level = 0) => {
        return nodes.map(node => {
            const isExpanded = expandedFolders.has(node.path);

            if (node.isDir) {
                return (
                    <div key={node.path}>
                        <div
                            className="flex items-center px-2 py-1.5 hover:bg-white/5 cursor-pointer text-gray-300 transition-colors group"
                            style={{ paddingLeft: `${level * 12 + 8}px` }}
                            onClick={() => toggleFolder(node.path)}
                        >
                            {isExpanded ? <FolderOpen size={14} className="mr-2 text-indigo-400" /> : <Folder size={14} className="mr-2 text-indigo-400 group-hover:text-indigo-300" />}
                            <span className="text-[11px] font-semibold tracking-wide truncate">{node.name}</span>
                        </div>
                        {isExpanded && node.children && renderTree(node.children, level + 1)}
                    </div>
                );
            }

            return (
                <div
                    key={node.path}
                    className="flex justify-between items-center px-2 py-1.5 hover:bg-indigo-500/20 cursor-grab text-gray-400 hover:text-white transition-colors group"
                    style={{ paddingLeft: `${level * 12 + 8}px` }}
                    draggable
                    onDragStart={(e) => {
                        e.dataTransfer.setData('application/x-frost-sample', node.path);
                    }}
                >
                    <div className="flex items-center overflow-hidden">
                        <FileAudio size={12} className="mr-2 opacity-60 text-cyan-400" />
                        <span className="text-[10px] uppercase font-mono tracking-tighter truncate">{node.name}</span>
                    </div>
                    <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button onClick={(e) => { e.stopPropagation(); handlePreview(node.path); }} className="p-0.5 hover:bg-white/10 rounded text-green-400" title="Preview">
                            <Play size={12} />
                        </button>
                        <button onClick={(e) => { e.stopPropagation(); handleAddToProject(node.path, node.name); }} className="p-0.5 hover:bg-white/10 rounded text-indigo-300" title="Add to Project">
                            <Plus size={12} />
                        </button>
                    </div>
                </div>
            );
        });
    };

    return (
        <div className="flex flex-col h-full bg-[#18181b] border-r border-[#27272a]/50 w-64 Frost-Studio select-none">
            <div className="flex justify-between items-center px-4 py-3 border-b border-[#27272a] shadow-sm bg-[#121214]">
                <h3 className="text-[10px] font-bold tracking-[0.2em] text-cyan-500 uppercase">Browser</h3>
            </div>
            <div className="p-2 border-b border-[#27272a] bg-[#1a1a1d]">
                <div className="relative">
                    <Search size={12} className="absolute left-2.5 top-2 text-gray-500" />
                    <input
                        type="text"
                        placeholder="Search samples..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="w-full bg-black/40 border border-white/5 rounded pl-7 pr-3 py-1.5 text-[10px] font-mono text-gray-300 placeholder-gray-600 focus:outline-none focus:border-indigo-500/50 focus:bg-black/60 transition-all shadow-inner"
                    />
                </div>
            </div>
            <div className="flex-1 overflow-y-auto custom-scrollbar py-2">
                {renderTree(samples)}
            </div>
            <div className="px-3 py-2 border-t border-[#27272a] bg-[#121214] text-center">
                <span className="text-[9px] text-gray-600 font-mono">Samples Folder Linked</span>
            </div>
        </div>
    );
}
