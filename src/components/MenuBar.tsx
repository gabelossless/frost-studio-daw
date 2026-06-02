import { useState } from 'react';
import { File, FolderOpen, Save, Download, Upload, Undo2, Redo2, Scissors, Copy, ClipboardPaste, Trash2, Headphones } from 'lucide-react';
import AudioSettingsModal from './AudioSettingsModal';
import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
import { useDawStore } from '../store/useDawStore';

export default function MenuBar() {
    const [activeMenu, setActiveMenu] = useState<string | null>(null);
    const [isSettingsOpen, setIsSettingsOpen] = useState(false);

    // Expose entire state to save/load
    const store = useDawStore();


    const handleNewProject = () => {
        setActiveMenu(null);
        if (window.confirm('Are you sure you want to start a new project? Unsaved changes will be lost.')) {
            useDawStore.setState({
                tracks: [],
                clips: [],
                notes: [],
                tempo: 120,
                timeSignature: [4, 4]
            });
        }
    };

    const handleSaveProject = async () => {
        setActiveMenu(null);
        try {
            const filePath = await save({
                filters: [{
                    name: 'Frost Project',
                    extensions: ['frost']
                }],
                defaultPath: 'New Project.frost',
            });
            if (filePath) {
                const stateToSave = useDawStore.getState();
                await writeTextFile(filePath, JSON.stringify({
                    tracks: stateToSave.tracks,
                    clips: stateToSave.clips,
                    notes: stateToSave.notes,
                    tempo: stateToSave.tempo,
                    timeSignature: stateToSave.timeSignature
                }));
                console.log('Project saved to', filePath);
            }
        } catch (e) {
            console.error('Save failed:', e);
        }
    };

    const handleOpenProject = async () => {
        setActiveMenu(null);
        try {
            const selected = await open({
                multiple: false,
                filters: [{
                    name: 'Frost Project',
                    extensions: ['frost']
                }]
            });
            if (selected && typeof selected === 'string') {
                const contents = await readTextFile(selected);
                const data = JSON.parse(contents);
                useDawStore.setState({
                    tracks: data.tracks || [],
                    clips: data.clips || [],
                    notes: data.notes || [],
                    tempo: data.tempo || 120,
                    timeSignature: data.timeSignature || [4, 4]
                });
            }
        } catch (e) {
            console.error('Open failed:', e);
        }
    };

    return (
        <div className="h-8 bg-[#18181b] border-b border-[#27272a] flex items-center px-4 text-xs font-medium text-gray-300 select-none z-50">
            <div className="flex items-center gap-2 mr-6 text-indigo-400 font-bold tracking-wider">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M12 2L4.5 20.29L5.21 21L12 18L18.79 21L19.5 20.29L12 2Z" fill="currentColor" fillOpacity="0.8" />
                    <path d="M12 5L16 16H8L12 5Z" fill="white" fillOpacity="0.1" />
                </svg>
                FROST
            </div>

            <div className="relative">
                <button
                    className={`px-3 py-1 rounded transition-colors ${activeMenu === 'file' ? 'bg-indigo-500/20 text-indigo-300' : 'hover:bg-[#27272a]'}`}
                    onClick={() => setActiveMenu(activeMenu === 'file' ? null : 'file')}
                >
                    File
                </button>
                {activeMenu === 'file' && (
                    <div className="absolute top-full left-0 mt-2 w-48 glass-panel py-1 flex flex-col z-50 animate-in fade-in slide-in-from-top-2 duration-150">
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={handleNewProject}>
                            <File size={14} className="text-gray-400" /> New Project
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={handleOpenProject}>
                            <FolderOpen size={14} className="text-indigo-400" /> Open Project...
                        </button>
                        <div className="my-1 border-b border-white/5"></div>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={handleSaveProject}>
                            <Save size={14} className="text-gray-400" /> Save Project
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={handleSaveProject}>
                            <Save size={14} className="text-gray-400" /> Save As...
                        </button>
                    </div>
                )}
            </div>

            <div className="relative">
                <button
                    className={`px-3 py-1 rounded transition-colors ${activeMenu === 'edit' ? 'bg-indigo-500/20 text-indigo-300' : 'hover:bg-[#27272a]'}`}
                    onClick={() => setActiveMenu(activeMenu === 'edit' ? null : 'edit')}
                >
                    Edit
                </button>
                {activeMenu === 'edit' && (
                    <div className="absolute top-full left-0 mt-2 w-48 glass-panel py-1 flex flex-col z-50 animate-in fade-in slide-in-from-top-2 duration-150">
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={() => { setActiveMenu(null); useDawStore.temporal.getState().undo(); }}>
                            <Undo2 size={14} className="text-gray-400" /> Undo
                            <span className="ml-auto text-[9px] text-gray-500">Ctrl+Z</span>
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={() => { setActiveMenu(null); useDawStore.temporal.getState().redo(); }}>
                            <Redo2 size={14} className="text-gray-400" /> Redo
                            <span className="ml-auto text-[9px] text-gray-500">Ctrl+Y</span>
                        </button>
                        <div className="my-1 border-b border-white/5"></div>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors text-gray-500" onClick={() => setActiveMenu(null)}>
                            <Scissors size={14} /> Cut
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors text-gray-500" onClick={() => setActiveMenu(null)}>
                            <Copy size={14} /> Copy
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors text-gray-500" onClick={() => setActiveMenu(null)}>
                            <ClipboardPaste size={14} /> Paste
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors text-gray-500" onClick={() => setActiveMenu(null)}>
                            <Trash2 size={14} /> Delete
                        </button>
                    </div>
                )}
            </div>

            <div className="relative">
                <button
                    className={`px-3 py-1 rounded transition-colors ${activeMenu === 'project' ? 'bg-indigo-500/20 text-indigo-300' : 'hover:bg-[#27272a]'}`}
                    onClick={() => setActiveMenu(activeMenu === 'project' ? null : 'project')}
                >
                    Project
                </button>
                {activeMenu === 'project' && (
                    <div className="absolute top-full left-0 mt-2 w-48 glass-panel py-1 flex flex-col z-50 animate-in fade-in slide-in-from-top-2 duration-150">
                        <button 
                            className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" 
                            onClick={async () => { 
                                setActiveMenu(null); 
                                try {
                                    const selected = await open({
                                        multiple: true,
                                        filters: [{ name: 'Audio', extensions: ['wav', 'mp3'] }]
                                    });
                                    if (selected) {
                                        const paths = Array.isArray(selected) ? selected : [selected];
                                        for (const path of paths) {
                                            await store.addAudioClipFromPath(path);
                                        }
                                    }
                                } catch (e) {
                                    console.error('Import failed:', e);
                                }
                            }}
                        >
                            <Upload size={14} className="text-gray-400" /> Import Audio...
                        </button>
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={() => { setActiveMenu(null); store.setExportModalOpen(true); }}>
                            <Download size={14} className="text-cyan-400" /> Export Mixdown...
                        </button>
                    </div>
                )}
            </div>

            <div className="relative">
                <button
                    className={`px-3 py-1 rounded transition-colors ${activeMenu === 'settings' ? 'bg-indigo-500/20 text-indigo-300' : 'hover:bg-[#27272a]'}`}
                    onClick={() => setActiveMenu(activeMenu === 'settings' ? null : 'settings')}
                >
                    Settings
                </button>
                {activeMenu === 'settings' && (
                    <div className="absolute top-full left-0 mt-2 w-48 glass-panel py-1 flex flex-col z-50 animate-in fade-in slide-in-from-top-2 duration-150">
                        <button className="flex items-center gap-2 px-4 py-2 hover:bg-white/10 w-full text-left transition-colors" onClick={() => { setActiveMenu(null); setIsSettingsOpen(true); }}>
                            <Headphones size={14} className="text-indigo-400" /> Audio Setup
                        </button>
                    </div>
                )}
            </div>

            {/* Global dismiss layer */}
            {
                activeMenu && (
                    <div className="fixed inset-0 z-40" onClick={() => setActiveMenu(null)} />
                )
            }

            <AudioSettingsModal isOpen={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} />
        </div >
    );
}
