export interface VirtualSlice {
  startIndex: number;
  endIndex: number;
  topSpacerHeight: number;
  bottomSpacerHeight: number;
}

/**
 * Computes windowed virtual slice for large lists.
 * Prevents bloated DOM and lagging UI when rendering tens of thousands of items.
 *
 * ponytail: Fixed-height virtualization covers 99.9% of table row use cases.
 * Upgrade to dynamic row measurement only if multiline variable-height row content is added.
 */
export function computeVirtualSlice(
  totalCount: number,
  scrollTop: number,
  viewportHeight: number,
  rowHeight = 52,
  overscan = 8
): VirtualSlice {
  if (totalCount <= 0) {
    return { startIndex: 0, endIndex: 0, topSpacerHeight: 0, bottomSpacerHeight: 0 };
  }

  const safeScrollTop = Math.max(0, scrollTop);
  const safeViewport = Math.max(0, viewportHeight);

  const rawStart = Math.floor(safeScrollTop / rowHeight) - overscan;
  const rawEnd = Math.ceil((safeScrollTop + safeViewport) / rowHeight) + overscan;

  const start = Math.min(Math.max(0, rawStart), totalCount);
  const end = Math.min(Math.max(start, rawEnd), totalCount);

  return {
    startIndex: start,
    endIndex: end,
    topSpacerHeight: start * rowHeight,
    bottomSpacerHeight: Math.max(0, (totalCount - end) * rowHeight),
  };
}
