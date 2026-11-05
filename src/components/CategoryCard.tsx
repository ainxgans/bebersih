import React from "react";
import {
  FileCode,
  Layers,
  Box,
  Database,
  Archive,
  FileText,
  Clock,
  Shield,
  Check,
  Minus,
} from "lucide-react";
import { CleanCategory, CleanItem, CATEGORY_DEFINITIONS } from "../types";
import { formatBytes } from "../utils/format";

interface CategoryCardProps {
  category: CleanCategory;
  items: CleanItem[];
  isActiveFilter: boolean;
  onSelectCategoryFilter: (category: CleanCategory | null) => void;
  onToggleSelectAll: (category: CleanCategory, select: boolean) => void;
}

const CATEGORY_ICONS: Record<CleanCategory, React.ElementType> = {
  DevRust: FileCode,
  DevGo: Layers,
  DevNode: Box,
  DockerCache: Layers,
  DockerVolume: Database,
  AptCache: Archive,
  SystemLogs: FileText,
  StaleCache: Clock,
};

export const CategoryCard: React.FC<CategoryCardProps> = ({
  category,
  items,
  isActiveFilter,
  onSelectCategoryFilter,
  onToggleSelectAll,
}) => {
  const meta = CATEGORY_DEFINITIONS[category];
  const IconComponent = CATEGORY_ICONS[category] || Box;

  const totalBytes = items.reduce((acc, item) => acc + item.size_bytes, 0);
  const selectedCount = items.filter((item) => item.selected).length;
  const isAllSelected = items.length > 0 && selectedCount === items.length;
  const isSomeSelected = selectedCount > 0 && selectedCount < items.length;
  const hasRoot = items.some((item) => item.requires_root);

  const handleCheckboxClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onToggleSelectAll(category, !isAllSelected);
  };

  const handleCardClick = () => {
    onSelectCategoryFilter(isActiveFilter ? null : category);
  };

  return (
    <div
      onClick={handleCardClick}
      className={`group relative rounded-xl border p-4 transition-all duration-150 cursor-pointer text-left select-none ${
        isActiveFilter
          ? "bg-slate-800/90 border-emerald-500/60 shadow-md shadow-emerald-950/30"
          : "bg-slate-900/60 border-slate-800/80 hover:bg-slate-850 hover:border-slate-700/80"
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        {/* Category icon & label */}
        <div className="flex items-center gap-3">
          <div
            className={`p-2 rounded-lg border transition-colors ${
              isActiveFilter
                ? "bg-emerald-500/20 text-emerald-300 border-emerald-500/30"
                : "bg-slate-800 text-slate-400 border-slate-700/50 group-hover:text-slate-200"
            }`}
          >
            <IconComponent className="w-4 h-4" />
          </div>
          <div>
            <div className="flex items-center gap-1.5">
              <h3 className="text-sm font-medium text-slate-200">
                {meta.label}
              </h3>
              {hasRoot && (
                <span
                  title="Requires root authorization via pkexec"
                  className="inline-flex items-center gap-0.5 text-[10px] font-semibold px-1.5 py-0.2 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20"
                >
                  <Shield className="w-2.5 h-2.5" />
                  root
                </span>
              )}
            </div>
            <p className="text-xs text-slate-500 line-clamp-1">
              {meta.description}
            </p>
          </div>
        </div>

        {/* Category select-all checkbox */}
        <button
          type="button"
          onClick={handleCheckboxClick}
          disabled={items.length === 0}
          title={isAllSelected ? "Deselect all in category" : "Select all in category"}
          className={`w-5 h-5 rounded flex items-center justify-center border transition-all ${
            isAllSelected
              ? "bg-emerald-600 border-emerald-500 text-white"
              : isSomeSelected
              ? "bg-emerald-600/30 border-emerald-500 text-emerald-300"
              : "bg-slate-950 border-slate-700 hover:border-slate-500 text-transparent"
          } disabled:opacity-30 disabled:pointer-events-none`}
        >
          {isAllSelected ? (
            <Check className="w-3.5 h-3.5 stroke-[2.5]" />
          ) : isSomeSelected ? (
            <Minus className="w-3.5 h-3.5 stroke-[2.5]" />
          ) : null}
        </button>
      </div>

      {/* Metrics footer */}
      <div className="mt-3 pt-3 border-t border-slate-800/60 flex items-center justify-between text-xs">
        <span className="text-slate-400">
          {items.length} {items.length === 1 ? "item" : "items"}{" "}
          {selectedCount > 0 && selectedCount < items.length && (
            <span className="text-emerald-400 font-mono">
              ({selectedCount} sel)
            </span>
          )}
        </span>
        <span className="font-mono tabular-nums font-semibold text-slate-200">
          {formatBytes(totalBytes)}
        </span>
      </div>
    </div>
  );
};
