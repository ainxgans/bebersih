import React, { useState, useEffect } from "react";
import { RefreshCw, Trash2, Sparkles, DownloadCloud } from "lucide-react";
import { formatBytes } from "../utils/format";
import { checkForUpdateSafe, installUpdateAndRelaunch, AvailableUpdate } from "../utils/updater";

interface HeaderProps {
  isScanning: boolean;
  isCleaning: boolean;
  selectedCount: number;
  selectedBytes: number;
  onScan: () => void;
  onOpenCleanModal: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  isScanning,
  isCleaning,
  selectedCount,
  selectedBytes,
  onScan,
  onOpenCleanModal,
}) => {
  const [availableUpdate, setAvailableUpdate] = useState<AvailableUpdate | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<string | null>(null);

  // Automatically check for releases on mount
  useEffect(() => {
    let isMounted = true;
    checkForUpdateSafe().then((up) => {
      if (isMounted && up) {
        setAvailableUpdate(up);
      }
    });
    return () => {
      isMounted = false;
    };
  }, []);

  const handleApplyUpdate = async () => {
    if (!availableUpdate || isUpdating) return;
    setIsUpdating(true);
    try {
      await installUpdateAndRelaunch(availableUpdate, (msg) => {
        setUpdateStatus(msg);
      });
    } catch (err) {
      setUpdateStatus("Update failed");
      setIsUpdating(false);
    }
  };

  return (
    <header className="border-b border-slate-800/80 bg-slate-900/60 backdrop-blur-md sticky top-0 z-30 px-6 py-4">
      <div className="max-w-7xl mx-auto flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        {/* App Title & Tagline */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center text-emerald-400 shadow-sm shadow-emerald-950">
            <Sparkles className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <h1 className="text-xl font-bold tracking-tight text-slate-100">
                Bebersih
              </h1>
              <span className="text-xs font-semibold px-2 py-0.5 rounded-full bg-slate-800 text-slate-400 border border-slate-700/60">
                v0.1.0
              </span>

              {availableUpdate && (
                <button
                  type="button"
                  onClick={handleApplyUpdate}
                  disabled={isUpdating}
                  className="inline-flex items-center gap-1.5 px-2.5 py-0.5 text-xs font-semibold rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 hover:bg-emerald-500/30 transition-all cursor-pointer animate-pulse"
                  title={`Install version ${availableUpdate.version} and restart`}
                >
                  <DownloadCloud className={`w-3.5 h-3.5 ${isUpdating ? "animate-bounce" : ""}`} />
                  <span>
                    {isUpdating ? updateStatus || "Updating..." : `Update v${availableUpdate.version}`}
                  </span>
                </button>
              )}
            </div>
            <p className="text-xs text-slate-400">
              Ubuntu Developer &amp; System Cache Cleaner
            </p>
          </div>
        </div>

        {/* Global Action Buttons */}
        <div className="flex items-center gap-3 w-full sm:w-auto justify-end">
          <button
            type="button"
            onClick={onScan}
            disabled={isScanning || isCleaning}
            className="flex-1 sm:flex-initial inline-flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium rounded-lg border border-slate-700/70 bg-slate-800/80 hover:bg-slate-700/80 text-slate-200 transition-all duration-150 active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none shadow-sm"
          >
            <RefreshCw
              className={`w-4 h-4 text-slate-400 ${
                isScanning ? "animate-spin text-emerald-400" : ""
              }`}
            />
            <span>{isScanning ? "Scanning..." : "Scan Now"}</span>
          </button>

          <button
            type="button"
            onClick={onOpenCleanModal}
            disabled={isScanning || isCleaning || selectedCount === 0}
            className="flex-1 sm:flex-initial inline-flex items-center justify-center gap-2 px-4 py-2 text-sm font-medium rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white transition-all duration-150 active:scale-[0.98] disabled:opacity-40 disabled:pointer-events-none shadow-sm shadow-emerald-950"
          >
            <Trash2 className="w-4 h-4" />
            <span>
              Clean Selected{" "}
              {selectedCount > 0 ? (
                <span className="font-mono text-xs opacity-90">
                  ({formatBytes(selectedBytes)})
                </span>
              ) : null}
            </span>
          </button>
        </div>
      </div>
    </header>
  );
};
