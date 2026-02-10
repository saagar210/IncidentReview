import { useCallback, useEffect, useMemo, useState } from "react";

import { invokeValidated, extractAppError } from "../../lib/tauri";
import {
  AiDraftResponseSchema,
  AiDraftArtifactListSchema,
  AiDraftArtifactSchema,
  AiHealthStatusSchema,
  AiIndexStatusSchema,
  AiModelInfoListSchema,
  BuildChunksResultSchema,
  EvidenceQueryResponseSchema,
  EvidenceChunkSchema,
  EvidenceContextResponseSchema,
  EvidenceChunkSummaryListSchema,
  EvidenceSourceListSchema,
} from "../../lib/schemas";
import { pickDirectory, pickTextFile } from "../../lib/pickers";
import { guidanceForAiErrorCode } from "../../lib/ai_guidance";
import { computeAiGate } from "./ai_gating";

type EvidenceSourceType = "sanitized_export" | "slack_transcript" | "incident_report_md" | "freeform_text";

function defaultOriginKindForType(t: EvidenceSourceType): "file" | "directory" | "paste" {
  if (t === "sanitized_export") return "directory";
  if (t === "freeform_text") return "paste";
  return "file";
}

export function AiSection(props: { onToast: (t: { kind: "success" | "error"; title: string; message: string }) => void }) {
  const { onToast } = props;

  const [persistDrafts, setPersistDrafts] = useState<boolean>(true);
  const [healthOk, setHealthOk] = useState<boolean | null>(null);
  const [healthMessage, setHealthMessage] = useState<string>("");
  const [healthErrorCode, setHealthErrorCode] = useState<string | null>(null);

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
    chunks_total?: number | null;
    source_id?: string | null;
    updated_at?: string | null;
  }>(null);
  const [availableModels, setAvailableModels] = useState<Array<{ name: string; size?: number | null; digest?: string | null; modified_at?: string | null }>>([]);
  const [draftModel, setDraftModel] = useState<string>("llama3.2:latest");
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
  const [viewerChunk, setViewerChunk] = useState<null | {
    chunk_id: string;
    source_id: string;
    ordinal: number;
    text: string;
    text_sha256: string;
    token_count_est: number;
    meta: { kind: string; incident_keys?: string[] | null; time_range?: { start_ts?: string | null; end_ts?: string | null } | null };
  }>(null);
  const [viewerContext, setViewerContext] = useState<null | {
    centerChunkId: string;
    chunks: Array<{
      chunk_id: string;
      source_id: string;
      ordinal: number;
      text_sha256: string;
      token_count_est: number;
      meta: { kind: string; incident_keys?: string[] | null; time_range?: { start_ts?: string | null; end_ts?: string | null } | null };
    }>;
  }>(null);
  const [selectedCitationChunkIds, setSelectedCitationChunkIds] = useState<string[]>([]);
  const [draftSectionId, setDraftSectionId] = useState<
    "exec_summary" | "incident_highlights_top_n" | "theme_analysis" | "action_plan_next_quarter" | "quarter_narrative_recap"
  >("exec_summary");
  const [draftQuarterLabel, setDraftQuarterLabel] = useState<string>("Q1 2026");
  const [draftPrompt, setDraftPrompt] = useState<string>("Draft an executive summary based on the evidence.");
  const [draftMarkdown, setDraftMarkdown] = useState<string>("");
  const [draftCitations, setDraftCitations] = useState<
    Array<{ chunk_id: string; locator: { source_id: string; ordinal: number; text_sha256: string; char_range?: [number, number] | null } }>
  >([]);
  const [draftArtifacts, setDraftArtifacts] = useState<
    Array<{
      id: number;
      quarter_label: string;
      section_type: string;
      draft_text: string;
      citation_chunk_ids: string[];
      model_name: string;
      model_params_hash: string;
      prompt_template_version: string;
      created_at: string;
      artifact_hash: string;
    }>
  >([]);

  useEffect(() => {
    // Local-only persistence for selected models. This is UI state only; it does not affect deterministic metrics.
    if (typeof window === "undefined") return;
    try {
      const m = window.localStorage.getItem("incidentreview.ai.indexModel");
      if (m) setIndexModel(m);
      const d = window.localStorage.getItem("incidentreview.ai.draftModel");
      if (d) setDraftModel(d);
    } catch {
      // ignore
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem("incidentreview.ai.indexModel", indexModel);
    } catch {
      // ignore
    }
  }, [indexModel]);

  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      window.localStorage.setItem("incidentreview.ai.draftModel", draftModel);
    } catch {
      // ignore
    }
  }, [draftModel]);
  const [draftArtifactsQuarterFilter, setDraftArtifactsQuarterFilter] = useState<string>("");
  const [selectedDraftArtifactId, setSelectedDraftArtifactId] = useState<number | null>(null);
  const selectedDraftArtifact = useMemo(() => {
    if (selectedDraftArtifactId == null) return null;
    return draftArtifacts.find((d) => d.id === selectedDraftArtifactId) ?? null;
  }, [draftArtifacts, selectedDraftArtifactId]);

  const defaultDraftPromptBySection = useMemo(() => {
    return {
      exec_summary: "Draft an executive summary based on the evidence.",
      incident_highlights_top_n: "Draft the top incident highlights based on the evidence.",
      theme_analysis: "Draft a theme analysis based on the evidence.",
      action_plan_next_quarter: "Draft an action plan for next quarter based on the evidence.",
      quarter_narrative_recap: "Draft a short narrative recap of the quarter based on the evidence.",
    } as const;
  }, []);

  useEffect(() => {
    // Only auto-update the prompt when the user hasn't customized it.
    const defaults = Object.values(defaultDraftPromptBySection);
    setDraftPrompt((cur) => {
      if (cur.trim() === "" || defaults.includes(cur as never)) {
        return defaultDraftPromptBySection[draftSectionId];
      }
      return cur;
    });
  }, [draftSectionId, defaultDraftPromptBySection]);

  const originKind = useMemo(() => defaultOriginKindForType(addType), [addType]);
  const sourceById = useMemo(() => new Map(sources.map((s) => [s.source_id, s] as const)), [sources]);
  const chunkSummaryById = useMemo(() => new Map(chunks.map((c) => [c.chunk_id, c] as const)), [chunks]);

  function dedupePreserveOrder(ids: string[]): string[] {
    const seen = new Set<string>();
    const out: string[] = [];
    for (const id of ids) {
      if (seen.has(id)) continue;
      seen.add(id);
      out.push(id);
    }
    return out;
  }

  const gate = useMemo(() => {
    return computeAiGate({
      healthOk,
      sourcesCount: sources.length,
      chunksCount: chunks.length,
      indexReady: indexStatus?.ready ?? null,
      selectedCitationsCount: selectedCitationChunkIds.length,
    });
  }, [healthOk, sources.length, chunks.length, indexStatus?.ready, selectedCitationChunkIds.length]);

  useEffect(() => {
    // Local-only privacy control; no telemetry.
    const v = localStorage.getItem("incidentreview.ai.persistDrafts");
    if (v === null) {
      setPersistDrafts(true);
      return;
    }
    setPersistDrafts(v === "true");
  }, []);

  useEffect(() => {
    localStorage.setItem("incidentreview.ai.persistDrafts", persistDrafts ? "true" : "false");
  }, [persistDrafts]);

  useEffect(() => {
    if (addType === "sanitized_export") setAddLabel("Sanitized export");
    else if (addType === "slack_transcript") setAddLabel("Slack transcript");
    else if (addType === "incident_report_md") setAddLabel("Incident report (MD)");
    else setAddLabel("Freeform text");
  }, [addType]);

  const runHealthCheck = useCallback(async (toast: boolean) => {
    try {
      const res = await invokeValidated("ai_health_check", undefined, AiHealthStatusSchema);
      setHealthOk(true);
      setHealthErrorCode(null);
      setHealthMessage(res.message);
      if (toast) onToast({ kind: "success", title: "AI OK", message: res.message });
    } catch (e) {
      const appErr = extractAppError(e);
      setHealthOk(false);
      setHealthErrorCode(appErr?.code ?? "AI_OLLAMA_UNHEALTHY");
      setHealthMessage(appErr ? `${appErr.code}: ${appErr.message}` : String(e));
      if (toast) {
        const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
        onToast({
          kind: "error",
          title: "AI unavailable",
          message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : String(e),
        });
      }
    }
  }, [onToast]);

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

  async function refreshDraftArtifacts(quarterLabel: string | null) {
    const res = await invokeValidated(
      "ai_drafts_list",
      { quarterLabel: quarterLabel ?? null },
      AiDraftArtifactListSchema
    );
    setDraftArtifacts(res);
    setSelectedDraftArtifactId((cur) => {
      if (cur == null) return null;
      return res.some((d) => d.id === cur) ? cur : null;
    });
    return res;
  }

  useEffect(() => {
    void (async () => {
      try {
        await runHealthCheck(false);
        const tags = await invokeValidated("ai_models_list", undefined, AiModelInfoListSchema);
        setAvailableModels(tags);
        // Default selection: prefer known good defaults when present.
        const names = new Set(tags.map((m) => m.name));
        if (names.has("nomic-embed-text:latest")) setIndexModel("nomic-embed-text:latest");
        if (names.has("llama3.2:latest")) setDraftModel("llama3.2:latest");
        const res = await invokeValidated("ai_evidence_list_sources", undefined, EvidenceSourceListSchema);
        setSources(res);
        setSelectedSourceId((cur) => cur || (res.length > 0 ? res[0].source_id : ""));
        const st = await invokeValidated("ai_index_status", undefined, AiIndexStatusSchema);
        setIndexStatus(st);
        await refreshDraftArtifacts(draftQuarterLabel);
      } catch {
        // Don't toast on first-load failure; this screen is gated later by health/index status.
      }
    })();
  }, [runHealthCheck, draftQuarterLabel]);

  useEffect(() => {
    if (!selectedSourceId) return;
    void (async () => {
      try {
        await refreshChunks(selectedSourceId);
      } catch {
        // ignore
      }
    })();
  }, [selectedSourceId]);

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
      onToast({ kind: "error", title: "Picker failed", message: String(e) });
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
      onToast({ kind: "success", title: "Evidence source added", message: addLabel });
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Add evidence failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
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
      onToast({
        kind: "success",
        title: "Chunks built",
        message: `chunk_count=${res.chunk_count}; updated_at=${res.updated_at}`,
      });
      await refreshChunks(selectedSourceId || null);
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Build chunks failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
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
      onToast({
        kind: "success",
        title: "Index built",
        message: `ready=${st.ready}; chunks=${st.chunk_count}; model=${st.model ?? "unknown"}`,
      });
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Build index failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
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
      onToast({ kind: "success", title: "Search complete", message: `${res.hits.length} hits` });
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Search failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  function addCitationId(id: string) {
    setSelectedCitationChunkIds((cur) => (cur.includes(id) ? cur : [...cur, id]));
  }

  function removeCitationId(id: string) {
    setSelectedCitationChunkIds((cur) => cur.filter((x) => x !== id));
  }

  function moveCitation(id: string, direction: -1 | 1) {
    setSelectedCitationChunkIds((cur) => {
      const idx = cur.indexOf(id);
      if (idx < 0) return cur;
      const nextIdx = idx + direction;
      if (nextIdx < 0 || nextIdx >= cur.length) return cur;
      const next = cur.slice();
      const tmp = next[idx];
      next[idx] = next[nextIdx];
      next[nextIdx] = tmp;
      return next;
    });
  }

  async function onRevealFullChunk(chunkId: string) {
    try {
      const chunk = await invokeValidated(
        "ai_evidence_get_chunk",
        { chunkId },
        EvidenceChunkSchema
      );
      setViewerChunk(chunk);
      onToast({ kind: "success", title: "Chunk loaded", message: `ordinal=${chunk.ordinal}; source_id=${chunk.source_id}` });
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Load chunk failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  async function onLoadContext(chunkId: string, window: number) {
    try {
      const ctx = await invokeValidated(
        "ai_evidence_get_context",
        { req: { chunkId, window } },
        EvidenceContextResponseSchema
      );
      setViewerContext(ctx);
      onToast({ kind: "success", title: "Context loaded", message: `${ctx.chunks.length} chunks` });
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Load context failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  async function onListChunks() {
    try {
      await refreshChunks(selectedSourceId || null);
      onToast({ kind: "success", title: "Chunks loaded", message: `${chunks.length} chunks` });
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "List chunks failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
      });
    }
  }

  async function onDraftSection() {
    try {
      const res = await invokeValidated(
        "ai_draft_section",
        {
          req: {
            sectionId: draftSectionId,
            quarterLabel: draftQuarterLabel,
            prompt: draftPrompt,
            citationChunkIds: selectedCitationChunkIds,
            model: draftModel,
          },
        },
        AiDraftResponseSchema
      );
      setDraftMarkdown(res.markdown);
      setDraftCitations(res.citations);
      onToast({ kind: "success", title: "Draft complete", message: `section=${draftSectionId}; citations=${res.citations.length}` });

      if (persistDrafts) {
        const citationChunkIds = dedupePreserveOrder(selectedCitationChunkIds);
        const artifact = await invokeValidated(
          "ai_drafts_create",
          {
            quarterLabel: draftQuarterLabel,
            sectionType: draftSectionId,
            draftText: res.markdown,
            citationChunkIds,
            modelName: res.model_name,
            modelParamsHash: res.model_params_hash,
            promptTemplateVersion: res.prompt_template_version,
          },
          AiDraftArtifactSchema
        );
        onToast({
          kind: "success",
          title: "Draft stored locally",
          message: `id=${artifact.id}; hash=${artifact.artifact_hash.slice(0, 12)}...`,
        });
        await refreshDraftArtifacts(draftQuarterLabel);
        setSelectedDraftArtifactId(artifact.id);
      }
    } catch (e) {
      const appErr = extractAppError(e);
      const guidance = appErr ? guidanceForAiErrorCode(appErr.code) : null;
      onToast({
        kind: "error",
        title: "Draft failed",
        message: appErr && guidance ? `${appErr.code}: ${appErr.message}\n\n${guidance}` : appErr ? `${appErr.code}: ${appErr.message}` : String(e),
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
      <p className="hint">
        Security: keep Ollama bound to <span className="mono">127.0.0.1</span> only. IncidentReview hard-rejects remote
        endpoints; do not expose Ollama to the network.
      </p>

      <div className="card card--sub">
        <h3>Preflight / Gating</h3>
        <p className="hint">Actions are gated by local health and index readiness. The app never calls external APIs.</p>
        <div className="actions">
          <button className="btn btn--accent" type="button" onClick={() => runHealthCheck(true)}>
            Health Check (Ollama)
          </button>
          <button className="btn" type="button" onClick={refreshSources}>
            Refresh Sources
          </button>
          <button className="btn" type="button" onClick={refreshIndexStatus}>
            Refresh Index Status
          </button>
        </div>
        <p className="hint">
          healthOk={String(healthOk)}; sources={sources.length}; chunks(selected)={chunks.length}; indexReady=
          {String(indexStatus?.ready ?? null)}; selectedCitations={selectedCitationChunkIds.length}
        </p>
        {availableModels.length > 0 ? (
          <p className="hint">Local models detected: {availableModels.map((m) => m.name).join(", ")}</p>
        ) : (
          <p className="hint">Local models: not loaded (use Health Check to confirm Ollama is running).</p>
        )}
        {gate.reasonCode ? (
          <p className="hint">
            Blocked: <code>{gate.reasonCode}</code> {gate.reasonMessage ? `- ${gate.reasonMessage}` : ""}
            {guidanceForAiErrorCode(gate.reasonCode) ? `\n\n${guidanceForAiErrorCode(gate.reasonCode)}` : ""}
          </p>
        ) : (
          <p className="hint">Ready: search and drafting are enabled.</p>
        )}
        {healthErrorCode && healthOk === false ? (
          <p className="hint">
            Last health error: <code>{healthErrorCode}</code> {healthMessage}
          </p>
        ) : null}
      </div>

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
            {availableModels.length > 0 ? (
              <select value={indexModel} onChange={(e) => setIndexModel(e.target.value)}>
                {availableModels.filter((m) => m.name.includes("embed")).length > 0 ? (
                  availableModels
                    .filter((m) => m.name.includes("embed"))
                    .map((m) => (
                      <option key={m.name} value={m.name}>
                        {m.name}
                      </option>
                    ))
                ) : (
                  <option value={indexModel}>{indexModel}</option>
                )}
              </select>
            ) : (
              <input value={indexModel} onChange={(e) => setIndexModel(e.target.value)} placeholder="e.g. nomic-embed-text:latest" />
            )}
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
            ready={String(indexStatus.ready)}; embedded={indexStatus.chunk_count}; total={indexStatus.chunks_total ?? "NULL"}; model=
            {indexStatus.model ?? "NULL"}; dims={indexStatus.dims ?? "NULL"}; updated_at={indexStatus.updated_at ?? "NULL"}
          </p>
        ) : (
          <p className="hint">Status not loaded.</p>
        )}
      </div>

      <div className="card card--sub">
        <h3>Evidence Viewer</h3>
        <p className="hint">
          Search uses the local embeddings index. Ordering is stable: score desc, then chunk_id asc. Snippets are shown by
          default; full chunk text is only loaded after an explicit click (it may contain sensitive evidence).
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
          <button className="btn btn--accent" type="button" onClick={onSearchEvidence} disabled={!gate.canSearch}>
            Search (selected source)
          </button>
          <button
            className="btn"
            type="button"
            onClick={() => {
              setViewerChunk(null);
              setViewerContext(null);
            }}
          >
            Clear Viewer
          </button>
        </div>

        <h4>Citation Set</h4>
        {selectedCitationChunkIds.length === 0 ? (
          <p className="hint">No citations selected. Add at least one chunk before drafting.</p>
        ) : (
          <>
            <div className="actions">
              <button className="btn" type="button" onClick={() => setSelectedCitationChunkIds([])}>
                Clear Citations
              </button>
            </div>
            <ul className="list">
              {selectedCitationChunkIds.map((id, idx) => (
                <li key={id}>
                  <code>{id}</code>{" "}
                  <span className="hint">
                    {idx + 1}/{selectedCitationChunkIds.length}
                  </span>
                  <div className="actions">
                    <button className="btn btn--small" type="button" onClick={() => moveCitation(id, -1)} disabled={idx === 0}>
                      Up
                    </button>
                    <button
                      className="btn btn--small"
                      type="button"
                      onClick={() => moveCitation(id, 1)}
                      disabled={idx === selectedCitationChunkIds.length - 1}
                    >
                      Down
                    </button>
                    <button className="btn btn--small" type="button" onClick={() => removeCitationId(id)}>
                      Remove
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          </>
        )}

        <h4>Search Results</h4>
        {searchHits.length === 0 ? (
          <p className="hint">No hits.</p>
        ) : (
          <ul className="list">
            {searchHits.map((h) => {
              const selected = selectedCitationChunkIds.includes(h.chunk_id);
              const src = sourceById.get(h.source_id);
              const sum = chunkSummaryById.get(h.chunk_id);
              return (
                <li key={h.chunk_id}>
                  <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
                    <code>{h.chunk_id}</code> <span className="pill pill--small">score {h.score.toFixed(4)}</span>{" "}
                    <span className="hint">
                      source_id={h.source_id}; ord={h.citation.locator.ordinal}
                    </span>
                    {src ? <span className="pill pill--small">{src.type}</span> : null}
                    {sum?.meta.kind ? <span className="pill pill--small">{sum.meta.kind}</span> : null}
                    {sum?.meta.incident_keys && sum.meta.incident_keys.length > 0 ? (
                      <span className="hint">incident_keys={sum.meta.incident_keys.join(",")}</span>
                    ) : null}
                  </div>
                  <div className="hint">{h.snippet}</div>
                  <div className="actions">
                    <button
                      className={selected ? "btn" : "btn btn--accent"}
                      type="button"
                      onClick={() => (selected ? removeCitationId(h.chunk_id) : addCitationId(h.chunk_id))}
                    >
                      {selected ? "Remove citation" : "Add to citation set"}
                    </button>
                    <button className="btn" type="button" onClick={() => void onLoadContext(h.chunk_id, 2)}>
                      Show context
                    </button>
                    <button className="btn" type="button" onClick={() => void onRevealFullChunk(h.chunk_id)}>
                      Reveal full chunk
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}

        <h4>Focused Chunk (Full Text)</h4>
        {viewerChunk ? (
          <>
            <p className="hint">
              Warning: this is raw evidence text. Handle carefully. chunk_id=<code>{viewerChunk.chunk_id}</code>; source_id=
              <code>{viewerChunk.source_id}</code>; ordinal={viewerChunk.ordinal}
            </p>
            <div className="actions">
              <button className="btn btn--accent" type="button" onClick={() => addCitationId(viewerChunk.chunk_id)}>
                Add focused chunk to citation set
              </button>
              <button className="btn" type="button" onClick={() => void onLoadContext(viewerChunk.chunk_id, 2)}>
                Show context for focused chunk
              </button>
            </div>
            <pre className="code">{viewerChunk.text}</pre>
          </>
        ) : (
          <p className="hint">No chunk loaded. Use “Reveal full chunk” from a search result or a context item.</p>
        )}

        <h4>Context (Prev / Next)</h4>
        {viewerContext ? (
          <>
            <p className="hint">
              centerChunkId=<code>{viewerContext.centerChunkId}</code>; count={viewerContext.chunks.length}
            </p>
            <ul className="list">
              {viewerContext.chunks.map((c) => (
                <li key={c.chunk_id}>
                  <code>{c.chunk_id}</code> <span className="pill pill--small">ord {c.ordinal}</span>{" "}
                  <span className="pill pill--small">{c.meta.kind}</span>
                  <div className="actions">
                    <button className="btn btn--small" type="button" onClick={() => addCitationId(c.chunk_id)}>
                      Add citation
                    </button>
                    <button className="btn btn--small" type="button" onClick={() => void onRevealFullChunk(c.chunk_id)}>
                      Reveal full chunk
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          </>
        ) : (
          <p className="hint">No context loaded.</p>
        )}
      </div>

      <div className="card card--sub">
        <h3>Draft</h3>
        <p className="hint">
          Drafting is local-only via Ollama. Draft requests must include at least one selected citation chunk; the server
          hard-fails with <code>AI_CITATION_REQUIRED</code> if missing. For list/narrative sections, citations are enforced
          per bullet/paragraph.
        </p>
        <label className="hint" style={{ display: "flex", gap: 8, alignItems: "center" }}>
          <input
            type="checkbox"
            checked={persistDrafts}
            onChange={(e) => setPersistDrafts(e.target.checked)}
          />{" "}
          Store AI drafts locally (audit trail)
        </label>
        <p className="hint">
          Privacy note: stored drafts may contain sensitive text. This setting affects only local persistence in the workspace DB.
        </p>
        <div className="grid">
          <label>
            Section
            <select value={draftSectionId} onChange={(e) => setDraftSectionId(e.target.value as never)}>
              <option value="exec_summary">exec_summary</option>
              <option value="incident_highlights_top_n">incident_highlights_top_n</option>
              <option value="theme_analysis">theme_analysis</option>
              <option value="action_plan_next_quarter">action_plan_next_quarter</option>
              <option value="quarter_narrative_recap">quarter_narrative_recap</option>
            </select>
          </label>
          <label>
            Quarter label
            <input value={draftQuarterLabel} onChange={(e) => setDraftQuarterLabel(e.target.value)} />
          </label>
          <label>
            Draft model
            {availableModels.length > 0 ? (
              <select value={draftModel} onChange={(e) => setDraftModel(e.target.value)}>
                {availableModels.filter((m) => !m.name.includes("embed")).length > 0 ? (
                  availableModels
                    .filter((m) => !m.name.includes("embed"))
                    .map((m) => (
                      <option key={m.name} value={m.name}>
                        {m.name}
                      </option>
                    ))
                ) : (
                  <option value={draftModel}>{draftModel}</option>
                )}
              </select>
            ) : (
              <input value={draftModel} onChange={(e) => setDraftModel(e.target.value)} placeholder="e.g. llama3.2:latest" />
            )}
          </label>
        </div>
        <label>
          Prompt
          <textarea rows={4} value={draftPrompt} onChange={(e) => setDraftPrompt(e.target.value)} />
        </label>
        <div className="actions">
          <button
            className="btn btn--accent"
            type="button"
            onClick={onDraftSection}
            disabled={!gate.canDraft}
          >
            Draft Section (requires citations)
          </button>
        </div>
        {draftMarkdown ? (
          <>
            <h4>Draft Markdown</h4>
            <pre className="code">{draftMarkdown}</pre>
            <h4>Citations</h4>
            {draftCitations.length === 0 ? (
              <p className="hint">No citations returned (this should not happen; server enforces citations).</p>
            ) : (
              <ul className="list">
                {draftCitations.map((c) => (
                  <li key={c.chunk_id}>
                    <code>{c.chunk_id}</code> (source_id={c.locator.source_id}; ordinal={c.locator.ordinal})
                  </li>
                ))}
              </ul>
            )}
          </>
        ) : (
          <p className="hint">No draft yet.</p>
        )}
      </div>

      <div className="card card--sub">
        <h3>Draft Artifacts (History / Provenance)</h3>
        <p className="hint">
          Draft artifacts are stored in the current workspace DB (local-only). Use this to audit what the model produced, with which citations and which model metadata.
        </p>
        <div className="grid">
          <label>
            Quarter filter (optional)
            <input
              value={draftArtifactsQuarterFilter}
              onChange={(e) => setDraftArtifactsQuarterFilter(e.target.value)}
              placeholder="e.g. Q1 2026"
            />
          </label>
        </div>
        <div className="actions">
          <button
            className="btn"
            type="button"
            onClick={async () => {
              try {
                const q = draftArtifactsQuarterFilter.trim();
                const res = await refreshDraftArtifacts(q ? q : null);
                onToast({ kind: "success", title: "Draft history loaded", message: `${res.length} items` });
              } catch (e) {
                const appErr = extractAppError(e);
                onToast({
                  kind: "error",
                  title: "Draft history failed",
                  message: appErr ? `${appErr.code}: ${appErr.message}` : String(e),
                });
              }
            }}
          >
            Refresh Draft History
          </button>
        </div>

        {draftArtifacts.length === 0 ? (
          <p className="hint">No stored drafts.</p>
        ) : (
          <div className="grid" style={{ gridTemplateColumns: "1fr 1fr" }}>
            <div>
              <ul className="list">
                {draftArtifacts.slice(0, 25).map((d) => (
                  <li key={d.id}>
                    <button
                      type="button"
                      className="btn btn--small"
                      onClick={() => setSelectedDraftArtifactId(d.id)}
                      style={{ marginRight: 8 }}
                    >
                      View
                    </button>
                    <code>#{d.id}</code> <span className="pill pill--small">{d.section_type}</span>{" "}
                    <span className="hint">{d.quarter_label}</span>
                    <div className="hint">
                      created_at={d.created_at}; model={d.model_name}; citations={d.citation_chunk_ids.length}; hash=
                      {d.artifact_hash.slice(0, 12)}...
                    </div>
                  </li>
                ))}
              </ul>
              {draftArtifacts.length > 25 ? <p className="hint">Showing first 25 drafts.</p> : null}
            </div>

            <div>
              {selectedDraftArtifact ? (
                <>
                  <h4>Provenance</h4>
                  <p className="hint">
                    <span className="pill pill--small">{selectedDraftArtifact.section_type}</span>{" "}
                    <span className="pill pill--small">{selectedDraftArtifact.quarter_label}</span>
                  </p>
                  <p className="hint">
                    created_at={selectedDraftArtifact.created_at}
                    <br />
                    model_name={selectedDraftArtifact.model_name}
                    <br />
                    model_params_hash=<code>{selectedDraftArtifact.model_params_hash}</code>
                    <br />
                    prompt_template_version=<code>{selectedDraftArtifact.prompt_template_version}</code>
                    <br />
                    artifact_hash=<code>{selectedDraftArtifact.artifact_hash}</code>
                  </p>
                  <h4>Citations</h4>
                  {selectedDraftArtifact.citation_chunk_ids.length === 0 ? (
                    <p className="hint">No citation chunk IDs stored (this should not happen; storage enforces citations).</p>
                  ) : (
                    <ul className="list">
                      {selectedDraftArtifact.citation_chunk_ids.map((id) => (
                        <li key={id}>
                          <code>{id}</code>{" "}
                          <button
                            className="btn btn--small"
                            type="button"
                            onClick={() => {
                              setSelectedCitationChunkIds((cur) => dedupePreserveOrder([...cur, id]));
                              onToast({
                                kind: "success",
                                title: "Citation selected",
                                message: `Added ${id} to the citation set`,
                              });
                            }}
                          >
                            Add to citation set
                          </button>
                          <button className="btn btn--small" type="button" onClick={() => void onLoadContext(id, 2)}>
                            Show context
                          </button>
                          <button className="btn btn--small" type="button" onClick={() => void onRevealFullChunk(id)}>
                            Reveal full chunk
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                  <h4>Draft Text</h4>
                  <pre className="code">{selectedDraftArtifact.draft_text}</pre>
                </>
              ) : (
                <p className="hint">Select a draft to view provenance.</p>
              )}
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
