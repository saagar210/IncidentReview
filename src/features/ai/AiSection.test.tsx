// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AiSection } from "./AiSection";

const mockInvokeValidated = vi.fn();
const mockExtractAppError = vi.fn();

function createMemoryStorage() {
  const data = new Map<string, string>();
  return {
    getItem(key: string) {
      return data.has(key) ? data.get(key)! : null;
    },
    setItem(key: string, value: string) {
      data.set(key, String(value));
    },
    removeItem(key: string) {
      data.delete(key);
    },
  };
}

vi.mock("../../lib/tauri", () => ({
  invokeValidated: (...args: unknown[]) => mockInvokeValidated(...args),
  extractAppError: (...args: unknown[]) => mockExtractAppError(...args),
}));

describe("AiSection", () => {
  beforeEach(() => {
    mockInvokeValidated.mockReset();
    mockExtractAppError.mockReset();
    mockExtractAppError.mockReturnValue(null);
    const storage = createMemoryStorage();
    Object.defineProperty(window, "localStorage", { value: storage, configurable: true });
    Object.defineProperty(globalThis, "localStorage", { value: storage, configurable: true });
    window.localStorage.removeItem?.("incidentreview.ai.persistDrafts");
    window.localStorage.removeItem?.("incidentreview.ai.indexModel");
    window.localStorage.removeItem?.("incidentreview.ai.draftModel");
  });

  it("keeps search/draft gated when evidence is missing and reports health success", async () => {
    mockInvokeValidated.mockImplementation(async (command: string) => {
      if (command === "ai_health_check") {
        return { ok: true, message: "healthy" };
      }
      if (command === "ai_models_list") {
        return [{ name: "llama3.2:latest" }];
      }
      if (command === "ai_evidence_list_sources") {
        return [];
      }
      if (command === "ai_index_status") {
        return { ready: false, chunk_count: 0, chunks_total: 0, model: null, dims: null, updated_at: null };
      }
      if (command === "ai_drafts_list") {
        return [];
      }
      throw new Error(`unexpected command: ${command}`);
    });

    const onToast = vi.fn();
    render(<AiSection onToast={onToast} />);

    await waitFor(() => {
      expect(screen.getByText(/AI_EVIDENCE_EMPTY/)).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Search (selected source)" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Build Chunks (selected)" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Draft Section (requires citations)" })).toBeDisabled();
    expect(screen.getByText("Add Evidence Source")).toBeInTheDocument();
    expect(screen.getByText("Evidence Viewer")).toBeInTheDocument();
    expect(screen.getByText("Draft Artifacts (History / Provenance)")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Health Check (Ollama)" }));
    await waitFor(() => {
      expect(onToast).toHaveBeenCalledWith(expect.objectContaining({ kind: "success", title: "AI OK" }));
    });
  });
});
