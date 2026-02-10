import { useEffect, useState } from "react";

import { invokeValidated } from "../../lib/tauri";
import { AiHealthStatusSchema, AiModelInfoListSchema, AppInfoSchema } from "../../lib/schemas";

type AppInfo = {
  app_version: string;
  git_commit_hash?: string | null;
  current_db_path: string;
  latest_migration: string;
  applied_migrations: string[];
};

type AiHealthStatus = { ok: boolean; message: string };
type AiModelInfo = { name: string; size?: number | null; digest?: string | null; modified_at?: string | null };

function safeReadLocalStorage(key: string): string | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function AboutSection() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [aiHealth, setAiHealth] = useState<AiHealthStatus | null>(null);
  const [aiModels, setAiModels] = useState<AiModelInfo[] | null>(null);
  const [loadErr, setLoadErr] = useState<string>("");

  const selectedDraftModel = safeReadLocalStorage("incidentreview.ai.draftModel");
  const selectedEmbeddingModel = safeReadLocalStorage("incidentreview.ai.indexModel");

  async function load() {
    setLoadErr("");
    try {
      const info = await invokeValidated("app_info", undefined, AppInfoSchema);
      setAppInfo(info);
    } catch (e) {
      setLoadErr(String(e));
      return;
    }

    try {
      const health = await invokeValidated("ai_health_check", undefined, AiHealthStatusSchema);
      setAiHealth(health);
      if (health.ok) {
        const models = await invokeValidated("ai_models_list", undefined, AiModelInfoListSchema);
        setAiModels(models);
      } else {
        setAiModels(null);
      }
    } catch {
      // AI is optional; About should still render.
      setAiHealth(null);
      setAiModels(null);
    }
  }

  useEffect(() => {
    void load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <section className="card" id="about">
      <h2>About</h2>
      <p className="hint">Build metadata is local-only. This app is offline-by-default and does not send telemetry.</p>
      <div className="actions">
        <button className="btn" type="button" onClick={() => void load()}>
          Refresh
        </button>
      </div>

      {loadErr ? (
        <p className="hint">
          <span className="mono">Error</span>: {loadErr}
        </p>
      ) : null}

      <div className="twoCol">
        <div>
          <div className="subhead">App</div>
          <ul className="list">
            <li>
              Version: <span className="mono">{appInfo?.app_version ?? "NULL"}</span>
            </li>
            <li>
              Git commit: <span className="mono">{appInfo?.git_commit_hash ?? "NULL"}</span>
            </li>
            <li>
              Workspace DB: <span className="mono">{appInfo?.current_db_path ?? "NULL"}</span>
            </li>
          </ul>

          <div className="subhead">Schema / Migrations</div>
          <ul className="list">
            <li>
              Latest migration: <span className="mono">{appInfo?.latest_migration ?? "NULL"}</span>
            </li>
            <li>
              Applied migrations: <span className="mono">{appInfo ? String(appInfo.applied_migrations.length) : "NULL"}</span>
            </li>
          </ul>
        </div>
        <div>
          <div className="subhead">Local AI (Ollama)</div>
          <p className="hint">
            Security: keep Ollama bound to <span className="mono">127.0.0.1</span> only. Do not expose it to the network.
          </p>
          <ul className="list">
            <li>
              Status: <span className="mono">{aiHealth ? (aiHealth.ok ? "ok" : "unhealthy") : "unknown"}</span>
            </li>
            <li>
              Draft model (selected): <span className="mono">{selectedDraftModel ?? "NULL"}</span>
            </li>
            <li>
              Embedding model (selected): <span className="mono">{selectedEmbeddingModel ?? "NULL"}</span>
            </li>
            <li>
              Installed models:{" "}
              <span className="mono">
                {aiModels ? aiModels.map((m) => m.name).join(", ") : aiHealth?.ok ? "loading/failed" : "unavailable"}
              </span>
            </li>
          </ul>
        </div>
      </div>
    </section>
  );
}

