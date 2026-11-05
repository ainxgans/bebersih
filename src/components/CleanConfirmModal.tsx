import React, { useEffect } from "react";
import {
  AlertTriangle,
  ShieldAlert,
  Trash2,
  X,
  CheckCircle2,
  RefreshCw,
  FolderSync,
} from "lucide-react";
import { CleanItem, CleanReport } from "../types";
import { formatBytes } from "../utils/format";

interface CleanConfirmModalProps {
  isOpen: boolean;
  isCleaning: boolean;
  selectedItems: CleanItem[];
  moveProjectToTrash: boolean;
  onToggleMoveProjectToTrash: (val: boolean) => void;
  onConfirmClean: () => void;
  onClose: () => void;
  cleanReport: CleanReport | null;
}

export const CleanConfirmModal: React.FC<CleanConfirmModalProps> = ({
  isOpen,
  isCleaning,
  selectedItems,
  moveProjectToTrash,
  onToggleMoveProjectToTrash,
  onConfirmClean,
  onClose,
  cleanReport,
}) => {
  // Close on Escape key press
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !isCleaning) {
        onClose();
      }
    };
    if (isOpen) {
      window.addEventListener("keydown", handleKeyDown);
    }
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, isCleaning, onClose]);

  if (!isOpen) return null;

  const totalBytes = selectedItems.reduce((acc, it) => acc + it.size_bytes, 0);
  const rootItems = selectedItems.filter((it) => it.requires_root);
  const trashItems = selectedItems.filter(
    (it) => it.supports_trash && moveProjectToTrash
  );
  const purgeItems = selectedItems.filter(
    (it) => !it.supports_trash || !moveProjectToTrash
  );

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        onClick={() => {
          if (!isCleaning) onClose();
        }}
        className="fixed inset-0 bg-slate-950/80 backdrop-blur-sm transition-opacity"
      />

      {/* Modal Dialog Card */}
      <div className="relative w-full max-w-lg bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl overflow-hidden z-10 flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-800/80 flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div
              className={`p-2 rounded-xl border ${
                cleanReport
                  ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20"
                  : "bg-rose-500/10 text-rose-400 border-rose-500/20"
              }`}
            >
              {cleanReport ? (
                <CheckCircle2 className="w-5 h-5" />
              ) : (
                <Trash2 className="w-5 h-5" />
              )}
            </div>
            <div>
              <h3 className="text-base font-semibold text-slate-100">
                {cleanReport ? "Cleanup Finished" : "Confirm Disk Cleanup"}
              </h3>
              <p className="text-xs text-slate-400">
                {cleanReport
                  ? "Reclamation summary and results"
                  : "Review selected items before proceeding"}
              </p>
            </div>
          </div>

          {!isCleaning && (
            <button
              type="button"
              onClick={onClose}
              className="p-1 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition-colors"
              title="Close modal"
            >
              <X className="w-5 h-5" />
            </button>
          )}
        </div>

        {/* Modal Body */}
        <div className="px-6 py-5 overflow-y-auto space-y-4 text-xs">
          {cleanReport ? (
            /* Result Report View */
            <div className="space-y-4">
              <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-300">
                <div className="text-lg font-bold font-mono tabular-nums">
                  {formatBytes(cleanReport.deleted_bytes)} Freed
                </div>
                <p className="text-xs text-emerald-400/80 mt-1">
                  Successfully cleaned {cleanReport.freed_items.length} items.
                </p>
              </div>

              {cleanReport.errors.length > 0 && (
                <div className="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 space-y-1">
                  <div className="font-semibold flex items-center gap-1.5">
                    <AlertTriangle className="w-4 h-4" />
                    Warnings during cleanup ({cleanReport.errors.length})
                  </div>
                  <ul className="list-disc list-inside space-y-0.5 text-slate-300 text-[11px] max-h-32 overflow-y-auto">
                    {cleanReport.errors.map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          ) : (
            /* Pre-clean Confirmation View */
            <div className="space-y-4">
              {/* Highlight summary card */}
              <div className="p-4 rounded-xl bg-slate-950/70 border border-slate-800 flex items-center justify-between">
                <div>
                  <span className="text-slate-400 block">Space to Reclaim</span>
                  <span className="text-2xl font-bold font-mono tabular-nums text-slate-100">
                    {formatBytes(totalBytes)}
                  </span>
                </div>
                <div className="text-right">
                  <span className="text-slate-400 block">Selected Items</span>
                  <span className="text-lg font-semibold font-mono text-slate-200">
                    {selectedItems.length}
                  </span>
                </div>
              </div>

              {/* Root Escalate Warning */}
              {rootItems.length > 0 && (
                <div className="p-3.5 rounded-xl bg-amber-500/10 border border-amber-500/20 text-amber-300 flex items-start gap-3">
                  <ShieldAlert className="w-5 h-5 flex-shrink-0 text-amber-400 mt-0.5" />
                  <div>
                    <span className="font-semibold block text-slate-200">
                      Root Authentication Required
                    </span>
                    <span className="text-slate-400 text-[11px] leading-relaxed block mt-0.5">
                      {rootItems.length} item(s) require system privileges (APT
                      archives, journalctl). Ubuntu Polkit will prompt for your
                      password once to execute in a single batch.
                    </span>
                  </div>
                </div>
              )}

              {/* Safety Trash Toggle */}
              <div className="p-3.5 rounded-xl bg-slate-950/40 border border-slate-800 flex items-center justify-between gap-3">
                <div className="flex items-center gap-2.5">
                  <FolderSync className="w-4 h-4 text-sky-400" />
                  <div>
                    <span className="font-medium text-slate-200 block">
                      Move Project Folders to Trash
                    </span>
                    <span className="text-slate-400 text-[11px] block">
                      Enables safe recovery via GNOME Trash for node_modules and
                      target/
                    </span>
                  </div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={moveProjectToTrash}
                    onChange={(e) =>
                      onToggleMoveProjectToTrash(e.target.checked)
                    }
                    className="sr-only peer"
                    disabled={isCleaning}
                  />
                  <div className="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-emerald-600" />
                </label>
              </div>

              {/* Summary of actions */}
              <div className="grid grid-cols-2 gap-3 text-[11px]">
                <div className="p-3 rounded-lg bg-slate-950/40 border border-slate-800/80">
                  <span className="text-slate-400 block">Move to Trash</span>
                  <span className="font-mono text-sm font-semibold text-sky-400">
                    {trashItems.length} items
                  </span>
                </div>
                <div className="p-3 rounded-lg bg-slate-950/40 border border-slate-800/80">
                  <span className="text-slate-400 block">Direct Purge</span>
                  <span className="font-mono text-sm font-semibold text-rose-400">
                    {purgeItems.length} items
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer Actions */}
        <div className="px-6 py-4 border-t border-slate-800/80 bg-slate-950/40 flex items-center justify-end gap-3">
          {cleanReport ? (
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs transition-colors"
            >
              Close
            </button>
          ) : (
            <>
              <button
                type="button"
                onClick={onClose}
                disabled={isCleaning}
                className="px-4 py-2 rounded-lg border border-slate-700 hover:bg-slate-800 text-slate-300 font-medium text-xs transition-colors disabled:opacity-40"
              >
                Cancel
              </button>

              <button
                type="button"
                onClick={onConfirmClean}
                disabled={isCleaning || selectedItems.length === 0}
                className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-rose-600 hover:bg-rose-500 text-white font-medium text-xs transition-all active:scale-[0.98] disabled:opacity-40 disabled:pointer-events-none shadow-sm shadow-rose-950"
              >
                {isCleaning ? (
                  <>
                    <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                    <span>Cleaning...</span>
                  </>
                ) : (
                  <>
                    <Trash2 className="w-3.5 h-3.5" />
                    <span>Proceed with Cleanup</span>
                  </>
                )}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
