import { useStore } from "../../store/useStore";
import { Disc, Layers, Monitor, Speaker } from "lucide-react";

export const MetadataOverlay = () => {
    const showMetadata = useStore((state) => state.showMetadata);
    const playlist = useStore((state) => state.playlist);
    const currentIndex = useStore((state) => state.currentIndex);
    const currentItem = currentIndex !== null ? playlist[currentIndex] : null;

    if (!showMetadata || !currentItem) return null;

    return (
        <div className="absolute top-6 right-6 z-50 animate-in fade-in slide-in-from-right-4 duration-500 max-w-xs">
            <div className="flex flex-col gap-1">
                <span className="text-[10px] font-bold text-brand-yellow/50 uppercase tracking-[0.2em] mb-1 text-right">
                    Stream Info
                </span>

                <div className="bg-pro-black/80 backdrop-blur-xl border border-white/10 rounded-xl p-4 shadow-2xl flex flex-col gap-4">

                    {/* Video Stream */}
                    {(currentItem.type === 'Video' || currentItem.type === 'Image') && (
                        <div className="space-y-2">
                            <div className="flex items-center gap-2 text-white/90 pb-1 border-b border-white/5">
                                <Monitor className="w-4 h-4 text-brand-yellow" />
                                <span className="text-xs font-bold uppercase tracking-wider">Video Stream</span>
                            </div>

                            <div className="grid grid-cols-2 gap-x-4 gap-y-2 text-[11px]">
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Codec</span>
                                    <span className="text-white/80 font-mono">{currentItem.videoCodec?.toUpperCase() || "N/A"}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Profile</span>
                                    <span className="text-white/80 font-mono truncate">{currentItem.videoProfile || "N/A"}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Resolution</span>
                                    <span className="text-white/80 font-mono">{currentItem.width}x{currentItem.height}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">FPS</span>
                                    <span className="text-white/80 font-mono">{currentItem.fps ? currentItem.fps.toFixed(2) : "N/A"}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Pixel Fmt</span>
                                    <span className="text-white/80 font-mono">{currentItem.pixelFormat || "N/A"}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Bitrate</span>
                                    <span className="text-white/80 font-mono">
                                        {currentItem.bitrate ? `${(currentItem.bitrate / 1_000_000).toFixed(1)} Mbps` : "N/A"}
                                    </span>
                                </div>

                                {(currentItem.colorSpace || currentItem.colorPrimaries) && (
                                    <div className="flex flex-col col-span-2 mt-1">
                                        <span className="text-white/40 uppercase text-[9px]">Colorimetry</span>
                                        <span className="text-white/60 font-mono text-[10px]">
                                            {currentItem.colorSpace}/{currentItem.colorPrimaries}/{currentItem.colorRange}
                                        </span>
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    {/* Audio Stream */}
                    {(currentItem.type === 'Video' || currentItem.type === 'Audio') && (
                        <div className="space-y-2">
                            <div className="flex items-center gap-2 text-white/90 pb-1 border-b border-white/5">
                                <Speaker className="w-4 h-4 text-brand-yellow" />
                                <span className="text-xs font-bold uppercase tracking-wider">Audio Stream</span>
                            </div>

                            <div className="grid grid-cols-2 gap-x-4 gap-y-2 text-[11px]">
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Codec</span>
                                    <span className="text-white/80 font-mono">{currentItem.audioCodec?.toUpperCase() || "N/A"}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Sample Rate</span>
                                    <span className="text-white/80 font-mono">
                                        {currentItem.sampleRate ? `${currentItem.sampleRate} Hz` : "N/A"}
                                    </span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Channels</span>
                                    <span className="text-white/80 font-mono">{currentItem.channels || "N/A"}</span>
                                </div>
                                <div className="flex flex-col">
                                    <span className="text-white/40 uppercase text-[9px]">Layout</span>
                                    <span className="text-white/80 font-mono truncate">{currentItem.audioLayout || "N/A"}</span>
                                </div>
                                <div className="flex flex-col col-span-2">
                                    <span className="text-white/40 uppercase text-[9px]">Format</span>
                                    <span className="text-white/80 font-mono">{currentItem.audioDepth || "N/A"}</span>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Container Info */}
                    <div className="pt-2 border-t border-white/5 flex items-center justify-between opacity-50">
                        <div className="flex items-center gap-1.5">
                            <Disc className="w-3 h-3" />
                            <span className="text-[10px] font-mono uppercase">{currentItem.container || "Unknown"}</span>
                        </div>
                        <div className="flex items-center gap-1.5">
                            <Layers className="w-3 h-3" />
                            <span className="text-[10px] font-mono">{(currentItem.size ? (currentItem.size / 1024 / 1024).toFixed(1) : 0)} MB</span>
                        </div>
                    </div>

                </div>
            </div>
        </div>
    );
};
