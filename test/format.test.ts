import test from "node:test";
import assert from "node:assert/strict";
import { formatBytes, truncatePath } from "../src/utils/format.ts";
import { CATEGORY_DEFINITIONS } from "../src/types.ts";

test("formatBytes formats byte values properly", () => {
  assert.equal(formatBytes(0), "0 B");
  assert.equal(formatBytes(1024), "1.0 KB");
  assert.equal(formatBytes(1024 * 1024 * 50), "50.0 MB");
  assert.equal(formatBytes(1024 * 1024 * 1024 * 2.5), "2.5 GB");
});

test("truncatePath handles null, short, and long paths", () => {
  assert.equal(truncatePath(null), "-");
  assert.equal(truncatePath("/short/path"), "/short/path");
  assert.equal(
    truncatePath("/home/user/very/long/nested/directory/structure/project/target", 30),
    "/home/.../project/target"
  );
});

test("CATEGORY_DEFINITIONS includes BrowserCache metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.BrowserCache);
  assert.equal(CATEGORY_DEFINITIONS.BrowserCache.id, "BrowserCache");
  assert.equal(CATEGORY_DEFINITIONS.BrowserCache.group, "Browser");
  assert.equal(CATEGORY_DEFINITIONS.BrowserCache.defaultTrash, false);
});

test("CATEGORY_DEFINITIONS includes UserTrash metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.UserTrash);
  assert.equal(CATEGORY_DEFINITIONS.UserTrash.id, "UserTrash");
  assert.equal(CATEGORY_DEFINITIONS.UserTrash.group, "System");
  assert.equal(CATEGORY_DEFINITIONS.UserTrash.defaultTrash, false);
});

test("CATEGORY_DEFINITIONS includes DevPython metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.DevPython);
  assert.equal(CATEGORY_DEFINITIONS.DevPython.id, "DevPython");
  assert.equal(CATEGORY_DEFINITIONS.DevPython.group, "Developer");
  assert.equal(CATEGORY_DEFINITIONS.DevPython.defaultTrash, true);
});

test("CATEGORY_DEFINITIONS includes SnapFlatpak metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.SnapFlatpak);
  assert.equal(CATEGORY_DEFINITIONS.SnapFlatpak.id, "SnapFlatpak");
  assert.equal(CATEGORY_DEFINITIONS.SnapFlatpak.group, "System");
  assert.equal(CATEGORY_DEFINITIONS.SnapFlatpak.defaultTrash, false);
});

test("CATEGORY_DEFINITIONS includes DevJvm metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.DevJvm);
  assert.equal(CATEGORY_DEFINITIONS.DevJvm.id, "DevJvm");
  assert.equal(CATEGORY_DEFINITIONS.DevJvm.group, "Developer");
  assert.equal(CATEGORY_DEFINITIONS.DevJvm.defaultTrash, true);
});

test("CATEGORY_DEFINITIONS includes IdeCache metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.IdeCache);
  assert.equal(CATEGORY_DEFINITIONS.IdeCache.id, "IdeCache");
  assert.equal(CATEGORY_DEFINITIONS.IdeCache.group, "Developer");
  assert.equal(CATEGORY_DEFINITIONS.IdeCache.defaultTrash, true);
});

test("CATEGORY_DEFINITIONS includes LargeFiles metadata", () => {
  assert.ok(CATEGORY_DEFINITIONS.LargeFiles);
  assert.equal(CATEGORY_DEFINITIONS.LargeFiles.id, "LargeFiles");
  assert.equal(CATEGORY_DEFINITIONS.LargeFiles.group, "Stale");
  assert.equal(CATEGORY_DEFINITIONS.LargeFiles.defaultTrash, true);
});
