#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const removableDirs = [
  "dist",
  "node_modules",
  "target",
  "src-tauri/target",
  ".pnpm-store",
  ".turbo",
  ".vite",
  "coverage",
  ".nyc_output",
  ".parcel-cache",
  ".cache",
  "_scaffold",
];

const removableFiles = [
  ".eslintcache",
  "pnpm-debug.log",
  "npm-debug.log",
  "yarn-debug.log",
  "yarn-error.log",
];

const removableGlobSuffixes = [
  ".tsbuildinfo",
  ".log",
];

const MAX_PASSES = 12;
const TRANSIENT_CODES = new Set(["ENOTEMPTY", "EBUSY", "EPERM", "EMFILE", "ENFILE"]);
const failures = new Map();

function relativePath(fullPath) {
  return path.relative(root, fullPath) || ".";
}

function rememberFailure(fullPath, error) {
  failures.set(relativePath(fullPath), {
    code: error?.code ?? "UNKNOWN_ERROR",
    message: error?.message ?? "unknown failure",
  });
}

function clearFailure(fullPath) {
  failures.delete(relativePath(fullPath));
}

function removePathOnce(fullPath) {
  try {
    fs.rmSync(fullPath, {
      recursive: true,
      force: true,
      maxRetries: 8,
      retryDelay: 120,
    });
    clearFailure(fullPath);
    return !fs.existsSync(fullPath);
  } catch (error) {
    rememberFailure(fullPath, error);
    return false;
  }
}

function removeWithPasses(fullPath) {
  if (!fs.existsSync(fullPath)) {
    clearFailure(fullPath);
    return true;
  }

  for (let pass = 1; pass <= MAX_PASSES; pass += 1) {
    const ok = removePathOnce(fullPath);
    if (ok) {
      return true;
    }

    if (!fs.existsSync(fullPath)) {
      clearFailure(fullPath);
      return true;
    }

    const failure = failures.get(relativePath(fullPath));
    if (!failure || !TRANSIENT_CODES.has(failure.code)) {
      return false;
    }
  }

  return !fs.existsSync(fullPath);
}

for (const relativeDir of removableDirs) {
  removeWithPasses(path.join(root, relativeDir));
}

for (const relativeFile of removableFiles) {
  removeWithPasses(path.join(root, relativeFile));
}

function removeDsStore(currentDir) {
  let entries;
  try {
    entries = fs.readdirSync(currentDir, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT") {
      return;
    }
    rememberFailure(currentDir, error);
    return;
  }

  for (const entry of entries) {
    const fullPath = path.join(currentDir, entry.name);

    if (entry.isDirectory()) {
      if (entry.name === ".git") {
        continue;
      }
      removeDsStore(fullPath);
      continue;
    }

    if (entry.isFile() && entry.name === ".DS_Store") {
      removeWithPasses(fullPath);
      continue;
    }

    if (
      entry.isFile() &&
      removableGlobSuffixes.some((suffix) => entry.name.endsWith(suffix))
    ) {
      removeWithPasses(fullPath);
    }
  }
}

removeDsStore(root);

const unresolved = [];
for (const relativeDir of removableDirs) {
  const fullPath = path.join(root, relativeDir);
  if (fs.existsSync(fullPath)) {
    unresolved.push(relativeDir);
  }
}
for (const relativeFile of removableFiles) {
  const fullPath = path.join(root, relativeFile);
  if (fs.existsSync(fullPath)) {
    unresolved.push(relativeFile);
  }
}

if (unresolved.length > 0) {
  for (const unresolvedPath of unresolved) {
    if (!failures.has(unresolvedPath)) {
      failures.set(unresolvedPath, {
        code: "LEFTOVER_PATH",
        message: "path still exists after cleanup passes",
      });
    }
  }
}

if (failures.size > 0) {
  console.error("Cleanup completed with failures:");
  for (const [failurePath, failure] of failures.entries()) {
    console.error(`- ${failurePath} (${failure.code}): ${failure.message}`);
  }
  process.exitCode = 1;
} else {
  console.log("Removed local build/cache artifacts.");
}
