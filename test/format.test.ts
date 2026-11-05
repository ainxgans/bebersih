import test from "node:test";
import assert from "node:assert/strict";
import { formatBytes, truncatePath } from "../src/utils/format.ts";

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
