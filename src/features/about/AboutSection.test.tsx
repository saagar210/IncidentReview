// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { AboutSection } from "./AboutSection";

const mockInvokeValidated = vi.fn();

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
}));

describe("AboutSection", () => {
  beforeEach(() => {
    mockInvokeValidated.mockReset();
    const storage = createMemoryStorage();
    Object.defineProperty(window, "localStorage", { value: storage, configurable: true });
    Object.defineProperty(globalThis, "localStorage", { value: storage, configurable: true });
    window.localStorage.removeItem?.("incidentreview.ai.draftModel");
    window.localStorage.removeItem?.("incidentreview.ai.indexModel");
  });

  it("loads app/AI metadata and refreshes on demand", async () => {
    window.localStorage.setItem("incidentreview.ai.draftModel", "llama3.2:latest");
    window.localStorage.setItem("incidentreview.ai.indexModel", "nomic-embed-text:latest");

    mockInvokeValidated.mockImplementation(async (command: string) => {
      if (command === "app_info") {
        return {
          app_version: "0.1.0",
          git_commit_hash: "abc123",
          current_db_path: "/tmp/incidentreview.sqlite",
          latest_migration: "0005_ai_drafts",
          applied_migrations: ["0001", "0002", "0003", "0004", "0005"],
        };
      }
      if (command === "ai_health_check") {
        return { ok: true, message: "healthy" };
      }
      if (command === "ai_models_list") {
        return [{ name: "llama3.2:latest" }, { name: "nomic-embed-text:latest" }];
      }
      throw new Error(`unexpected command: ${command}`);
    });

    render(<AboutSection />);

    await waitFor(() => {
      expect(screen.getByText("abc123")).toBeInTheDocument();
    });
    expect(screen.getByText("/tmp/incidentreview.sqlite")).toBeInTheDocument();
    expect(screen.getByText("llama3.2:latest, nomic-embed-text:latest")).toBeInTheDocument();
    expect(screen.getByText("nomic-embed-text:latest")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    await waitFor(() => {
      const appInfoCalls = mockInvokeValidated.mock.calls.filter((call) => call[0] === "app_info");
      expect(appInfoCalls.length).toBeGreaterThanOrEqual(2);
    });
  });

  it("surfaces app-info load errors and avoids AI follow-up calls", async () => {
    mockInvokeValidated.mockImplementation(async (command: string) => {
      if (command === "app_info") {
        throw new Error("app info unavailable");
      }
      throw new Error(`unexpected command: ${command}`);
    });

    render(<AboutSection />);

    await waitFor(() => {
      expect(screen.getByText(/app info unavailable/)).toBeInTheDocument();
    });
    expect(mockInvokeValidated.mock.calls.some((call) => call[0] === "ai_health_check")).toBe(false);
  });
});
