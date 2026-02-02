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
        // Skip if already running
        if (item.processing || processingQueue.current.has(item.path)) return;

        // Skip if fully processed (has metadata)
        if (item.processed) {
            const hasMetadata = item.type === 'Image' || (item.fps !== undefined && item.bitrate !== undefined);
            if (hasMetadata) return;
        }

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
                '-show_entries', 'format=duration,size,bit_rate,format_name:stream=codec_type,width,height,sample_rate,r_frame_rate,codec_name,channels,channel_layout,pix_fmt,profile,level,sample_fmt,bit_rate',
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
            let fps = 0;
            let videoCodec = '';
            let audioCodec = '';
            let channels = 0;
            let pixelFormat = '';
            let audioLayout = '';
            let videoProfile = '';
            let audioDepth = '';
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
                        videoCodec = videoStream.codec_name;
                        pixelFormat = videoStream.pix_fmt;
                        if (videoStream.r_frame_rate) {
                            const [num, den] = videoStream.r_frame_rate.split('/').map(Number);
                            if (den > 0) fps = num / den;
                        }
                        // Video Profile (e.g. "Main 4.1")
                        if (videoStream.profile) {
                            videoProfile = videoStream.profile;
                            if (videoStream.level && videoStream.level !== 'unknown') {
                                // converting level 51 -> 5.1 if needed, usually ffprobe gives raw number or string
                                // For H.264, level is often just a number like 41. Let's keep it simple.
                                videoProfile += ` ${videoStream.level}`;
                            }
                        }
                        // Prefer stream bitrate if available, else format bitrate
                        if (videoStream.bit_rate) bitrate = parseInt(videoStream.bit_rate, 10);
                    }

                    if (audioStream) {
                        sampleRate = parseInt(audioStream.sample_rate || '0', 10);
                        audioCodec = audioStream.codec_name;
                        channels = audioStream.channels;
                        audioLayout = audioStream.channel_layout; // e.g. "stereo"
                        audioDepth = audioStream.sample_fmt; // e.g. "fltp"

                        // Map simple layouts if missing
                        if (!audioLayout && channels > 0) {
                            if (channels === 1) audioLayout = 'mono';
                            if (channels === 2) audioLayout = 'stereo';
                        }
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
                        container,
                        fps,
                        videoCodec,
                        audioCodec,
                        channels,
                        pixelFormat,
                        audioLayout,
                        videoProfile,
                        audioDepth
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
                container,
                fps,
                videoCodec,
                audioCodec,
                channels,
                pixelFormat,
                audioLayout,
                videoProfile,
                audioDepth
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
            // Re-process if not processed OR if it's a non-image missing FPS (stale metadata)
            const needsProcessing = !item.processed ||
                (!item.processing && item.type !== 'Image' && item.fps === undefined);

            if (needsProcessing && !item.processing) {
                processItem(item, index);
            }
        });
    }, [playlist, processItem]);

    return { processItem };
}
