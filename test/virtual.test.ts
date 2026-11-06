import test from "node:test";
import assert from "node:assert/strict";
import { computeVirtualSlice } from "../src/utils/virtual.ts";

test("computeVirtualSlice returns zero bounds for empty list", () => {
  const slice = computeVirtualSlice(0, 0, 500);
  assert.deepEqual(slice, {
    startIndex: 0,
    endIndex: 0,
    topSpacerHeight: 0,
    bottomSpacerHeight: 0,
  });
});

test("computeVirtualSlice calculates initial window with overscan for 28,000 items", () => {
  const totalItems = 28682;
  const rowHeight = 52;
  const viewportHeight = 520; // 10 items in view
  const overscan = 8;

  const slice = computeVirtualSlice(totalItems, 0, viewportHeight, rowHeight, overscan);

  // At top: start is 0, end is 10 + 8 = 18
  assert.equal(slice.startIndex, 0);
  assert.equal(slice.endIndex, 18);
  assert.equal(slice.topSpacerHeight, 0);
  assert.equal(slice.bottomSpacerHeight, (totalItems - 18) * rowHeight);

  // Total height conserved
  const totalHeight = slice.topSpacerHeight + ((slice.endIndex - slice.startIndex) * rowHeight) + slice.bottomSpacerHeight;
  assert.equal(totalHeight, totalItems * rowHeight);
});

test("computeVirtualSlice calculates middle scroll position accurately", () => {
  const totalItems = 28682;
  const rowHeight = 52;
  const viewportHeight = 520;
  const overscan = 8;
  const scrollTop = 5200; // 100th item

  const slice = computeVirtualSlice(totalItems, scrollTop, viewportHeight, rowHeight, overscan);

  // start = 100 - 8 = 92
  assert.equal(slice.startIndex, 92);
  // end = (100 + 10) + 8 = 118
  assert.equal(slice.endIndex, 118);
  assert.equal(slice.topSpacerHeight, 92 * rowHeight);
  assert.equal(slice.bottomSpacerHeight, (totalItems - 118) * rowHeight);

  // Total height conserved
  const totalHeight = slice.topSpacerHeight + ((slice.endIndex - slice.startIndex) * rowHeight) + slice.bottomSpacerHeight;
  assert.equal(totalHeight, totalItems * rowHeight);
});

test("computeVirtualSlice clamps correctly at list end", () => {
  const totalItems = 100;
  const rowHeight = 50;
  const viewportHeight = 500;
  const overscan = 5;
  const scrollTop = 100000; // Past bottom

  const slice = computeVirtualSlice(totalItems, scrollTop, viewportHeight, rowHeight, overscan);

  assert.equal(slice.endIndex, 100);
  assert.equal(slice.bottomSpacerHeight, 0);
  assert.ok(slice.startIndex <= slice.endIndex);
});
