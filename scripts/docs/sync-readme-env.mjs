import fs from "node:fs";

const readmePath = "README.md";
const envPath = ".env.example";
const start = "<!-- AUTO-GENERATED:ENV START -->";
const end = "<!-- AUTO-GENERATED:ENV END -->";

const envVars = fs
  .readFileSync(envPath, "utf8")
  .split("\n")
  .map((line) => line.trim())
  .filter((line) => line && !line.startsWith("#") && line.includes("="))
  .map((line) => line.split("=")[0]?.trim())
  .filter(Boolean);

const table = [
  "| Variable | Required | Description |",
  "|---|---|---|",
  ...envVars.map((name) => `| \`${name}\` | optional | TODO |`),
].join("\n");

const readme = fs.readFileSync(readmePath, "utf8");
const replacement = `${start}\n${table}\n${end}`;

const updated =
  readme.includes(start) && readme.includes(end)
    ? readme.replace(new RegExp(`${start}[\\s\\S]*?${end}`), replacement)
    : `${readme.trimEnd()}\n\n## Environment Variables\n\n${replacement}\n`;

fs.writeFileSync(readmePath, updated);
