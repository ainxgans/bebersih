import { useState, useEffect, useMemo, useCallback } from "react";
import { Header } from "./components/Header";
import { DiskMeter } from "./components/DiskMeter";
import { CategoryCard } from "./components/CategoryCard";
import { ItemTable } from "./components/ItemTable";
import { CleanConfirmModal } from "./components/CleanConfirmModal";
import { CleanItem, CleanCategory, CleanReport } from "./types";
import {
  scanAll,
  cleanSelected,
  getDiskFreeSpace,
} from "./services/api";
import { formatBytes } from "./utils/format";
import { AlertCircle, Trash2 } from "lucide-react";

const ALL_CATEGORIES: CleanCategory[] = [
  "DevRust",
  "DevGo",
  "DevNode",
  "DockerCache",
  "DockerVolume",
  "AptCache",
  "SystemLogs",
  "StaleCache",
];

export default function App() {
  const [items, setItems] = useState<CleanItem[]>([]);
  const [freeDiskBytes, setFreeDiskBytes] = useState<number>(0);
  const [isScanning, setIsScanning] = useState<boolean>(false);
  const [isCleaning, setIsCleaning] = useState<boolean>(false);
  const [scanError, setScanError] = useState<string | null>(null);

  // Filters & selection
  const [activeCategoryFilter, setActiveCategoryFilter] =
    useState<CleanCategory | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [moveProjectToTrash, setMoveProjectToTrash] = useState<boolean>(true);

  // Modal
  const [isCleanModalOpen, setIsCleanModalOpen] = useState<boolean>(false);
  const [cleanReport, setCleanReport] = useState<CleanReport | null>(null);

  // Initial scan and disk space check on mount
  useEffect(() => {
    let mounted = true;

    async function init() {
      try {
        const freeSpace = await getDiskFreeSpace();
        if (mounted) setFreeDiskBytes(freeSpace);
      } catch (err) {
        console.warn("Failed to query disk free space:", err);
      }
    }

    init();
    return () => {
      mounted = false;
    };
  }, []);

  // Trigger scan
  const handleScan = useCallback(async () => {
    setIsScanning(true);
    setScanError(null);
    setCleanReport(null);

    try {
      const [scannedItems, freeSpace] = await Promise.all([
        scanAll(),
        getDiskFreeSpace().catch(() => 0),
      ]);
      setItems(scannedItems);
      if (freeSpace > 0) setFreeDiskBytes(freeSpace);
    } catch (err) {
      console.error("Scan error:", err);
      setScanError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsScanning(false);
    }
  }, []);

  // Toggle item selection
  const handleToggleItemSelect = useCallback((id: string) => {
    setItems((prev) =>
      prev.map((item) =>
        item.id === id ? { ...item, selected: !item.selected } : item
      )
    );
  }, []);

  // Toggle all items in a specific category
  const handleToggleSelectAllCategory = useCallback(
    (category: CleanCategory, select: boolean) => {
      setItems((prev) =>
        prev.map((item) =>
          item.category === category ? { ...item, selected: select } : item
        )
      );
    },
    []
  );

  // Filtered items based on active category and search
  const filteredItems = useMemo(() => {
    return items.filter((item) => {
      if (activeCategoryFilter && item.category !== activeCategoryFilter) {
        return false;
      }
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        const matchLabel = item.label.toLowerCase().includes(query);
        const matchPath = item.path?.toLowerCase().includes(query) ?? false;
        if (!matchLabel && !matchPath) return false;
      }
      return true;
    });
  }, [items, activeCategoryFilter, searchQuery]);

  // Toggle all visible filtered items
  const handleToggleSelectAllFiltered = useCallback(
    (select: boolean) => {
      const filteredIds = new Set(filteredItems.map((item) => item.id));
      setItems((prev) =>
        prev.map((item) =>
          filteredIds.has(item.id) ? { ...item, selected: select } : item
        )
      );
    },
    [filteredItems]
  );

  // Aggregated calculations
  const totalFoundBytes = useMemo(
    () => items.reduce((acc, it) => acc + it.size_bytes, 0),
    [items]
  );

  const selectedItems = useMemo(
    () => items.filter((it) => it.selected),
    [items]
  );

  const selectedBytes = useMemo(
    () => selectedItems.reduce((acc, it) => acc + it.size_bytes, 0),
    [selectedItems]
  );

  const selectedFilteredCount = useMemo(
    () => filteredItems.filter((it) => it.selected).length,
    [filteredItems]
  );

  // Confirm and execute cleanup
  const handleConfirmClean = useCallback(async () => {
    if (selectedItems.length === 0) return;

    setIsCleaning(true);
    try {
      const report = await cleanSelected(selectedItems, moveProjectToTrash);
      setCleanReport(report);

      // Remove successfully cleaned items from table
      const freedIds = new Set(report.freed_items);
      setItems((prev) => prev.filter((item) => !freedIds.has(item.id)));

      // Refresh free disk space
      const freeSpace = await getDiskFreeSpace().catch(() => 0);
      if (freeSpace > 0) setFreeDiskBytes(freeSpace);
    } catch (err) {
      console.error("Clean error:", err);
      setCleanReport({
        deleted_bytes: 0,
        freed_items: [],
        errors: [err instanceof Error ? err.message : String(err)],
      });
    } finally {
      setIsCleaning(false);
    }
  }, [selectedItems, moveProjectToTrash]);

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans">
      {/* Header bar */}
      <Header
        isScanning={isScanning}
        isCleaning={isCleaning}
        selectedCount={selectedItems.length}
        selectedBytes={selectedBytes}
        onScan={handleScan}
        onOpenCleanModal={() => {
          setCleanReport(null);
          setIsCleanModalOpen(true);
        }}
      />

      {/* Main container */}
      <main className="flex-1 max-w-7xl w-full mx-auto p-4 sm:p-6 space-y-6">
        {/* Error notification if scan failed */}
        {scanError && (
          <div className="p-4 rounded-xl bg-rose-500/10 border border-rose-500/20 text-rose-300 flex items-start gap-3 text-xs">
            <AlertCircle className="w-5 h-5 flex-shrink-0 text-rose-400 mt-0.5" />
            <div>
              <span className="font-semibold block text-slate-200">
                Scan Failed
              </span>
              <span className="text-slate-400 mt-0.5 block">{scanError}</span>
            </div>
          </div>
        )}

        {/* Disk Usage Overview Meter */}
        <DiskMeter
          freeBytes={freeDiskBytes}
          totalFoundBytes={totalFoundBytes}
          selectedBytes={selectedBytes}
          itemCount={items.length}
          isScanning={isScanning}
        />

        {/* Categories Checklist Grid */}
        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-semibold text-slate-200">
                Cache Categories
              </h2>
              <p className="text-xs text-slate-400">
                Select categories to clean or click a card to filter the items
                table
              </p>
            </div>
            {activeCategoryFilter && (
              <button
                type="button"
                onClick={() => setActiveCategoryFilter(null)}
                className="text-xs text-emerald-400 hover:text-emerald-300 font-medium"
              >
                Show All Categories
              </button>
            )}
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
            {ALL_CATEGORIES.map((cat) => {
              const categoryItems = items.filter((it) => it.category === cat);
              return (
                <CategoryCard
                  key={cat}
                  category={cat}
                  items={categoryItems}
                  isActiveFilter={activeCategoryFilter === cat}
                  onSelectCategoryFilter={setActiveCategoryFilter}
                  onToggleSelectAll={handleToggleSelectAllCategory}
                />
              );
            })}
          </div>
        </section>

        {/* Items Table Section */}
        <section className="space-y-3 pb-20">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-semibold text-slate-200">
                Granular Items
              </h2>
              <p className="text-xs text-slate-400">
                Inspect specific directories and build caches before deleting
              </p>
            </div>
          </div>

          <ItemTable
            items={filteredItems}
            allFilteredCount={filteredItems.length}
            selectedFilteredCount={selectedFilteredCount}
            activeCategoryFilter={activeCategoryFilter}
            onClearCategoryFilter={() => setActiveCategoryFilter(null)}
            searchQuery={searchQuery}
            onSearchChange={setSearchQuery}
            onToggleItemSelect={handleToggleItemSelect}
            onToggleSelectAllFiltered={handleToggleSelectAllFiltered}
            moveProjectToTrash={moveProjectToTrash}
          />
        </section>
      </main>

      {/* Sticky Bottom Action Bar (When items are selected) */}
      {selectedItems.length > 0 && (
        <aside className="fixed bottom-0 inset-x-0 z-40 bg-slate-900/90 backdrop-blur-md border-t border-slate-800 p-4 shadow-lg">
          <div className="max-w-7xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-4 text-xs">
              <span className="text-slate-300">
                <strong className="font-semibold text-emerald-400">
                  {selectedItems.length}
                </strong>{" "}
                {selectedItems.length === 1 ? "item" : "items"} selected
              </span>
              <span className="h-4 w-px bg-slate-700" />
              <span className="text-slate-300">
                Reclaim:{" "}
                <strong className="font-mono tabular-nums text-sm font-bold text-slate-100">
                  {formatBytes(selectedBytes)}
                </strong>
              </span>
            </div>

            <div className="flex items-center gap-4 w-full sm:w-auto justify-end">
              {/* Trash vs Direct Purge switch */}
              <label className="flex items-center gap-2 text-xs text-slate-300 cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={moveProjectToTrash}
                  onChange={(e) => setMoveProjectToTrash(e.target.checked)}
                  className="rounded bg-slate-800 border-slate-700 text-emerald-500 focus:ring-0"
                />
                <span>Safe Trash for project folders</span>
              </label>

              <button
                type="button"
                onClick={() => {
                  setCleanReport(null);
                  setIsCleanModalOpen(true);
                }}
                disabled={isCleaning}
                className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs transition-all active:scale-[0.98] shadow-sm shadow-emerald-950"
              >
                <Trash2 className="w-4 h-4" />
                <span>Clean Selected</span>
              </button>
            </div>
          </div>
        </aside>
      )}

      {/* Confirmation & Result Modal */}
      <CleanConfirmModal
        isOpen={isCleanModalOpen}
        isCleaning={isCleaning}
        selectedItems={selectedItems}
        moveProjectToTrash={moveProjectToTrash}
        onToggleMoveProjectToTrash={setMoveProjectToTrash}
        onConfirmClean={handleConfirmClean}
        onClose={() => setIsCleanModalOpen(false)}
        cleanReport={cleanReport}
      />
    </div>
  );
}
