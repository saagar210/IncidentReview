import { execSync } from "node:child_process";

function getChangedFiles() {
  const base = process.env.GITHUB_BASE_REF ? `origin/${process.env.GITHUB_BASE_REF}` : "origin/main";
  const candidates = [`git diff --name-only ${base}...HEAD`, "git diff --name-only HEAD~1...HEAD", "git diff --name-only HEAD"];

  for (const cmd of candidates) {
    try {
      const out = execSync(cmd, { encoding: "utf8" });
      const files = out
        .split("\n")
        .map((line) => line.trim())
        .filter(Boolean);
      if (files.length > 0) return files;
    } catch {
      // try next strategy
    }
  }

  return [];
}

const files = getChangedFiles();
if (files.length === 0) {
  console.log("No changed files detected. Skipping policy checks.");
  process.exit(0);
}

const isProdCode = (f) => /^(src|src-tauri|crates)\//.test(f) && !/(\.test\.|\.spec\.|\/tests\/)/.test(f);
const isTest = (f) =>
  /^tests\//.test(f) || /\.(test|spec)\.[cm]?[jt]sx?$/.test(f) || /^crates\/.+\/tests\//.test(f);
const isDoc = (f) => /^docs\//.test(f) || /^openapi\//.test(f) || f === "README.md";
const isApiSurface = (f) =>
  /^src-tauri\/src\//.test(f) || f === "src/lib/schemas.ts" || f === "src/lib/tauri.ts" || /^crates\/.+\/src\//.test(f);
const isArchitecture = (f) => /^crates\//.test(f) || /^migrations\//.test(f) || /^src-tauri\//.test(f);
const isAdr = (f) => /^docs\/adr\/\d{4}-.*\.md$/.test(f);

const prodChanged = files.some(isProdCode);
const testsChanged = files.some(isTest);
const apiChanged = files.some(isApiSurface);
const docsChanged = files.some(isDoc);
const architectureChanged = files.some(isArchitecture);
const adrChanged = files.some(isAdr);

let failed = false;

if (prodChanged && !testsChanged) {
  console.error("Policy failure: production code changed without test updates.");
  failed = true;
}

if (apiChanged && !docsChanged) {
  console.error("Policy failure: command/API surface changed without docs/openapi updates.");
  failed = true;
}

if (architectureChanged && !adrChanged) {
  console.error("Policy failure: architecture-impacting changes require a new ADR under docs/adr/.");
  failed = true;
}

if (failed) {
  process.exit(1);
}

console.log("Policy checks passed.");
