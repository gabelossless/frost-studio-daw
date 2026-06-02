// src/audioContext.ts
let ctx: AudioContext | null = null;

/**
 * Returns the singleton AudioContext, creating it on first call.
 */
export function getAudioContext(): AudioContext {
  if (typeof window === 'undefined') return {} as AudioContext;
  if (!ctx) {
    ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
  }
  return ctx;
}

/**
 * Call this from your UI (e.g. when the user presses Play) to make sure the
 * context is running. It is safe to call repeatedly – it will just resume if
 * the context is suspended.
 */
export function resumeAudio(): void {
  if (typeof window === 'undefined') return;
  const audioCtx = getAudioContext();
  if (audioCtx.state === 'suspended') {
    audioCtx.resume().catch(console.error);
  }
}
