import { useRef, useState, useEffect } from 'react';
import '../styles/Knob.css';
import React from 'react';

interface KnobProps {
    value: number; // For Pan: -1 to 1. For EQ: e.g., -24 to +24
    min: number;
    max: number;
    onChange: (val: number) => void;
    label: string;
    unit?: string;
    size?: number;
    step?: number;
    isPan?: boolean;
}

export function Knob({ value, min, max, step = 1, onChange, label, unit = '', size = 32, isPan = false }: KnobProps) {
    const [isDragging, setIsDragging] = useState(false);
    const startY = useRef(0);
    const startVal = useRef(0);

    const displayVal = isPan ?
        (value === 0 ? 'C' : value < 0 ? `L${Math.round(Math.abs(value * 100))}` : `R${Math.round(value * 100)}`) :
        `${value > 0 ? '+' : ''}${Math.round(value)}${unit}`;

    // Map value to angle (-135 to 135 degrees)
    const percent = (value - min) / (max - min);
    const angle = percent * 270 - 135;

    useEffect(() => {
        const handleUp = () => setIsDragging(false);
        const handleMove = (e: MouseEvent) => {
            if (!isDragging) return;
            const delta = startY.current - e.clientY;
            const range = max - min;
            // 100 pixels = full range
            let newVal = startVal.current + (delta / 100) * range;
            newVal = Math.max(min, Math.min(max, newVal));
            if (step !== 0) {
                newVal = Math.round(newVal / step) * step;
            }
            onChange(newVal);
        };

        if (isDragging) {
            window.addEventListener('mousemove', handleMove);
            window.addEventListener('mouseup', handleUp);
        }
        return () => {
            window.removeEventListener('mousemove', handleMove);
            window.removeEventListener('mouseup', handleUp);
        };
    }, [isDragging, min, max, step, onChange]);

    return (
        <div className="knob-container">
            <div
                className="knob-disk"
                style={{
                    '--knob-size': `${size}px`,
                    '--knob-angle': `${angle}deg`
                } as React.CSSProperties}
                onMouseDown={(e) => {
                    setIsDragging(true);
                    startY.current = e.clientY;
                    startVal.current = value;
                }}
                onDoubleClick={() => onChange(isPan ? 0 : 0)}
            >
                <div className="knob-indicator" />
                {isPan && <div className="knob-center-detent" />}
            </div>
            <div className="flex flex-col items-center leading-none">
                <span className="text-[9px] text-gray-500 font-medium uppercase tracking-[0.05em]">{label}</span>
                <span className="text-[10px] text-gray-300 font-mono">{displayVal}</span>
            </div>
        </div>
    );
}
