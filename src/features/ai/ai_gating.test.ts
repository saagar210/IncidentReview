import { describe, expect, it } from "vitest";

import { computeAiGate } from "./ai_gating";

describe("computeAiGate", () => {
  it("blocks when Ollama unhealthy", () => {
    const g = computeAiGate({ healthOk: false, sourcesCount: 1, chunksCount: 1, indexReady: true, selectedCitationsCount: 1 });
    expect(g.canSearch).toBe(false);
    expect(g.reasonCode).toBe("AI_OLLAMA_UNHEALTHY");
  });

  it("blocks when evidence empty", () => {
    const g = computeAiGate({ healthOk: true, sourcesCount: 0, chunksCount: 0, indexReady: false, selectedCitationsCount: 0 });
    expect(g.canSearch).toBe(false);
    expect(g.reasonCode).toBe("AI_EVIDENCE_EMPTY");
  });

  it("blocks when chunks missing", () => {
    const g = computeAiGate({ healthOk: true, sourcesCount: 1, chunksCount: 0, indexReady: false, selectedCitationsCount: 0 });
    expect(g.canSearch).toBe(false);
    expect(g.reasonCode).toBe("AI_INDEX_NOT_READY");
  });

  it("blocks when index not ready", () => {
    const g = computeAiGate({ healthOk: true, sourcesCount: 1, chunksCount: 1, indexReady: false, selectedCitationsCount: 0 });
    expect(g.canSearch).toBe(false);
    expect(g.reasonCode).toBe("AI_INDEX_NOT_READY");
  });

  it("allows search but blocks draft without citations", () => {
    const g = computeAiGate({ healthOk: true, sourcesCount: 1, chunksCount: 1, indexReady: true, selectedCitationsCount: 0 });
    expect(g.canSearch).toBe(true);
    expect(g.canDraft).toBe(false);
    expect(g.reasonCode).toBe("AI_CITATION_REQUIRED");
  });
});

