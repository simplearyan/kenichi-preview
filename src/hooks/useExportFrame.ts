import { useCallback, useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { Command } from '@tauri-apps/plugin-shell';
import { useStore } from '../store/useStore';

export const useExportFrame = () => {
    const isPlaying = useStore((state) => state.isPlaying);
    const storeSetIsPlaying = useStore((state) => state.setIsPlaying);
    const currentTime = useStore((state) => state.currentTime);
    const currentIndex = useStore((state) => state.currentIndex);
    const playlist = useStore((state) => state.playlist);

    const [isExporting, setIsExporting] = useState(false);

    const handleExportFrame = useCallback(async () => {
        if (currentIndex === null) return;
        const currentItem = playlist[currentIndex];
        if (!currentItem || currentItem.type !== 'Video') return;

        // Pause playback during export
        const wasPlaying = isPlaying;
        if (wasPlaying) storeSetIsPlaying(false);

        try {
            // Suggest filename
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
            const suggestedName = `${currentItem.name?.replace(/\.[^/.]+$/, "")}_snapshot_${timestamp}.png`;

            // Open Save Dialog
            const filePath = await save({
                title: 'Export Frame',
                defaultPath: suggestedName,
                filters: [{
                    name: 'Image',
                    extensions: ['png', 'jpg']
                }]
            });

            if (!filePath) {
                if (wasPlaying) storeSetIsPlaying(true);
                return;
            }

            setIsExporting(true);

            // Execute FFmpeg Sidecar
            // Note: -ss before -i is faster (input seeking)
            const args = [
                '-y',
                '-ss', currentTime.toString(),
                '-i', currentItem.path,
                '-vframes', '1',
                '-q:v', '2', // High quality for JPEG, ignored for PNG but good safety
                filePath
            ];

            console.log('[ExportFrame] Running FFmpeg:', args.join(' '));
            const command = Command.sidecar('bin/ffmpeg', args);
            const output = await command.execute();

            if (output.code !== 0) {
                console.error('[ExportFrame] FFmpeg failed:', output.stderr);
                // Optionally show toast error here
            } else {
                console.log('[ExportFrame] Success:', filePath);
                // Optionally show toast success here
            }

        } catch (error) {
            console.error('[ExportFrame] Error:', error);
        } finally {
            setIsExporting(false);
            if (wasPlaying) storeSetIsPlaying(true);
        }

    }, [currentIndex, playlist, currentTime, isPlaying, storeSetIsPlaying]);

    return {
        handleExportFrame,
        isExporting
    };
};
