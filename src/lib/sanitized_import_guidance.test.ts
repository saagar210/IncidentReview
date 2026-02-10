import { describe, expect, it } from "vitest";

import { guidanceForSanitizedImportErrorCode } from "./sanitized_import_guidance";

describe("guidanceForSanitizedImportErrorCode", () => {
  it("returns deterministic guidance for DB not empty", () => {
    const msg = guidanceForSanitizedImportErrorCode("INGEST_SANITIZED_DB_NOT_EMPTY");
    expect(msg).toContain("non-empty");
    expect(msg).toContain("fresh DB");
  });

  it("returns guidance for manifest mismatches", () => {
    expect(guidanceForSanitizedImportErrorCode("INGEST_SANITIZED_MANIFEST_VERSION_MISMATCH")).toBeTruthy();
    expect(guidanceForSanitizedImportErrorCode("INGEST_SANITIZED_MANIFEST_HASH_MISMATCH")).toBeTruthy();
    expect(guidanceForSanitizedImportErrorCode("INGEST_SANITIZED_MANIFEST_BYTES_MISMATCH")).toBeTruthy();
  });

  it("returns guidance for metrics mismatch", () => {
    const msg = guidanceForSanitizedImportErrorCode("INGEST_SANITIZED_METRICS_MISMATCH");
    expect(msg).toContain("metrics");
  });
});

