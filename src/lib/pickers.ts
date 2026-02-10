import { open } from "@tauri-apps/plugin-dialog";

export async function pickDirectory(): Promise<string | null> {
  const res = await open({ directory: true, multiple: false });
  if (!res) return null;
  if (Array.isArray(res)) return res[0] ?? null;
  return res;
}

export async function pickDbFile(): Promise<string | null> {
  const res = await open({
    directory: false,
    multiple: false,
    filters: [{ name: "SQLite DB", extensions: ["sqlite", "db"] }],
  });
  if (!res) return null;
  if (Array.isArray(res)) return res[0] ?? null;
  return res;
}

