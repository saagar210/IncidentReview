import { describe, expect, it } from "vitest";
import { renderToString } from "react-dom/server";

import { SanitizedImportSection } from "./SanitizedImportSection";

describe("SanitizedImportSection", () => {
  it("renders with no dataset selected", () => {
    const html = renderToString(
      <SanitizedImportSection
        sanitizedExport={null}
        sanitizedImportDir=""
        sanitizedImportManifest={null}
        sanitizedImportSummary={null}
        onExportSanitizedDataset={() => {}}
        onPickSanitizedDatasetForImport={() => {}}
        onImportSanitizedDataset={() => {}}
      />
    );

    expect(html).toContain("Sanitized Dataset");
    expect(html).toContain("Import Sanitized Dataset");
  });
});

