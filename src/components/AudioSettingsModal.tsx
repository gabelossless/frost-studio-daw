import { useEffect, useState } from 'react';
import { X, Headphones, Check } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface AudioSettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
}

const BUFFER_SIZES = [64, 128, 256, 512, 1024, 2048];

export default function AudioSettingsModal({ isOpen, onClose }: AudioSettingsModalProps) {
    const [hosts, setHosts] = useState<string[]>([]);
    const [selectedHost, setSelectedHost] = useState<string>('');
    const [devices, setDevices] = useState<string[]>([]);
    const [selectedDevice, setSelectedDevice] = useState<string>('');
    const [bufferSize, setBufferSize] = useState<number | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!isOpen) return;

        const loadHosts = async () => {
            try {
                const availableHosts = await invoke<string[]>('get_audio_hosts');
                setHosts(availableHosts);
                if (availableHosts.length > 0) {
                    // Default to first host (usually Wasapi on windows)
                    const initialHost = availableHosts[0];
                    setSelectedHost(initialHost);
                    loadDevices(initialHost);
                }
            } catch (err: any) {
                console.error('Failed to load audio hosts:', err);
                setError(err.toString());
            }
        };

        loadHosts();
    }, [isOpen]);

    const loadDevices = async (host: string) => {
        try {
            const availableDevices = await invoke<string[]>('get_audio_devices', { host });
            setDevices(availableDevices);
            if (availableDevices.length > 0) {
                // Try to find if default already playing, or just pick first
                setSelectedDevice(availableDevices[0]);
            }
        } catch (err: any) {
            console.error('Failed to load devices for host:', host, err);
            setError(`Failed to load devices for ${host}`);
        }
    };

    const handleHostChange = (newHost: string) => {
        setSelectedHost(newHost);
        loadDevices(newHost);
    };

    const handleApplySettings = async () => {
        try {
            await invoke('set_audio_device', {
                host: selectedHost,
                device: selectedDevice,
                bufferSize: bufferSize
            });
            onClose();
        } catch (err: any) {
            console.error('Failed to set audio device:', err);
            setError(err.toString());
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-md flex items-center justify-center z-[100] animate-in fade-in duration-200">
            <div className="bg-[#141416] border border-[#27272a] rounded-2xl w-full max-w-md p-6 shadow-2xl flex flex-col gap-5">
                <div className="flex items-center justify-between border-b border-[#27272a] pb-4">
                    <div className="flex items-center gap-2 text-indigo-400 font-bold text-lg">
                        <Headphones size={20} />
                        Audio Driver Settings
                    </div>
                    <button onClick={onClose} title="Close" className="text-gray-400 hover:text-white transition-colors">
                        <X size={18} />
                    </button>
                </div>

                {error && (
                    <div className="text-red-400 text-xs bg-red-500/10 p-2 rounded border border-red-500/20">
                        {error}
                    </div>
                )}

                {/* Driver / Host Selection */}
                <div className="flex flex-col gap-2">
                    <label className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Audio System / Driver</label>
                    <div className="flex gap-2">
                        {hosts.map(host => (
                            <button
                                key={host}
                                onClick={() => handleHostChange(host)}
                                className={`px-3 py-2 rounded-lg border text-xs font-medium transition-all ${
                                    selectedHost === host
                                        ? 'bg-indigo-500/10 border-indigo-500 text-white'
                                        : 'bg-[#1c1c1e] border-transparent text-gray-400 hover:bg-[#27272a]'
                                }`}
                            >
                                {host}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Device Selection */}
                <div className="flex flex-col gap-2">
                    <label className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Output Device</label>
                    <div className="flex flex-col gap-1 max-h-40 overflow-y-auto custom-scrollbar">
                        {devices.map((device) => (
                            <button
                                key={device}
                                onClick={() => setSelectedDevice(device)}
                                className={`flex items-center justify-between px-3 py-2 rounded-lg border transition-all text-left ${
                                    selectedDevice === device
                                        ? 'bg-indigo-500/10 border-indigo-500 text-white'
                                        : 'bg-[#1c1c1e] border-transparent text-gray-400 hover:bg-[#27272a] hover:text-gray-200'
                                }`}
                            >
                                <div className="text-xs font-medium truncate pr-4">{device}</div>
                                {selectedDevice === device && <Check size={14} className="text-indigo-400 flex-shrink-0" />}
                            </button>
                        ))}
                    </div>
                </div>

                {/* Buffer Size Selection */}
                <div className="flex flex-col gap-2">
                    <label className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Buffer Size (Latency)</label>
                    <p className="text-[10px] text-gray-500">Lower values reduce lag but place higher demand on CPU.</p>
                    <div className="grid grid-cols-6 gap-1 mt-1">
                        {BUFFER_SIZES.map(size => (
                            <button
                                key={size}
                                onClick={() => setBufferSize(size)}
                                className={`py-1.5 rounded-lg border text-[11px] font-medium text-center transition-all ${
                                    bufferSize === size
                                        ? 'bg-indigo-500/10 border-indigo-500 text-white'
                                        : 'bg-[#1c1c1e] border-transparent text-gray-400 hover:bg-[#27272a]'
                                }`}
                            >
                                {size}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="flex justify-end gap-2 mt-2 pt-4 border-t border-[#27272a]">
                    <button onClick={onClose} className="px-4 py-2 border border-[#27272a] hover:bg-[#1c1c1e] text-gray-400 hover:text-white rounded-xl text-xs font-medium transition-all">
                        Cancel
                    </button>
                    <button onClick={handleApplySettings} className="px-4 py-2 bg-indigo-500 hover:bg-indigo-600 text-white rounded-xl text-xs font-semibold shadow-lg shadow-indigo-500/20 transition-all">
                        Apply Settings
                    </button>
                </div>
            </div>
        </div>
    );
}
