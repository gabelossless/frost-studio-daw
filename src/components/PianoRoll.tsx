import { useState, useRef } from 'react';
import { useDawStore } from '../store/useDawStore';
import { MousePointer2, Eraser, Scissors } from 'lucide-react';
import '../styles/PianoRoll.css';

const MIDI_NOTES = ['B', 'A#', 'A', 'G#', 'G', 'F#', 'F', 'E', 'D#', 'D', 'C#', 'C'];
const OCTAVES = [8, 7, 6, 5, 4, 3, 2, 1, 0];
const PIANO_KEYS = OCTAVES.flatMap(octave => MIDI_NOTES.map(note => `${note}${octave}`));

const CELL_WIDTH = 40;
const CELL_HEIGHT = 20;
const KEY_WIDTH = 60;

const SCALES = {
    'Major': [0, 2, 4, 5, 7, 9, 11],
    'Minor': [0, 2, 3, 5, 7, 8, 10],
    'Dorian': [0, 2, 3, 5, 7, 9, 10],
    'Phrygian': [0, 1, 3, 5, 7, 8, 10],
    'Lydian': [0, 2, 4, 6, 7, 9, 11],
    'Mixolydian': [0, 2, 4, 5, 7, 9, 10],
    'Aeolian': [0, 2, 3, 5, 7, 8, 10],
    'Locrian': [0, 1, 3, 5, 6, 8, 10],
    'Blues': [0, 3, 5, 6, 7, 10]
};

const NOTE_OFFSETS: { [key: string]: number } = {
    'C': 0, 'C#': 1, 'D': 2, 'D#': 3, 'E': 4, 'F': 5,
    'F#': 6, 'G': 7, 'G#': 8, 'A': 9, 'A#': 10, 'B': 11
};

