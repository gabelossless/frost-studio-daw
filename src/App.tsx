import { useEffect, useRef, useState } from 'react';
import TopBar from './components/TopBar';
import MenuBar from './components/MenuBar';
import Mixer from './components/Mixer';
import Arrangement from './components/Arrangement';
import PianoRoll from './components/PianoRoll';
import ExportModal from './components/ExportModal';
import InstrumentView from './components/InstrumentView';
import PluginManager from './components/PluginManager';
import DetailTabs from './components/DetailTabs';
import SampleBrowser from './components/SampleBrowser';
import LoadingScreen from './components/LoadingScreen';
import PluginInsert from './components/PluginInsert';
import VisualizerView from './components/VisualizerView';
import { useDawStore } from './store/useDawStore';
import { listen } from '@tauri-apps/api/event';
import { AudioTrackEditor } from './components/AudioTrackEditor';

export default function App() {
    const { isFullScreenMixer, activeDetailTab, isExportModalOpen, setExportModalOpen, tracks } = useDawStore();
    const [mainPanelHeight, setMainPanelHeight] = useState(65); // percent
    const dragging = useRef(false);
    const [isDraggingFile, setIsDraggingFile] = useState(false);

    useEffect(() => {
        const onMouseMove = (e: MouseEvent) => {
            if (!dragging.current) return;
            const pct = (e.clientY / window.innerHeight) * 100;
            setMainPanelHeight(Math.max(30, Math.min(85, pct)));
        };
        const onMouseUp = () => { dragging.current = false; };
        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('mouseup', onMouseUp);
        const unlistenDrop = listen<{ paths: string[] }>('tauri://file-drop', (event) => {
            const paths = event.payload.paths;
            if (paths && paths.length > 0) {
                handleFilesDropped(paths);
            }
        });

        // Also check Tauri v2 drop event which might be 'tauri://drag-drop'
        const unlistenDragDrop = listen<{ paths: string[] }>('tauri://drag-drop', (event) => {
            const paths = event.payload.paths;
            if (paths && paths.length > 0) {
                handleFilesDropped(paths);
            }
        });

        return () => {
            window.removeEventListener('mousemove', onMouseMove);
            window.removeEventListener('mouseup', onMouseUp);
            unlistenDrop.then(f => f());
            unlistenDragDrop.then(f => f());
        };
    }, [tracks]);

    const handleFilesDropped = async (paths: string[]) => {
        setIsDraggingFile(false);
        const audioFiles = paths.filter(p => !!p.match(/\.(wav|mp3|ogg|flac)$/i));
        
        if (audioFiles.length > 0) {
            for (const path of audioFiles) {
                await useDawStore.getState().addAudioClipFromPath(path);
            }
        }
    };

    const handleWebDrop = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDraggingFile(false);
        const files = Array.from(e.dataTransfer.files);
        // In browser, files.path isn't guaranteed, we use name as fallback, 
        // but backend might not be able to load unless it's sent via tauri commands.
        // We'll pass the files' names in web mode.
        const paths = files.map(f => (f as any).path || f.name);
        handleFilesDropped(paths);
    };

    const handleWebDragOver = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDraggingFile(true);
    };

    const handleWebDragLeave = (e: React.DragEvent) => {
        e.preventDefault();
        setIsDraggingFile(false);
    };


    if (isFullScreenMixer) {
        return (
            <div className="app-shell bg-[#0A0A0B] text-white overflow-hidden h-screen flex flex-col font-sans select-none Frost-Studio">
                <LoadingScreen />
                <MenuBar />
                <TopBar />
                <div className="flex-1 overflow-hidden">
                    <Mixer />
                </div>
                <ExportModal isOpen={isExportModalOpen} onClose={() => setExportModalOpen(false)} />
            </div>
        );
    }

    return (
        <div 
            className="app-shell bg-[#0A0A0B] text-white overflow-hidden h-screen flex flex-col font-sans select-none Frost-Studio relative"
            onDrop={handleWebDrop}
            onDragOver={handleWebDragOver}
            onDragLeave={handleWebDragLeave}
        >
            {isDraggingFile && (
                <div className="absolute inset-0 bg-indigo-500/20 z-[100] border-4 border-indigo-500 border-dashed pointer-events-none flex items-center justify-center">
                    <h2 className="text-4xl font-bold bg-black/50 px-8 py-4 rounded-2xl backdrop-blur-sm">Drop Audio Here</h2>
                </div>
            )}
            <LoadingScreen />
            <MenuBar />
            <TopBar />

            <div className="flex-1 flex overflow-hidden">
                <SampleBrowser />

                <div className="main-layout flex-1 flex flex-col overflow-hidden relative border-l border-[#27272a]/50">
                    <div
                        className="main-layout-panel"
                        style={{ height: `${mainPanelHeight}%` }}
                    >
                        <Arrangement />
                    </div>

                    <div
                        className="h-1 bg-[#27272a] hover:bg-indigo-500 cursor-row-resize transition-colors z-30"
                        onMouseDown={() => { dragging.current = true; }}
                    />

                    <DetailTabs />

                    <div className="flex-1 overflow-hidden bg-[#121214]">
                        {activeDetailTab === 'piano-roll' && <PianoRoll />}
                        {activeDetailTab === 'mixer' && <Mixer />}
                        {activeDetailTab === 'instrument' && <InstrumentView />}
                        {activeDetailTab === 'plugins' && <PluginManager />}
                        {activeDetailTab === 'visualizer' && <VisualizerView />}
                        {activeDetailTab === 'audio' && <AudioTrackEditor />}
                    </div>
                </div>
            </div>
            <ExportModal isOpen={isExportModalOpen} onClose={() => setExportModalOpen(false)} />
            <PluginInsert />
        </div>
    );
}
