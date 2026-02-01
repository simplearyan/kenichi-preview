import { useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Command } from '@tauri-apps/plugin-shell';
import { exists, mkdir, readFile } from '@tauri-apps/plugin-fs';
import { useStore, MediaItem } from '../store/useStore';

export function useFileProcessing() {
    const playlist = useStore((state) => state.playlist);
    const updateMediaItem = useStore((state) => state.updateMediaItem);
    const processingQueue = useRef<Set<string>>(new Set());

    const hashString = async (str: string) => {
        const msgUint8 = new TextEncoder().encode(str);
        const hashBuffer = await crypto.subtle.digest('SHA-256', msgUint8);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    };

    const processItem = useCallback(async (item: MediaItem, index: number) => {
        if (item.thumbnail || processingQueue.current.has(item.path)) return;

        processingQueue.current.add(item.path);
        updateMediaItem(index, { processing: true });

        try {
            const cacheDir = await invoke<string>('get_app_cache_dir');
            const thumbDir = `${cacheDir}${navigator.userAgent.includes('Windows') ? '\\' : '/'}thumbnails`;

            // Ensure directory exists
            if (!(await exists(thumbDir))) {
                await mkdir(thumbDir, { recursive: true });
            }

            const hash = await hashString(item.path);
            const sep = navigator.userAgent.includes('Windows') ? '\\' : '/';
            const thumbPath = `${thumbDir}${sep}${hash}.jpg`;

            // Check if already exists in cache
            if (await exists(thumbPath)) {
                const fileBytes = await readFile(thumbPath);
                const base64String = btoa(
                    new Uint8Array(fileBytes)
                        .reduce((data, byte) => data + String.fromCharCode(byte), '')
                );
                updateMediaItem(index, {
                    thumbnail: `data:image/jpeg;base64,${base64String}`,
                    processing: false
                });
                return;
            }

            // Generate thumbnail using FFmpeg sidecar
            const command = Command.sidecar('bin/ffmpeg', [
                '-y',
                '-ss', '00:00:01', // Sample at 1s
                '-i', item.path,
                '-vframes', '1',
                '-vf', 'scale=320:-1', // Scale to width 320, maintain aspect
                '-q:v', '4', // Quality 4 (1-31, lower is better)
                thumbPath
            ]);

            const result = await command.execute();

            if (result.code === 0) {
                // Get duration too
                const ffprobe = Command.sidecar('bin/ffprobe', [
                    '-v', 'error',
                    '-show_entries', 'format=duration',
                    '-of', 'default=noprint_wrappers=1:nokey=1',
                    item.path
                ]);
                const probeResult = await ffprobe.execute();
                let duration = 0;
                if (probeResult.code === 0) {
                    duration = parseFloat(probeResult.stdout);
                }

                const fileBytes = await readFile(thumbPath);
                const base64String = btoa(
                    new Uint8Array(fileBytes)
                        .reduce((data, byte) => data + String.fromCharCode(byte), '')
                );

                updateMediaItem(index, {
                    thumbnail: `data:image/jpeg;base64,${base64String}`,
                    duration,
                    processing: false
                });
            } else {
                console.error('FFmpeg error:', result.stderr);
                updateMediaItem(index, { processing: false });
            }
        } catch (e) {
            console.error('Processing error:', e);
            updateMediaItem(index, { processing: false });
        } finally {
            processingQueue.current.delete(item.path);
        }
    }, [updateMediaItem]);

    useEffect(() => {
        playlist.forEach((item, index) => {
            if (!item.thumbnail && !item.processing) {
                processItem(item, index);
            }
        });
    }, [playlist, processItem]);
}
