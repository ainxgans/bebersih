import React, { useState, useRef, useEffect, useMemo } from "react";
import {
  Search,
  Trash2,
  Shield,
  Check,
  X,
  Archive,
} from "lucide-react";
import { CleanItem, CleanCategory, CATEGORY_DEFINITIONS } from "../types";
import { formatBytes, truncatePath } from "../utils/format";
import { computeVirtualSlice } from "../utils/virtual";

interface ItemTableProps {
  items: CleanItem[];
  allFilteredCount: number;
  selectedFilteredCount: number;
  activeCategoryFilter: CleanCategory | null;
  onClearCategoryFilter: () => void;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onToggleItemSelect: (id: string) => void;
  onToggleSelectAllFiltered: (select: boolean) => void;
  moveProjectToTrash: boolean;
}

const ROW_HEIGHT = 52;
const OVERSCAN = 10;

export const ItemTable: React.FC<ItemTableProps> = ({
  items,
  allFilteredCount,
  selectedFilteredCount,
  activeCategoryFilter,
  onClearCategoryFilter,
  searchQuery,
  onSearchChange,
  onToggleItemSelect,
  onToggleSelectAllFiltered,
  moveProjectToTrash,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(560);

  // Measure visible scroll container height
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const updateHeight = () => {
      if (el.clientHeight > 0) {
        setViewportHeight(el.clientHeight);
      }
    };

    updateHeight();
    const observer = new ResizeObserver(updateHeight);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // Reset scroll to top when filters or search change
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = 0;
      setScrollTop(0);
    }
  }, [activeCategoryFilter, searchQuery]);

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  };

  const isAllFilteredSelected =
    allFilteredCount > 0 && selectedFilteredCount === allFilteredCount;
  const isSomeFilteredSelected =
    selectedFilteredCount > 0 && selectedFilteredCount < allFilteredCount;

  // Windowed virtual slice calculations
  const slice = useMemo(
    () =>
      computeVirtualSlice(
        items.length,
        scrollTop,
        viewportHeight,
        ROW_HEIGHT,
        OVERSCAN
      ),
    [items.length, scrollTop, viewportHeight]
  );

  const visibleItems = useMemo(
    () => items.slice(slice.startIndex, slice.endIndex),
    [items, slice.startIndex, slice.endIndex]
  );

  return (
    <div className="bg-slate-900/80 border border-slate-800/80 rounded-2xl overflow-hidden shadow-sm flex flex-col">
      {/* Table Toolbar */}
      <div className="p-4 border-b border-slate-800/80 flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 bg-slate-900/40">
        <div className="flex items-center gap-2">
          {/* Active Category Filter Tag if set */}
          {activeCategoryFilter ? (
            <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-emerald-500/15 border border-emerald-500/30 text-emerald-300 text-xs font-medium">
              <span>Category: {CATEGORY_DEFINITIONS[activeCategoryFilter].label}</span>
              <button
                type="button"
                onClick={onClearCategoryFilter}
                className="hover:text-emerald-100 p-0.5 rounded"
                title="Clear category filter"
              >
                <X className="w-3.5 h-3.5" />
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2 text-xs font-medium text-slate-400">
              <span>
                Showing {allFilteredCount.toLocaleString()}{" "}
                {allFilteredCount === 1 ? "item" : "items"}
              </span>
              {allFilteredCount > 50 && (
                <span className="text-[10px] text-emerald-400/90 font-medium px-1.5 py-0.5 rounded bg-emerald-500/10 border border-emerald-500/20">
                  virtualized
                </span>
              )}
            </div>
          )}
        </div>

        {/* Search bar */}
        <div className="relative flex-1 max-w-xs">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 pointer-events-none" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search items or paths..."
            className="w-full pl-9 pr-8 py-1.5 bg-slate-950/70 border border-slate-800 rounded-lg text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-emerald-500/60 focus:ring-1 focus:ring-emerald-500/30 transition-colors"
          />
          {searchQuery && (
            <button
              type="button"
              onClick={() => onSearchChange("")}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
              title="Clear search"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* Table Virtual Scroll Content */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="overflow-y-auto overflow-x-auto h-[580px] max-h-[calc(100vh-320px)] min-h-[380px] relative scroll-smooth select-text"
      >
        <table className="w-full table-fixed text-left text-xs border-collapse">
          <thead>
            <tr className="select-none">
              <th className="sticky top-0 z-10 bg-slate-950 border-b border-slate-800 py-3 px-4 w-12 text-left">
                <button
                  type="button"
                  onClick={() => onToggleSelectAllFiltered(!isAllFilteredSelected)}
                  disabled={allFilteredCount === 0}
                  className={`w-4 h-4 rounded border flex items-center justify-center transition-all ${
                    isAllFilteredSelected
                      ? "bg-emerald-600 border-emerald-500 text-white"
                      : isSomeFilteredSelected
                      ? "bg-emerald-600/30 border-emerald-500 text-emerald-300"
                      : "bg-slate-950 border-slate-700 hover:border-slate-500 text-transparent"
                  } disabled:opacity-30 disabled:pointer-events-none`}
                  title={
                    isAllFilteredSelected
                      ? "Deselect all visible"
                      : "Select all visible"
                  }
                >
                  {isAllFilteredSelected && <Check className="w-3 h-3 stroke-[3]" />}
                </button>
              </th>
              <th className="sticky top-0 z-10 bg-slate-950 border-b border-slate-800 py-3 px-4 font-medium w-56 sm:w-64 text-left text-slate-400">
                Item &amp; Category
              </th>
              <th className="sticky top-0 z-10 bg-slate-950 border-b border-slate-800 py-3 px-4 font-medium text-left text-slate-400">
                Path / Source
              </th>
              <th className="sticky top-0 z-10 bg-slate-950 border-b border-slate-800 py-3 px-4 font-medium text-center w-24 text-slate-400">
                Mode
              </th>
              <th className="sticky top-0 z-10 bg-slate-950 border-b border-slate-800 py-3 px-4 font-medium text-right w-28 text-slate-400">
                Size
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-800/60">
            {items.length === 0 ? (
              <tr>
                <td colSpan={5} className="py-16 text-center text-slate-500">
                  <div className="max-w-xs mx-auto flex flex-col items-center">
                    <Archive className="w-8 h-8 text-slate-600 mb-2 stroke-1" />
                    <p className="text-sm font-medium text-slate-400">
                      No items to display
                    </p>
                    <p className="text-xs text-slate-500 mt-1">
                      {searchQuery
                        ? "No results matching your search query."
                        : "Run a scan to detect developer and system caches."}
                    </p>
                  </div>
                </td>
              </tr>
            ) : (
              <>
                {/* Top Virtual Spacer */}
                {slice.topSpacerHeight > 0 && (
                  <tr style={{ height: `${slice.topSpacerHeight}px` }} aria-hidden="true">
                    <td
                      colSpan={5}
                      style={{ height: `${slice.topSpacerHeight}px`, padding: 0, border: 0 }}
                    />
                  </tr>
                )}

                {/* Visible Windowed Items */}
                {visibleItems.map((item) => {
                  const meta = CATEGORY_DEFINITIONS[item.category];
                  return (
                    <tr
                      key={item.id}
                      style={{ height: `${ROW_HEIGHT}px` }}
                      onClick={() => onToggleItemSelect(item.id)}
                      className={`group transition-colors cursor-pointer ${
                        item.selected
                          ? "bg-emerald-500/[0.04] hover:bg-emerald-500/[0.08]"
                          : "hover:bg-slate-800/40 opacity-75 hover:opacity-100"
                      }`}
                    >
                      {/* Checkbox column */}
                      <td
                        className="py-2.5 px-4 w-12"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <button
                          type="button"
                          onClick={() => onToggleItemSelect(item.id)}
                          className={`w-4 h-4 rounded border flex items-center justify-center transition-all ${
                            item.selected
                              ? "bg-emerald-600 border-emerald-500 text-white"
                              : "bg-slate-950 border-slate-700 hover:border-slate-500 text-transparent"
                          }`}
                        >
                          {item.selected && <Check className="w-3 h-3 stroke-[3]" />}
                        </button>
                      </td>

                      {/* Label & Category */}
                      <td className="py-2.5 px-4 overflow-hidden">
                        <div
                          className="font-medium text-slate-200 truncate"
                          title={item.label}
                        >
                          {item.label}
                        </div>
                        <div className="text-[11px] text-slate-400 flex items-center gap-1.5 mt-0.5 truncate">
                          <span className="text-slate-400 truncate">
                            {meta?.label || item.category}
                          </span>
                          {item.requires_root && (
                            <span className="inline-flex items-center gap-0.5 text-[10px] font-semibold px-1.5 py-0.2 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 shrink-0">
                              <Shield className="w-2.5 h-2.5" />
                              root
                            </span>
                          )}
                        </div>
                      </td>

                      {/* Path / Source */}
                      <td className="py-2.5 px-4 font-mono text-slate-400 text-[11px] overflow-hidden">
                        {item.path ? (
                          <span
                            title={item.path}
                            className="hover:text-slate-200 truncate block"
                          >
                            {truncatePath(item.path)}
                          </span>
                        ) : (
                          <span className="italic text-slate-500">
                            Managed by engine / daemon
                          </span>
                        )}
                      </td>

                      {/* Action Mode (Trash vs Direct Purge) */}
                      <td className="py-2.5 px-4 text-center w-24">
                        {item.supports_trash && moveProjectToTrash ? (
                          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium bg-sky-500/10 text-sky-400 border border-sky-500/20">
                            <Trash2 className="w-3 h-3" />
                            Trash
                          </span>
                        ) : (
                          <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium bg-rose-500/10 text-rose-400 border border-rose-500/20">
                            Purge
                          </span>
                        )}
                      </td>

                      {/* Size bytes */}
                      <td className="py-2.5 px-4 text-right font-mono tabular-nums font-semibold text-slate-200 text-xs w-28">
                        {formatBytes(item.size_bytes)}
                      </td>
                    </tr>
                  );
                })}

                {/* Bottom Virtual Spacer */}
                {slice.bottomSpacerHeight > 0 && (
                  <tr style={{ height: `${slice.bottomSpacerHeight}px` }} aria-hidden="true">
                    <td
                      colSpan={5}
                      style={{ height: `${slice.bottomSpacerHeight}px`, padding: 0, border: 0 }}
                    />
                  </tr>
                )}
              </>
            )}
          </tbody>
        </table>
      </div>
    </div>
  );
};
