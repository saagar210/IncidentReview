import { useEffect, useMemo, useState } from "react";

import { invokeValidated, extractAppError } from "../../lib/tauri";
import {
  AiIndexStatusSchema,
  BuildChunksResultSchema,
  EvidenceQueryResponseSchema,
  EvidenceChunkSummaryListSchema,
  EvidenceSourceListSchema,
} from "../../lib/schemas";
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
  const [indexModel, setIndexModel] = useState<string>("nomic-embed-text");
  const [indexStatus, setIndexStatus] = useState<null | {
    ready: boolean;
    model?: string | null;
    dims?: number | null;
    chunk_count: number;
    updated_at?: string | null;
  }>(null);
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [searchTopK, setSearchTopK] = useState<number>(8);
  const [searchHits, setSearchHits] = useState<
    Array<{
      chunk_id: string;
      source_id: string;
      score: number;
      snippet: string;
      citation: { chunk_id: string; locator: { source_id: string; ordinal: number; text_sha256: string; char_range?: [number, number] | null } };
    }>
  >([]);
  const [selectedCitationChunkIds, setSelectedCitationChunkIds] = useState<string[]>([]);

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

  async function refreshIndexStatus() {
    const st = await invokeValidated("ai_index_status", undefined, AiIndexStatusSchema);
    setIndexStatus(st);
  }

  async function onBuildIndex() {
    try {
      const st = await invokeValidated(
        "ai_index_build",
        { req: { model: indexModel, sourceId: selectedSourceId || null } },
        AiIndexStatusSchema
      );
      setIndexStatus(st);
      props.onToast({
        kind: "success",
        title: "Index built",
        message: `ready=${st.ready}; chunks=${st.chunk_count}; model=${st.model ?? "unknown"}`,
      });
    } catch (e) {
      const appErr = extractAppError(e);
      props.onToast({
        kind: "error",
        title: "Build index failed",
        message: appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  async function onSearchEvidence() {
    try {
      const res = await invokeValidated(
        "ai_evidence_query",
        {
          req: {
            query: searchQuery,
            topK: Math.max(1, Math.min(50, searchTopK | 0)),
            sourceFilter: selectedSourceId ? [selectedSourceId] : null,
          },
        },
        EvidenceQueryResponseSchema
      );
      setSearchHits(res.hits);
      props.onToast({ kind: "success", title: "Search complete", message: `${res.hits.length} hits` });
    } catch (e) {
      const appErr = extractAppError(e);
      props.onToast({
        kind: "error",
        title: "Search failed",
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

      <div className="card card--sub">
        <h3>Index (Embeddings)</h3>
        <p className="hint">Build an embeddings index locally via Ollama. Unit tests do not require Ollama.</p>
        <div className="grid">
          <label>
            Embedding model
            <input value={indexModel} onChange={(e) => setIndexModel(e.target.value)} placeholder="e.g. nomic-embed-text" />
          </label>
        </div>
        <div className="actions">
          <button className="btn" type="button" onClick={refreshIndexStatus}>
            Refresh Status
          </button>
          <button className="btn btn--accent" type="button" onClick={onBuildIndex} disabled={!selectedSourceId}>
            Build Index (selected source)
          </button>
        </div>
        {indexStatus ? (
          <p className="hint">
            ready={String(indexStatus.ready)}; chunks={indexStatus.chunk_count}; model={indexStatus.model ?? "NULL"}; dims=
            {indexStatus.dims ?? "NULL"}; updated_at={indexStatus.updated_at ?? "NULL"}
          </p>
        ) : (
          <p className="hint">Status not loaded.</p>
        )}
      </div>

      <div className="card card--sub">
        <h3>Search Evidence (Top-K)</h3>
        <p className="hint">
          Retrieval uses the local index. Ordering is stable: score desc, then chunk_id asc. Select chunks here to use as
          citations for drafting in DS4.
        </p>
        <div className="grid">
          <label>
            Query
            <input value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search query" />
          </label>
          <label>
            top_k
            <input
              type="number"
              value={searchTopK}
              onChange={(e) => setSearchTopK(parseInt(e.target.value || "8", 10))}
              min={1}
              max={50}
            />
          </label>
        </div>
        <div className="actions">
          <button className="btn btn--accent" type="button" onClick={onSearchEvidence} disabled={!selectedSourceId}>
            Search (selected source)
          </button>
        </div>
        {searchHits.length === 0 ? (
          <p className="hint">No hits.</p>
        ) : (
          <ul className="list">
            {searchHits.map((h) => {
              const checked = selectedCitationChunkIds.includes(h.chunk_id);
              return (
                <li key={h.chunk_id}>
                  <label>
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={(e) => {
                        const next = e.target.checked
                          ? Array.from(new Set([...selectedCitationChunkIds, h.chunk_id]))
                          : selectedCitationChunkIds.filter((id) => id !== h.chunk_id);
                        setSelectedCitationChunkIds(next);
                      }}
                    />{" "}
                    <code>{h.chunk_id}</code> <span className="pill pill--small">score {h.score.toFixed(4)}</span>
                  </label>
                  <div className="hint">{h.snippet}</div>
                </li>
              );
            })}
          </ul>
        )}
        {selectedCitationChunkIds.length > 0 ? (
          <p className="hint">Selected citations: {selectedCitationChunkIds.join(", ")}</p>
        ) : (
          <p className="hint">Selected citations: none</p>
        )}
      </div>
    </section>
  );
}
