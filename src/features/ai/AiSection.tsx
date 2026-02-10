import { useEffect, useMemo, useState } from "react";

import { invokeValidated, extractAppError } from "../../lib/tauri";
import { BuildChunksResultSchema, EvidenceChunkSummaryListSchema, EvidenceSourceListSchema } from "../../lib/schemas";
import { pickDirectory, pickTextFile } from "../../lib/pickers";

type EvidenceSourceType = "sanitized_export" | "slack_transcript" | "incident_report_md" | "freeform_text";

function defaultOriginKindForType(t: EvidenceSourceType): "file" | "directory" | "paste" {
  if (t === "sanitized_export") return "directory";
  if (t === "freeform_text") return "paste";
  return "file";
}

export function AiSection(props: { onToast: (t: { kind: "success" | "error"; title: string; message: string }) => void }) {
  const [sources, setSources] = useState<Array<{ source_id: string; type: EvidenceSourceType; origin: { kind: string; path?: string | null }; label: string; created_at: string }>>(
    []
  );
  const [chunks, setChunks] = useState<
    Array<{
      chunk_id: string;
      source_id: string;
      ordinal: number;
      text_sha256: string;
      token_count_est: number;
      meta: { kind: string; incident_keys?: string[] | null; time_range?: { start_ts?: string | null; end_ts?: string | null } | null };
    }>
  >([]);

  const [addType, setAddType] = useState<EvidenceSourceType>("sanitized_export");
  const [addLabel, setAddLabel] = useState<string>("Sanitized export");
  const [addPath, setAddPath] = useState<string>("");
  const [addText, setAddText] = useState<string>("");
  const [selectedSourceId, setSelectedSourceId] = useState<string>("");

  const originKind = useMemo(() => defaultOriginKindForType(addType), [addType]);

  useEffect(() => {
    if (addType === "sanitized_export") setAddLabel("Sanitized export");
    else if (addType === "slack_transcript") setAddLabel("Slack transcript");
    else if (addType === "incident_report_md") setAddLabel("Incident report (MD)");
    else setAddLabel("Freeform text");
  }, [addType]);

  async function refreshSources() {
    const res = await invokeValidated("ai_evidence_list_sources", undefined, EvidenceSourceListSchema);
    setSources(res);
    if (!selectedSourceId && res.length > 0) setSelectedSourceId(res[0].source_id);
  }

  async function refreshChunks(sourceId: string | null) {
    const res = await invokeValidated(
      "ai_evidence_list_chunks",
      { sourceId: sourceId ?? null },
      EvidenceChunkSummaryListSchema
    );
    setChunks(res);
  }

  useEffect(() => {
    void (async () => {
      try {
        const res = await invokeValidated("ai_evidence_list_sources", undefined, EvidenceSourceListSchema);
        setSources(res);
        setSelectedSourceId((cur) => cur || (res.length > 0 ? res[0].source_id : ""));
      } catch {
        // Don't toast on first-load failure; this screen is gated later by health/index status.
      }
    })();
  }, []);

  async function onPickPath() {
    try {
      if (originKind === "directory") {
        const dir = await pickDirectory();
        if (!dir) return;
        setAddPath(dir);
      } else if (originKind === "file") {
        const file = await pickTextFile();
        if (!file) return;
        setAddPath(file);
      }
    } catch (e) {
      props.onToast({ kind: "error", title: "Picker failed", message: String(e) });
    }
  }

  async function onAddSource() {
    try {
      const origin =
        originKind === "paste"
          ? { kind: "paste", path: null }
          : originKind === "directory"
            ? { kind: "directory", path: addPath }
            : { kind: "file", path: addPath };

      const res = await invokeValidated(
        "ai_evidence_add_source",
        {
          req: {
            type: addType,
            origin,
            label: addLabel,
            text: originKind === "paste" ? addText : null,
          },
        },
        null
      );
      // res is an EvidenceSource; avoid schema churn here and just refresh from list endpoint.
      void res;
      await refreshSources();
      props.onToast({ kind: "success", title: "Evidence source added", message: addLabel });
    } catch (e) {
      const appErr = extractAppError(e);
      props.onToast({
        kind: "error",
        title: "Add evidence failed",
        message: appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  async function onBuildChunks() {
    try {
      const res = await invokeValidated(
        "ai_evidence_build_chunks",
        { sourceId: selectedSourceId || null },
        BuildChunksResultSchema
      );
      props.onToast({
        kind: "success",
        title: "Chunks built",
        message: `chunk_count=${res.chunk_count}; updated_at=${res.updated_at}`,
      });
      await refreshChunks(selectedSourceId || null);
    } catch (e) {
      const appErr = extractAppError(e);
      props.onToast({
        kind: "error",
        title: "Build chunks failed",
        message: appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  async function onListChunks() {
    try {
      await refreshChunks(selectedSourceId || null);
      props.onToast({ kind: "success", title: "Chunks loaded", message: `${chunks.length} chunks` });
    } catch (e) {
      const appErr = extractAppError(e);
      props.onToast({
        kind: "error",
        title: "List chunks failed",
        message: appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  return (
    <section className="card" id="ai">
      <h2>AI (Phase 5)</h2>
      <p className="hint">
        Phase 5 is local-only (Ollama on 127.0.0.1). AI must never compute deterministic metrics; it can only draft text
        with citations to evidence chunks.
      </p>

      <div className="card card--sub">
        <h3>Add Evidence Source</h3>
        <div className="grid">
          <label>
            Type
            <select value={addType} onChange={(e) => setAddType(e.target.value as EvidenceSourceType)}>
              <option value="sanitized_export">sanitized_export</option>
              <option value="slack_transcript">slack_transcript</option>
              <option value="incident_report_md">incident_report_md</option>
              <option value="freeform_text">freeform_text</option>
            </select>
          </label>
          <label>
            Label
            <input value={addLabel} onChange={(e) => setAddLabel(e.target.value)} />
          </label>
        </div>

        {originKind === "paste" ? (
          <label>
            Text (paste)
            <textarea rows={6} value={addText} onChange={(e) => setAddText(e.target.value)} placeholder="Paste text here" />
          </label>
        ) : (
          <label>
            Path ({originKind})
            <div className="actions">
              <input value={addPath} onChange={(e) => setAddPath(e.target.value)} placeholder="Pick a path" />
              <button className="btn" type="button" onClick={onPickPath}>
                Pick
              </button>
            </div>
          </label>
        )}

        <div className="actions">
          <button className="btn btn--accent" type="button" onClick={onAddSource}>
            Add Evidence
          </button>
          <button className="btn" type="button" onClick={refreshSources}>
            Refresh Sources
          </button>
        </div>
      </div>

      <div className="card card--sub">
        <h3>Sources</h3>
        {sources.length === 0 ? (
          <p className="hint">No sources yet.</p>
        ) : (
          <>
            <label>
              Selected source
              <select value={selectedSourceId} onChange={(e) => setSelectedSourceId(e.target.value)}>
                {sources.map((s) => (
                  <option key={s.source_id} value={s.source_id}>
                    {s.label} ({s.type})
                  </option>
                ))}
              </select>
            </label>
            <ul className="list">
              {sources.map((s) => (
                <li key={s.source_id}>
                  <code>{s.source_id}</code> <span className="pill pill--small">{s.type}</span> {s.label}
                  <div className="hint">
                    origin={s.origin.kind}
                    {s.origin.path ? `:${s.origin.path}` : ""}
                  </div>
                </li>
              ))}
            </ul>
          </>
        )}
      </div>

      <div className="card card--sub">
        <h3>Chunks</h3>
        <div className="actions">
          <button className="btn btn--accent" type="button" onClick={onBuildChunks} disabled={!selectedSourceId}>
            Build Chunks (selected)
          </button>
          <button className="btn" type="button" onClick={onListChunks} disabled={!selectedSourceId}>
            List Chunks (selected)
          </button>
        </div>
        {chunks.length === 0 ? (
          <p className="hint">No chunks loaded.</p>
        ) : (
          <ul className="list">
            {chunks.slice(0, 50).map((c) => (
              <li key={c.chunk_id}>
                <code>{c.chunk_id}</code> <span className="pill pill--small">ord {c.ordinal}</span>{" "}
                <span className="pill pill--small">{c.meta.kind}</span>
                {c.meta.incident_keys && c.meta.incident_keys.length > 0 ? (
                  <span className="hint"> incident_keys={c.meta.incident_keys.join(",")}</span>
                ) : null}
              </li>
            ))}
          </ul>
        )}
        {chunks.length > 50 ? <p className="hint">Showing first 50 chunks.</p> : null}
      </div>
    </section>
  );
}
