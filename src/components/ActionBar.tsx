import React from "react";
import { Trash2, ShieldCheck, ShieldAlert, Sparkles, AlertOctagon } from "lucide-react";
import { formatBytes } from "../utils/format";

interface ActionBarProps {
  selectedCount: number;
  selectedBytes: number;
  moveProjectToTrash: boolean;
  onToggleMoveProjectToTrash: (val: boolean) => void;
  onOpenCleanModal: () => void;
  onEmptyTrash?: () => void;
  hasTrashItems?: boolean;
  isCleaning: boolean;
  isScanning: boolean;
}

export const ActionBar: React.FC<ActionBarProps> = ({
  selectedCount,
  selectedBytes,
  moveProjectToTrash,
  onToggleMoveProjectToTrash,
  onOpenCleanModal,
  onEmptyTrash,
  hasTrashItems,
  isCleaning,
  isScanning,
}) => {
  return (
    <aside
      aria-label="Action controls"
      className="sticky bottom-0 z-30 bg-slate-900/90 backdrop-blur-md border-t border-slate-800/80 px-4 sm:px-6 py-3.5 shadow-2xl shadow-black"
    >
      <div className="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
        {/* Left: Reclaim Selection Summary */}
        <div className="flex items-center gap-4 w-full sm:w-auto justify-between sm:justify-start">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
              <Sparkles className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-xs text-slate-400">Selected to Free:</span>
                <span className="font-mono tabular-nums text-sm font-bold text-emerald-400">
                  {formatBytes(selectedBytes)}
                </span>
                <span className="text-slate-500 text-xs tabular-nums">
                  ({selectedCount} {selectedCount === 1 ? "item" : "items"})
                </span>
              </div>
              <p className="text-[11px] text-slate-500 hidden md:block">
                Choose safety strategy before executing cleanup
              </p>
            </div>
          </div>
        </div>

        {/* Right: Safety Strategy & Action Buttons */}
        <div className="flex flex-wrap items-center gap-3 w-full sm:w-auto justify-end">
          {/* Safety mode toggle */}
          <button
            type="button"
            onClick={() => onToggleMoveProjectToTrash(!moveProjectToTrash)}
            className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg border text-xs font-medium transition-all ${
              moveProjectToTrash
                ? "bg-emerald-950/40 border-emerald-500/30 text-emerald-300 hover:bg-emerald-900/40"
                : "bg-amber-950/40 border-amber-500/30 text-amber-300 hover:bg-amber-900/40"
            }`}
            title="Toggle between moving project artifacts to trash first vs permanent purge"
          >
            {moveProjectToTrash ? (
              <>
                <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
                <span>Trash Fallback (Safer)</span>
              </>
            ) : (
              <>
                <ShieldAlert className="w-3.5 h-3.5 text-amber-400" />
                <span>Permanent Purge</span>
              </>
            )}
          </button>

          {/* Quick Empty Trash button if trash items exist */}
          {hasTrashItems && onEmptyTrash && (
            <button
              type="button"
              onClick={onEmptyTrash}
              disabled={isCleaning || isScanning}
              className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-lg border border-slate-700/80 bg-slate-800/80 hover:bg-slate-700 text-slate-300 transition-all disabled:opacity-40"
              title="Permanently empty user trash bin"
            >
              <AlertOctagon className="w-3.5 h-3.5 text-rose-400" />
              <span>Empty Trash</span>
            </button>
          )}

          {/* Primary Clean Selected Button */}
          <button
            type="button"
            onClick={onOpenCleanModal}
            disabled={isCleaning || isScanning || selectedCount === 0}
            className="inline-flex items-center gap-2 px-4 py-2 text-sm font-semibold rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white transition-all shadow-md shadow-emerald-950 active:scale-[0.98] disabled:opacity-40 disabled:pointer-events-none"
          >
            <Trash2 className="w-4 h-4" />
            <span>
              Clean Selected
              {selectedCount > 0 ? (
                <span className="font-mono tabular-nums text-xs opacity-90 ml-1">
                  ({formatBytes(selectedBytes)})
                </span>
              ) : null}
            </span>
          </button>
        </div>
      </div>
    </aside>
  );
};
