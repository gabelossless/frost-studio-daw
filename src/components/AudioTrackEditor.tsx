import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useDawStore } from '../store/useDawStore';
import { Scissors, ZoomIn, ZoomOut, AudioWaveform } from 'lucide-react';

export const AudioTrackEditor: React.FC = () => {
    const { selectedClipId, clips } = useDawStore();
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const [waveform, setWaveform] = useState<number[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [zoomX, setZoomX] = useState(1);

    const clip = clips.find(c => c.id === selectedClipId);

    useEffect(() => {
        if (clip?.samplePath) {
            setIsLoading(true);
            invoke<number[]>('get_sample_waveform', { path: clip.samplePath, buckets: 1000 })
                .then(data => {
                    setWaveform(data);
                    setIsLoading(false);
                })
                .catch(err => {
                    console.error("Failed to load waveform:", err);
                    setIsLoading(false);
                });
        } else {
            setWaveform([]);
        }
    }, [clip?.samplePath]);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas || waveform.length === 0) return;

        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        // Handle high DPI
        const dpr = window.devicePixelRatio || 1;
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        ctx.scale(dpr, dpr);

        const width = rect.width;
        const height = rect.height;
        const midY = height / 2;

        ctx.clearRect(0, 0, width, height);

        // Draw Background Grid Mesh
        ctx.strokeStyle = '#1e1e24';
        ctx.lineWidth = 1;
        const gridStep = 40;
        for (let x = 0; x < width; x += gridStep) {
            ctx.beginPath();
            ctx.moveTo(x, 0);
            ctx.lineTo(x, height);
            ctx.stroke();
        }

        // Draw Centerline
        ctx.strokeStyle = '#2a2a35';
        ctx.beginPath();
        ctx.moveTo(0, midY);
        ctx.lineTo(width, midY);
        ctx.stroke();

        if (waveform.length === 0) return;

        // Draw Waveform Glow
        ctx.strokeStyle = 'rgba(16, 185, 129, 0.8)'; // Emerald
        ctx.lineWidth = 2;
        ctx.beginPath();

        const step = width / waveform.length;
        
        for (let i = 0; i < waveform.length; i++) {
            const x = i * step;
            const peak = waveform[i]; // 0.0 to 1.0
            const amp = (peak * (height / 2)) * 0.9; // Scale to 90% of half height

            // Draw line from center to top peak
            ctx.moveTo(x, midY - amp);
            ctx.lineTo(x, midY + amp);
        }
        ctx.stroke();

    }, [waveform, zoomX]);

    if (!clip) {
        return (
            <div className="flex flex-col items-center justify-center h-full text-gray-500 bg-[#070709] italic">
                <AudioWaveform className="w-12 h-12 mb-2 opacity-30 text-emerald-500" />
                No audio clip selected. Double-click an audio clip to edit.
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full bg-[#09090b] text-xs font-mono select-none">
            {/* Toolbar */}
            <div className="flex items-center justify-between h-9 px-3 bg-[#0c0c10] border-b border-[#16161c]">
                <div className="flex items-center gap-2">
                    <span className="font-bold text-gray-400 capitalize">{clip.name}</span>
                    <span className="text-[10px] text-gray-600">Sample Path: {clip.samplePath}</span>
                </div>

                <div className="flex items-center gap-1">
                    <button className="p-1 rounded hover:bg-[#1a1a22] text-gray-400">
                        <Scissors className="w-3.5 h-3.5" />
                    </button>
                    <div className="h-4 w-[1px] bg-[#1a1a22] mx-1"></div>
                    <button 
                        onClick={() => setZoomX(prev => Math.max(0.5, prev - 0.25))}
                        className="p-1 rounded hover:bg-[#1a1a22] text-gray-400"
                    >
                        <ZoomOut className="w-3.5 h-3.5" />
                    </button>
                    <button 
                        onClick={() => setZoomX(prev => Math.min(5, prev + 0.25))}
                        className="p-1 rounded hover:bg-[#1a1a22] text-gray-400"
                    >
                        <ZoomIn className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Canvas Container */}
            <div className="flex-1 relative overflow-hidden p-2">
                {isLoading && (
                    <div className="absolute inset-0 flex items-center justify-center bg-black/50 z-10 text-emerald-400 text-[10px] animate-pulse">
                        Generating Waveform Peaks...
                    </div>
                )}
                
                <div className="w-full h-full bg-[#050507] rounded-md border border-[#121216] relative overflow-hidden">
                    <canvas 
                        ref={canvasRef} 
                        className="w-full h-full cursor-crosshair"
                    />
                </div>
            </div>
        </div>
    );
};