export default function PianoRoll() {
    const {
        notes,
        addNote,
        updateNote,
        deleteNote,
        selectedClipId,
        gridSize,
        clips
    } = useDawStore();

    const currentClip = clips.find(c => c.id === selectedClipId);
    const clipNotes = notes.filter(n => n.clipId === selectedClipId);

    const [isDrawing, setIsDrawing] = useState(false);
    const [dragMode, setDragMode] = useState<'move' | 'resize' | 'velocity' | null>(null);
    const [activeNoteId, setActiveNoteId] = useState<string | null>(null);
    const scrollRef = useRef<HTMLDivElement>(null);

    // Scale Highlighting State
    const [rootNote, setRootNote] = useState('C');
    const [scaleType, setScaleType] = useState<'Major' | 'Minor'>('Minor');

    const isInScale = (pitch: string) => {
        const noteName = pitch.replace(/[0-9]/g, '');
        const offset = NOTE_OFFSETS[noteName];
        const rootOffset = NOTE_OFFSETS[rootNote];
        const interval = (offset - rootOffset + 12) % 12;
        return SCALES[scaleType].includes(interval);
    };

    const handleMouseDown = (e: React.MouseEvent, pitch: string, beat: number) => {
        if (e.button === 2) return; // Right click handled separately

        const existingNote = clipNotes.find(n => n.pitch === pitch && beat >= n.start && beat < n.start + n.duration);

        if (existingNote) {
            setActiveNoteId(existingNote.id);
            const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
            const edgeThreshold = 10;
            if (e.clientX > rect.right - edgeThreshold) {
                setDragMode('resize');
            } else {
                setDragMode('move');
            }
        } else {
            const newNoteId = `n${Date.now()}`;
            addNote({
                clipId: selectedClipId!,
                pitch,
                start: Math.floor(beat / gridSize) * gridSize,
                duration: gridSize,
                velocity: 100
            });
            setActiveNoteId(newNoteId);
            setDragMode('resize');
        }
        setIsDrawing(true);
    };

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!isDrawing || !activeNoteId || !currentClip) return;

        const container = scrollRef.current;
        if (!container) return;

        const rect = container.getBoundingClientRect();
        const x = e.clientX - rect.left + container.scrollLeft - KEY_WIDTH;
        const y = e.clientY - rect.top + container.scrollTop;

        const currentBeat = Math.max(0, x / CELL_WIDTH);
        const snappedBeat = Math.floor(currentBeat / gridSize) * gridSize;

        const noteIndex = Math.floor(y / CELL_HEIGHT);
        const currentPitch = PIANO_KEYS[noteIndex];

        if (dragMode === 'resize') {
            const note = notes.find(n => n.id === activeNoteId);
            if (note) {
                const newDuration = Math.max(gridSize, snappedBeat - note.start + gridSize);
                updateNote(activeNoteId, { duration: newDuration });
            }
        } else if (dragMode === 'move') {
            updateNote(activeNoteId, {
                start: snappedBeat,
                pitch: currentPitch
            });
        } else if (dragMode === 'velocity') {
            const lane = document.getElementById('velocity-lane-canvas');
            if (lane) {
                const rect = lane.getBoundingClientRect();
                const y = Math.max(0, Math.min(rect.height, e.clientY - rect.top));
                const percent = 1.0 - (y / rect.height);
                const newVel = Math.floor(percent * 127);
                updateNote(activeNoteId, { velocity: Math.max(1, newVel) });
            }
        }
    };

    const handleMouseUp = () => {
        setIsDrawing(false);
        setDragMode(null);
        setActiveNoteId(null);
    };

    const handleContextMenu = (e: React.MouseEvent, noteId: string) => {
        e.preventDefault();
        deleteNote(noteId);
    };

    const handleChopToGrid = () => {
        if (!currentClip) return;
        
        // Iterate over notes and subdivide
        clipNotes.forEach(note => {
            if (note.duration > gridSize) {
                const divisions = Math.floor(note.duration / gridSize);
                if (divisions > 1) {
                    deleteNote(note.id);
                    for (let i = 0; i < divisions; i++) {
                        addNote({
                            clipId: selectedClipId!,
                            pitch: note.pitch,
                            start: note.start + (i * gridSize),
                            duration: gridSize,
                            velocity: note.velocity
                        });
                    }
                }
            }
        });
    };

    if (!selectedClipId) {
        return (
            <div className="flex-1 flex items-center justify-center bg-[#121214] text-gray-500 italic">
                Select a clip to edit notes
            </div>
        );
    }

    return (
        <div className="flex flex-col h-full bg-[#121214] border-t border-[#27272a] Frost-UI">
            {/* Toolbar */}
            <div className="h-10 border-b border-[#27272a] bg-[#1a1a1e] flex items-center px-4 gap-4">
                <div className="flex bg-[#27272a] rounded p-0.5">
                    <button className="p-1 px-2 bg-indigo-600 rounded shadow-sm" title="Select Tool"><MousePointer2 size={14} /></button>
                    <button className="p-1 px-2 hover:bg-[#3f3f46] transition-colors" title="Eraser Tool"><Eraser size={14} /></button>
                </div>

                <div className="h-4 w-px bg-gray-700"></div>

                <div className="flex items-center gap-2 text-[10px] text-gray-400 font-bold uppercase tracking-wider">
                    Snap:
                    <select
                        className="bg-[#27272a] border border-[#3f3f46] rounded text-white px-1 outline-none"
                        value={gridSize}
                        onChange={(e) => useDawStore.getState().setGridSize(parseFloat(e.target.value))}
                        title="Grid Snap Size"
                    >
                        <option value={1}>1/1</option>
                        <option value={0.5}>1/2</option>
                        <option value={0.25}>1/4</option>
                        <option value={0.125}>1/8</option>
                        <option value={0.0625}>1/16</option>
                    </select>
                </div>

                <div className="h-4 w-px bg-gray-700"></div>

                <button 
                    onClick={handleChopToGrid} 
                    className="p-1 px-2 bg-indigo-600 hover:bg-indigo-500 rounded text-white text-[10px] font-bold flex items-center gap-1 shadow-sm transition-colors"
                    title="Chop all notes down to the current grid snap size"
                >
                    <Scissors size={12} /> Chop to Grid
                </button>

                <div className="h-4 w-px bg-gray-700"></div>

                <div className="flex items-center gap-2 text-[10px] text-gray-400 font-bold uppercase tracking-wider">
                    Scale:
                    <select
                        className="bg-[#27272a] border border-[#3f3f46] rounded text-white px-1 outline-none"
                        value={rootNote}
                        onChange={(e) => setRootNote(e.target.value)}
                        title="Root Note"
                    >
                        {Object.keys(NOTE_OFFSETS).map(n => <option key={n} value={n}>{n}</option>)}
                    </select>
                    <select
                        className="bg-[#27272a] border border-[#3f3f46] rounded text-white px-1 outline-none"
                        value={scaleType}
                        onChange={(e) => setScaleType(e.target.value as any)}
                        title="Scale Type"
                    >
                        {Object.keys(SCALES).map(s => <option key={s} value={s}>{s}</option>)}
                    </select>
                </div>
            </div>

            {/* Editor Area */}
            <div className="flex-1 flex overflow-hidden relative" ref={scrollRef}>
                {/* Keyboard Sidebar */}
                <div className="w-[60px] bg-[#1a1a1e] border-r border-[#27272a] z-20 sticky left-0 overflow-y-auto custom-scrollbar no-scrollbar">
                    {PIANO_KEYS.map((key) => {
                        const isBlack = key.includes('#');
                        return (
                            <div
                                key={key}
                                className={`h-[20px] w-full border-b border-[#27272a] flex items-center justify-end pr-1 text-[8px] font-bold ${isBlack ? 'bg-black text-gray-400' : 'bg-white text-gray-800'}`}
                            >
                                {key.startsWith('C') && !isBlack ? key : ''}
                            </div>
                        );
                    })}
                </div>

                {/* Grid Canvas */}
                <div
                    className="flex-1 overflow-auto custom-scrollbar relative"
                    onMouseMove={handleMouseMove}
                    onMouseUp={handleMouseUp}
                    onMouseLeave={handleMouseUp}
                >
                    <div
                        className="piano-roll-grid"
                        style={{
                            '--grid-width': `${(currentClip?.duration || 16) * CELL_WIDTH}px`,
                            '--grid-height': `${PIANO_KEYS.length * CELL_HEIGHT}px`,
                            '--cell-width': `${CELL_WIDTH}px`,
                            '--cell-height': `${CELL_HEIGHT}px`
                        } as React.CSSProperties}
                    >
                        {/* Scale Highlighting Backdrop */}
                        {PIANO_KEYS.map((key, index) => {
                            const inScale = isInScale(key);
                            return (
                                <div
                                    key={`bg-${key}`}
                                    className={`absolute left-0 right-0 h-[20px] pointer-events-none ${!inScale ? 'bg-black/40' : 'bg-transparent'}`}
                                    style={{ top: `${index * CELL_HEIGHT}px` }}
                                />
                            );
                        })}

                        {/* Note Rendering */}
                        {clipNotes.map((note) => {
                            const pitchIndex = PIANO_KEYS.indexOf(note.pitch);
                            if (pitchIndex === -1) return null;

                            return (
                                <div
                                    key={note.id}
                                    onMouseDown={(e) => handleMouseDown(e, note.pitch, note.start)}
                                    onContextMenu={(e) => handleContextMenu(e, note.id)}
                                    className="piano-roll-note group"
                                    style={{
                                        '--note-left': `${note.start * CELL_WIDTH}px`,
                                        '--note-top': `${pitchIndex * CELL_HEIGHT}px`,
                                        '--note-width': `${note.duration * CELL_WIDTH}px`,
                                        '--note-height': `${CELL_HEIGHT - 1}px`
                                    } as React.CSSProperties}
                                >
                                    <div className="piano-roll-note-resizer hover:bg-white/20" />
                                </div>
                            );
                        })}

                        {/* Click Surface */}
                        <div
                            className="absolute inset-0 z-0"
                            onMouseDown={(e) => {
                                const rect = e.currentTarget.getBoundingClientRect();
                                const x = e.clientX - rect.left;
                                const y = e.clientY - rect.top;
                                const beat = x / CELL_WIDTH;
                                const pitchIndex = Math.floor(y / CELL_HEIGHT);
                                handleMouseDown(e, PIANO_KEYS[pitchIndex], beat);
                            }}
                        />
                    </div>

                    {/* Velocity Lane drawer */}
                    <div 
                        className="sticky bottom-0 z-30 bg-[#141417]/95 border-t border-[#27272a] backdrop-blur-sm shadow-[0_-4px_12px_rgba(0,0,0,0.5)] flex"
                        style={{ height: '80px', width: `${(currentClip?.duration || 16) * CELL_WIDTH + 60}px` }}
                    >
                        {/* Lane Label */}
                        <div className="w-[60px] bg-[#1a1a1e] border-r border-[#27272a] h-full flex items-center justify-center text-[8px] font-bold text-gray-500 uppercase tracking-wider sticky left-0 z-40">
                            Velocity
                        </div>
                        
                        {/* Bars Canvas */}
                        <div id="velocity-lane-canvas" className="flex-1 h-full relative" style={{ width: `${(currentClip?.duration || 16) * CELL_WIDTH}px` }}>
                            {clipNotes.map((note) => {
                                const barL = note.start * CELL_WIDTH + 4; // slight offset centering
                                const barH = (note.velocity / 127) * 60; // scale to container height-padding
                                return (
                                    <div
                                        key={`vel-${note.id}`}
                                        className="absolute bottom-1 w-2 rounded-t bg-indigo-500 hover:bg-indigo-400 cursor-ns-resize transition-all shadow-[0_0_4px_rgba(99,102,241,0.4)]Group"
                                        style={{ 
                                            left: `${barL}px`, 
                                            height: `${Math.max(4, barH)}px`,
                                            opacity: note.velocity / 127 * 0.8 + 0.2 
                                        }}
                                        title={`Velocity: ${note.velocity}`}
                                        onMouseDown={(e) => {
                                            e.stopPropagation();
                                            setActiveNoteId(note.id);
                                            setDragMode('velocity');
                                        }}
                                    />
                                );
                            })}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
