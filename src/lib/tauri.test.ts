import { describe, expect, it, vi } from "vitest";

import { AppErrorException, invokeValidated } from "./tauri";

vi.mock("@tauri-apps/api/core", () => {
  return {
    invoke: vi.fn(),
  };
});

describe("invokeValidated error contract", () => {
  it("preserves structured AppError codes to callers", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as unknown as ReturnType<typeof vi.fn>).mockRejectedValueOnce({
      code: "INGEST_SANITIZED_DB_NOT_EMPTY",
      message: "Refusing to import into non-empty DB",
      details: "incidents=1; timeline_events=0; artifacts=0",
      retryable: false,
    });

    let caught: unknown = null;
    try {
      await invokeValidated("import_sanitized_dataset", { datasetDir: "/tmp/x" }, null);
    } catch (e) {
      caught = e;
    }

    expect(caught).toBeInstanceOf(AppErrorException);
    expect((caught as AppErrorException).code).toBe("INGEST_SANITIZED_DB_NOT_EMPTY");
  });
});
