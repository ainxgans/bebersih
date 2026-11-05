import React from "react";
import { HardDrive, CheckCircle2, AlertCircle } from "lucide-react";
import { formatBytes } from "../utils/format";

interface DiskMeterProps {
  freeBytes: number;
  totalFoundBytes: number;
  selectedBytes: number;
  itemCount: number;
  isScanning: boolean;
}

export const DiskMeter: React.FC<DiskMeterProps> = ({
  freeBytes,
  totalFoundBytes,
  selectedBytes,
  itemCount,
  isScanning,
}) => {
  // Estimated base size for reference (or total proportional bar calculation)
  // Assume a baseline drive size for meter representation: freeBytes + foundBytes
  const totalDriveEst = Math.max(freeBytes + totalFoundBytes, 100 * 1024 * 1024 * 1024);
  const selectedPct = Math.min((selectedBytes / totalDriveEst) * 100, 100);
  const foundPct = Math.min((totalFoundBytes / totalDriveEst) * 100, 100);
  const unselectedPct = Math.max(0, foundPct - selectedPct);

  return (
    <section className="bg-slate-900/80 border border-slate-800/80 rounded-2xl p-5 shadow-sm">
      <div className="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4 mb-4">
        <div className="flex items-center gap-3">
          <div className="p-2.5 rounded-xl bg-slate-800 text-slate-300 border border-slate-700/50">
            <HardDrive className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-slate-200">
              Disk Usage &amp; Reclamation
            </h2>
            <p className="text-xs text-slate-400">
              Ubuntu root filesystem storage &amp; potential reclaimed space
            </p>
          </div>
        </div>

        {/* Quick summary stats */}
        <div className="flex items-center gap-6 text-xs">
          <div>
            <span className="text-slate-400 block mb-0.5">Found Caches</span>
            <span className="font-mono tabular-nums text-sm font-semibold text-slate-200">
              {formatBytes(totalFoundBytes)}
            </span>
            <span className="text-slate-500 text-[11px] ml-1">
              ({itemCount} items)
            </span>
          </div>

          <div className="h-8 w-px bg-slate-800 hidden sm:block" />

          <div>
            <span className="text-slate-400 block mb-0.5">Selected to Free</span>
            <span className="font-mono tabular-nums text-sm font-semibold text-emerald-400">
              {formatBytes(selectedBytes)}
            </span>
          </div>

          <div className="h-8 w-px bg-slate-800 hidden sm:block" />

          <div>
            <span className="text-slate-400 block mb-0.5">Free Space</span>
            <span className="font-mono tabular-nums text-sm font-semibold text-slate-200">
              {formatBytes(freeBytes)}
            </span>
          </div>
        </div>
      </div>

      {/* Visual meter bar */}
      <div className="space-y-2">
        <div className="h-3 w-full bg-slate-950 rounded-full overflow-hidden flex border border-slate-800/70 p-0.5">
          {/* Selected reclaimable space */}
          <div
            className="h-full bg-emerald-500 rounded-l-full transition-all duration-300 relative group"
            style={{ width: `${Math.max(selectedPct, selectedBytes > 0 ? 1.5 : 0)}%` }}
            title={`Selected to clean: ${formatBytes(selectedBytes)}`}
          />
          {/* Unselected detected cache */}
          <div
            className="h-full bg-amber-500/60 transition-all duration-300 relative group"
            style={{ width: `${Math.max(unselectedPct, 0)}%` }}
            title={`Unselected cache: ${formatBytes(totalFoundBytes - selectedBytes)}`}
          />
          {/* Remaining free space / drive */}
          <div className="h-full bg-slate-800/50 flex-1 rounded-r-full" />
        </div>

        {/* Legend */}
        <div className="flex flex-wrap items-center justify-between text-[11px] text-slate-400 pt-1">
          <div className="flex items-center gap-4">
            <span className="inline-flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-emerald-500" />
              Selected for Cleanup ({formatBytes(selectedBytes)})
            </span>
            <span className="inline-flex items-center gap-1.5">
              <span className="w-2 h-2 rounded-full bg-amber-500/60" />
              Unselected Cache ({formatBytes(Math.max(0, totalFoundBytes - selectedBytes))})
            </span>
          </div>

          <div className="text-slate-500 flex items-center gap-1">
            {isScanning ? (
              <span className="text-emerald-400 animate-pulse">
                Analyzing directory trees...
              </span>
            ) : totalFoundBytes > 0 ? (
              <span className="inline-flex items-center gap-1 text-slate-400">
                <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                Scan complete
              </span>
            ) : (
              <span className="inline-flex items-center gap-1 text-slate-500">
                <AlertCircle className="w-3.5 h-3.5" />
                Click &quot;Scan Now&quot; to inspect caches
              </span>
            )}
          </div>
        </div>
      </div>
    </section>
  );
};
