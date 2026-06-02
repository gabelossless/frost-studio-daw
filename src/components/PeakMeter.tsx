import { MeterLevel } from '../store/useDawStore';
import '../styles/Mixer.css';

interface PeakMeterProps {
    level?: MeterLevel;
    height?: number; // CSS string
    width?: number; // px
}

export function PeakMeter({ level, height = 200, width = 12 }: PeakMeterProps) {
    // Level is 0.0 to 1.0 (linear amplitude)
    // Convert amplitude to dB scale for visual (-60dB to 0dB)
    const toLinearBg = (amp: number) => {
        if (amp <= 0) return '0%';
        const db = 20 * Math.log10(amp);
        const minDb = -60;
        const clamped = Math.max(minDb, Math.min(0, db));
        // map -60..0 to 0..100%
        return `${((clamped - minDb) / -minDb) * 100}%`;
    };

    const lRms = level ? toLinearBg(level.rms_left) : '0%';
    const lPeak = level ? toLinearBg(level.peak_left) : '0%';

    return (
        <div
            className="mixer-peak-meter"
            style={{
                '--meter-width': `${width}px`,
                '--meter-height': `${height}px`,
                '--meter-fill-height': lRms,
                '--meter-peak-bottom': lPeak
            } as React.CSSProperties}
        >
            <div className="mixer-meter-bg-gradient" />
            <div className="mixer-meter-fill">
                <div className="mixer-meter-gradient" />
            </div>
            <div className="mixer-meter-peak" />
        </div>
    );
}
