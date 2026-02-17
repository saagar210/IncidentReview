import { describe, expect, it, vi } from "vitest";

import { AppErrorException, extractAppError, invokeValidated, isAppError } from "../../src/lib/tauri";
import { AppErrorSchema } from "../../src/lib/schemas";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("tauri error integration", () => {
  it("extracts nested error objects from wrapped invoke payloads", () => {
    const payload = {
      error: {
        code: "WORKSPACE_DB_LOCKED",
        message: "Workspace is locked",
        details: "database is locked",
        retryable: true,
      },
    };

    const extracted = extractAppError(payload);

    expect(extracted).toBeTruthy();
    expect(extracted?.code).toBe("WORKSPACE_DB_LOCKED");
    expect(extracted?.retryable).toBe(true);
  });

  it("extracts JSON-serialized AppError payloads", () => {
    const serialized = JSON.stringify({
      code: "VALIDATION_TIMESTAMP_ORDER",
      message: "Timestamp ordering is invalid",
      retryable: false,
    });

    const extracted = extractAppError(serialized);

    expect(isAppError(extracted)).toBe(true);
    expect(extracted?.message).toContain("Timestamp");
  });

  it("throws AppErrorException with stable code on invoke rejection", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;
    invokeMock.mockRejectedValueOnce({
      code: "INGEST_SANITIZED_DB_NOT_EMPTY",
      message: "Refusing to import into non-empty DB",
      details: "incidents=1; timeline_events=0",
      retryable: false,
    });

    await expect(
      invokeValidated("import_sanitized_dataset", { datasetDir: "/tmp/sanitized" }, AppErrorSchema)
    ).rejects.toMatchObject({
      name: AppErrorException.name,
      code: "INGEST_SANITIZED_DB_NOT_EMPTY",
    });
  });
});
