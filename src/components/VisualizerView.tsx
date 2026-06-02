import { useEffect, useRef, useState } from 'react';
import { useDawStore } from '../store/useDawStore';
import * as THREE_LIB from 'three';
import { Upload, Video, Share2, Smartphone } from 'lucide-react';

export default function VisualizerView() {
    const mountRef = useRef<HTMLDivElement>(null);
    const { meters, isPlaying } = useDawStore();
    const [bgImage, setBgImage] = useState<string | null>(null);
    const [isVertical, setIsVertical] = useState(false);
    const [isRecording, setIsRecording] = useState(false);
    const sceneRef = useRef<THREE_LIB.Scene | null>(null);
    const rendererRef = useRef<THREE_LIB.WebGLRenderer | null>(null);
    const mediaRecorderRef = useRef<MediaRecorder | null>(null);
    const recordedChunksRef = useRef<Blob[]>([]);

    const uniformsRef = useRef<{
        uTime: { value: number };
        uBass: { value: number };
        uMid: { value: number };
        uHigh: { value: number };
    }>({
        uTime: { value: 0 },
        uBass: { value: 0 },
        uMid: { value: 0 },
        uHigh: { value: 0 }
    });

    useEffect(() => {
        if (!mountRef.current) return;

        // --- 1. Setup ---
        const width = mountRef.current.clientWidth;
        const height = mountRef.current.clientHeight;

        const scene = new THREE_LIB.Scene();
        scene.background = new THREE_LIB.Color('#030304');

        const camera = new THREE_LIB.PerspectiveCamera(60, width / height, 0.1, 1000);
        camera.position.z = 4;

        const renderer = new THREE_LIB.WebGLRenderer({ antialias: true, alpha: true, preserveDrawingBuffer: true });
        renderer.setSize(width, height);
        renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        mountRef.current.appendChild(renderer.domElement);
        rendererRef.current = renderer;

        // --- 2. Custom Shader Mesh ---
        const vertexShader = `
            uniform float uTime;
            uniform float uBass;
            varying vec2 vUv;
            varying vec3 vNormal;

            // Simple noise function
            vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
            vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
            vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }
            float snoise(vec2 v) {
                const vec4 C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
                vec2 i  = floor(v + dot(v, C.yy) );
                vec2 x0 = v -   i + dot(i, C.xx);
                vec2 i1;
                i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
                vec4 x12 = x0.xyxy + C.xxzz;
                x12.xy -= i1;
                i = mod289(i);
                vec3 p = permute( permute( i.y + vec3(0.0, i1.y, 1.0 )) + i.x + vec3(0.0, i1.x, 1.0 ));
                vec3 m = max(0.5 - vec3(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), 0.0);
                m = m*m ;
                m = m*m ;
                vec3 x = 2.0 * fract(p * C.www) - 1.0;
                vec3 h = abs(x) - 0.5;
                vec3 a0 = x - floor(x + 0.5);
                vec3 m0 = 1.79284291400159 - 0.85373472095314 * ( a0*a0 + h*h );
                vec3 g;
                g.x  = a0.x  * x0.x  + h.x  * x0.y;
                g.yz = a0.yz * x12.xz + h.yz * x12.yw;
                return 130.0 * dot(m * m0, g);
            }

            void main() {
                vUv = uv;
                vNormal = normal;
                
                // Displace vertex based on noise and bass
                float noise = snoise(uv * 3.0 + uTime * 0.5);
                vec3 pos = position + normal * noise * uBass * 0.4;
                
                gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
            }
        `;

        const fragmentShader = `
            uniform float uTime;
            uniform float uBass;
            varying vec2 vUv;
            varying vec3 vNormal;

            void main() {
                // Glow effect based on normal orientation and bass pulse
                float intensity = pow(1.0 - dot(vNormal, vec3(0, 0, 1.0)), 2.0);
                vec3 color = vec3(0.3, 0.1, 0.8) * intensity; // Dark Purple base
                
                // Add reactive neon cyan core
                color += vec3(0.0, 0.8, 0.9) * (uBass * 0.5) * (1.0 - intensity);
                
                gl_FragColor = vec4(color, 1.0);
            }
        `;

        const material = new THREE_LIB.ShaderMaterial({
            vertexShader,
            fragmentShader,
            uniforms: uniformsRef.current,
            wireframe: true,
            transparent: true,
            blending: THREE_LIB.AdditiveBlending
        });

        const geometry = new THREE_LIB.IcosahedronGeometry(1.5, 4);
        const mesh = new THREE_LIB.Mesh(geometry, material);
        scene.add(mesh);

        // --- 3. Particle System ---
        const particleCount = 2000;
        const positions = new Float32Array(particleCount * 3);
        const rand = () => (Math.random() - 0.5) * 10;
        for(let i = 0; i < particleCount * 3; i++) {
            positions[i] = rand();
        }

        const particleGeo = new THREE_LIB.BufferGeometry();
        particleGeo.setAttribute('position', new THREE_LIB.BufferAttribute(positions, 3));

        const particleMat = new THREE_LIB.PointsMaterial({
            color: '#00f2fe',
            size: 0.02,
            transparent: true,
            opacity: 0.6,
            blending: THREE_LIB.AdditiveBlending
        });

        const particles = new THREE_LIB.Points(particleGeo, particleMat);
        scene.add(particles);

        sceneRef.current = scene;

        // --- 3.5 Background Image Mesh ---
        const bgGeometry = new THREE_LIB.PlaneGeometry(16, 9);
        const bgMaterial = new THREE_LIB.MeshBasicMaterial({
            color: '#111',
            transparent: true,
            opacity: 0.1,
            depthWrite: false
        });
        const bgMesh = new THREE_LIB.Mesh(bgGeometry, bgMaterial);
        bgMesh.position.z = -5;
        scene.add(bgMesh);

        // --- 4. Animation Frame ---
        let animationFrameId: number;
        const clock = new THREE_LIB.Clock();

        const animate = () => {
            animationFrameId = requestAnimationFrame(animate);
            const elapsedTime = clock.getElapsedTime();

            // Update time uniform
            uniformsRef.current.uTime.value = elapsedTime;

            // Rotate objects
            mesh.rotation.y = elapsedTime * 0.1;
            mesh.rotation.x = elapsedTime * 0.05;
            particles.rotation.y = -elapsedTime * 0.02;

            // Pulse particles slightly to bass
            const bassScale = 1.0 + uniformsRef.current.uBass.value * 0.1;
            particles.scale.set(bassScale, bassScale, bassScale);

            renderer.render(scene, camera);
        };

        animate();

        // --- 5. Resize Handler ---
        const handleResize = () => {
            if (!mountRef.current) return;
            const w = mountRef.current.clientWidth;
            const h = mountRef.current.clientHeight;
            camera.aspect = w / h;
            camera.updateProjectionMatrix();
            renderer.setSize(w, h);
        };
        window.addEventListener('resize', handleResize);

        // Cleanup
        return () => {
            window.removeEventListener('resize', handleResize);
            cancelAnimationFrame(animationFrameId);
            renderer.dispose();
            geometry.dispose();
            material.dispose();
            particleGeo.dispose();
            particleMat.dispose();
            if (mountRef.current) {
                mountRef.current.removeChild(renderer.domElement);
            }
        };
    }, []);

    // --- 5.5 Setup Background Texture Update ---
    useEffect(() => {
        if (!sceneRef.current || !bgImage) return;

        const loader = new THREE_LIB.TextureLoader();
        loader.load(bgImage, (texture) => {
            if (sceneRef.current) {
                const bgMesh = sceneRef.current.children.find(c => c.type === 'Mesh' && (c as THREE_LIB.Mesh).geometry.type === 'PlaneGeometry') as THREE_LIB.Mesh;
                if (bgMesh) {
                    const mat = bgMesh.material as THREE_LIB.MeshBasicMaterial;
                    mat.map = texture;
                    mat.opacity = 0.4; // subtle background overlay
                    mat.needsUpdate = true;
                }
            }
        });
    }, [bgImage]);

    // Update aspect ratio on isVertical toggle
    useEffect(() => {
        if (!rendererRef.current || !mountRef.current) return;
        const bgMesh = sceneRef.current?.children.find(c => c.type === 'Mesh' && (c as THREE_LIB.Mesh).geometry.type === 'PlaneGeometry') as THREE_LIB.Mesh;
        
        if (bgMesh) {
            if (isVertical) {
                bgMesh.scale.set(0.5, 1.7, 1); // approximate 9:16 fit
            } else {
                bgMesh.scale.set(1, 1, 1);
            }
        }
    }, [isVertical]);

    // --- 6. React Audio Tick Integration ---
    useEffect(() => {
        // Find master peak or RMS level (channel 0 generally signifies main output)
        const masterMeter = meters[0];
        if (masterMeter && isPlaying) {
            // Use average RMS or max RMS of left and right channels
            const rms = (masterMeter.rms_left + masterMeter.rms_right) / 2;
            const norm = Math.max(0, Math.min(1, rms)); // Ensure bounded [0, 1]
            
            // Linear interpolate for smooth mesh frames
            const currentBass = uniformsRef.current.uBass.value;
            uniformsRef.current.uBass.value = currentBass + (norm - currentBass) * 0.3; // smoothing factor
        } else if (!isPlaying) {
             uniformsRef.current.uBass.value *= 0.8; // Decay back down to 0 when paused
        }
    }, [meters, isPlaying]);

    const handleImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file) {
            const url = URL.createObjectURL(file);
            setBgImage(url);
        }
    };

    const handleRecordToggle = () => {
        if (!rendererRef.current) return;
        
        if (isRecording) {
            mediaRecorderRef.current?.stop();
            setIsRecording(false);
        } else {
            const canvas = rendererRef.current.domElement;
            const stream = canvas.captureStream(60); // 60 fps
            recordedChunksRef.current = [];
            
            const options = { mimeType: 'video/webm; codecs=vp9' };
            const recorder = new MediaRecorder(stream, options);
            
            recorder.ondataavailable = (e) => {
                if (e.data.size > 0) {
                    recordedChunksRef.current.push(e.data);
                }
            };
            
            recorder.onstop = () => {
                const blob = new Blob(recordedChunksRef.current, { type: 'video/webm' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.style.display = 'none';
                a.href = url;
                a.download = `frost-viz-${Date.now()}.webm`;
                document.body.appendChild(a);
                a.click();
                setTimeout(() => {
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                }, 100);
            };

            recorder.start();
            mediaRecorderRef.current = recorder;
            setIsRecording(true);
        }
    };

    return (
        <div ref={mountRef} className="w-full h-full relative overflow-hidden bg-[#030304] flex items-center justify-center">
            {/* Top Badge */}
            <div className="absolute top-4 left-4 z-10 bg-black/60 backdrop-blur-md px-3 py-1.5 rounded-lg border border-white/5">
                <span className="text-[10px] font-bold tracking-wider text-indigo-400 uppercase">3D Render Mode: ACTIVE</span>
            </div>

            {/* Bottom Controls Panel */}
            <div className="absolute bottom-4 left-4 right-4 z-10 flex items-center justify-between bg-black/50 backdrop-blur-md p-3 rounded-xl border border-white/5 mx-auto max-w-xl">
                <div className="flex items-center gap-2">
                    <label title="Add Cover Image" className="flex items-center gap-2 px-3 py-1.5 bg-indigo-500/20 hover:bg-indigo-500/30 text-indigo-400 rounded-lg cursor-pointer border border-indigo-500/30 transition-all text-xs font-semibold">
                        <Upload size={14} />
                        <span className="hidden sm:inline">Cover Image</span>
                        <input type="file" accept="image/*" onChange={handleImageUpload} className="hidden" />
                    </label>
                    <button 
                        title="Toggle Vertical (9:16) format"
                        onClick={() => setIsVertical(!isVertical)}
                        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all text-xs font-semibold ${
                            isVertical ? 'bg-[#00f2fe]/20 text-[#00f2fe] border-[#00f2fe]/30' : 'bg-white/5 text-gray-400 border-white/10 hover:bg-white/10'
                        }`}
                    >
                        <Smartphone size={14} />
                        <span className="hidden sm:inline">Vertical</span>
                    </button>
                    {bgImage && (
                        <span className="text-[10px] text-gray-400">BG ON</span>
                    )}
                </div>

                <div className="flex items-center gap-2">
                    <button 
                        title={isRecording ? "Stop Recording" : "Start 1080p WebM Record"}
                        onClick={handleRecordToggle}
                        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-all text-xs font-semibold ${
                            isRecording 
                                ? 'bg-rose-500 animate-pulse text-white border-rose-600' 
                                : 'bg-rose-500/20 hover:bg-rose-500/30 text-rose-400 border border-rose-500/30'
                        }`}
                    >
                        <Video size={14} />
                        <span>{isRecording ? 'Stop' : 'Rec'}</span>
                    </button>
                    <button title="Share link" className="p-1.5 bg-white/5 hover:bg-white/10 rounded-lg transition-all text-gray-400">
                        <Share2 size={14} />
                    </button>
                </div>
            </div>
            
            {/* Overlay Grid */}
            <div className="absolute inset-0 pointer-events-none bg-[radial-gradient(circle_at_center,rgba(0,0,0,0)_0%,rgba(0,0,0,0.8)_100%)]" />

            {/* Force 9:16 framing box overlay if in vertical view to reference social captures clearly */}
            {isVertical && (
                <div className="absolute inset-y-0 w-[56.25vh] max-w-full border-x border-dashed border-[#00f2fe]/40 pointer-events-none z-0" />
            )}
        </div>
    );
}
