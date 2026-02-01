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
        if (item.processed || item.processing || processingQueue.current.has(item.path)) return;

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
                    processing: false,
                    processed: true
                });
                return;
            }

            // 1. First, Probing Metadata (Fast)
            const ffprobe = Command.sidecar('bin/ffprobe', [
                '-v', 'error',
                '-show_entries', 'format=duration,size,bit_rate,format_name:stream=codec_type,width,height,sample_rate',
                '-print_format', 'json',
                item.path
            ]);
            const probeResult = await ffprobe.execute();
            // console.log(`[useFileProcessing] Probe Result for ${item.name}:`, probeResult);

            let duration = 0;
            let size = 0;
            let bitrate = 0;
            let container = '';
            let width = 0;
            let height = 0;
            let sampleRate = 0;
            let type: 'Video' | 'Audio' | 'Image' = 'Video';

            if (probeResult.code === 0) {
                try {
                    const data = JSON.parse(probeResult.stdout);
                    const format = data.format || {};
                    const videoStream = data.streams?.find((s: any) => s.codec_type === 'video');
                    const audioStream = data.streams?.find((s: any) => s.codec_type === 'audio');

                    duration = parseFloat(format.duration || '0');
                    size = parseInt(format.size || '0', 10);
                    bitrate = parseInt(format.bit_rate || '0', 10);
                    container = format.format_name;

                    if (videoStream) {
                        width = videoStream.width;
                        height = videoStream.height;
                    }

                    if (audioStream) {
                        sampleRate = parseInt(audioStream.sample_rate || '0', 10);
                    }

                    const ext = item.path.split('.').pop()?.toLowerCase();
                    const isImageExt = ['jpg', 'jpeg', 'png', 'webp', 'tiff', 'tif', 'bmp'].includes(ext || '');

                    if (isImageExt) {
                        type = 'Image';
                    } else if (!videoStream && audioStream) {
                        type = 'Audio';
                    } else {
                        type = 'Video';
                    }
                } catch (e) {
                    console.error("Failed to parse ffprobe json:", e);
                }
            }

            // 2. Generate thumbnail if not audio
            if (type !== 'Audio') {
                const hash = await hashString(item.path);
                const sep = navigator.userAgent.includes('Windows') ? '\\' : '/';
                const thumbPath = `${thumbDir}${sep}${hash}.jpg`;

                if (!(await exists(thumbPath))) {
                    const command = Command.sidecar('bin/ffmpeg', [
                        '-y',
                        '-ss', type === 'Image' ? '0' : '00:00:01',
                        '-i', item.path,
                        '-vframes', '1',
                        '-vf', 'scale=320:-1',
                        '-q:v', '4',
                        thumbPath
                    ]);
                    await command.execute();
                }

                if (await exists(thumbPath)) {
                    const fileBytes = await readFile(thumbPath);
                    const base64String = btoa(
                        new Uint8Array(fileBytes)
                            .reduce((data, byte) => data + String.fromCharCode(byte), '')
                    );
                    updateMediaItem(index, {
                        thumbnail: `data:image/jpeg;base64,${base64String}`,
                        duration,
                        type,
                        processing: false,
                        processed: true,
                        size,
                        width,
                        height,
                        sampleRate,
                        bitrate,
                        container
                    });
                    return;
                }
            }

            // If audio or thumbnail failed
            updateMediaItem(index, {
                duration,
                type,
                processing: false,
                processed: true,
                size,
                width,
                height,
                sampleRate,
                bitrate,
                container
            });
        } catch (e) {
            console.error('Processing error:', e);
            updateMediaItem(index, { processing: false, processed: true });
        } finally {
            processingQueue.current.delete(item.path);
        }
    }, [updateMediaItem]);

    useEffect(() => {
        playlist.forEach((item, index) => {
            if (!item.processed && !item.processing) {
                processItem(item, index);
            }
        });
    }, [playlist, processItem]);
}
