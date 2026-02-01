import { Volume2, VolumeX, Volume1 } from "lucide-react";
import { usePlayback } from "../hooks/usePlayback";

export const VolumeControl = () => {
    const { volume, isMuted, handleSetVolume, handleToggleMute } = usePlayback();

    const getVolumeIcon = () => {
        if (isMuted || volume === 0) return <VolumeX size={18} />;
        if (volume < 0.5) return <Volume1 size={18} />;
        return <Volume2 size={18} />;
    };

    return (
        <div className="flex items-center gap-2 group relative">
            <button
                onClick={handleToggleMute}
                className={`p-2 rounded-lg transition-colors ${isMuted ? "text-red-400 bg-red-400/10" : "text-gray-400 hover:text-white hover:bg-white/5"
                    }`}
            >
                {getVolumeIcon()}
            </button>

            <div className="w-0 group-hover:w-24 overflow-hidden transition-all duration-300 ease-in-out flex items-center">
                <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    value={isMuted ? 0 : volume}
                    onChange={(e) => handleSetVolume(parseFloat(e.target.value))}
                    className="w-24 h-1 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-yellow-400"
                />
            </div>
        </div>
    );
};
