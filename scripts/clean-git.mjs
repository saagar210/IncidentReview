#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();

function runGit(args) {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.status !== 0) {
    const message = result.stderr?.trim() || `git ${args.join(" ")} failed`;
    throw new Error(message);
  }

  return result.stdout.trim();
}

function removeIfPresent(relativePath) {
  const fullPath = path.join(root, relativePath);
  if (!fs.existsSync(fullPath)) {
    return false;
  }

  fs.rmSync(fullPath, { force: true });
  return true;
}

const removedDsStore = [".DS_Store", ".git/.DS_Store"].filter(removeIfPresent);
const before = runGit(["count-objects", "-vH"]);

runGit(["reflog", "expire", "--expire=now", "--expire-unreachable=now", "--all"]);
runGit(["gc", "--aggressive", "--prune=now"]);
runGit(["prune-packed"]);
runGit(["repack", "-Ad"]);

const after = runGit(["count-objects", "-vH"]);

if (removedDsStore.length > 0) {
  console.log(`Removed: ${removedDsStore.join(", ")}`);
}

console.log("Git storage before cleanup:\n" + before + "\n");
console.log("Git storage after cleanup:\n" + after);
