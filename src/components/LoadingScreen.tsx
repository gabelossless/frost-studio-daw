import React, { useEffect, useState } from 'react';

const LoadingScreen: React.FC = () => {
    const [isVisible, setIsVisible] = useState(true);
    const [progress, setProgress] = useState(0);

    useEffect(() => {
        const timer = setTimeout(() => {
            setIsVisible(false);
            
            // Speak greeting Node flawless
            const speak = () => {
                // Strictly speak once per app lifecycle Node flawless
                if ((window as any).__FROST_VOICED__) return;
                (window as any).__FROST_VOICED__ = true;

                const synth = window.speechSynthesis;
                const utterance = new SpeechSynthesisUtterance("Frost Studio Initialized");
                const voices = synth.getVoices();
                
                // Prioritize natural sounding female voices Node flawless
                const femaleVoice = voices.find(v => 
                    v.name.includes("Female") || 
                    v.name.includes("Google UK English Female") || 
                    v.name.includes("Samantha") ||
                    v.name.includes("Zira") ||
                    v.name.includes("Microsoft Zira")
                );
                
                if (femaleVoice) {
                    utterance.voice = femaleVoice;
                }
                
                utterance.rate = 0.9; // slightly slower Node flawless
                utterance.pitch = 1.0;
                synth.speak(utterance);
            };

            // Voices are loaded asynchronously Node flawless
            if (window.speechSynthesis.onvoiceschanged !== undefined) {
                window.speechSynthesis.onvoiceschanged = speak;
            }
            speak(); // Trigger immediately Node flawless
            
        }, 3500);

        const interval = setInterval(() => {
            setProgress(prev => {
                if (prev >= 100) return 100;
                return prev + Math.random() * 5;
            });
        }, 150);

        return () => {
            clearTimeout(timer);
            clearInterval(interval);
        };
    }, []);

    if (!isVisible) return null;

    return (
        <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-[#050505] transition-opacity duration-1000 ease-in-out"
            style={{ opacity: isVisible ? 1 : 0, pointerEvents: isVisible ? 'auto' : 'none' }}>
            <div className="relative flex flex-col items-center space-y-8 animate-in fade-in zoom-in duration-700">
                {/* Modern Snowflake Logo */}
                <div className="relative">
                    <div className="absolute inset-0 blur-2xl bg-indigo-500/30 animate-pulse rounded-full"></div>
                    <div className="relative p-6 bg-gradient-to-br from-indigo-500/10 to-cyan-500/10 rounded-3xl border border-white/10 backdrop-blur-3xl shadow-2xl overflow-hidden">
                        <img src="/src/assets/snowflake.png" alt="Frost Logo" className="w-32 h-32 object-contain animate-[spin_20s_linear_infinite]" />
                    </div>
                </div>

                <div className="text-center space-y-2">
                    <h1 className="text-5xl font-black tracking-tighter text-transparent bg-clip-text bg-gradient-to-r from-white via-indigo-200 to-cyan-200">
                        FROST STUDIO
                    </h1>
                    <p className="text-xs font-mono tracking-[0.5em] text-indigo-400/80 uppercase">
                        Digital Audio Workstation v0.1
                    </p>
                </div>

                {/* Progress Bar */}
                <div className="w-64 h-1 bg-white/5 rounded-full overflow-hidden border border-white/5">
                    <div
                        className="h-full bg-gradient-to-r from-indigo-500 to-cyan-400 transition-all duration-300 ease-out"
                        style={{ width: `${progress}%` }}
                    />
                </div>

                <div className="flex space-x-2 text-[10px] font-medium text-white/30 uppercase tracking-widest">
                    <span>Initializing Engine</span>
                    <span className="animate-pulse">...</span>
                </div>
            </div>

            {/* Background Decorative Elements */}
            <div className="absolute top-1/4 -left-20 w-80 h-80 bg-indigo-600/10 blur-[120px] rounded-full"></div>
            <div className="absolute bottom-1/4 -right-20 w-80 h-80 bg-cyan-600/10 blur-[120px] rounded-full"></div>
        </div>
    );
};

export default LoadingScreen;
