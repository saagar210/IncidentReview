import { execSync } from "node:child_process";

function changedFiles() {
  const output = execSync("git status --porcelain -- README.md docs openapi", { encoding: "utf8" });
  return output
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => line.slice(3))
    .sort();
}

const before = changedFiles();
execSync("pnpm docs:generate", { stdio: "inherit" });
const after = changedFiles();

if (before.join("\n") !== after.join("\n")) {
  console.error("Documentation drift detected: docs:generate changed tracked docs artifacts.");
  process.exit(1);
}

console.log("Documentation artifacts are in sync.");
