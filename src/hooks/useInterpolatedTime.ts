import { useEffect, useRef } from "react";

/**
 * useInterpolatedTime
 * 
 * Linearly interpolates time between backend updates to provide 60fps+ UI smoothness.
 * Backend updates (typically ~24fps) serve as "sync anchors" to correct drift.
 * 
 * @param backendTime - The raw time received from Rust/Video Engine
 * @param isPlaying - Whether the engine is currently playing
 * @param onUpdate - Callback to update the DOM directly (bypassing React render cycle)
 */
export const useInterpolatedTime = (
    backendTime: number,
    isPlaying: boolean,
    onUpdate: (time: number) => void
) => {
    // Refs to hold the "truth" without triggering re-renders
    const lastBackendTimeRef = useRef(backendTime);
    const lastUpdateTimestampRef = useRef(performance.now());
    const visualTimeRef = useRef(backendTime);

    // Sync Ref with Prop updates (Backend Heartbeat)
    useEffect(() => {
        const now = performance.now();
        lastBackendTimeRef.current = backendTime;
        lastUpdateTimestampRef.current = now;

        // Calculate drift
        const currentVisual = visualTimeRef.current;
        const drift = Math.abs(currentVisual - backendTime);

        // DEBUG LOG: Analyze why glitches happen
        if (drift > 0.05) {
            // console.log(`[Sync] Backend Update: ${backendTime.toFixed(3)}s | Visual: ${currentVisual.toFixed(3)}s | Drift: ${drift.toFixed(3)}s`);
        }

        // Drift Correction:
        // If large drift (> 0.5s), it's likely a SEEK or a major Lag Spike.
        // We Hard Snap to the backend time. 
        // We use 0.5s (relaxed from 0.1s) to avoid snapping back when the backend just lags slightly.
        if (drift > 0.5) {
            // console.warn(`[Sync] Hard Snap! Drift ${drift.toFixed(3)}s > 0.5s`);
            visualTimeRef.current = backendTime;
            onUpdate(backendTime);
        }
    }, [backendTime, onUpdate]);

    // Animation Loop
    useEffect(() => {
        if (!isPlaying) return;

        let frameId: number;

        const loop = () => {
            const now = performance.now();

            // How much time has passed since the last backend update?
            const elapsedSinceUpdate = (now - lastUpdateTimestampRef.current) / 1000; // ms to s

            // Project where we should be: Anchor + Elapsed
            const projected = lastBackendTimeRef.current + elapsedSinceUpdate;

            // Enforce Monotonicity: Never rewind time unless a seek happened (handled by drift check above)
            // If backend lags (so 'projected' calculates to a time in the past), 
            // we just HOLD the current frame until the backend catches up.
            const nextTime = Math.max(projected, visualTimeRef.current);

            visualTimeRef.current = nextTime;
            onUpdate(nextTime);

            frameId = requestAnimationFrame(loop);
        };

        frameId = requestAnimationFrame(loop);

        return () => cancelAnimationFrame(frameId);
    }, [isPlaying, onUpdate]);
};
