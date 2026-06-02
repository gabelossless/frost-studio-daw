const fs = require('fs');
const path = require('path');

const generatePresets = (name, count, folder) => {
    for (let i = 1; i <= count; i++) {
        const patch = {
            name: `${name}_Patch_${i.toString().padStart(2, '0')}`,
            category: i < 15 ? 'Bass' : i < 35 ? 'Lead' : 'Pad',
            cutoff: Math.random() * 0.8 + 0.1,
            resonance: Math.random() * 0.6,
            attack: Math.random() * 0.1,
            decay: Math.random() * 0.4 + 0.1,
            sustain: Math.random() * 0.5 + 0.3,
            release: Math.random() * 0.5 + 0.1,
            osc_mix: Math.random()
        };
        fs.writeFileSync(path.join(folder, `patch_${i.toString().padStart(2, '0')}.json`), JSON.stringify(patch, null, 2));
    }
};

const baseDir = 'src-tauri/presets';
generatePresets('SUMMIT', 50, path.join(baseDir, 'summit'));
generatePresets('ERUPTION', 50, path.join(baseDir, 'eruption'));
generatePresets('NEBULA', 50, path.join(baseDir, 'nebula'));

console.log('Generated 150 presets.');
