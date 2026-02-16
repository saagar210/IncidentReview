#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const buildDirs = ["target", "src-tauri/target"];
const failures = [];

for (const dir of buildDirs) {
  const fullPath = path.join(root, dir);
  try {
    fs.rmSync(fullPath, { recursive: true, force: true });
  } catch (error) {
    failures.push({
      dir,
      code: error?.code ?? "UNKNOWN_ERROR",
      message: error?.message ?? "unknown failure",
    });
  }
}

if (failures.length > 0) {
  console.error("Build cleanup completed with failures:");
  for (const failure of failures) {
    console.error(`- ${failure.dir} (${failure.code}): ${failure.message}`);
  }
  process.exitCode = 1;
} else {
  console.log("Removed Rust/Tauri build artifacts.");
}
