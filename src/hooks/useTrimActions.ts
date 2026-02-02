import { useCallback, useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { Command } from '@tauri-apps/plugin-shell';
import { useStore } from '../store/useStore';

export const useTrimActions = () => {
    const isPlaying = useStore((state) => state.isPlaying);
    const storeSetIsPlaying = useStore((state) => state.setIsPlaying);
    const currentTime = useStore((state) => state.currentTime);
    const currentIndex = useStore((state) => state.currentIndex);
    const playlist = useStore((state) => state.playlist);
    const updateMediaItem = useStore((state) => state.updateMediaItem);

    const [isExportingClip, setIsExportingClip] = useState(false);

    // Get current item's trim state
    const currentItem = currentIndex !== null ? playlist[currentIndex] : null;
    const trimStart = currentItem?.trimStart;
    const trimEnd = currentItem?.trimEnd;

    const setMarkIn = useCallback(() => {
        if (currentIndex === null || !currentItem) return;

        // Validation: Start must be before End
        if (trimEnd !== undefined && currentTime >= trimEnd) {
            // If dragging start past end, reset end? or block? 
            // For now, let's just update start. If it conflicts, maybe clear end?
            // Pro behavior: Shift end? No, usually Start > End is invalid.
            updateMediaItem(currentIndex, { trimStart: currentTime, trimEnd: undefined });
        } else {
            updateMediaItem(currentIndex, { trimStart: currentTime });
        }
    }, [currentIndex, currentItem, currentTime, trimEnd, updateMediaItem]);

    const setMarkOut = useCallback(() => {
        if (currentIndex === null || !currentItem) return;

        // Validation: End must be after Start
        if (trimStart !== undefined && currentTime <= trimStart) {
            // If cursor before start, maybe update start instead? 
            // Or just ignore/clear start.
            updateMediaItem(currentIndex, { trimEnd: currentTime, trimStart: undefined });
        } else {
            updateMediaItem(currentIndex, { trimEnd: currentTime });
        }
    }, [currentIndex, currentItem, currentTime, trimStart, updateMediaItem]);

    const clearMarks = useCallback(() => {
        if (currentIndex === null) return;
        updateMediaItem(currentIndex, { trimStart: undefined, trimEnd: undefined });
    }, [currentIndex, updateMediaItem]);

    const handleExportClip = useCallback(async () => {
        if (currentIndex === null || !currentItem || currentItem.type !== 'Video') return;
        if (trimStart === undefined || trimEnd === undefined) return;

        // Pause playback
        const wasPlaying = isPlaying;
        if (wasPlaying) storeSetIsPlaying(false);

        try {
            const duration = trimEnd - trimStart;
            if (duration <= 0) return;

            // Suggest filename
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
            const suggestedName = `${currentItem.name?.replace(/\.[^/.]+$/, "")}_clip_${timestamp}.mp4`;

            // Open Save Dialog
            const filePath = await save({
                title: 'Export Clip',
                defaultPath: suggestedName,
                filters: [{
                    name: 'Video',
                    extensions: ['mp4']
                }]
            });

            if (!filePath) {
                if (wasPlaying) storeSetIsPlaying(true);
                return;
            }

            setIsExportingClip(true);

            // Execute FFmpeg Sidecar (Re-encode for accuracy)
            // ffmpeg -ss {start} -i {input} -t {duration} -c:v libx264 -preset fast -crf 23 -c:a aac {output}
            // Note: Using -ss before -i for fast seek to keyframe, then re-encoding from there.
            // This is FRAME ACCURATE because we re-encode.
            const args = [
                '-y',
                '-ss', trimStart.toString(),
                '-i', currentItem.path,
                '-t', duration.toString(),
                '-c:v', 'libx264',
                '-preset', 'fast',
                '-crf', '23',
                '-c:a', 'aac',
                filePath
            ];

            console.log('[Trim] Exporting Clip:', args.join(' '));
            const command = Command.sidecar('bin/ffmpeg', args);
            const output = await command.execute();

            if (output.code !== 0) {
                console.error('[Trim] Export failed:', output.stderr);
            } else {
                console.log('[Trim] Export success:', filePath);
            }

        } catch (error) {
            console.error('[Trim] Error:', error);
        } finally {
            setIsExportingClip(false);
            if (wasPlaying) storeSetIsPlaying(true);
        }
    }, [currentIndex, currentItem, trimStart, trimEnd, isPlaying, storeSetIsPlaying]);

    return {
        trimStart,
        trimEnd,
        setMarkIn,
        setMarkOut,
        clearMarks,
        handleExportClip,
        isExportingClip
    };
};
