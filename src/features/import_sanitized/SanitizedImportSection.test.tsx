// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SanitizedImportSection } from "./SanitizedImportSection";

describe("SanitizedImportSection", () => {
  it("requires manifest before import and surfaces deterministic import preview/result", () => {
    const onExportSanitizedDataset = vi.fn();
    const onPickSanitizedDatasetForImport = vi.fn();
    const onImportSanitizedDataset = vi.fn();

    const { rerender } = render(
      <SanitizedImportSection
        sanitizedExport={null}
        sanitizedImportDir=""
        sanitizedImportManifest={null}
        sanitizedImportSummary={null}
        onExportSanitizedDataset={onExportSanitizedDataset}
        onPickSanitizedDatasetForImport={onPickSanitizedDatasetForImport}
        onImportSanitizedDataset={onImportSanitizedDataset}
      />
    );

    expect(screen.getByRole("heading", { name: "Sanitized Dataset Import/Export (Deterministic)" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import Sanitized Dataset" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "Export Sanitized Dataset..." }));
    fireEvent.click(screen.getByRole("button", { name: "Pick Sanitized Dataset..." }));
    expect(onExportSanitizedDataset).toHaveBeenCalledTimes(1);
    expect(onPickSanitizedDatasetForImport).toHaveBeenCalledTimes(1);

    rerender(
      <SanitizedImportSection
        sanitizedExport={{ export_dir: "/tmp/sanitized-export", incident_count: 5 }}
        sanitizedImportDir="/tmp/sanitized-export"
        sanitizedImportManifest={{
          manifest_version: 1,
          app_version: "0.1.0",
          export_time: "2026-02-17T00:00:00Z",
          incident_count: 5,
          files: [
            { filename: "incidents.json", bytes: 128, sha256: "a".repeat(64) },
            { filename: "manifest.json", bytes: 32, sha256: "b".repeat(64) },
          ],
        }}
        sanitizedImportSummary={{
          inserted_incidents: 5,
          inserted_timeline_events: 12,
          import_warnings: [{ code: "IMPORT_WARN", message: "One record had missing optional fields." }],
        }}
        onExportSanitizedDataset={onExportSanitizedDataset}
        onPickSanitizedDatasetForImport={onPickSanitizedDatasetForImport}
        onImportSanitizedDataset={onImportSanitizedDataset}
      />
    );

    expect(screen.getByRole("button", { name: "Import Sanitized Dataset" })).toBeEnabled();
    expect(screen.getAllByText("/tmp/sanitized-export")).toHaveLength(2);
    expect(screen.getByText(/incidents.json, manifest.json/)).toBeInTheDocument();
    expect(screen.getByText(/missing optional fields/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Import Sanitized Dataset" }));
    expect(onImportSanitizedDataset).toHaveBeenCalledTimes(1);
  });
});
