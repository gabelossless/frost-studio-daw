import { useState } from 'react';
import { useDawStore } from '../store/useDawStore';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { X, Download, ShieldCheck, Loader2 } from 'lucide-react';

interface ExportModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export default function ExportModal({ isOpen, onClose }: ExportModalProps) {
    const [isExporting, setIsExporting] = useState(false);
    const [exportProgress, setExportProgress] = useState(0);
    const [error, setError] = useState<string | null>(null);
    const { tempo, clips } = useDawStore();

    if (!isOpen) return null;

    // Calculate maximum duration based on clip ends
    const maxBeats = clips.length > 0
        ? Math.max(...clips.map(c => c.start + c.duration))
        : 16;
    const roundedBeats = Math.ceil(maxBeats / 4) * 4; // Round up to nearest bar

    const handleExport = async () => {
        setIsExporting(true);
        setError(null);
        setExportProgress(10); // Fake start pulse

        try {
            // 1. Show save dialog
            const filePath = await save({
                filters: [{ name: 'Audio', extensions: ['wav'] }],
                defaultPath: 'composition.wav'
            });

            if (!filePath) {
                setIsExporting(false);
                return;
            }

            setExportProgress(30);

            // 2. Trigger Rust export
            await invoke('export_project', {
                path: filePath,
                durationBeats: roundedBeats
            });

            setExportProgress(100);
            setTimeout(() => {
                setIsExporting(false);
                onClose();
            }, 500);

        } catch (e: any) {
            console.error(e);
            setError(e.toString());
            setIsExporting(false);
        }
    };

    return (
        <div className="export-modal-overlay animate-in fade-in duration-200">
            <div className="w-full max-w-md bg-[#18181B] rounded-2xl border border-white/10 shadow-2xl overflow-hidden shadow-indigo-500/20">
                <div className="flex items-center justify-between p-6 border-b border-white/5">
                    <div className="flex items-center gap-3">
                        <div className="p-2 bg-indigo-500/20 rounded-lg text-indigo-400">
                            <Download size={20} />
                        </div>
                        <div>
                            <h2 className="text-xl font-bold text-white">Export Audio</h2>
                            <p className="text-xs text-gray-400 uppercase tracking-widest font-mono">32-bit Float WAV • {tempo} BPM</p>
                        </div>
                    </div>
                    <button
                        onClick={onClose}
                        className="text-gray-500 hover:text-white transition-colors"
                        title="Close"
                    >
                        <X size={20} />
                    </button>
                </div>

                <div className="p-6 space-y-6">
                    {isExporting ? (
                        <div className="space-y-4 py-8 flex flex-col items-center text-center">
                            <div className="relative">
                                <Loader2 size={48} className="text-indigo-400 animate-spin" />
                                <div className="absolute inset-0 flex items-center justify-center text-[10px] font-bold">
                                    {exportProgress}%
                                </div>
                            </div>
                            <div>
                                <h3 className="text-lg font-semibold text-white">Rendering Composition...</h3>
                                <p className="text-sm text-gray-500 mt-1">Applying Master Limiter and Summing Channels</p>
                            </div>
                            <div className="export-progress-bar-container">
                                <div
                                    className="export-progress-bar-fill"
                                    style={{ width: `${exportProgress}%` }}
                                />
                            </div>
                        </div>
                    ) : (
                        <>
                            <div className="space-y-3">
                                <label className="text-[10px] font-bold text-gray-500 uppercase tracking-tighter">Export Settings</label>
                                <div className="grid grid-cols-2 gap-3">
                                    <div className="p-3 bg-white/5 rounded-xl border border-white/5 flex flex-col">
                                        <span className="text-[10px] text-gray-500 font-mono">Format</span>
                                        <span className="text-sm font-bold text-gray-200">WAV (PCM)</span>
                                    </div>
                                    <div className="p-3 bg-white/5 rounded-xl border border-white/5 flex flex-col">
                                        <span className="text-[10px] text-gray-500 font-mono">Bit Depth</span>
                                        <span className="text-sm font-bold text-gray-200">32-bit Float</span>
                                    </div>
                                    <div className="p-3 bg-white/5 rounded-xl border border-white/5 flex flex-col">
                                        <span className="text-[10px] text-gray-500 font-mono">Sample Rate</span>
                                        <span className="text-sm font-bold text-gray-200">44.1 kHz</span>
                                    </div>
                                    <div className="p-3 bg-white/5 rounded-xl border border-white/5 flex flex-col">
                                        <span className="text-[10px] text-gray-500 font-mono">Length</span>
                                        <span className="text-sm font-bold text-gray-200">{roundedBeats / 4} Bars</span>
                                    </div>
                                </div>
                            </div>

                            <div className="flex items-center gap-3 p-4 bg-indigo-500/10 rounded-xl border border-indigo-500/20 text-indigo-300">
                                <ShieldCheck size={20} className="shrink-0" />
                                <p className="text-xs leading-relaxed">
                                    The **Master Limiter** will be applied during export to ensure your audio stays below 0dBFS without clipping.
                                </p>
                            </div>

                            {error && (
                                <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-xs">
                                    {error}
                                </div>
                            )}

                            <button
                                onClick={handleExport}
                                className="w-full bg-indigo-500 hover:bg-indigo-400 text-white font-bold py-4 rounded-xl shadow-lg shadow-indigo-500/20 transition-all active:scale-[0.98] flex items-center justify-center gap-2"
                                title="Start Export"
                            >
                                <Download size={18} />
                                START EXPORT
                            </button>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
}
